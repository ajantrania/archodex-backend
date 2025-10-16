# Self-Hosted Authentication Flow Diagrams

This document provides visual representations of the self-hosted authentication flows to complement the [main design document](./self-hosted-auth-design.md).

---

## 1. Complete End-to-End Flow

```mermaid
graph TB
    subgraph "Phase 1: Account Creation & Credential Generation"
        A[User] -->|Login with Cognito| B[Dashboard]
        B -->|Create Account| C[archodex.com API]
        C -->|Auto-generate| D[Self-Hosted Credential]
        C -->|Store metadata| E[(SurrealDB accounts)]
        C -->|Display once| B
        B -->|Show credential| A
        A -->|Copy to clipboard| A
    end

    subgraph "Phase 2: Self-Hosted Backend Configuration"
        A -->|Set env var| F[Self-Hosted Backend]
        F -->|ARCHODEX_SELF_HOSTED_CREDENTIAL| F
        F -->|Startup| G[Plan Fetch Module]
    end

    subgraph "Phase 3: Plan Limit Fetching"
        G -->|GET /api/v1/self-hosted/plan-limits| C
        C -->|Validate credential| H[Credential Validator]
        H -->|Decrypt & verify| H
        H -->|Check revocation| E
        H -->|Fetch plan| I[Plan Table]
        C -->|Return limits| G
        G -->|Cache 72 hours| J[Local Cache]
    end

    subgraph "Phase 4: Rate Limiting Enforcement"
        K[Agent] -->|Send events/resources| F
        F -->|Check limits| J
        F -->|Enforce limits| L[Rate Limiter]
        L -->|Apply limits| M[Ingestion Pipeline]
    end

    style D fill:#90EE90
    style J fill:#FFD700
    style L fill:#FF6B6B
```

---

## 2. Credential Structure Deep Dive

```mermaid
graph LR
    subgraph "Credential String Format"
        A["archodex_selfhosted_123456_dGVzdGRhdGE=..."]
    end

    A --> B[Prefix: archodex_selfhosted_]
    A --> C[Credential ID: 123456]
    A --> D[Base64 Protobuf]

    subgraph "Protobuf Structure"
        D --> E[SelfHostedCredential]
        E --> F[version: 1]
        E --> G[account_id: 1234567890]
        E --> H[nonce: 12 bytes]
        E --> I[encrypted_contents]
    end

    subgraph "Encrypted Contents (AES-GCM)"
        I --> J[account_id: 1234567890]
        I --> K[credential_id: 123456]
        I --> L[secret_bytes: 16 random bytes]
    end

    subgraph "AAD (Additional Authenticated Data)"
        M[account_id: 1234567890]
        N[purpose: self-hosted-plan-fetch]
        O[credential_id: 123456]
    end

    H -.->|AES-GCM| I
    M -.->|Authenticates| I
    N -.->|Authenticates| I
    O -.->|Authenticates| I

    style I fill:#FFD700
    style M fill:#90EE90
    style N fill:#90EE90
    style O fill:#90EE90
```

---

## 3. Multi-Backend Support

```mermaid
graph TB
    A[User Account 1234567890]

    A -->|Generate credential #1| B[Prod Credential: 123456]
    A -->|Generate credential #2| C[Staging Credential: 234567]
    A -->|Generate credential #3| D[Dev Credential: 345678]

    B -->|Configure| E[Production Backend]
    C -->|Configure| F[Staging Backend]
    D -->|Configure| G[Development Backend]

    E -->|Fetch limits| H[archodex.com API]
    F -->|Fetch limits| H
    G -->|Fetch limits| H

    H -->|Return plan limits| I[Max Resources: 500<br/>Max Events/hr: 1000<br/>Update Freq: 1200s]

    I -.-> E
    I -.-> F
    I -.-> G

    subgraph "All backends get same plan limits"
        I
    end

    subgraph "Audit Trail"
        E -->|last_used_at| J[(Database)]
        F -->|last_used_at| J
        G -->|last_used_at| J
    end

    style B fill:#90EE90
    style C fill:#FFD700
    style D fill:#87CEEB
```

