# Self-Hosted Authentication - Summary

**Feature**: 001-rate-limits-we (Phase 2)
**Date**: 2025-10-10
**Status**: ⚠️ **WIP - DEFERRED TO PHASE 2**

---

## ⚠️ WORK IN PROGRESS NOTICE

**This document is marked as WIP and deferred to Phase 2 implementation.**

**Current Status**:
- ✅ Design concepts outlined
- ⚠️ Implementation details pending Phase 2 work
- ⚠️ API routes may change (see contracts/plan-fetch.yaml for latest)
- ⚠️ Self-hosted auth is NOT included in Phase 1 MVP

**Phase 1 Focus**: Managed archodex.com service only (no self-hosted plan fetching)

**Phase 2 Scope**: Complete self-hosted authentication design and implementation

**See also**: plan.md for Phase 2 implementation timeline

---

## TL;DR

**Approach**: Enhanced Shared Secret with AES-GCM (single credential per account)

**Key Points**:
- ✅ Self-service credential generation during account creation
- ✅ One active credential per self-hosted account (normal use case)
- ✅ Credential stored in environment variable (`ARCHODEX_SELF_HOSTED_CREDENTIAL`)
- ✅ Tamper-proof via AES-GCM authenticated encryption
- ⚠️ Multiple backends per account = abuse scenario (acceptable risk)

---

## Requirements

