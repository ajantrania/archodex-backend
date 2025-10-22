# Quickstart: Account Plans & Rate Limiting

**Feature**: 001-rate-limits-we
**Audience**: Developers implementing this feature
**Prerequisites**: Familiarity with Rust, axum, SurrealDB, and the Archodex codebase

This guide provides a high-level overview of how to implement account plans and rate limiting.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Phase 1: Managed Service (MVP)](#phase-1-managed-service-mvp)
3. [Phase 2: Self-Hosted Extension](#phase-2-self-hosted-extension)
4. [Testing Strategy](#testing-strategy)
5. [Deployment Checklist](#deployment-checklist)

---

## Architecture Overview

### Three Backend Plan Tiers

- **Team**: 500 resources, 1000 events/hour, 20-minute update frequency (free tier)
- **Organization**: 5000 resources, 10000 events/hour, 1-minute update frequency (paid)
- **Custom**: Unlimited resources/events, 1-minute update frequency (paid)

**Note**: Stand Alone Mode (50 resources, 100 events/hour) is agent-only with no backend involvement.

### Key Components

```
┌─────────────────────┐
│  accounts DB        │
│  ┌──────────────┐   │
│  │ plan table   │   │  ← Plan configs (limits, audit trail)
│  └──────────────┘   │
│  ┌──────────────┐   │
│  │ usage        │   │  ← Efficient counters (no COUNT queries)
│  └──────────────┘   │
└─────────────────────┘

┌─────────────────────┐
│ src/rate_limits/    │  ← License-isolated rate limiting module
│  ├── plan.rs        │     (Plan CRUD, queries)
│  ├── counters.rs    │     (Counter management, hour reset logic)
│  ├── enforcement.rs │     (Limit checks, drop logic)
│  └── plan_fetch.rs  │     (Self-hosted plan fetching - Phase 2)
└─────────────────────┘

┌─────────────────────┐
│ report_api_key.proto│  ← Extended with PlanLimits in AAD
│  └── PlanLimits     │     (max_resources, max_events_per_hour, update_frequency)
└─────────────────────┘
```

---

## Phase 1: Managed Service (MVP)

Implement core rate limiting for archodex.com managed backends only. Self-hosted support comes in Phase 2.

### Step 1: Database Schema (Migration)

**File**: `migrator/src/migrations/m20251010_create_plan_table.rs`

Create migration that defines both `plan` and `usage` tables:

```surrealql
-- Plan table
DEFINE TABLE IF NOT EXISTS plan SCHEMAFULL TYPE NORMAL;
DEFINE FIELD IF NOT EXISTS account_id ON TABLE plan TYPE record<account> READONLY;
DEFINE FIELD IF NOT EXISTS name ON TABLE plan TYPE string;
DEFINE FIELD IF NOT EXISTS max_resources ON TABLE plan TYPE option<int>;
DEFINE FIELD IF NOT EXISTS max_events_per_hour ON TABLE plan TYPE option<int>;
DEFINE FIELD IF NOT EXISTS update_frequency_seconds ON TABLE plan TYPE int
  ASSERT $value >= 60 AND $value <= 1200;
DEFINE FIELD IF NOT EXISTS start ON TABLE plan TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS end ON TABLE plan TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE plan TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS created_by ON TABLE plan TYPE string;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE plan TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS updated_by ON TABLE plan TYPE option<record<user>>;
DEFINE INDEX IF NOT EXISTS plan_account_active_idx ON TABLE plan FIELDS account_id, start, end;

-- Usage counter table (multiple records per account)
DEFINE TABLE IF NOT EXISTS usage SCHEMAFULL TYPE NORMAL;
DEFINE FIELD IF NOT EXISTS account_id ON TABLE usage TYPE record<account> READONLY;
DEFINE FIELD IF NOT EXISTS resource_count ON TABLE usage TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS event_hour_window ON TABLE usage TYPE option<string>;
DEFINE FIELD IF NOT EXISTS event_count_this_hour ON TABLE usage TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS last_updated_at ON TABLE usage TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS usage_account_hour_idx ON TABLE usage FIELDS account_id, event_hour_window UNIQUE;
```

**Key Schema Changes**:
- Plan table: Added `start`/`end` fields for plan history, `name` field (not `plan_name`), changed `created_by` to string
- Usage table: Changed unique index to composite (account_id, event_hour_window) to support multiple hour records per account

Register migration in `migrator/src/lib.rs`.

---

### Step 2: Create Rate Limits Module

**File**: `src/rate_limits/mod.rs`

```rust
//! Rate limiting module (Fair Core License isolated)
//!
//! This module contains all rate limiting functionality to support
//! license restrictions.

pub mod plan;
pub mod counters;
pub mod enforcement;

pub use plan::{Plan, PlanQueries};
pub use counters::{Counter, get_counter_values, increment_resource_counter, check_and_increment_event_counters};
pub use enforcement::{check_rate_limits, RateLimitCheckResult, RateLimitError};
```

Add to `src/lib.rs`:
```rust
pub mod rate_limits;
```

---

### Step 3: Implement Plan Model

**File**: `src/rate_limits/plan.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::Connection;
use tracing::instrument;

use crate::{next_binding, user::User};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Plan {
    pub account_id: String,
    pub name: Option<String>,
    pub max_resources: Option<i64>,
    pub max_events_per_hour: Option<i64>,
    pub update_frequency_seconds: i32,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,  // String, not User (supports "system", "archodex-migrator", etc.)
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<User>,
}

pub trait PlanQueries<'r, C: Connection> {
    fn get_plan_by_account(self, account_id: &str) -> surrealdb::method::Query<'r, C>;
    fn create_plan_query(self, plan: &Plan, created_by: &User) -> surrealdb::method::Query<'r, C>;
    fn update_plan_query(self, account_id: &str, updates: PlanUpdates, updated_by: &User) -> surrealdb::method::Query<'r, C>;
}

// Implement query methods following existing patterns in codebase
// See src/report_api_key.rs and src/account.rs for examples
```

---

### Step 4: Implement Counter Management

**File**: `src/rate_limits/counters.rs`

Implement functions from data-model.md:
- `get_counter_values(account_id)` - Read resource counter from `usage:{account_id}:resources`
- `increment_resource_counter(account_id, increment)` - Atomic resource count update using RETURN BEFORE
- `check_and_increment_event_counters(account_id, events, plan)` - Multi-hour event counting with per-hour limit enforcement

**Key Design**:
- Resource counter: Single record `usage:{account_id}:resources` with `event_hour_window = NONE`
- Event counters: One record per hour `usage:{account_id}:{hour}` (e.g., `usage:1234567890:2025-10-10T14`)
- Query by composite index: `WHERE account_id = $account_id AND event_hour_window = $hour`

See **data-model.md** "Event Counter with Hour Segregation" section for complete implementation.

---

### Step 5: Implement Limit Enforcement

**File**: `src/rate_limits/enforcement.rs`

```rust
use tracing::instrument;

pub struct RateLimitCheckResult {
    pub resources_ok: bool,
    pub events_ok: bool,
    pub current_resource_count: i64,
    pub current_event_count: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Resource limit exceeded: {current}/{limit} resources")]
    ResourceLimitExceeded { current: i64, limit: i64 },

    #[error("Event rate limit exceeded: {current}/{limit} events this hour")]
    EventLimitExceeded { current: i64, limit: i64 },
}

#[instrument(err)]
pub async fn check_rate_limits(
    account_id: &str,
    plan: &Plan,
    new_resource_count: usize,
    new_event_count: usize,
) -> Result<RateLimitCheckResult> {
    // 1. Get current counter values
    let counters = get_counter_values(account_id).await?;

    // 2. Check resource limit
    let resources_ok = plan.max_resources
        .map(|max| counters.resource_count + new_resource_count as i64 <= max)
        .unwrap_or(true); // null = unlimited

    // 3. Check event limit
    let events_ok = plan.max_events_per_hour
        .map(|max| counters.event_count_this_hour + new_event_count as i64 <= max)
        .unwrap_or(true); // null = unlimited

    Ok(RateLimitCheckResult {
        resources_ok,
        events_ok,
        current_resource_count: counters.resource_count,
        current_event_count: counters.event_count_this_hour,
    })
}
```

---

### Step 6: Integrate with Report Ingestion

**File**: `src/report.rs` (modified)

```rust
use crate::rate_limits::{check_rate_limits, increment_resource_counter, increment_event_counter};

#[instrument(err, skip(account))]
pub(crate) async fn report(
    Extension(account): Extension<Account>,
    Json(req): Json<Request>,
) -> Result<()> {
    // NEW: Pre-flight rate limit check
    let plan = fetch_plan_for_account(&account).await?;
    let new_resource_count = count_unique_resources(&req);
    let new_event_count = count_total_events(&req);

    let limit_check = check_rate_limits(
        account.id(),
        &plan,
        new_resource_count,
        new_event_count,
    ).await?;

    if !limit_check.resources_ok {
        return Err(RateLimitError::ResourceLimitExceeded {
            current: limit_check.current_resource_count,
            limit: plan.max_resources.unwrap(),
        }.into());
    }

    if !limit_check.events_ok {
        return Err(RateLimitError::EventLimitExceeded {
            current: limit_check.current_event_count,
            limit: plan.max_events_per_hour.unwrap(),
        }.into());
    }

    // NOTE: Agents receive plan limits in their report API keys (plaintext).
    // When backend returns rate limit errors, agents can self-regulate:
    // - Reduce update frequency
    // - Filter out low-priority events
    // - Alert user to upgrade plan

    // Existing transaction logic for resource/event ingestion
    let db = account.resources_db().await?;
    let mut query = db.query(BeginStatement::default());
    // ... existing resource/event upsert logic ...
    query = query.query(CommitStatement::default());
    query.await?.check_first_real_error()?;

    // NEW: Update counters after successful ingestion
    increment_resource_counter(account.id(), new_resource_count as i64).await?;
    increment_event_counter(account.id(), new_event_count as i64).await?;

    Ok(())
}
```

---

### Step 7: Extend Report API Key Protobuf

**File**: `src/report_api_key.proto` (modified)

Add PlanLimits message to both outer message and encrypted contents:

```protobuf
message ReportApiKey {
  uint32 version = 1;
  optional string endpoint = 2;
  bytes account_salt = 3;
  bytes nonce = 4;
  bytes encrypted_contents = 5;
  PlanLimits plan_limits = 6; // NEW: Plaintext for agent-side limiting
}

message ReportApiKeyEncryptedContents {
  fixed64 account_id = 1;
  PlanLimits plan_limits = 2; // NEW: Encrypted for backend tamper detection
}

message PlanLimits {
  optional uint64 max_resources = 1;       // Absent = unlimited
  optional uint64 max_events_per_hour = 2; // Absent = unlimited
  uint32 update_frequency_seconds = 3;     // Required, 60-1200
}
```

**Encoding**: "Absent = unlimited" (not "0 = unlimited"). Use protobuf optional fields.

**File**: `src/report_api_key.rs` (modified)

Update `generate_value()` to include plan limits in both outer message and encrypted contents:

```rust
pub(crate) async fn generate_value(
    &self,
    account_id: &str,
    account_salt: Vec<u8>,
    plan: &Plan, // NEW parameter
) -> anyhow::Result<String> {
    // ... existing nonce/cipher setup ...

    let plan_limits = proto::PlanLimits {
        max_resources: plan.max_resources.map(|v| v as u64),
        max_events_per_hour: plan.max_events_per_hour.map(|v| v as u64),
        update_frequency_seconds: plan.update_frequency_seconds as u32,
    };

    // Add to encrypted contents
    let contents = proto::ReportApiKeyEncryptedContents {
        account_id: account_id.parse()?,
        plan_limits: Some(plan_limits.clone()),
    };

    // Encrypt with AAD (existing AAD structure unchanged)
    let encrypted = cipher.encrypt(&nonce, Payload {
        msg: &contents.encode_to_vec(),
        aad: &aad.encode_to_vec(),
    })?;

    // Build outer message with plaintext plan_limits
    let api_key = proto::ReportApiKey {
        version: 1,
        endpoint: /* ... */,
        account_salt,
        nonce: nonce.to_vec(),
        encrypted_contents: encrypted,
        plan_limits: Some(plan_limits), // NEW: Plaintext for agent
    };

    // ... encode and return ...
}
```

---

### Step 8: Plan Management (Direct Database Access)

**Note**: Plan management is handled via direct database manipulation by Archodex employees (no API endpoints for MVP).

**Creating a plan** (via database query):
```surrealql
CREATE plan CONTENT {
    account_id: account:{account_id},
    name: "Team",
    max_resources: 500,
    max_events_per_hour: 1000,
    update_frequency_seconds: 1200,
    start: time::now(),
    end: NONE,
    created_by: "employee-{user_id}"
};
```

**Query active plan**:
```surrealql
SELECT * FROM plan
WHERE account_id = account:{account_id}
AND start <= time::now()
AND (end IS NONE OR end > time::now())
LIMIT 1;
```

**Updating a plan** (creates new record, ends old one):
```surrealql
-- 1. End old plan
UPDATE plan:{old_plan_id} SET end = time::now();

-- 2. Create new plan
CREATE plan CONTENT {
    account_id: account:{account_id},
    name: "Organization",
    max_resources: 5000,
    max_events_per_hour: 10000,
    update_frequency_seconds: 60,
    start: time::now(),
    end: NONE,
    created_by: "employee-{user_id}"
};
```

Employee access to production database required. Admin API endpoints are out of scope for this iteration.

---

## Phase 2: Self-Hosted Extension

After Phase 1 is validated on archodex.com managed service, extend to support self-hosted backends.

### Step 1: Self-Hosted Credential Generation

⚠️ **NOTE**: Self-hosted authentication design is WIP and deferred to Phase 2 implementation. See `self-hosted-auth-summary.md` for current design status.

**File**: `src/rate_limits/self_hosted_credential.rs` (new)

Implement credential structure following enhanced shared secret pattern:
- Protobuf schema for SelfHostedCredential
- AES-GCM encryption with AAD
- Format: `archodex_selfhosted_{credential_id}_{base64}`

See **self-hosted-auth-summary.md** for complete design (marked as WIP).

---

### Step 2: Plan Fetch Endpoint (archodex.com)

**File**: `src/rate_limits/plan_fetch.rs` (new)

```rust
#[instrument(err)]
pub async fn fetch_plan_limits_handler(
    Path(account_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<PlanLimitsResponse>> {
    // 1. Detect auth type and validate
    let auth_header = extract_bearer_token(&headers)?;

    if auth_header.starts_with("archodex_selfhosted_") {
        // Self-hosted credential authentication
        let validated_account_id = validate_self_hosted_credential(&auth_header).await?;
        ensure!(validated_account_id == account_id, "Account ID mismatch");
    } else {
        // Cognito JWT authentication (for frontend)
        let claims = validate_cognito_jwt(&auth_header).await?;
        ensure!(claims.has_account_access(&account_id), "Unauthorized");
    }

    // 2. Fetch active plan from database
    let plan = get_active_plan_by_account(&account_id).await?;

    // 3. Calculate cache hint (current_time + 60 min)
    let cached_until = Utc::now() + Duration::hours(1);

    Ok(Json(PlanLimitsResponse {
        account_id,
        plan: plan.into(),
        cached_until,
    }))
}
```

Add route to `src/router.rs`:
```rust
.route("/{account_id}/plan", get(fetch_plan_limits_handler))
```

**Dual Authentication**: Route supports both self-hosted credentials and Cognito JWT (for frontend dashboard).

---

### Step 3: Plan Fetch Client (Self-Hosted Backend)

**File**: `src/rate_limits/plan_fetch_client.rs` (new)

```rust
pub struct PlanFetchClient {
    credential: String,
    archodex_com_url: String,
    fetch_interval: Duration,
    cache: Arc<RwLock<CachedPlan>>,
}

struct CachedPlan {
    plan: Plan,
    fetched_at: DateTime<Utc>,
}

impl PlanFetchClient {
    pub async fn new() -> Result<Self> {
        let credential = env::var("ARCHODEX_SELF_HOSTED_CREDENTIAL")?;
        let client = Self {
            credential,
            archodex_com_url: env::var("ARCHODEX_COM_URL")
                .unwrap_or_else(|_| "https://api.archodex.com".to_string()),
            fetch_interval: Duration::seconds(
                env::var("PLAN_FETCH_INTERVAL_SECONDS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()?
            ),
            cache: Arc::new(RwLock::new(None)),
        };

        // Initial fetch (blocking)
        client.fetch_and_cache().await?;

        Ok(client)
    }

    #[instrument(err)]
    async fn fetch_and_cache(&self) -> Result<()> {
        // Extract account_id from credential for route parameter
        let account_id = extract_account_id_from_credential(&self.credential)?;

        let response = reqwest::Client::new()
            .get(format!("{}/{}/plan", self.archodex_com_url, account_id))
            .header("Authorization", format!("Bearer {}", self.credential))
            .send()
            .await?
            .json::<PlanLimitsResponse>()
            .await?;

        let mut cache = self.cache.write().await;
        *cache = Some(CachedPlan {
            plan: response.plan.into(),
            fetched_at: Utc::now(),
        });

        Ok(())
    }

    pub async fn get_plan(&self) -> Result<Plan> {
        let cache = self.cache.read().await;
        let cached = cache.as_ref().ok_or(anyhow!("No cached plan"))?;

        // Check cache age
        let age = Utc::now() - cached.fetched_at;
        if age > Duration::days(3) {
            bail!("Cached plan expired (> 3 days old). Cannot enforce limits.");
        }

        if age > Duration::hours(1) {
            warn!("Using stale cached plan (age: {})", age);
        }

        Ok(cached.plan.clone())
    }

    pub async fn start_background_refresh(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.fetch_interval.to_std().unwrap());
            loop {
                interval.tick().await;
                if let Err(e) = self.fetch_and_cache().await {
                    error!("Failed to refresh plan limits: {}", e);
                }
            }
        });
    }
}
```

---

### Step 4: Environment Configuration

Add to `src/env.rs`:

```rust
// Self-hosted only
#[cfg(not(feature = "archodex-com"))]
pub(crate) fn self_hosted_credential() -> Option<&'static str> {
    static CREDENTIAL: LazyLock<Option<String>> = LazyLock::new(|| {
        env::var("ARCHODEX_SELF_HOSTED_CREDENTIAL").ok()
    });
    CREDENTIAL.as_deref()
}
```

---

## Testing Strategy

### Unit Tests

1. **Counter logic**:
   - Hour window reset (14:59 → 15:00)
   - Atomic increment operations
   - Concurrent counter updates

2. **Limit enforcement**:
   - Resource limit checks (at limit, under limit, unlimited)
   - Event limit checks (at limit, hour reset)
   - Partial batch handling (request atomicity)

3. **Plan CRUD**:
   - Plan creation with validation
   - Plan updates (partial updates)
   - Audit trail metadata

### Integration Tests

1. **End-to-end limit enforcement**:
   - Create account with plan
   - Ingest resources until limit
   - Verify rejection behavior
   - Verify existing resources continue accepting events

2. **API key with plan limits**:
   - Generate key for account with plan
   - Decode protobuf and extract limits
   - Verify AAD tamper protection

3. **Self-hosted plan fetching** (Phase 2):
   - Mock archodex.com endpoint
   - Fetch limits with credential
   - Cache behavior and expiration
   - Background refresh

### Performance Tests

1. **Counter query performance**:
   - Measure counter read time with 10k resources
   - Must be <5ms (SC-009)

2. **Rate limit overhead**:
   - Measure end-to-end ingestion latency with rate limiting
   - Must add <10ms overhead (SC-008)

---

## Deployment Checklist

### Phase 1 (Managed Service)

- [ ] Run `cargo fmt` and `cargo clippy`
- [ ] Database migration tested on staging
- [ ] Unit tests passing (>80% coverage for rate_limits module)
- [ ] Integration tests passing
- [ ] Performance benchmarks meet SC-008 and SC-009
- [ ] Plan management API tested via Postman/curl
- [ ] Report API keys include plan limits in AAD
- [ ] Rate limit violations return explicit errors (SC-010)
- [ ] Structured logging with #[instrument] on all rate limit functions
- [ ] Update DATAMODEL.md with plan and account_counter tables
- [ ] Deploy to staging, smoke test core scenarios
- [ ] Deploy to production with feature flag (gradual rollout)

### Phase 2 (Self-Hosted Extension)

- [ ] Self-hosted credential generation endpoint tested
- [ ] Plan fetch endpoint tested with mock credentials
- [ ] Self-hosted backend startup with plan fetch tested
- [ ] Cache expiration logic tested (3-day limit)
- [ ] Background refresh tested (60-min interval)
- [ ] Documentation for self-hosted operators (environment variables, setup)
- [ ] Test archodex.com unavailability scenarios
- [ ] Deploy plan fetch endpoint to archodex.com
- [ ] Provide self-hosted test environment setup guide

---

## Key Files Reference

| Component | File Path | Purpose |
|-----------|-----------|---------|
| Migration | `migrator/src/migrations/m20251010_create_plan_table.rs` | Create plan and usage tables |
| Plan model | `src/rate_limits/plan.rs` | Plan struct, queries, CRUD |
| Counters | `src/rate_limits/counters.rs` | Multi-hour counter management |
| Enforcement | `src/rate_limits/enforcement.rs` | Limit checks, drop logic |
| Report ingestion | `src/report.rs` | Integration point for rate limiting |
| API key protobuf | `src/report_api_key.proto` | Extended with PlanLimits (outer + encrypted) |
| API key generation | `src/report_api_key.rs` | Include plan limits in both locations |
| Routes | `src/router.rs` | `/{account_id}/plan` endpoint (Phase 2) |
| Self-hosted fetch (Phase 2) | `src/rate_limits/plan_fetch.rs` | Plan fetch endpoint with dual auth |
| Self-hosted client (Phase 2) | `src/rate_limits/plan_fetch_client.rs` | Plan fetch client with caching |

---

## Additional Resources

- **Spec**: [spec.md](./spec.md) - Full feature specification
- **Data Model**: [data-model.md](./data-model.md) - Database schema details
- **Research**: [research.md](./research.md) - Technical decisions and alternatives
- **API Contracts**: [contracts/](./contracts/) - OpenAPI specifications

For questions, reference existing patterns in:
- `src/report_api_key.rs` - Protobuf + AES-GCM authentication
- `src/account.rs` - Model with audit trail
- `src/report.rs` - Transaction-based ingestion