---

## 4. Credential Validation Flow

```mermaid
sequenceDiagram
    participant SH as Self-Hosted Backend
    participant API as archodex.com API
    participant Val as Credential Validator
    participant KMS as AWS KMS (via api_private_key)
    participant DB as SurrealDB

    SH->>API: GET /api/v1/self-hosted/plan-limits<br/>Authorization: Bearer archodex_selfhosted_123456_...

    API->>Val: validate_self_hosted_credential(credential)

    Val->>Val: 1. Parse format (prefix, credential_id, base64)

    Val->>Val: 2. Decode protobuf

    Val->>KMS: 3. Get api_private_key
    KMS-->>Val: AES-GCM key

    Val->>Val: 4. Decrypt with AES-GCM<br/>(using AAD: account_id + purpose + credential_id)

    alt Decryption fails
        Val-->>API: Error: Invalid credential
        API-->>SH: 401 Unauthorized
    end

    Val->>Val: 5. Validate account_id matches<br/>(encrypted vs protobuf)

    Val->>DB: 6. SELECT * FROM self_hosted_credential:123456<br/>WHERE revoked_at IS NONE

    alt Credential not found or revoked
        DB-->>Val: None
        Val-->>API: Error: Not found or revoked
        API-->>SH: 401 Unauthorized
    else Credential valid
        DB-->>Val: Credential record
        Val->>DB: UPDATE last_used_at = time::now()<br/>(fire-and-forget)
        Val-->>API: Valid (account_id: 1234567890)
        API->>DB: SELECT * FROM plan WHERE account_id = 1234567890
        DB-->>API: Plan limits
        API-->>SH: 200 OK + plan limits
    end
```

---

## 5. Credential Lifecycle Management

```mermaid
stateDiagram-v2
    [*] --> NotExists: User creates account

    NotExists --> Active: Auto-generate credential

    Active --> Active: Used by self-hosted backend
    Active --> Active: User generates additional credentials

    Active --> Revoked: User revokes credential
    Active --> Revoked: Admin revokes credential
    Active --> Revoked: Auto-revoke (e.g., detected in public repo)

    Revoked --> [*]: Credential deleted after retention period

    note right of Active
        Credential stored in:
        - Database (metadata only)
        - Self-hosted backend env var
        - User's secrets manager
    end note

    note right of Revoked
        Immediate effect:
        - Next API call fails
        - Backend must use different credential
    end note
```

---

## 6. Security Model Comparison

### Option 1: Auto-Generate (Basic)

```mermaid
graph LR
    A[Account Created] -->|Auto-generate| B[Credential 1]
    B --> C[?]

    style C fill:#FF6B6B,stroke:#333,stroke-width:4px

    D[Question: How to support multiple backends?]
    E[Generate 10 upfront? Wasteful]
    F[Generate on demand? Not automatic]
```

### Option 2: Backend Self-Registration

```mermaid
sequenceDiagram
    participant BE as Backend
    participant API as archodex.com
    participant User as User

    BE->>API: POST /api/register-backend<br/>(account_id + generated_credential)
    API-->>BE: Pending. Enter code "XYZ123"
    BE->>BE: Wait for approval...

    User->>API: Enter code "XYZ123"
    API->>API: Approve registration

    BE->>API: Poll: Is approved?
    API-->>BE: Yes, credential active

    Note over BE,User: Problems:<br/>1. How does backend know account_id?<br/>2. Complex UX (manual code entry)<br/>3. Not restart-safe by default
```

### Option 3 Enhanced: Shared Secret with AES-GCM (RECOMMENDED)

