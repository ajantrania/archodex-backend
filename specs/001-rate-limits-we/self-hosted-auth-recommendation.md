# Self-Hosted Authentication: Executive Recommendation

**Date**: 2025-10-10
**Feature**: Rate Limiting (001-rate-limits-we)
**Decision Required**: Self-service authentication flow for self-hosted backends

---

## TL;DR

**Recommended Approach**: **Enhanced Shared Secret with AES-GCM Authenticated Encryption**

This approach provides:
- ✅ True self-service (no employee involvement)
- ✅ Multi-backend support (generate multiple credentials)
- ✅ Restart-safe (credential in environment variable)
- ✅ Abuse-tolerant (detection possible, acceptable risk)
- ✅ Proven security (reuses existing report API key pattern)
- ✅ Simple operations (one env var to configure)

**Implementation complexity**: Moderate (reuses existing crypto infrastructure)
**Timeline**: 4-6 weeks across 6 phases

---

## Problem Statement

Self-hosted Archodex backends need to fetch plan limits from archodex.com to enforce rate limiting. The authentication mechanism must be:

1. **Self-service**: Users can generate credentials without Archodex employee involvement
2. **Multi-backend support**: One account can have multiple backends (staging, production, etc.)
3. **Restart-safe**: Backends can restart without re-authentication
4. **Abuse-tolerant**: Some risk of abuse is acceptable (free tier users)

---

## Options Evaluated

### ❌ Option 1: Auto-Generate During Account Creation (Basic)

**Approach**: Generate credential automatically when account is created

**Pros**:
- Simple implementation
- Zero-friction onboarding

**Cons**:
- Multi-backend support unclear (generate 10 upfront? wasteful)
- Cannot add more backends later without manual step

**Verdict**: Insufficient for multi-backend requirement

---

### ❌ Option 2: Backend Self-Registration

**Approach**: Backend generates credential, calls archodex.com to register, user approves with code

**Pros**:
- Truly automatic (backend-initiated)
- Each backend gets unique credential

**Cons**:
- Complex UX (user must enter approval code)
- Still needs shared secret to prove account ownership (defeats purpose)
- Not restart-safe by default
- Race conditions with multiple backends

**Verdict**: Over-engineered, doesn't solve the core distribution problem

---

### ❌ Option 3: Account-Wide Shared Secret (Basic)

**Approach**: One shared secret for all backends, user generates via dashboard

**Pros**:
- Simple to implement
- Simple to configure (one secret everywhere)