### Functional
1. **Self-service**: No employee involvement in credential generation
2. **Restart-safe**: Backend can restart with same credential
3. **Single backend**: One credential per self-hosted account (not a multi-backend feature)
4. **Abuse tolerance**: Some risk acceptable (won't be paying customers anyway)

### Non-Functional
- Reuse existing AES-GCM pattern from report API keys
- Simple operator experience (one env var)
- Clear audit trail for security incidents
- Optional abuse detection (not blocking)

---

## Credential Structure

### Format
```
archodex_selfhosted_{credential_id}_{base64_protobuf}
```

Example:
```
archodex_selfhosted_123456_ChAKDjEyMzQ1Njc4OTAxMBIQa3J5cHRvZ3JhcGhpYw==
```

### Protobuf Schema

```protobuf
message SelfHostedCredential {
  uint32 version = 1;  // Always 1
  fixed64 account_id = 2;
  bytes nonce = 3;  // 12 bytes for AES-GCM
  bytes encrypted_contents = 4;
}

message SelfHostedCredentialEncryptedContents {
  fixed64 account_id = 1;
  fixed32 credential_id = 2;
  bytes secret_bytes = 3;  // 16 random bytes
}

message SelfHostedCredentialAAD {
  fixed64 account_id = 1;
  string purpose = 2;  // Always "self-hosted-plan-fetch"
  fixed32 credential_id = 3;
}
```

### Security Properties
- **Tamper-proof**: AES-GCM authenticated encryption with AAD
- **Account-bound**: Cannot use credential for different account
- **Purpose-bound**: Only valid for "self-hosted-plan-fetch" purpose
- **Revocable**: Instant invalidation via database flag

---

## User Flow

### 1. Account Creation (Auto-Generate)
```
User creates account → Backend auto-generates credential → Display once in UI → User copies
```

### 2. Self-Hosted Backend Setup
```bash
# User sets environment variable
export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_123456_..."

# Start backend
./archodex-backend

# Backend automatically:
# 1. Validates credential on startup
# 2. Fetches plan limits from archodex.com
# 3. Caches limits (72-hour expiration)
# 4. Begins serving with rate limiting enabled
```

### 3. Credential Lost/Compromised
```
User regenerates credential via dashboard → Old credential invalidated → Update env var → Restart backend
```

---

## Database Schema

### Table: `self_hosted_credential`

Located in **accounts database** (archodex.com):

```surrealql
DEFINE TABLE IF NOT EXISTS self_hosted_credential SCHEMAFULL TYPE NORMAL;
DEFINE FIELD IF NOT EXISTS credential_id ON TABLE self_hosted_credential TYPE int;
DEFINE FIELD IF NOT EXISTS account_id ON TABLE self_hosted_credential TYPE record<account> READONLY;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE self_hosted_credential TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS created_by ON TABLE self_hosted_credential TYPE record<user>;
DEFINE FIELD IF NOT EXISTS revoked_at ON TABLE self_hosted_credential TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS last_used_at ON TABLE self_hosted_credential TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS last_used_ip ON TABLE self_hosted_credential TYPE option<string>;

-- Unique credential ID (6-digit random)
DEFINE INDEX IF NOT EXISTS credential_id_idx ON TABLE self_hosted_credential FIELDS credential_id UNIQUE;

-- One active credential per account (enforce via unique index on non-revoked)
DEFINE INDEX IF NOT EXISTS active_credential_per_account_idx ON TABLE self_hosted_credential FIELDS account_id
  WHERE revoked_at IS NONE UNIQUE;
```

**Record ID**: `self_hosted_credential:{credential_id}`

**Key Constraints**:
- ✅ Unique credential_id (6-digit range: 100000-999999)
- ✅ One active (non-revoked) credential per account
- ✅ Revoked credentials remain in table for audit trail

---

## API Endpoints

### Archodex.com Endpoints

#### 1. Generate Credential (Internal - Auto-Called)

```
POST /api/internal/accounts/{account_id}/self-hosted-credential
Authorization: Internal service token

Response 201:
{
  "credential": "archodex_selfhosted_123456_...",
  "credential_id": 123456,
  "created_at": "2025-10-10T12:00:00Z"
}
```

**Called automatically during account creation**. User sees credential once in dashboard.

#### 2. Regenerate Credential (Dashboard API)

```
POST /api/v1/accounts/{account_id}/regenerate-self-hosted-credential
Authorization: Bearer {user_jwt_token}

Response 200:
{
  "credential": "archodex_selfhosted_789012_...",
  "credential_id": 789012,
  "created_at": "2025-10-10T14:30:00Z",
  "previous_credential_revoked": true
}
```

**User-initiated** when credential is lost or compromised. Invalidates previous credential.

#### 3. Fetch Plan Limits (Self-Hosted Backend)

```
GET /v1/self-hosted/plan-limits
Authorization: Bearer archodex_selfhosted_123456_...

Response 200:
{
  "account_id": "1234567890",
  "plan": {
    "plan_name": "Team",
    "max_resources": 500,
    "max_events_per_hour": 1000,
    "update_frequency_seconds": 1200
  },
  "cached_until": "2025-10-10T15:00:00Z"
}
```

---

## Abuse Detection (Optional)

### Scenario: Multiple Backends Per Account

**Normal**: One self-hosted backend per account
**Abuse**: User runs credential on 10+ backends to bypass limits

**Detection Mechanisms**:
1. **Last Used IP Tracking**: Record `last_used_ip` on each plan fetch
   - Alert if IP changes frequently (>5 IPs in 24 hours)
   - Not blocking (some legitimate multi-region use)

2. **Fetch Frequency Monitoring**: CloudWatch metrics on fetch rate
   - Alert if >10 fetches per hour per account
   - Rate limit to 1 fetch per minute per credential

3. **Credential Count Per Account**: Query metrics
   - Alert if account generates >3 credentials in 7 days
   - Could indicate credential sharing abuse

**Enforcement**: Manual review and account suspension if abuse confirmed. Acceptable risk per requirements.

---

## Implementation Checklist

### Phase 2.1: Credential Generation
- [ ] Database migration for `self_hosted_credential` table
- [ ] Credential generation logic (AES-GCM encryption)
- [ ] Auto-generate during account creation
- [ ] Dashboard UI: Display credential once with copy button
- [ ] Regeneration API endpoint
- [ ] Unit tests for encryption/decryption

### Phase 2.2: Plan Fetch Endpoint
- [ ] Validation middleware for self-hosted credentials
- [ ] GET /v1/self-hosted/plan-limits endpoint
- [ ] Update `last_used_at` and `last_used_ip` on successful fetch
- [ ] Error handling (revoked, invalid, expired)
- [ ] Integration tests

### Phase 2.3: Self-Hosted Backend Client
- [ ] Environment variable: `ARCHODEX_SELF_HOSTED_CREDENTIAL`
- [ ] Startup: Validate credential and fetch limits
- [ ] Background refresh (default: 1 hour)
- [ ] Cache with 72-hour expiration
- [ ] Error handling (unreachable, invalid credential)
- [ ] Structured logging

### Phase 2.4: Abuse Detection (Optional)
- [ ] CloudWatch metrics for fetch frequency
- [ ] Alert on unusual IP patterns
- [ ] Dashboard for employee review
- [ ] Rate limiting (1 fetch/min per credential)

---

## Security Analysis

### Threats & Mitigations

| Threat | Likelihood | Impact | Mitigation | Residual Risk |
|--------|------------|--------|------------|---------------|
| **Credential theft** | Medium | Medium | One-time display, env var storage, revocation | **Low** - Limited to plan metadata |
| **Credential tampering** | Low | None | AES-GCM authentication | **None** - Cryptographically impossible |
| **Account impersonation** | Low | None | Account ID encrypted, validated on decrypt | **None** |
| **Brute force** | Low | Low | 6-digit ID space, rate limiting | **Very Low** |
| **Abuse (multiple backends)** | Medium | Low | Detection via metrics, acceptable risk | **Acceptable** per requirements |
| **Credential sharing** | Medium | Low | Regeneration invalidates old, audit trail | **Acceptable** per requirements |

### What's Protected
- ✅ Plan metadata (limits, update frequency)
- ✅ License enforcement (can't bypass by modifying local DB)

### What's NOT Protected (Acceptable)
- ⚠️ If credential is leaked, attacker can fetch plan limits (not sensitive data)
- ⚠️ User could run multiple backends with same credential (acceptable abuse risk)
- ⚠️ User could share credential with others (acceptable abuse risk)

**Justification**: Per requirements, "some risk of abuse is acceptable (won't be paying customers anyway)". Focus is on preventing _accidental_ bypass and providing license enforcement, not preventing determined abuse.

---

## Code Examples

### Generate Credential (archodex.com)

```rust
#[instrument(err)]
pub async fn generate_self_hosted_credential(
    account_id: &str,
    created_by: &User,
) -> Result<(String, u32)> {
    let credential_id = rand::thread_rng().gen_range(100_000..=999_999);

    let contents = SelfHostedCredentialEncryptedContents {
        account_id: account_id.parse()?,
        credential_id,
        secret_bytes: rand::thread_rng().gen::<[u8; 16]>().to_vec(),
    };

    let aad = SelfHostedCredentialAAD {
        account_id: account_id.parse()?,
        purpose: "self-hosted-plan-fetch".to_string(),
        credential_id,
    };

    let cipher = Aes128Gcm::new(api_private_key().await);
    let nonce = Aes128Gcm::generate_nonce(&mut OsRng);

    let encrypted = cipher.encrypt(&nonce, Payload {
        msg: &contents.encode_to_vec(),
        aad: &aad.encode_to_vec(),
    })?;

    let proto = SelfHostedCredential {
        version: 1,
        account_id: account_id.parse()?,
        nonce: nonce.to_vec(),
        encrypted_contents: encrypted,
    };

    let credential_value = format!(
        "archodex_selfhosted_{}_{}",
        credential_id,
        BASE64_STANDARD.encode(proto.encode_to_vec())
    );

    // Store in database
    db.query("CREATE self_hosted_credential:{credential_id} CONTENT {
        credential_id: {credential_id},
        account_id: account:{account_id},
        created_by: {created_by}
    }")
    .await?;

    Ok((credential_value, credential_id))
}
```

### Validate Credential (archodex.com)

```rust
#[instrument(err, skip_all)]
pub async fn validate_self_hosted_credential(
    credential_value: &str,
) -> Result<(String, u32)> {
    // 1. Parse format
    let (credential_id, payload) = parse_credential_format(credential_value)?;

    // 2. Decrypt
    let proto = SelfHostedCredential::decode(BASE64_STANDARD.decode(payload)?)?;
    let cipher = Aes128Gcm::new(api_private_key().await);

    let aad = SelfHostedCredentialAAD {
        account_id: proto.account_id,
        purpose: "self-hosted-plan-fetch".to_string(),
        credential_id,
    };

    let decrypted = cipher.decrypt(
        &Nonce::from_slice(&proto.nonce),
        Payload {
            msg: &proto.encrypted_contents,
            aad: &aad.encode_to_vec(),
        },
    )?;

    let contents = SelfHostedCredentialEncryptedContents::decode(&*decrypted)?;

    // 3. Verify account_id matches
    ensure!(contents.account_id == proto.account_id, "Account ID mismatch");
    ensure!(contents.credential_id == credential_id, "Credential ID mismatch");

    // 4. Check if revoked
    let is_revoked: bool = db.query(
        "SELECT type::is::some(revoked_at) AS revoked
         FROM self_hosted_credential:{credential_id}"
    ).await?.take(0)?;

    ensure!(!is_revoked, "Credential has been revoked");

    // 5. Update last used
    db.query("UPDATE self_hosted_credential:{credential_id} SET
        last_used_at = time::now(),
        last_used_ip = {ip}
    ").await?;

    Ok((proto.account_id.to_string(), credential_id))
}
```

### Self-Hosted Backend (Fetch on Startup)

```rust
#[instrument(err)]
pub async fn startup_fetch_plan_limits() -> Result<PlanLimits> {
    let credential = env::var("ARCHODEX_SELF_HOSTED_CREDENTIAL")
        .context("Missing ARCHODEX_SELF_HOSTED_CREDENTIAL env var")?;

    let url = format!(
        "https://api.{}/v1/self-hosted/plan-limits",
        Env::archodex_domain()
    );

    let response = Client::new()
        .get(&url)
        .header("Authorization", format!("Bearer {}", credential))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .context("Failed to fetch plan limits from archodex.com")?;

    if !response.status().is_success() {
        let error_body = response.text().await?;
        bail!("Plan fetch failed: {}", error_body);
    }

    let data: PlanLimitsResponse = response.json().await?;

    // Cache with expiration
    PLAN_LIMITS_CACHE.write().await.set(
        data.plan.clone(),
        data.cached_until,
    );

    info!(
        "Fetched plan limits: max_resources={:?}, max_events_per_hour={:?}",
        data.plan.max_resources, data.plan.max_events_per_hour
    );

    Ok(data.plan)
}
```

---

## Timeline

**Total**: 2-3 weeks (Phase 2 only, after Phase 1 MVP complete)

| Week | Tasks | Deliverables |
|------|-------|--------------|
| **Week 1** | Database, credential generation, dashboard UI | Credential auto-generation working |
| **Week 2** | Plan fetch endpoint, self-hosted client | Self-hosted backend can fetch limits |
| **Week 3** | Testing, abuse detection, documentation | Phase 2 complete |

---

## Next Steps

1. **Approve design**: Confirm single-backend-per-account approach acceptable
2. **Phase 1 first**: Complete managed service MVP before Phase 2
3. **Implement Phase 2**: Follow 3-week timeline above
4. **Beta test**: Select self-hosted customers for testing
5. **Monitor**: Track abuse patterns, adjust detection if needed

---

## Related Documents

- **spec.md**: Feature requirements (FR-019 through FR-025)
- **plan.md**: Implementation plan and constitution check
- **contracts/plan-fetch.yaml**: OpenAPI spec for plan fetch endpoint
- **quickstart.md**: Phase 2 implementation guide

For detailed protobuf definitions, API specifications, and additional code examples, see the full self-hosted auth design documents generated during research phase.