```mermaid
graph TB
    A[User] -->|Self-service generate| B[Credential]
    B -->|AES-GCM encrypted| C[Tamper-proof]
    B -->|Account-bound| D[Cannot impersonate]
    B -->|Purpose-bound| E[Cannot reuse for other APIs]
    B -->|Revocable| F[Instant invalidation]
    B -->|Multi-backend| G[Generate multiple]

    C --> H[Security ✓]
    D --> H
    E --> H
    F --> I[Operations ✓]
    G --> I
    B --> J[UX ✓]

    style B fill:#90EE90
    style H fill:#90EE90
    style I fill:#90EE90
    style J fill:#90EE90
```

---

## 7. Abuse Detection & Mitigation

```mermaid
graph TB
    subgraph "Detection Layer"
        A[CloudWatch Metrics] --> B{Suspicious Activity?}
        B -->|>10 active credentials| C[Alert: Too many credentials]
        B -->|Multiple IPs per credential| D[Alert: Credential sharing?]
        B -->|High API call frequency| E[Alert: Potential abuse]
    end

    subgraph "Mitigation Actions"
        C --> F[Send email to user]
        D --> F
        E --> F

        F --> G{User Response?}
        G -->|Legitimate| H[Allowlist account]
        G -->|No response| I[Rate limit API calls]
        G -->|Confirmed abuse| J[Revoke all credentials]
    end

    subgraph "Automated Protections"
        K[GitHub Secret Scanning] -->|Credential detected| L[Auto-revoke]
        L --> M[Notify user]
    end

    style B fill:#FFD700
    style F fill:#FF6B6B
    style J fill:#FF6B6B
```

---

## 8. Implementation Phases

```mermaid
gantt
    title Self-Hosted Authentication Implementation Timeline
    dateFormat  YYYY-MM-DD

    section Phase 1: Core Infrastructure
    Database schema (self_hosted_credential table)    :p1a, 2025-10-10, 2d
    Credential generation logic                      :p1b, after p1a, 2d
    Credential validation logic                      :p1c, after p1a, 2d
    Unit tests                                       :p1d, after p1b, 1d

    section Phase 2: API Endpoints
    POST /api/accounts/{id}/self-hosted-credentials  :p2a, after p1c, 2d
    GET /api/v1/self-hosted/plan-limits             :p2b, after p1c, 2d
    DELETE /api/accounts/{id}/self-hosted-credentials/{id} :p2c, after p2a, 1d
    GET /api/accounts/{id}/self-hosted-credentials (list)  :p2d, after p2a, 1d
    Integration tests                                :p2e, after p2d, 2d

    section Phase 3: Dashboard UI
    Account creation credential display              :p3a, after p2b, 2d
    Self-hosted credentials settings page            :p3b, after p3a, 3d
    Credential generation UI                         :p3c, after p3b, 2d
    Credential revocation UI                         :p3d, after p3b, 2d

    section Phase 4: Self-Hosted Integration
    Plan fetch module implementation                 :p4a, after p2b, 3d
    Background task for periodic fetching            :p4b, after p4a, 2d
    Cache management (72-hour expiration)            :p4c, after p4a, 2d
    Integration with rate limiting enforcement       :p4d, after p4c, 2d
    End-to-end tests                                 :p4e, after p4d, 2d

    section Phase 5: Security & Monitoring
    GitHub secret scanning registration              :p5a, after p4e, 1d
    CloudWatch metrics & alarms                      :p5b, after p4e, 2d
    Rate limiting on plan fetch API                  :p5c, after p5b, 2d

    section Phase 6: Documentation
    Self-hosted setup guide                          :p6a, after p5c, 2d
    Credential management best practices             :p6b, after p6a, 1d
    Troubleshooting guide                            :p6c, after p6a, 1d
```

---

## 9. Comparison Matrix

