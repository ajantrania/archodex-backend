# Self-Hosted Authentication: Quick Reference

**For**: Developers, Operators, and Support Staff
**Last Updated**: 2025-10-10

---

## For Self-Hosted Operators

### Initial Setup (5 minutes)

1. **Get your credential**:
   - Log into archodex.com dashboard
   - Go to Settings > Self-Hosted Credentials
   - Click "Generate New Credential"
   - **Copy credential immediately** (shown only once)

2. **Configure your backend**:
   ```bash
   export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_123456_..."
   ```

3. **Start your backend**:
   ```bash
   ./archodex-backend
   ```

4. **Verify it works**:
   - Check logs for: `Successfully fetched plan limits from archodex.com`
   - If successful, backend is configured correctly

### Multiple Backends (Staging + Production)

**Scenario**: You have 3 backends (dev, staging, prod)

**Steps**:
1. Generate 3 separate credentials in dashboard
2. Label each credential (e.g., "Production backend")
3. Configure each backend with its own credential

**Example**:
```bash
# Production server
export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_123456_..."

# Staging server
export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_234567_..."

# Development laptop
export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_345678_..."
```

### Credential Lost or Compromised

**Steps**:
1. Log into dashboard
2. Go to Settings > Self-Hosted Credentials
3. Click "Revoke" on compromised credential
4. Click "Generate New Credential"
5. Update backend env var with new credential
6. Restart backend

**Effect**: Old credential invalidated immediately

### Troubleshooting

#### Error: "Missing ARCHODEX_SELF_HOSTED_CREDENTIAL environment variable"

**Solution**: Set environment variable before starting backend
```bash
export ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_..."
./archodex-backend
```

#### Error: "Failed to fetch plan limits: HTTP 401"

**Possible causes**:
- Credential revoked (check dashboard)
- Credential malformed (copy-paste error)
- Credential for wrong account

**Solution**:
1. Verify credential is active in dashboard
2. Re-copy credential from dashboard
3. Ensure no extra whitespace in env var

#### Error: "Cached plan limits expired"

**Explanation**: Backend couldn't reach archodex.com for >72 hours

**Solution**:
1. Check internet connectivity
2. Verify archodex.com is reachable
3. Check firewall rules (allow HTTPS to api.archodex.com)
4. Restart backend to retry

#### Backend works but no limits enforced

**Possible causes**:
- Using old backend version (before rate limiting feature)
- Custom plan with unlimited limits (max_resources=0, max_events_per_hour=0)

**Solution**:
1. Check backend version: `./archodex-backend --version`
2. Check plan limits in dashboard: Settings > Plan

---

## For Dashboard Users

### Generate a Credential

1. Log into archodex.com
2. Select your account
3. Go to Settings > Self-Hosted Credentials
4. Click "Generate New Credential"
5. Enter description (e.g., "Production backend")
6. Click "Generate"
7. **Copy credential immediately** (shown only once)
8. Save in password manager or secrets vault

### View Active Credentials

1. Log into archodex.com
2. Select your account
3. Go to Settings > Self-Hosted Credentials
4. See list of all credentials with:
   - Credential ID
   - Description
   - Created date
   - Last used date

### Revoke a Credential

1. Log into archodex.com
2. Select your account
3. Go to Settings > Self-Hosted Credentials
4. Find credential to revoke
5. Click "Revoke"
6. Confirm revocation

**Effect**: Credential invalidated immediately. Backend must use different credential.

### Best Practices

1. **One credential per backend** - Don't share credentials across backends
2. **Use descriptive labels** - "Production backend", "Staging backend", etc.
3. **Store securely** - Save in password manager or secrets vault
4. **Revoke unused credentials** - Clean up old credentials regularly
5. **Monitor usage** - Check "Last used" to detect suspicious activity

---

## For Developers

### Credential Format

**String**: `archodex_selfhosted_{credential_id}_{base64_protobuf}`

**Example**: `archodex_selfhosted_123456_dGVzdGRhdGE=...`

**Components**:
- **Prefix**: `archodex_selfhosted_`
- **Credential ID**: 6-digit number (100,000-999,999)
- **Payload**: Base64-encoded protobuf

### Protobuf Definition