**Cons**:
- Security concern: If one backend compromised, all are compromised
- No granular revocation (must update all backends)
- No audit trail (can't distinguish which backend made which request)

**Verdict**: Poor security posture, no multi-backend benefits

---

### ✅ Option 3 Enhanced: Shared Secret with AES-GCM (RECOMMENDED)

**Approach**: Cryptographically-protected credentials with account binding, multiple credentials per account

**Pros**:
- ✅ Self-service credential generation
- ✅ Multi-backend support (generate multiple credentials)
- ✅ Restart-safe (credential in env var)
- ✅ Tamper-proof (AES-GCM authenticated encryption)
- ✅ Audit trail (credential_id, last_used_at tracking)
- ✅ Granular revocation (revoke individual credentials)
- ✅ Aligns with existing architecture (report API keys use same pattern)

**Cons**:
- Long-lived credentials (no expiration by default)
- User must securely store credential
- If leaked, attacker can fetch plan limits until revoked

**Verdict**: Best balance of security, simplicity, and functionality

---

### ❌ Option 4: Backend-Specific Registration Tokens

**Approach**: User generates short-lived token, backend exchanges for long-lived credential

**Pros**:
- Granular control (one token per backend)
- Short-lived tokens reduce leak risk

**Cons**:
- Complex token lifecycle (generation, exchange, storage)
- UX friction (generate token before starting backend)
- Not restart-safe by default (must persist exchanged credential)
- Over-engineered (no significant benefit over enhanced shared secret)

**Verdict**: Too complex, minimal benefit over recommended approach

---

## Detailed Design: Enhanced Shared Secret

### Architecture Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                         Account Creation                             │
│  User → Dashboard → API → Auto-generate credential → Display once    │
└──────────────────────────────────────────────────────────────────────┘
                                    ↓
┌──────────────────────────────────────────────────────────────────────┐
│                    Self-Hosted Configuration                         │
│  User → Set ARCHODEX_SELF_HOSTED_CREDENTIAL env var → Start backend  │
└──────────────────────────────────────────────────────────────────────┘
                                    ↓
┌──────────────────────────────────────────────────────────────────────┐
│                      Plan Limit Fetching                             │
│  Backend → GET /api/v1/self-hosted/plan-limits → Validate credential │
│  → Check revocation → Return limits → Cache 72 hours                 │
└──────────────────────────────────────────────────────────────────────┘
                                    ↓
┌──────────────────────────────────────────────────────────────────────┐
│                     Rate Limiting Enforcement                        │
│  Agent → Send data → Backend checks limits → Enforce limits → Ingest │
└──────────────────────────────────────────────────────────────────────┘
```

### Credential Format

**String**: `archodex_selfhosted_{credential_id}_{base64_protobuf}`

**Example**: `archodex_selfhosted_123456_dGVzdGRhdGE=...`

**Structure**:
- **Version**: Protocol version (currently 1)
- **Account ID**: Encrypted in payload
- **Credential ID**: Random 6-digit ID (100,000-999,999)
- **Nonce**: 12 bytes for AES-GCM
- **Encrypted Contents**: Account ID + credential ID + random bytes
- **AAD** (Additional Authenticated Data): Account ID + purpose + credential ID

**Security Properties**:
- **Tamper-proof**: AES-GCM prevents modification
- **Account-bound**: Credential only valid for specific account
- **Purpose-bound**: Scoped to "self-hosted-plan-fetch" only
- **Revocable**: Instant invalidation via database flag

### Database Schema

**New table**: `self_hosted_credential`

**Fields**:
- `account_id`: Link to account
- `credential_id`: Unique 6-digit ID
- `description`: Optional label (e.g., "Production backend")
- `created_at`, `created_by`: Audit trail
- `last_used_at`: Usage tracking
- `revoked_at`, `revoked_by`: Revocation metadata

**Indexes**:
- Unique index on `credential_id` (fast lookups)
- Index on `account_id` (list all credentials for account)

### API Endpoints

#### 1. Generate Credential (Self-Service)

**Endpoint**: `POST /api/accounts/{account_id}/self-hosted-credentials`

**Auth**: AWS Cognito JWT (user must have access to account)

**Request**:
```json
{
  "description": "Production backend"
}
```

**Response**:
```json
{
  "credential_id": 123456,
  "credential_value": "archodex_selfhosted_123456_dGVzdGRhdGE=...",
  "description": "Production backend",
  "created_at": "2025-10-10T12:00:00Z"
}
```

**Note**: Credential value displayed **once** - never shown again

#### 2. Fetch Plan Limits (Self-Hosted Backend)

**Endpoint**: `GET /api/v1/self-hosted/plan-limits`

**Auth**: Bearer token with self-hosted credential

**Response**:
```json
{
  "account_id": "1234567890",
  "plan": {
    "max_resources": 500,
    "max_events_per_hour": 1000,
    "update_frequency_seconds": 1200
  },
  "fetched_at": "2025-10-10T12:00:00Z",
  "cache_until": "2025-10-13T12:00:00Z"
}
```

**Caching**: 72-hour cache on self-hosted backend, refreshed every hour by default

#### 3. List Credentials

**Endpoint**: `GET /api/accounts/{account_id}/self-hosted-credentials`

**Auth**: AWS Cognito JWT

**Response**:
```json
{
  "credentials": [
    {
      "credential_id": 123456,
      "description": "Production backend",
      "created_at": "2025-10-10T12:00:00Z",
      "last_used_at": "2025-10-10T14:23:15Z"
    },
    {
      "credential_id": 234567,
      "description": "Staging backend",
      "created_at": "2025-10-10T13:00:00Z",
      "last_used_at": null
    }
  ]
}
```

**Note**: Credential values NOT included (security best practice)

#### 4. Revoke Credential

**Endpoint**: `DELETE /api/accounts/{account_id}/self-hosted-credentials/{credential_id}`

**Auth**: AWS Cognito JWT

**Response**: 204 No Content

**Effect**: Immediate invalidation - next API call from backend will fail

### Multi-Backend Support

**Scenario**: User has production + staging + development backends

**Solution**: Generate 3 separate credentials via dashboard

**Configuration**:

```bash
# Production backend
export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_123456_..."

# Staging backend
export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_234567_..."

# Development backend
export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_345678_..."
```

**Benefits**:
- Independent revocation (revoke staging without affecting production)
- Audit trail (see last_used_at for each backend)
- Clear labeling (descriptions distinguish backends)

### Security Analysis

#### Threat: Credential Theft

**Mitigation**:
- Stored only in env var (not in git)
- One-time display in UI
- Revocation available

**Residual Risk**: **Medium** - If credential leaked, attacker can fetch plan limits until revoked. Impact limited to plan metadata (not customer data).

**Acceptable**: Yes - per requirements, some abuse risk is tolerable for free tier users.

#### Threat: Credential Tampering

**Mitigation**:
- AES-GCM authenticated encryption
- AAD binding to account_id and purpose
- Any modification causes decryption failure

**Residual Risk**: **None** - Cryptographically impossible to modify credential

#### Threat: Account Impersonation

**Mitigation**:
- Account ID encrypted in credential
- Account ID validated during decryption
- Database check ensures credential belongs to account

**Residual Risk**: **None** - Cannot use credential for different account

#### Threat: Brute Force Attacks

**Mitigation**:
- 6-digit credential ID space (900,000 possibilities)
- Rate limiting on API endpoint
- Account ID unknown to attacker

**Residual Risk**: **Low** - Expensive to brute force, low value target (free tier limits)

#### Threat: Abuse (100 backends per account)

**Detection**:
- Monitor number of active credentials per account
- CloudWatch metrics on API call frequency
- Alert if >10 active credentials

**Mitigation**:
- Rate limit credential generation (e.g., max 10 per account)
- Rate limit plan fetch API (e.g., 1 request per minute per credential)
- Dashboard warning for excessive credentials

**Residual Risk**: **Acceptable** - Per requirements, some abuse risk is tolerable.

---

## Implementation Plan

### Phase 1: Database Schema and Core Logic (Week 1)
- Add `self_hosted_credential` table to migrator
- Implement credential generation function
- Implement credential validation function
- Write unit tests

### Phase 2: archodex.com API Endpoints (Week 2)
- Implement POST, GET, DELETE endpoints
- Add authentication/authorization middleware
- Write integration tests

### Phase 3: Dashboard UI (Week 3)
- Add credential display to account creation page
- Build "Self-Hosted Credentials" settings page
- Implement credential generation/revocation UI
- Add copy-to-clipboard functionality

### Phase 4: Self-Hosted Backend Integration (Week 4)
- Implement plan fetch module
- Add background task for periodic fetching
- Implement 72-hour cache with expiration
- Integrate with rate limiting enforcement
- Write end-to-end tests

### Phase 5: Security and Monitoring (Week 5)
- Register credential pattern with GitHub secret scanning
- Add CloudWatch metrics and alarms
- Implement rate limiting on plan fetch API

### Phase 6: Documentation (Week 6)
- Write self-hosted setup guide
- Document credential management best practices
- Create troubleshooting guide
- Update DATAMODEL.md

**Total Timeline**: 4-6 weeks

---

## Comparison Matrix

| Criterion | Option 1: Auto-Gen | Option 2: Self-Reg | Option 3: Basic Secret | **Option 3 Enhanced (Recommended)** |
|-----------|-------------------|-------------------|------------------------|-------------------------------------|
| Self-service | ⚠️ Partial | ✅ Yes | ✅ Yes | ✅ **Yes** |
| Multi-backend | ❌ Unclear | ✅ Yes (complex) | ⚠️ Same credential | ✅ **Multiple credentials** |
| Restart-safe | ✅ Yes | ⚠️ Needs persistence | ✅ Yes | ✅ **Yes** |
| Abuse-tolerant | ✅ Yes | ⚠️ Risk of spam | ❌ All-or-nothing | ✅ **Detection possible** |
| Security | ⚠️ Basic | ⚠️ Complex | ❌ No tamper protection | ✅ **AES-GCM tamper-proof** |
| Audit trail | ❌ No | ✅ Yes | ❌ No | ✅ **Yes** |
| Granular revocation | ❌ No | ✅ Yes | ❌ No | ✅ **Yes** |
| Aligns with existing | ⚠️ Partial | ❌ No | ❌ No | ✅ **Yes (report API keys)** |
| Implementation | ⭐ Simple | ⭐⭐⭐⭐ Complex | ⭐⭐ Moderate | ⭐⭐⭐ **Moderate-High** |
| Operations | ⭐⭐ Moderate | ⭐⭐⭐⭐⭐ Very Complex | ⭐ Simple | ⭐⭐ **Moderate** |

**Legend**: ✅ Fully supported | ⚠️ Partial support | ❌ Not supported | ⭐ Complexity rating

---

## Open Questions & Recommendations

### 1. Should we support credential rotation?

**Recommendation**: No rotation support initially. Add later if requested.

**Rationale**: Rotation adds complexity (multiple active credentials, grace periods). Can be added later without breaking changes. Simple revoke-and-regenerate workflow is sufficient for MVP.

### 2. Should we rate-limit credential generation?

**Recommendation**: Yes - soft limit of 10 credentials per account with warning.

**Rationale**: Hard limit could block legitimate use cases. Soft limit + monitoring is sufficient for free tier. Can tighten later if abuse observed.

### 3. Should we store credential value in database?

**Recommendation**: No - credential shown once, never stored.

**Rationale**: Security best practice. Forces users to save credentials properly. If lost, user can generate new credential and revoke old one.

### 4. Should we support credential expiration?

**Recommendation**: No expiration initially. Add later as opt-in feature if requested.

**Rationale**: Self-hosted operators prefer "set and forget". Expiration adds operational burden (rotation required). Can be added later without breaking changes.

---

## Success Metrics

1. **Time to first successful plan fetch**: <30 seconds from account creation
2. **Credential generation success rate**: >99%
3. **Plan fetch API latency**: <100ms p99
4. **Credential validation success rate**: >99.9% (excluding revoked)
5. **User error rate during setup**: <5%
6. **Support tickets related to credentials**: <1% of total support volume

---

## Why This Approach?

### 1. Meets All Requirements

- ✅ **Self-service**: User generates credentials via dashboard without employee involvement
- ✅ **Multi-backend support**: Generate multiple credentials for staging, production, etc.
- ✅ **Restart-safe**: Credential stored in environment variable, persists across restarts
- ✅ **Abuse-tolerant**: Detection possible, revocation available, acceptable risk for free tier

### 2. Aligns with Existing Architecture

- Reuses proven AES-GCM pattern from report API keys
- Same cryptographic primitives (api_private_key from AWS KMS)
- Consistent credential format and validation
- Familiar to developers and operators

### 3. Appropriate Security for Threat Model

- Self-hosted backends fetching plan limits (not customer data)
- Free tier users (low-value target for attackers)
- Abuse detection and revocation available
- Tamper-proof credentials

### 4. Operational Simplicity

- One environment variable to configure
- No token refresh logic
- No OAuth infrastructure
- No certificate management
- Clear troubleshooting path

### 5. Good User Experience

- Credential auto-generated on account creation
- Can generate additional credentials for staging/prod
- Revocation available via dashboard
- Clear audit trail (last_used_at tracking)
- One-time display enforces security best practice

---

## Conclusion

The **Enhanced Shared Secret with AES-GCM** approach provides the optimal balance of security, simplicity, and functionality for self-hosted backend authentication.

**Recommendation**: Proceed with implementation of Option 3 Enhanced.

**Timeline**: 4-6 weeks across 6 phases

**Risk**: Low - proven cryptographic pattern, aligns with existing architecture

**Next Steps**:
1. Review and approve this recommendation
2. Begin Phase 1 implementation (database schema and core logic)
3. Iterate on dashboard UI mockups during Phase 2
4. Launch beta with selected self-hosted users during Phase 4

---

## Appendix: Related Documents

- [Detailed Design Document](./self-hosted-auth-design.md): Complete technical specification
- [Visual Diagrams](./self-hosted-auth-diagrams.md): Flow diagrams and architecture visualizations
- [Rate Limiting Spec](./spec.md): Overall feature specification
- [Implementation Plan](./plan.md): Rate limiting implementation plan
- [Research Document](./research.md): Technical research findings
