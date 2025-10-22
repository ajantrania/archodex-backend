# Research: Account Plans & Rate Limiting

**Feature**: 001-rate-limits-we
**Date**: 2025-10-10
**Phase**: Phase 0 Research

This document captures research findings for key technical decisions needed to implement the rate limiting feature.

## Table of Contents

1. [Efficient Counter Mechanisms for SurrealDB 2.x](#1-efficient-counter-mechanisms-for-surrealdb-2x)
2. [Self-Hosted Authentication Pattern](#2-self-hosted-authentication-pattern)
3. [Protobuf Extension for Plan Limits in API Keys](#3-protobuf-extension-for-plan-limits-in-api-keys)

---

## 1. Efficient Counter Mechanisms for SurrealDB 2.x

### Decision

Use a dedicated `usage` table in the accounts database with atomic UPSERT operations for both resource counts and hourly event counts.

### Rationale

**COUNT() Performance Issues in SurrealDB 2.x:**
- COUNT indexes are NOT available in SurrealDB 2.x (introduced in v3.0.0-alpha.10)
- Without COUNT indexes, COUNT() queries require full table scans
- Requirement: <5ms counter queries with up to 10k resources (SC-009)

**UPSERT with += Provides Atomic Increments:**
```surrealql
UPSERT counter:account_123 SET resource_count += 1;
```
- Automatically type-infers numeric fields with default 0
- Works within transactions for consistency
- More performant than SELECT COUNT(*) pattern
- Avoids race conditions during concurrent increments

**Separate Counter Table Performance Benefits:**
- Direct record access by ID: `SELECT count FROM counter:account_123` is O(1) lookup
- No table scan required
- Transaction-safe counter updates
- Minimal storage overhead (one record per account vs scanning thousands of resources)

### Implementation Approach

**Storage Structure:**

```surrealql
-- In accounts DB (shared across all accounts)
DEFINE TABLE IF NOT EXISTS usage SCHEMAFULL TYPE NORMAL;
DEFINE FIELD IF NOT EXISTS account_id ON TABLE usage TYPE record<account> READONLY;
DEFINE FIELD IF NOT EXISTS resource_count ON TABLE usage TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS event_hour_window ON TABLE usage TYPE string;
DEFINE FIELD IF NOT EXISTS event_count_this_hour ON TABLE usage TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS last_updated_at ON TABLE usage TYPE datetime;
DEFINE INDEX IF NOT EXISTS usage_account_idx ON TABLE usage FIELDS account_id UNIQUE;
```

**Counter ID Format:** `usage:{account_id}` (e.g., `usage:1234567890`)

**Resource Counter Update Pattern (Only Count NEW Resources):**

```rust
// Within existing transaction in report.rs
// Step 1: Upsert resources with RETURN BEFORE to detect new vs updated
async fn upsert_resources_and_count_new(
    resources: Vec<ResourceTreeNode>
) -> Result<i64> {
    let mut new_resource_count = 0;

    for resource in resources {
        let before: Option<Resource> = db
            .query(
                "INSERT INTO resource (id, first_seen_at, last_seen_at)
                 VALUES ($id, $first_seen, $last_seen)
                 ON DUPLICATE KEY UPDATE last_seen_at = $input.last_seen_at
                 RETURN BEFORE"
            )
            .bind(("id", resource.id))
            .bind(("first_seen", resource.first_seen_at))
            .bind(("last_seen", resource.last_seen_at))
            .await?
            .take(0)?;

        // If BEFORE is null, this was a new insert
        if before.is_none() {
            new_resource_count += 1;
        }
    }

    Ok(new_resource_count)
}

// Step 2: Increment counter by new resource count only
async fn increment_resource_counter(
    account_id: &str,
    new_resource_increment: i64
) -> Result<i64> {
    let db = accounts_db().await?;

    let new_count: Option<i64> = db
        .query(
            "UPSERT $counter_id SET
                resource_count = (resource_count ?? 0) + $increment,
                last_updated_at = time::now()
             RETURN resource_count"
        )
        .bind(("counter_id", format!("usage:{}", account_id)))
        .bind(("increment", new_resource_increment))
        .await?
        .take(0)?;

    Ok(new_count.unwrap_or(0))
}
```

**RETURN BEFORE Logic:**
- If `BEFORE` is `null` → new resource was created → count it
- If `BEFORE` has data → existing resource was updated → don't count
- Ensures `resource_count` = unique resources, not total upserts
- Critical for accurate limit enforcement

**Hourly Event Counter with Hour Segregation (Multi-Hour Support):**

```rust
async fn check_and_increment_event_counters(
    account_id: &str,
    events: Vec<Event>,
    plan: &Plan,
) -> Result<()> {
    let db = accounts_db().await?;

    // Step 1: Segregate events by hour from last_seen_at timestamps
    let mut events_by_hour: HashMap<String, Vec<Event>> = HashMap::new();
    for event in events {
        let hour = event.last_seen_at.format("%Y-%m-%dT%H").to_string();
        events_by_hour.entry(hour).or_default().push(event);
    }

    // Step 2: Check limits and update counter for each hour segment
    for (hour, events_in_hour) in events_by_hour {
        let counter: Option<Counter> = db
            .query("SELECT * FROM $counter_id LIMIT 1")
            .bind(("counter_id", format!("usage:{}", account_id)))
            .await?
            .take(0)?;

        // Determine current count for this hour
        let current_count = match counter {
            Some(c) if c.event_hour_window == Some(hour.clone()) => c.event_count_this_hour,
            _ => 0,  // Different hour or no counter, start fresh
        };

        // Check if adding these events would exceed limit
        if let Some(max) = plan.max_events_per_hour {
            if current_count + events_in_hour.len() as i64 > max {
                return Err(RateLimitError::EventLimitExceeded {
                    hour,
                    current: current_count,
                    attempted: events_in_hour.len() as i64,
                    limit: max,
                });
            }
        }

        // Step 3: Update counter for this hour
        db.query(
            "LET $counter = (SELECT * FROM $counter_id LIMIT 1)[0];
             IF $counter IS NONE OR $counter.event_hour_window != $hour_param THEN
                UPSERT $counter_id SET
                    event_hour_window = $hour_param,
                    event_count_this_hour = $count,
                    last_updated_at = time::now()
             ELSE
                UPSERT $counter_id SET
                    event_count_this_hour += $count,
                    last_updated_at = time::now()
             END;"
        )
        .bind(("counter_id", format!("usage:{}", account_id)))
        .bind(("hour_param", hour))
        .bind(("count", events_in_hour.len() as i64))
        .await?;
    }

    Ok(())
}
```

**Hour Segregation Logic (Handles Multi-Hour Requests):**
- Segregate events by hour extracted from `last_seen_at` (occurrence time)
- Check and enforce limits for each hour segment independently
- **Handles events spanning multiple hour blocks** in single request
- Prevents issues with requests crossing hour boundaries
- More accurate limit enforcement per clock hour
- Events reported late still count against their occurrence hour

**Pre-Ingestion Limit Check Flow (Granular):**

```rust
// 1. Pre-flight limit check (read-only, no transaction)
let plan_limits = fetch_plan_limits(&account).await?;
let new_resource_count = count_unique_resources(&req);
let new_event_count = count_total_events(&req);

let limit_check = check_rate_limits(
    account.id(),
    &plan_limits,
    new_resource_count,
    new_event_count
).await?;

// Separate rejection logic for resources vs events
if !limit_check.resources_ok {
    // Resource limit exceeded - filter out resource_captures, keep event_captures
    req.resource_captures.clear();
    // Continue processing events if event limit OK
}

if !limit_check.events_ok {
    // Event limit exceeded - filter out event_captures, keep resource_captures
    req.event_captures.clear();
    // Continue processing resources if resource limit OK
}

if req.resource_captures.is_empty() && req.event_captures.is_empty() {
    return Err(RateLimitExceeded::Both);
}

// 2. Proceed with ingestion transaction (partial request if needed)
// 3. Update counters in accounts DB (separate connection)

// 4. Return response to agent with limit breach notification (for self-regulation)
let response = ReportResponse {
    accepted: true,
    resources_limited: !limit_check.resources_ok,
    events_limited: !limit_check.events_ok,
    message: build_limit_message(&limit_check),
};

return Ok(response);
```

**Agent Notification for Self-Regulation:**

After ingestion completes (successfully or partially), the backend must inform the agent which limits were breached:

```rust
#[derive(Debug, Serialize)]
pub struct ReportResponse {
    /// Whether the report was accepted (even if partially)
    pub accepted: bool,

    /// True if resource limit was hit (resource_captures were dropped)
    pub resources_limited: bool,

    /// True if event limit was hit (event_captures were dropped)
    pub events_limited: bool,

    /// Human-readable message describing limit status
    pub message: String,
}

fn build_limit_message(limit_check: &LimitCheck) -> String {
    match (limit_check.resources_ok, limit_check.events_ok) {
        (true, true) => "Report accepted".to_string(),
        (false, true) => "Resource limit exceeded - only events ingested".to_string(),
        (true, false) => "Event limit exceeded - only resources ingested".to_string(),
        (false, false) => "Both limits exceeded - report rejected".to_string(),
    }
}
```

**Agent Self-Regulation:**

Agents use the response flags to adjust their behavior:

```rust
// In agent after receiving response
if response.resources_limited {
    warn!("Resource limit hit - throttling resource discovery");
    // Reduce resource polling frequency
    // Prioritize events for existing resources
}

if response.events_limited {
    warn!("Event limit hit - reducing event reporting rate");
    // Increase batching interval
    // Apply event sampling/filtering
}
```

**Why Agent Notification Matters:**
- Enables cooperative limiting (agents self-regulate before hitting limits)
- Better user experience (agents adapt instead of silently failing)
- Reduces backend enforcement burden (fewer rejected requests)
- Provides visibility into limit status for debugging

**Why Granular Checks Matter:**
- Resource limit hit: Still accept events for existing resources
- Event limit hit: Still accept new resource discoveries
- Better UX: Partial functionality vs total rejection

**Performance Characteristics:**

| Operation | Mechanism | Complexity | Expected Time |
|-----------|-----------|------------|---------------|
| Check resource count | Direct record lookup by ID | O(1) | <1ms |
| Check event count | Direct record lookup + hour comparison | O(1) | <2ms |
| Increment resource counter | UPSERT with += operator | O(1) | <2ms |
| Increment event counter | UPSERT with hour window logic | O(1) | <3ms |
| **Total overhead per ingestion** | Pre-check + 2x counter updates | - | **<10ms** ✅ |

### Alternatives Considered

**Alternative 1: Embedded Counter Fields in Account Table**
- **Rejected**: Account table is in accounts DB, resources are in separate per-account DB; would require cross-database updates on every insert; mixes concerns with audit fields

**Alternative 2: COUNT INDEX (when available in v3.0)**
- **Not viable**: COUNT indexes introduced in v3.0 (not in stable 2.x); cannot handle hourly event windows; future migration path possible

**Alternative 3: Periodic Background Counter Sync**
- **Rejected**: Race conditions during concurrent ingestion; counter loss on restart; eventual consistency violates enforcement requirements

**Alternative 4: Approximate Counting with HyperLogLog**
- **Rejected**: Spec requires exact counts (SC-001, SC-002); approximation could allow bypass or false rejections

---

## 2. Self-Hosted Authentication Pattern

### Decision

Use **Enhanced Shared Secret (Option A+)** - Account ID + cryptographically-secure shared secret with AES-GCM authenticated encryption, following the same proven pattern as report API keys.

### Rationale

**Context-Appropriate Security:**
- This is machine-to-machine (M2M) authentication for config fetching, not user authentication
- Industry precedents: GitLab activation codes, HashiCorp Vault AppRole, Kubernetes bootstrap tokens all use shared secrets for M2M

**Operational Simplicity for Stealth-Stage Startup:**
- Zero infrastructure overhead (no OAuth server, no token refresh logic, no JWKS endpoints)
- Simple integration for self-hosted operators (paste one credential during setup)
- Predictable behavior (no token expiration failures during critical operations)
- Easy debugging (clear authentication failures)

**Existing Architecture Alignment:**
- Codebase already implements sophisticated API key pattern for agent authentication
- Uses AES-GCM authenticated encryption with account-specific salts
- Same proven pattern can secure self-hosted→central auth
- Reuses existing crypto infrastructure

**Security Enhancements Over Simple Shared Secret:**
1. Authenticated Encryption (AES-GCM): Prevents tampering, guarantees authenticity
2. Account-Specific Binding: Secret only valid for one account (via AAD)
3. Purpose Binding: Secret scoped to "self-hosted-config-fetch" only
4. Format Versioning: Protocol can evolve without breaking changes
5. Revocation Support: Instant invalidation via database flag
6. Multiple Active Secrets: Zero-downtime rotation (grace period)

### Implementation Approach

**Credential Structure (Similar to Report API Keys):**

```protobuf
message SelfHostedCredential {
  uint32 version = 1;  // Always 1 initially
  fixed64 account_id = 2;
  bytes credential_nonce = 3;  // 12 bytes for AES-GCM
  bytes encrypted_secret = 4;  // Contains account_id + random bytes
}

message SelfHostedCredentialAAD {
  fixed64 account_id = 1;
  string purpose = 2;  // Always "self-hosted-config-fetch"
}
```

**Credential Format:** `archodex_selfhosted_{account_id}_{base64_protobuf}`

**API Flow:**

```bash
# 1. Backend auto-generates credential during account creation
# (Credential displayed once in UI, like report API keys)

# 2. User regenerates credential if needed (self-service)
POST /api/accounts/{account_id}/self-hosted-credentials/regenerate
Authorization: Bearer {user_cognito_token}
Response: {
  "account_id": "1234567890",
  "secret_key": "archodex_selfhosted_1234567890_<base64>",
  "created_at": "2025-10-10T12:00:00Z",
  "warning": "Previous credential has been invalidated"
}

# 3. Self-hosted backend fetches plan limits
GET /api/v1/self-hosted/plan-limits
Authorization: Bearer archodex_selfhosted_1234567890_<base64>

Response: {
  "account_id": "1234567890",
  "plan": {
    "max_resources": 500,
    "max_events_per_hour": 1000,
    "update_frequency_seconds": 1200
  },
  "cached_until": "2025-10-11T12:00:00Z"  // 24 hour cache hint
}
```

**Server-Side Validation (archodex.com):**

```rust
async fn validate_self_hosted_credential(credential: &str) -> Result<AccountId> {
    // 1. Parse format: "archodex_selfhosted_{account_id}_{base64_payload}"
    let (account_id, payload) = parse_credential(credential)?;

    // 2. Decrypt using archodex.com's private key (from KMS)
    let cipher = Aes128Gcm::new(api_private_key().await);
    let decrypted = cipher.decrypt(
        &nonce,
        Payload {
            msg: &encrypted_secret,
            aad: &build_aad(account_id, "self-hosted-config-fetch"),
        },
    )?;

    // 3. Verify account_id matches
    ensure!(decrypted.account_id == account_id, "Account ID mismatch");

    // 4. Check if credential is revoked
    ensure!(!is_revoked(account_id, credential_id).await?, "Credential revoked");

    Ok(account_id)
}
```

**Client-Side (Self-Hosted Backend):**

```rust
// Environment variable: ARCHODEX_SELF_HOSTED_CREDENTIAL
async fn fetch_plan_limits() -> Result<PlanLimits> {
    let credential = env::var("ARCHODEX_SELF_HOSTED_CREDENTIAL")
        .context("Missing ARCHODEX_SELF_HOSTED_CREDENTIAL")?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/self-hosted/plan-limits", ARCHODEX_COM_URL))
        .header("Authorization", format!("Bearer {}", credential))
        .send()
        .await?;

    let plan_data: PlanLimitsResponse = response.json().await?;

    // Cache with 24-hour TTL (or until cached_until timestamp)
    cache_plan_limits(plan_data).await?;

    Ok(plan_data.plan)
}
```

**Credential Lifecycle Management:**

```rust
// Single active credential per account (simplest approach)
struct SelfHostedCredential {
    account_id: u64,
    credential_id: String,
    created_at: DateTime<Utc>,
    revoked: bool,
}

// Auto-creation workflow:
// 1. Account created → Backend automatically generates credential
// 2. Credential displayed once to user in dashboard
// 3. User copies and sets in self-hosted environment variable

// Regeneration workflow:
// 1. User requests regeneration via API (self-service)
// 2. Previous credential immediately invalidated
// 3. New credential displayed once (must be copied immediately)
// 4. Self-hosted operator updates env var at their convenience
```

**Bootstrap/Initial Secret Distribution:**

1. Account creation in dashboard → auto-generate credential
2. Display credential **once** in UI (like report API key)
3. Self-hosted operator sets environment variable:
   ```bash
   export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_..."
   ./archodex-backend  # Automatically fetches limits on startup
   ```

### Alternatives Considered

**Option B: Full OAuth/JWT**
- **Pros**: Industry standard, rich token metadata
- **Cons**: Massive complexity (auth server, token refresh, JWKS), 3-5x longer implementation, operational burden (token expiration failures)
- **Verdict**: Overkill for M2M config fetch, benefits don't justify complexity

**Option C: Mutual TLS (mTLS)**
- **Pros**: Strong cryptographic auth, no bearer tokens
- **Cons**: Certificate management burden, complex setup, PKI infrastructure needed
- **Verdict**: Too operationally heavy for stealth-stage product

**Option D: Signed JWTs without OAuth**
- **Pros**: Simple JWT structure, self-verifiable
- **Cons**: Expiration handling, JWKS rotation logic, marginally better than enhanced shared secret
- **Verdict**: Adds complexity without significant security gain

**Security vs Complexity Analysis:**

| Approach | Security | Complexity | Operator Burden | Recommendation |
|----------|----------|------------|-----------------|----------------|
| **Option A+ (Enhanced Secret)** | ⭐⭐⭐⭐☆ (8/10) | ⭐⭐⭐⭐⭐ (9/10 simple) | Low | ✅ **Selected** |
| Option B (OAuth/JWT) | ⭐⭐⭐⭐⭐ (9/10) | ⭐⭐☆☆☆ (3/10 complex) | High | ❌ |
| Option C (mTLS) | ⭐⭐⭐⭐⭐ (10/10) | ⭐⭐☆☆☆ (2/10 complex) | Very High | ❌ |
| Option D (Simple JWT) | ⭐⭐⭐⭐☆ (8/10) | ⭐⭐⭐☆☆ (6/10) | Medium | ❌ |

---

## 3. Protobuf Extension for Plan Limits in API Keys

### Decision

Add `PlanLimits` message to **both** outer `ReportApiKey` message (plaintext for agent-side limiting) and `ReportApiKeyEncryptedContents` (encrypted for backend tamper detection). Use optional uint64 fields with "absent = unlimited" semantics. Keep version field at 1 (backward compatible extension).

### Rationale

**Dual Placement (Outer + Encrypted):**
- ✅ **Outer message (plaintext)**: Agents read for client-side limiting
- ✅ **Encrypted contents**: Backend verifies no tampering by comparing outer vs decrypted
- ✅ NOT in AAD: Simpler encoding, backward compatibility
- ✅ Tamper detection: If outer ≠ encrypted, backend rejects as tampered
- ✅ Security boundary: Backend always fetches authoritative limits from `plan` table (key limits are hints only)

**Simplified Unlimited Encoding (Absent = Unlimited):**
- Field absent (`optional` not set): Unlimited resources/events (Custom plan)
- Field present with value > 0: Specific limit (Team: 500, Organization: 5000)
- **No sentinel value needed**: Cleaner semantics using protobuf's optional feature
- No backward compatibility concerns (no active customers yet)

**Why "Absent = Unlimited" instead of "0 = Unlimited":**
- More intuitive semantics (absence naturally means "no limit")
- Avoids potential confusion with zero as a meaningful value
- Clearer in code and debugging
- Standard protobuf pattern for optional limits

**No Version Bump:**
- Adding optional fields is backward compatible
- Old agents ignore unknown protobuf fields
- Version bump reserved for breaking changes

### Updated Protobuf Schema

```protobuf
syntax = "proto3";

package archodex.report_api_key;

message ReportApiKey {
  uint32 version = 1; // Always 1
  optional string endpoint = 2;
  bytes account_salt = 3; // Always 16 bytes long
  bytes nonce = 4; // Always 12 bytes long for AES128-GCM
  bytes encrypted_contents = 5;
  PlanLimits plan_limits = 6; // NEW: Plan limits in plaintext (for agent-side limiting)
}

// Encrypted with AES128-GCM.
message ReportApiKeyEncryptedContents {
  fixed64 account_id = 1;
  PlanLimits plan_limits = 2; // NEW: Plan limits encrypted (for backend tamper detection)
}

message ReportApiKeyEncryptedAAD {
  fixed32 key_id = 1;
  optional string endpoint = 2;
  bytes account_salt = 3;
  // Note: plan_limits NOT in AAD (in outer message + encrypted contents instead)
}

// Plan limits transmitted to agents within the report API key
// Included in both outer message (agent-readable) and encrypted contents (tamper detection)
message PlanLimits {
  // Maximum number of resources this account can track
  // - absent (not set): Unlimited resources (Custom plan)
  // - >0: Specific limit (e.g., 500 for Team, 5000 for Organization)
  optional uint64 max_resources = 1;

  // Maximum number of events per hour this account can ingest
  // - absent (not set): Unlimited events (Custom plan)
  // - >0: Specific limit (e.g., 1000 for Team, 10000 for Organization)
  optional uint64 max_events_per_hour = 2;

  // Minimum seconds between agent updates to backend
  // Required field, valid range: 60-1200 seconds
  // Example: 1200 for Team (20 min), 60 for Organization/Custom (1 min)
  uint32 update_frequency_seconds = 3;
}
```

### Agent Interpretation

```rust
// Agent receives and decodes report API key
let key = ReportApiKey::decode(base64_decode(key_value))?;

// Read plan limits from outer message (plaintext, no decryption needed)
if let Some(limits) = &key.plan_limits {
    // Validate update frequency
    if limits.update_frequency_seconds < 60 || limits.update_frequency_seconds > 1200 {
        return Err(anyhow!("Invalid update_frequency in API key"));
    }

    // Interpret resource limits (absent = unlimited)
    let max_resources = match limits.max_resources {
        None => ResourceLimit::Unlimited,
        Some(n) if n > 0 => ResourceLimit::Limited(n),
        Some(0) => {
            warn!("max_resources=0 is invalid, treating as unlimited");
            ResourceLimit::Unlimited
        }
    };

    // Interpret event limits (absent = unlimited)
    let max_events = match limits.max_events_per_hour {
        None => EventLimit::Unlimited,
        Some(n) if n > 0 => EventLimit::Limited(n),
        Some(0) => {
            warn!("max_events_per_hour=0 is invalid, treating as unlimited");
            EventLimit::Unlimited
        }
    };

    // Configure agent behavior
    agent_config.set_update_interval(Duration::from_secs(limits.update_frequency_seconds));
    agent_config.set_resource_limit(max_resources);
    agent_config.set_event_limit(max_events);
} else {
    // Old key without plan_limits - proceed without client-side limiting
    warn!("Report API key does not contain plan limits");
}
```

### Backend Validation

```rust
// Backend validates incoming report with API key
pub async fn validate_and_extract_limits(
    report_api_key_value: &str,
) -> anyhow::Result<(String, u32, PlanLimits)> {
    // 1. Decode outer message to get plaintext plan limits
    let key = ReportApiKey::decode(base64_decode(key_value))?;
    let outer_limits = key.plan_limits.clone();

    // 2. Decrypt contents to verify tamper-proof limits
    let decrypted = decrypt_report_api_key(&key).await?;
    let inner_limits = decrypted.plan_limits.clone();

    // 3. Verify outer limits match encrypted limits (tamper detection)
    if outer_limits != inner_limits {
        return Err(anyhow!("Plan limits tampered - outer != encrypted"));
    }

    // 4. Extract account_id from decrypted contents
    let account_id = decrypted.account_id;

    // Return limits (for informational purposes only - backend fetches authoritative limits from plan table)
    Ok((account_id.to_string(), key.key_id, outer_limits))
}

// Rate limit enforcement (uses limits fetched from plan table, NOT from API key)
pub async fn enforce_limits(
    account_id: &str,
) -> EnforcementResult {
    // ALWAYS fetch authoritative limits from plan table
    let plan = fetch_active_plan(account_id).await?;

    // Check resource limit (absent = unlimited)
    if let Some(max_res) = plan.max_resources {
        let current = get_resource_count(account_id).await?;
        if current >= max_res {
            return EnforcementResult::ResourceLimitExceeded;
        }
    }

    // Check event limit (absent = unlimited)
    if let Some(max_events) = plan.max_events_per_hour {
        let current = get_event_count_this_hour(account_id).await?;
        if current >= max_events {
            return EnforcementResult::EventLimitExceeded;
        }
    }

    EnforcementResult::Allowed
}
```

### Backward Compatibility

**Compatibility Matrix:**

| Scenario | Result |
|----------|--------|
| New key, new agent, new backend | ✅ Full functionality (agent reads outer limits, backend verifies + enforces) |
| New key, old agent, new backend | ✅ Backend enforces server-side (agent unaware of limits) |

**Simplified Compatibility:**
- No backward compatibility concerns (no active customers yet)
- All new keys include `plan_limits` in both outer message and encrypted contents
- Old agents ignore unknown protobuf fields (graceful degradation)
- Backend always enforces limits server-side regardless of agent support

**Key Guarantees:**
1. Protobuf forward compatibility - old agents ignore unknown fields
2. Backend always fetches authoritative limits from `plan` table (security boundary)
3. Tamper detection via outer vs encrypted comparison

**Operational Deployment:**
```
Phase 1: Deploy new backend with rate limiting
- Backend generates keys with plan_limits in outer + encrypted
- All enforcement happens server-side initially

Phase 2: Deploy new agents (optional)
- Agents read outer plan_limits for client-side optimization
- Agents self-regulate based on embedded limits
- Server-side enforcement continues as fallback
```

### Alternatives Considered

**Alternative: Plan limits in AAD instead of outer message**
```protobuf
// REJECTED
message ReportApiKeyEncryptedAAD {
  fixed32 key_id = 1;
  optional string endpoint = 2;
  bytes account_salt = 3;
  optional PlanLimits plan_limits = 4; // In AAD
}
```
- **Rejected**: More complex AAD construction, no clear benefit over outer message approach
- Outer message + encrypted contents achieves same tamper detection with simpler encoding

**Alternative: Use 0 as sentinel for unlimited**
```protobuf
// REJECTED
message PlanLimits {
  optional uint64 max_resources = 1; // 0 = unlimited
}
```
- **Rejected**: Less intuitive (0 is confusing as "unlimited"), requires more documentation
- "Absent = unlimited" is cleaner and more standard protobuf pattern

**Alternative: Add boolean flags + separate limit fields**
```protobuf
// REJECTED
message PlanLimits {
  bool unlimited_resources = 1;
  optional uint64 max_resources = 2; // Only used if unlimited=false
}
```
- **Rejected**: Doubles field count, potential inconsistent states, more complex validation

**Alternative: Always-present fields with max uint64 for unlimited**
- **Rejected**: Less intuitive, could cause overflow bugs, harder to debug

---

## Summary

All major technical unknowns have been researched and decisions made:

1. **Counter Mechanism**: Dedicated `usage` table with atomic UPSERT operations, achieving <10ms overhead. Hour segregation for events handles multi-hour request batches. RETURN BEFORE logic ensures only NEW resources counted.

2. **Self-Hosted Auth**: Enhanced shared secret pattern (AES-GCM), auto-generated during account creation, with self-service regeneration. Single credential per account, balancing security and simplicity.

3. **Protobuf Extension**: Plan limits in **both** outer message (agent-readable plaintext) and encrypted contents (tamper detection). Absent = unlimited encoding. Backend always fetches authoritative limits from plan table.

These decisions provide a solid foundation for Phase 1 design artifacts (data-model.md, contracts/, quickstart.md).

---

## Supplemental Research (Post-Planning)

### 4. HyperLogLog for Approximate Counting - NOT RECOMMENDED

**Question**: Could HyperLogLog provide acceptable short-term performance for counting?

**Answer**: **NO** - Exact counters are superior in every dimension for this use case.

**Key Findings**:
- **Accuracy unacceptable**: At 500 resources (Team limit), HLL has ~10.6% false rejection rate and ~11.1% bypass rate with standard precision
- **Memory worse**: HLL uses 750× MORE memory (12 KB vs 16 bytes) at target scale
- **Performance no better**: Exact UPSERT already achieves <5ms (meets requirements)
- **Migration impossible**: No path from HLL back to exact counts without data loss

**Detailed Analysis**:
- Standard error at 500 resources: ±8-10 resources (2% relative error)
- At 495/500 limit: 10.6% probability of false rejection
- At 505/500 (over limit): 11.1% probability of bypass
- HLL only wins at >10M items where memory is constrained

**Recommendation**: Stick with exact counter table design. HLL provides no benefits and significant downsides at your scale (50-5000 resources).

---

### 5. SurrealDB v3 COUNT Index Migration Path - PARTIAL COMPATIBILITY

**Question**: How easy is migration to COUNT index when moving to SurrealDB v3?

**Answer**: **COUNT index solves resource counting but NOT hourly event counting** due to per-account namespaces.

**Updated Analysis (Per-Account Namespaces)**:

Your architecture uses per-account namespaces (`a{account_id}:resources`), which changes the COUNT index assessment:

✅ **Resource Counting**: COUNT index COULD work
- Each account namespace has its own `resource` table
- `DEFINE INDEX resource_count ON resource COUNT` per namespace
- O(1) reads via pre-computed count

❌ **Hourly Event Counting**: COUNT index CANNOT work
- No time-based filtering (cannot count "events in current hour")
- No conditional counting (WHERE clauses don't work with COUNT index)
- Known bugs in v3.0-alpha with COUNT WHERE operations

**Hybrid Approach Possible (Future)**:
```
Resource counting: v3 COUNT index (when stable)
Event counting: Custom hourly counter (keep usage table)
```

**Migration Complexity**: Medium
- Can add COUNT index alongside counter table
- Gradual transition for resource counts only
- Keep hourly counter logic indefinitely

**Recommendation**:
- **Now**: Use counter table for both resources and events (consistent, proven)
- **Future (v3 stable)**: Consider COUNT index for resources only, keep hourly counter
- **Timeline**: v3.0 stable likely 2026+, reassess then

**Key Constraint**: Time-based counting (hourly windows) requires custom solution regardless of SurrealDB version.

---

### 6. Self-Hosted Authentication - Single Backend Per Account

**Question**: How to enable self-service authentication for self-hosted backends?

**Answer**: **Enhanced shared secret with single credential per account** (abuse detection optional).

**Clarified Requirements**:
- ✅ One self-hosted backend per account (normal case)
- ⚠️ Multiple backends per account is ABUSE scenario (not a supported feature)
- ✅ Acceptable to have some risk if prevention is complex

**Recommended Approach**:
- One active credential per account at a time
- Auto-generated during account creation (self-service)
- Displayed once to user, stored in environment variable
- Regeneration invalidates previous credential
- Optional abuse detection: Alert on unusual usage patterns (multiple IPs, high frequency)

**Credential Structure**: `archodex_selfhosted_{credential_id}_{base64_protobuf}`
- AES-GCM authenticated encryption (tamper-proof)
- Account-bound (cannot use for different account)
- Purpose-bound ("self-hosted-plan-fetch")
- Revocable (instant invalidation)

**Implementation**: See detailed design documents for complete specification, API endpoints, and code examples.

---

These supplemental findings validate the core design decisions and provide guidance for future optimizations when SurrealDB v3 becomes stable.