```protobuf
message SelfHostedCredential {
  uint32 version = 1;
  fixed64 account_id = 2;
  bytes nonce = 3;  // 12 bytes for AES-GCM
  bytes encrypted_contents = 4;
}

message SelfHostedCredentialEncryptedContents {
  fixed64 account_id = 1;
  uint32 credential_id = 2;
  bytes secret_bytes = 3;  // 16 random bytes
}

message SelfHostedCredentialAAD {
  fixed64 account_id = 1;
  string purpose = 2;  // "self-hosted-plan-fetch"
  uint32 credential_id = 3;
}
```

### API Endpoints

#### Generate Credential

```http
POST /api/accounts/{account_id}/self-hosted-credentials
Authorization: Bearer {cognito_jwt}
Content-Type: application/json

{
  "description": "Production backend"
}
```

**Response**:
```json
{
  "credential_id": 123456,
  "credential_value": "archodex_selfhosted_123456_...",
  "description": "Production backend",
  "created_at": "2025-10-10T12:00:00Z"
}
```

#### Fetch Plan Limits

```http
GET /api/v1/self-hosted/plan-limits
Authorization: Bearer archodex_selfhosted_123456_...
```

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

#### List Credentials

```http
GET /api/accounts/{account_id}/self-hosted-credentials
Authorization: Bearer {cognito_jwt}
```

**Response**:
```json
{
  "credentials": [
    {
      "credential_id": 123456,
      "description": "Production backend",
      "created_at": "2025-10-10T12:00:00Z",
      "last_used_at": "2025-10-10T14:23:15Z"
    }
  ]
}
```

#### Revoke Credential

```http
DELETE /api/accounts/{account_id}/self-hosted-credentials/{credential_id}
Authorization: Bearer {cognito_jwt}
```

**Response**: 204 No Content

### Validation Logic

**Rust Example**:

```rust
use aes_gcm::{Aes128Gcm, Aead};
use prost::Message;

async fn validate_credential(credential_value: &str) -> Result<String> {
    // 1. Parse format
    let credential_value = credential_value
        .strip_prefix("archodex_selfhosted_")?;

    let parts: Vec<&str> = credential_value.splitn(2, '_').collect();
    let credential_id: u32 = parts[0].parse()?;
    let payload = BASE64.decode(parts[1])?;

    // 2. Decode protobuf
    let proto = SelfHostedCredential::decode(payload.as_slice())?;

    // 3. Decrypt with AES-GCM
    let cipher = Aes128Gcm::new(api_private_key().await);
    let nonce = Nonce::from_slice(&proto.nonce);

    let aad = SelfHostedCredentialAAD {
        account_id: proto.account_id,
        purpose: "self-hosted-plan-fetch".to_string(),
        credential_id,
    };

    let decrypted = cipher.decrypt(
        nonce,
        Payload {
            msg: &proto.encrypted_contents,
            aad: &aad.encode_to_vec(),
        },
    )?;

    let contents = SelfHostedCredentialEncryptedContents::decode(
        decrypted.as_slice()
    )?;

    // 4. Validate
    ensure!(contents.account_id == proto.account_id);
    ensure!(contents.credential_id == credential_id);

    // 5. Check revocation
    let db = accounts_db().await?;
    let record = db.query("
        SELECT * FROM self_hosted_credential:$credential_id
        WHERE revoked_at IS NONE
    ")
    .bind(("credential_id", credential_id))
    .await?
    .take::<Option<_>>(0)?
    .ok_or(anyhow!("Credential not found or revoked"))?;

    Ok(proto.account_id.to_string())
}
```

### Database Schema

```sql
DEFINE TABLE self_hosted_credential SCHEMAFULL TYPE NORMAL;

DEFINE FIELD account_id ON TABLE self_hosted_credential
    TYPE record<account> ASSERT $value != NONE;

DEFINE FIELD credential_id ON TABLE self_hosted_credential
    TYPE int ASSERT $value >= 100000 AND $value <= 999999;

DEFINE FIELD description ON TABLE self_hosted_credential
    TYPE option<string>;

DEFINE FIELD created_at ON TABLE self_hosted_credential
    TYPE datetime DEFAULT time::now();

DEFINE FIELD created_by ON TABLE self_hosted_credential
    TYPE record<user> ASSERT $value != NONE;

DEFINE FIELD last_used_at ON TABLE self_hosted_credential
    TYPE option<datetime>;

DEFINE FIELD revoked_at ON TABLE self_hosted_credential
    TYPE option<datetime>;

DEFINE FIELD revoked_by ON TABLE self_hosted_credential
    TYPE option<record<user>>;

DEFINE INDEX credential_id_idx ON TABLE self_hosted_credential
    FIELDS credential_id UNIQUE;

DEFINE INDEX account_id_idx ON TABLE self_hosted_credential
    FIELDS account_id;
```