| Feature | Option 1: Auto-Gen Basic | Option 2: Self-Reg | Option 3: Shared Secret Basic | Option 3 Enhanced (RECOMMENDED) |
|---------|--------------------------|-------------------|------------------------------|--------------------------------|
| **Self-service** | ⚠️ Partial (unclear multi-backend) | ✅ Yes | ✅ Yes | ✅ Yes |
| **Multi-backend** | ❌ Unclear | ✅ Yes (complex UX) | ⚠️ Yes (same credential) | ✅ Yes (multiple credentials) |
| **Restart-safe** | ✅ Yes | ⚠️ Needs persistence | ✅ Yes | ✅ Yes |
| **Abuse-tolerant** | ✅ Yes | ⚠️ Risk of spam | ❌ All-or-nothing | ✅ Yes (detection possible) |
| **Security** | ⚠️ Basic | ⚠️ Complex flow | ❌ No tamper protection | ✅ AES-GCM tamper-proof |
| **Audit trail** | ❌ No | ✅ Yes | ❌ No | ✅ Yes (credential_id + last_used_at) |
| **Granular revocation** | ❌ No | ✅ Yes | ❌ No (all-or-nothing) | ✅ Yes |
| **Implementation complexity** | ⭐ Simple | ⭐⭐⭐⭐ Complex | ⭐⭐ Moderate | ⭐⭐⭐ Moderate-High |
| **Operational complexity** | ⭐⭐ Moderate | ⭐⭐⭐⭐⭐ Very Complex | ⭐ Simple | ⭐⭐ Moderate |
| **Aligns with existing patterns** | ⚠️ Partial | ❌ No | ❌ No | ✅ Yes (report API keys) |

**Legend**:
- ✅ Fully supported
- ⚠️ Partially supported or has caveats
- ❌ Not supported or has significant issues
- ⭐ Rating (more stars = more complex)

---

## 10. Threat Model Visualization

```mermaid
graph TB
    subgraph "Attack Vectors"
        A1[Credential Theft]
        A2[Credential Tampering]
        A3[Account Impersonation]
        A4[Replay Attacks]
        A5[Brute Force]
        A6[Abuse: 100 Backends]
    end

    subgraph "Mitigations"
        M1[One-time display<br/>Env var storage<br/>Revocation]
        M2[AES-GCM authenticated encryption<br/>AAD binding]
        M3[Account ID encrypted<br/>Validated on decrypt]
        M4[Long-lived credentials<br/>Revocation available]
        M5[6-digit ID space<br/>Rate limiting<br/>Account ID hidden]
        M6[Detection via metrics<br/>Rate limiting<br/>Acceptable risk]
    end

    subgraph "Residual Risk"
        R1[Medium:<br/>Limited to plan metadata]
        R2[None:<br/>Cryptographically impossible]
        R3[None:<br/>Cannot use for different account]
        R4[Low:<br/>Acceptable trade-off]
        R5[Low:<br/>Expensive, low value]
        R6[Acceptable:<br/>Per requirements]
    end

    A1 --> M1 --> R1
    A2 --> M2 --> R2
    A3 --> M3 --> R3
    A4 --> M4 --> R4
    A5 --> M5 --> R5
    A6 --> M6 --> R6

    style R1 fill:#FFD700
    style R2 fill:#90EE90
    style R3 fill:#90EE90
    style R4 fill:#90EE90
    style R5 fill:#90EE90
    style R6 fill:#FFD700
```

---

## Summary

These diagrams illustrate the complete self-hosted authentication system design, showing:

1. **End-to-end flow** from account creation to rate limiting enforcement
2. **Credential structure** with AES-GCM encryption and AAD
3. **Multi-backend support** with independent credentials
4. **Validation flow** with security checks and revocation
5. **Lifecycle management** from creation to revocation
6. **Security comparison** across all options
7. **Abuse detection** and mitigation strategies
8. **Implementation timeline** across 6 phases
9. **Feature comparison matrix** for all options
10. **Threat model** with mitigations and residual risks

The **Enhanced Shared Secret with AES-GCM (Option 3 Enhanced)** provides the best balance of security, usability, and operational simplicity for self-hosted Archodex backends.