### Environment Variables

```bash
# Required for self-hosted backends
ARCHODEX_SELF_HOSTED_CREDENTIAL="archodex_selfhosted_123456_..."

# Optional: Fetch interval in seconds (default: 3600 = 1 hour)
ARCHODEX_PLAN_FETCH_INTERVAL_SECONDS=3600
```

### Integration Example

```rust
// In main.rs (self-hosted backend)

#[tokio::main]
async fn main() -> Result<()> {
    // Start plan fetch background task
    tokio::spawn(async {
        start_plan_fetch_background_task().await;
    });

    // Initial fetch (blocks startup until limits available)
    get_cached_plan_limits().await?;

    info!("Self-hosted backend initialized with plan limits");

    // Start HTTP server
    start_server().await
}
```

---

## For Support Staff

### Common Issues

#### Issue: User lost credential

**Resolution**:
1. User cannot retrieve lost credential (security design)
2. Instruct user to:
   - Revoke old credential in dashboard
   - Generate new credential
   - Update backend env var
   - Restart backend

#### Issue: Credential not working

**Debug steps**:
1. Check if credential is revoked (dashboard)
2. Verify credential format (starts with `archodex_selfhosted_`)
3. Check if credential belongs to correct account
4. Verify backend can reach archodex.com

#### Issue: Backend not enforcing limits

**Debug steps**:
1. Check backend version (must have rate limiting feature)
2. Verify plan limits are not unlimited (max_resources=0 means unlimited)
3. Check backend logs for plan fetch success
4. Verify cached limits have not expired

#### Issue: "Too many credentials" warning

**Explanation**: Account has >10 active credentials

**Resolution**:
1. Review credentials in dashboard
2. Revoke unused credentials
3. If legitimately need >10, note in support ticket for monitoring

### Escalation Criteria

**Escalate to engineering if**:
- Multiple users report credential validation failures
- Plan fetch API returning errors consistently
- Suspected security breach (credential leaked)
- Database issues with `self_hosted_credential` table

---

## Security Guidelines

### For Users

1. **Never commit credentials to git** - Use environment variables or secrets manager
2. **Store in password manager** - Don't save in plaintext files
3. **One credential per backend** - Don't share credentials
4. **Revoke unused credentials** - Clean up regularly
5. **Monitor last used date** - Detect suspicious activity

### For Developers

1. **Credential in AAD** - Never remove AAD binding
2. **No credential logging** - Redact credentials from logs
3. **Constant-time comparison** - Prevent timing attacks
4. **Rate limit endpoints** - Prevent brute force
5. **Monitor CloudWatch** - Alert on suspicious patterns

### For Operators

1. **Rotate api_private_key annually** - KMS key rotation
2. **Monitor credential generation rate** - Alert on spikes
3. **Review audit logs monthly** - Detect anomalies
4. **Test revocation regularly** - Ensure immediate effect
5. **Backup credentials table** - Include in disaster recovery

---

## Monitoring

### Key Metrics

- **credential_generation_count**: Number of credentials generated per day
- **credential_validation_success_rate**: % of successful validations
- **credential_validation_latency_p99**: 99th percentile latency
- **plan_fetch_success_rate**: % of successful plan fetches
- **active_credentials_per_account**: Distribution of credentials per account

### Alerts

- **High credential generation rate**: >100 credentials/hour
- **Low validation success rate**: <95%
- **High validation latency**: p99 >500ms
- **Many credentials per account**: >10 credentials for single account
- **Plan fetch failures**: >5% failure rate

### Dashboards

1. **Credential Health**: Generation rate, validation success, active credentials
2. **Security**: Failed validations, revocations, suspicious IPs
3. **Usage**: Plan fetches, cache hits, API latency

---

## Related Documents

- [Detailed Design](./self-hosted-auth-design.md): Complete technical specification
- [Visual Diagrams](./self-hosted-auth-diagrams.md): Architecture diagrams
- [Recommendation](./self-hosted-auth-recommendation.md): Executive summary and comparison
- [Rate Limiting Spec](./spec.md): Overall feature requirements
