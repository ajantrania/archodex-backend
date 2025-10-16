# Research: Testing Framework Setup and Validation

**Feature**: 002-specs-001-rate
**Date**: 2025-10-14
**Status**: Research Complete

---

## Executive Summary

This research resolves all "NEEDS CLARIFICATION" items from the Technical Context and evaluates testing approaches for the Archodex backend. Based on comprehensive investigation of SurrealDB testing options and Rust testing best practices, we recommend a **layered testing strategy** prioritizing simplicity and speed while maintaining production confidence.

**Key Decision: Use SurrealDB in-memory mode (`kv-mem`) as primary testing approach**

**Rationale**:
- Zero infrastructure dependencies (no Docker required)
- Fast test execution (<30 seconds for full suite)
- High production fidelity (SurrealDB maintains API parity across backends)
- Minimal maintenance burden (aligns with Constitution's anti-over-engineering principle)
- Works perfectly in GitHub Actions CI

---

## 1. SurrealDB Testing Strategy (RESOLVED)

### Research Question
How do we test database interactions with SurrealDB 2.3.7 (Archodex fork with DynamoDB backend)?

### Options Evaluated

#### Option A: In-Memory SurrealDB (`kv-mem`) ⭐ **RECOMMENDED**

**Viability**: ✅ Highly Viable

**Setup**:
```rust
use surrealdb::{Surreal, engine::local::Mem};

async fn create_test_db() -> Surreal<Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    db
}
```

**Dependencies**: None (already have surrealdb 2.3.7)

**Key Findings**:
- SurrealDB's test suite confirms **unified API behavior across all backends** (mem, rocksdb, dynamodb)
- Archodex fork at `/Users/ajantrania/code/archodex/surrealdb` runs identical tests across backends
- Evidence: `surrealdb/crates/core/src/kvs/tests/mod.rs` validates behavioral parity
- Performance: In-memory can be 3-4x slower than server mode for large queries (acceptable tradeoff)
- Production fidelity: **HIGH** for functional correctness, **MEDIUM** for performance/network testing

**Pros**:
- Zero setup time (no Docker, no containers)
- Fast test execution (microseconds vs seconds)
- Perfect test isolation (each test gets fresh DB)
- Works on all platforms (Linux, macOS, Windows)
- Automatic cleanup (no resources to manage)
- Same SurrealDB version as production (2.3.7)

**Cons**:
- Cannot test DynamoDB-specific behaviors
- Cannot test network failures or latency
- Cannot test server-only features (live queries over WebSocket)

**Assessment**: **PRIMARY APPROACH** - Use for 90-95% of tests

---

#### Option B: Testcontainers with SurrealDB Docker Image

**Viability**: ✅ Viable (secondary approach)

**Setup**:
```rust
use testcontainers_modules::{surrealdb::SurrealDb, testcontainers::runners::AsyncRunner};

async fn create_container_db() -> String {
    let container = SurrealDb::default().start().await.unwrap();
    format!("http://127.0.0.1:{}", container.get_host_port_ipv4(8000).await)
}
```

**Dependencies**: `testcontainers-modules = { version = "0.11", features = ["surrealdb"] }`

**Pros**:
- Tests actual SurrealDB server binary
- Network protocol testing (HTTP/WebSocket)
- Server-specific features (live queries, auth)
- Higher production fidelity

**Cons**:
- Requires Docker in development and CI
- Slower (2-5 seconds container startup per test)
- More complex debugging
- 10-50x slower than in-memory

**Assessment**: **SECONDARY APPROACH** - Use for 5-10% of tests (server-specific features)

---

#### Option C: LocalStack + DynamoDB Backend

**Viability**: ⚠️ Viable but Not Recommended

**Setup Complexity**: HIGH (requires LocalStack container + DynamoDB table creation)

**Evidence**: Archodex fork already uses LocalStack for DynamoDB tests:
- Location: `surrealdb/crates/sdk/tests/api_integration/mod.rs` lines 630-731
- Creates/deletes DynamoDB tables for each test
- Uses LocalStack endpoint: `http://localhost:8001`

**Pros**:
- Tests actual DynamoDB backend
- Validates fork-specific behaviors
- Highest production fidelity for archodex.com

**Cons**:
- Very slow (5-10s LocalStack startup + 1-2s per table)
- Complex setup and maintenance
- 100-500x slower than in-memory
- LocalStack ≠ real DynamoDB (behavioral differences)

**Assessment**: **NOT RECOMMENDED** - Only use for critical fork validation (outside regular test suite)

---

#### Option D: Mocking Database Layer

**Viability**: ⚠️ Viable but Requires Refactoring

**Current Architecture Barrier**: Archodex extensively uses SurrealDB-specific types and method chaining patterns that don't lend themselves to mocking without significant refactoring.

**Pros**:
- Very fast unit tests
- No external dependencies

**Cons**:
- Requires introducing trait abstraction layer (major refactor)
- Risk of mock/real implementation divergence
- Doesn't test actual database interactions
- Low production fidelity

**Assessment**: **NOT RECOMMENDED** - Architecture doesn't support this approach

---

### Decision: Layered Testing Strategy

```
┌─────────────────────────────────────────────────────────┐
│ Layer 1: In-Memory Tests (kv-mem)                      │
│ • 90-95% of test coverage                               │
│ • Run on every commit                                   │
│ • Execution time: <30 seconds                           │
│ • Validates: business logic, queries, API correctness   │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│ Layer 2: Server Tests (Testcontainers) [OPTIONAL]      │
│ • 5-10% additional coverage                             │
│ • Run on PR or nightly                                  │
│ • Execution time: 2-5 minutes                           │
│ • Validates: server features, network, WebSocket        │
└─────────────────────────────────────────────────────────┘
```

**Rationale**:
- Aligns with Constitution principle: avoid over-engineering
- Fast feedback loop encourages TDD practices
- Minimal infrastructure dependencies
- Can add Layer 2 later if needed (not required for MVP)

---

## 2. Testing Framework Selection (RESOLVED)

### Research Question
Which testing framework and libraries should we use for Rust backend testing?

### Recommendation: Minimal Tooling Approach

**Core Stack** (Start Here):
- ✅ `cargo test` (built-in, no dependencies)
- ✅ `tokio::test` (already have tokio 1.47)
- ✅ Axum `oneshot()` pattern (Tower trait, minimal dependency)
- ✅ SurrealDB in-memory mode (no new dependencies)

**Optional Additions** (Add As Needed):
- `rstest` - Fixtures and parameterized tests (when >20 tests)
- `fake` - Test data generation (when >50 tests)
- `testcontainers-modules` - Docker containers (only for Layer 2)

### HTTP Testing Pattern

**Use Axum's Built-in `oneshot()` Method** (No external dependencies):

```rust
use tower::ServiceExt; // oneshot trait
use axum::body::Body;
use axum::http::{Request, StatusCode};

#[tokio::test]
async fn test_endpoint() {
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

**Alternative**: `axum-test` crate provides slightly cleaner API but adds dependency (defer until needed).

### Test Organization

**Rust Standard Pattern**:
- **Unit tests**: `#[cfg(test)] mod tests` inside source files (e.g., `src/report.rs`)
- **Integration tests**: `tests/` directory at project root
- **Shared helpers**: `tests/common/mod.rs` (NOT `tests/common.rs` to avoid test discovery)

**Example Structure**:
```
archodex-backend/
├── src/
│   ├── report.rs              # Contains #[cfg(test)] mod tests
│   └── account.rs             # Contains #[cfg(test)] mod tests
├── tests/
│   ├── common/
│   │   ├── mod.rs             # Shared test utilities
│   │   ├── db.rs              # create_test_db()
│   │   └── fixtures.rs        # Test data builders
│   ├── report_ingestion_test.rs
│   └── report_api_key_test.rs
└── Cargo.toml
```

### Test Data Fixture Patterns

**Phase 1: Factory Functions** (Simplest):
```rust
#[cfg(test)]
pub fn create_test_report(num_resources: usize, num_events: usize) -> Report {
    Report {
        resources: (0..num_resources)
            .map(|i| Resource {
                id: format!("res{}", i),
                first_seen_at: Utc::now(),
                last_seen_at: Utc::now(),
            })
            .collect(),
        events: (0..num_events)
            .map(|i| Event { /* ... */ })
            .collect(),
    }
}
```

**Phase 2: Builder Pattern** (When tests need flexibility):
```rust
pub struct TestReportBuilder {
    resources: Vec<Resource>,
    events: Vec<Event>,
}

impl TestReportBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn with_resources(mut self, count: usize) -> Self { /* ... */ }
    pub fn build(self) -> Report { /* ... */ }
}
```

---

## 3. CI Configuration (RESOLVED)

### Research Question
How do we configure GitHub Actions for Rust testing?

### Recommendation: Standard GitHub Actions Workflow

**File**: `.github/workflows/test.yml`

```yaml
name: Test Suite

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
          components: clippy

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check
```

**Key Features**:
- Automatic Rust toolchain caching (via `setup-rust-toolchain`)
- Runs on every push and PR
- Enforces clippy and formatting
- No Docker required (using in-memory SurrealDB)
- Fast execution (<3 minutes expected)

**ACT Compatibility** (Local CI Testing):
- In-memory tests work perfectly with ACT
- Docker-based tests require ACT with Docker socket mounting

---

## 4. Dependencies to Add (RESOLVED)

### Minimal Setup (Phase 1)

**Cargo.toml** additions:
```toml
[dev-dependencies]
# Core testing (already have most dependencies)
tower = { version = "0.5", features = ["util"] }  # For oneshot()

# Optional - only if needed later
# rstest = "0.22"                                  # Fixtures
# fake = "2.9"                                     # Test data generation
# testcontainers-modules = { version = "0.11", features = ["surrealdb"] }  # Layer 2
```

**Note**: We already have tokio, surrealdb, axum, and other core dependencies in workspace.

---

## 5. Best Practices Research Findings

### Test Isolation

**SurrealDB In-Memory Approach**:
- Each test creates fresh `Surreal<Mem>` instance
- Use unique namespace/database per test: `db.use_ns("test").use_db(&uuid::Uuid::new_v4().to_string())`
- Automatic cleanup (no resources to manage)
- Safe for parallel execution

### Async Testing

**Use `tokio::test` attribute**:
```rust
#[tokio::test]
async fn test_async_function() {
    // async test code
}
```

**Time testing** (for rate limiting):
```rust
#[tokio::test(start_paused = true)]
async fn test_rate_limit() {
    tokio::time::pause();
    // Make requests
    tokio::time::advance(Duration::from_secs(60)).await;
    // Time-dependent test runs instantly
}
```

### Performance Goals

Based on research findings:
- **Unit tests**: <1 second per test
- **Integration tests (in-memory)**: <5 seconds per test
- **Full test suite**: <30 seconds (target for 20-30 tests)
- **CI pipeline**: <3 minutes total (test + clippy + fmt)

---

## 6. Revised Example Test Implementation (Based on Implementation Feedback)

### Example Test 1: Unit Test for PrincipalChainIdPart Conversion (Pure Logic)

**File**: `src/principal_chain.rs` (inline `#[cfg(test)]` module)

**Why This Test**:
- ✅ Tests pure logic (no DB, no AWS, no external dependencies)
- ✅ Tests existing `TryFrom<surrealdb::sql::Object>` and `From` trait implementations
- ✅ Can access private types and methods
- ✅ Fast (<1ms execution time)
- ✅ Perfect idiomatic Rust unit test pattern

```rust
// In src/principal_chain.rs (at bottom of file)

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::sql::{Object, Value, Strand, Array};

    #[test]
    fn test_principal_chain_id_part_round_trip() {
        // Create test ResourceId (mocking without external dependencies)
        let resource_id = ResourceId::from(vec![
            ("partition".to_string(), "aws".to_string()),
            ("account".to_string(), "123456789012".to_string()),
        ]);

        // Create PrincipalChainIdPart
        let original = PrincipalChainIdPart {
            id: resource_id.clone(),
            event: Some("s3:PutObject".to_string()),
        };

        // Convert to SurrealDB Object
        let surreal_value: Value = original.clone().into();
        let surreal_object = match surreal_value {
            Value::Object(obj) => obj,
            _ => panic!("Expected Object"),
        };

        // Convert back to PrincipalChainIdPart
        let parsed = PrincipalChainIdPart::try_from(surreal_object).unwrap();

        // Verify round-trip correctness
        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.event, original.event);
    }

    #[test]
    fn test_principal_chain_id_part_without_event() {
        let resource_id = ResourceId::from(vec![
            ("partition".to_string(), "aws".to_string()),
        ]);

        let original = PrincipalChainIdPart {
            id: resource_id.clone(),
            event: None,
        };

        let surreal_value: Value = original.clone().into();
        let surreal_object = match surreal_value {
            Value::Object(obj) => obj,
            _ => panic!("Expected Object"),
        };

        let parsed = PrincipalChainIdPart::try_from(surreal_object).unwrap();

        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.event, None);
    }

    #[test]
    fn test_principal_chain_id_part_invalid_object_missing_id() {
        let mut obj = Object::default();
        obj.insert("event".to_string(), Value::Strand(Strand::from("test")));

        let result = PrincipalChainIdPart::try_from(obj);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing the `id` key"));
    }

    #[test]
    fn test_principal_chain_id_part_invalid_event_type() {
        let resource_id = ResourceId::from(vec![("partition".to_string(), "aws".to_string())]);

        let mut obj = Object::default();
        obj.insert("id".to_string(), Value::from(resource_id));
        obj.insert("event".to_string(), Value::from(123)); // Invalid: should be string

        let result = PrincipalChainIdPart::try_from(obj);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid `event` value"));
    }
}
```

**Benefits**:
- No external dependencies (no DB connection, no AWS credentials)
- Tests real production code (`TryFrom`, `From` implementations)
- Validates error handling (missing fields, invalid types)
- Fast feedback loop for TDD

---

### Example Test 2: Integration Test with Auth Bypass (HTTP → DB)

**File**: `tests/health_check_integration_test.rs` (start simple, then escalate)

**Authentication Strategy**: Use test-specific router without auth middleware

```rust
// tests/common/test_router.rs
use axum::{Router, routing::get};

/// Creates router for testing WITHOUT authentication middleware
pub fn create_test_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "Ok" }))
    // Add more routes as needed for testing, bypassing DashboardAuth/ReportApiKeyAuth
}
```

**Simple Integration Test** (no auth needed):

```rust
// tests/health_check_integration_test.rs
mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt; // oneshot

#[tokio::test]
async fn test_health_endpoint() {
    let app = common::create_test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Ok");
}
```

**Advanced Integration Test** (with mock authenticated routes):

For routes requiring authentication, create test-specific versions:

```rust
// tests/common/test_router.rs (extended)
use axum::{Router, routing::post, Json, Extension};
use crate::account::Account;

/// Creates test router with pre-authenticated context
pub fn create_authenticated_test_router(account: Account) -> Router {
    Router::new()
        .route("/test/report", post(test_report_handler))
        .layer(Extension(account)) // Inject test account directly, bypassing auth
}

async fn test_report_handler(
    Extension(account): Extension<Account>,
    Json(req): Json<ReportRequest>,
) -> Result<()> {
    // Reuse actual report handler logic or call it directly
    report::report(Extension(account), Json(req)).await
}
```

**Full Integration Test** (HTTP → Handler → DB):

```rust
// tests/report_integration_test.rs
mod common;

use common::{create_test_db_with_account, create_authenticated_test_router, create_test_report_request};
use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[tokio::test]
async fn test_report_endpoint_with_mock_auth() {
    // Setup: Create test DB + account
    let (db, account) = create_test_db_with_account("test_account").await;

    // Create router with pre-authenticated account (bypasses Cognito)
    let app = create_authenticated_test_router(account);

    // Build test report request
    let report_json = create_test_report_request(3, 5); // 3 resources, 5 events

    // Execute: POST to /test/report
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/test/report")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&report_json).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    // Verify: Response successful
    assert_eq!(response.status(), StatusCode::OK);

    // Verify: Data stored in database
    let resources: Vec<Resource> = db.select("resource").await.unwrap();
    assert_eq!(resources.len(), 3);
}
```

---

### Authentication Bypass Strategy (Detailed Design)

**Problem**: All production routes use `DashboardAuth::authenticate` or `ReportApiKeyAuth::authenticate` middleware, which require:
- Valid Cognito JWT tokens (DashboardAuth)
- Valid encrypted Report API keys with SSM/KMS access (ReportApiKeyAuth)

**Solution**: Test-specific router without authentication middleware

#### Approach 1: Separate Test Router (RECOMMENDED)

**Benefits**:
- ✅ No modification to production code
- ✅ Security-safe (test routes never deployed)
- ✅ Full control over test setup
- ✅ Can inject mock `DashboardAuth` or `ReportApiKeyAuth` extensions directly

**Implementation**:

```rust
// tests/common/test_router.rs

/// Test router that bypasses authentication by injecting auth extensions directly
pub fn create_test_router_with_mock_auth(
    account_id: &str,
    user_id: &str,
) -> Router {
    let mock_dashboard_auth = DashboardAuth::new_for_testing(User::new(user_id));
    let mock_account = Account::new_for_testing(account_id);

    Router::new()
        .route("/test/accounts", get(accounts::list_accounts))
        .route("/test/report", post(report::report))
        .layer(Extension(mock_dashboard_auth))  // Inject mock auth
        .layer(Extension(mock_account))          // Inject mock account
}
```

**Required Changes to Production Code** (minimal):

```rust
// src/auth.rs - Add test-only constructors
impl DashboardAuth {
    #[cfg(test)]
    pub fn new_for_testing(principal: User) -> Self {
        Self { principal }
    }
}

// src/account.rs - Add test-only constructor
impl Account {
    #[cfg(test)]
    pub fn new_for_testing(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}
```

#### Approach 2: Mock JWT Tokens (NOT RECOMMENDED)

**Why Rejected**:
- ❌ Requires setting up mock JWKS server
- ❌ Complex token generation
- ❌ Tests network layer unnecessarily
- ❌ Slower execution

---

### Final Test Structure

```
tests/
├── common/
│   ├── mod.rs                # Re-exports
│   ├── db.rs                 # Database helpers (in-memory SurrealDB)
│   ├── fixtures.rs           # Test data builders
│   └── test_router.rs        # Test routers with auth bypass
│
├── health_check_test.rs      # Simple integration (no auth)
└── report_integration_test.rs # Full integration (mock auth)

src/
└── principal_chain.rs        # Contains #[cfg(test)] mod tests (unit tests)
```

---

### Summary of Revised Approach

**Example Test 1 (Unit)**:
- Location: `src/principal_chain.rs` (#[cfg(test)] module)
- Tests: `PrincipalChainIdPart` conversions (TryFrom/From traits)
- Dependencies: None (pure logic)
- Execution: <1ms

**Example Test 2 (Integration)**:
- Location: `tests/report_integration_test.rs`
- Tests: HTTP request → Handler → Database
- Auth Bypass: Test router with mock `Extension<Account>`
- Dependencies: In-memory SurrealDB
- Execution: ~50-100ms

**Benefits of This Approach**:
- ✅ No AWS credentials or SSM/KMS access needed
- ✅ No Cognito setup required
- ✅ Security-safe (test helpers marked #[cfg(test)])
- ✅ Fast execution (in-memory DB)
- ✅ Reusable pattern for future tests

---

### Example Test 3: Full Auth Middleware + Report Ingestion (Integration)

**Purpose**: Test the COMPLETE authentication flow including both middleware layers

**File**: `tests/report_with_auth_test.rs`

**What Makes This Different**:
- ✅ Tests actual auth middleware chain (not bypassed)
- ✅ Tests `ReportApiKeyAuth::authenticate` middleware (src/auth.rs:196)
- ✅ Tests `report_api_key_account` middleware (src/db.rs:296)
- ✅ Tests full request flow: Auth → Account Loading → Handler → DB
- ✅ Validates that `Extension<ReportApiKeyAuth>` and `Extension<Account>` are correctly injected

**Architecture Understanding**:

Production router for `/report` endpoint has TWO middleware layers:
```rust
// src/router.rs:74-77
let report_api_key_authed_router = Router::new()
    .route("/report", post(report::report))
    .layer(middleware::from_fn(report_api_key_account))      // Layer 2: Loads Account
    .layer(middleware::from_fn(ReportApiKeyAuth::authenticate)); // Layer 1: Auth
```

**Middleware Flow**:
1. `ReportApiKeyAuth::authenticate` → Validates auth header → Injects `Extension<ReportApiKeyAuth>`
2. `report_api_key_account` → Uses auth extension → Loads account from DB → Injects `Extension<Account>`
3. `report::report` handler → Uses both extensions → Processes report

**Test Strategy**: Use production router with test-specific setup helpers

**Test Implementation**:

```rust
// tests/report_with_auth_test.rs
mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use common::{create_test_db_with_account, create_test_report_request, setup_test_env};

#[tokio::test]
async fn test_report_endpoint_with_full_auth_middleware() {
    // Setup: Create test environment
    setup_test_env();  // Sets required env vars (ARCHODEX_DOMAIN, etc.)

    // Setup: Create test DB + account
    let (db, account) = create_test_db_with_account("test_account_123").await;

    // Setup: Store account in accounts DB (required by report_api_key_account middleware)
    let accounts_db = common::get_test_accounts_db().await;
    accounts_db.create(("account", &account.id))
        .content(&account)
        .await
        .unwrap();

    // Setup: Create test router using PRODUCTION router function
    // This includes actual auth middleware!
    let app = crate::router::router();  // Uses real production router

    // Setup: Build test report
    let report_json = create_test_report_request(3, 5);

    // Execute: POST to /report with mock Authorization header
    // We still need to bypass ReportApiKeyAuth validation, so we use a test-specific token
    let mock_auth_token = common::create_test_auth_token(&account.id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/report")
                .header("content-type", "application/json")
                .header("authorization", mock_auth_token)  // Mock auth token
                .body(Body::from(serde_json::to_string(&report_json).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    // Verify: Response successful
    assert_eq!(response.status(), StatusCode::OK);

    // Verify: Middleware properly loaded account
    // (Account extension was used by handler to access resources_db)

    // Verify: Data stored in database via the authenticated path
    let resources_db = account.resources_db().await.unwrap();
    let resources: Vec<Resource> = resources_db.select("resource").await.unwrap();
    assert_eq!(resources.len(), 3);
}

#[tokio::test]
async fn test_report_endpoint_rejects_invalid_auth() {
    setup_test_env();

    let app = crate::router::router();

    let report_json = create_test_report_request(1, 1);

    // Execute: POST with invalid/missing auth
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/report")
                .header("content-type", "application/json")
                // NO Authorization header
                .body(Body::from(serde_json::to_string(&report_json).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    // Verify: Auth middleware rejected request
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

**Required Test Helpers**:

```rust
// tests/common/auth.rs

/// Creates a mock auth token that bypasses ReportApiKeyAuth validation in tests
pub fn create_test_auth_token(account_id: &str) -> String {
    // Option 1: If ReportApiKey::validate_value has #[cfg(test)] bypass
    format!("test_token_{}", account_id)

    // Option 2: Actually generate valid encrypted key (more realistic)
    // Requires implementing ReportApiKey::generate_for_testing()
}
```

**Required Production Code Changes** (minimal, secure):

**Option A: Bypass in validate_value (simpler)**

```rust
// src/report_api_key.rs
impl ReportApiKey {
    pub async fn validate_value(value: &str) -> Result<(String, u32)> {
        // Test bypass: allows "test_token_{account_id}" format
        #[cfg(test)]
        if let Some(account_id) = value.strip_prefix("test_token_") {
            return Ok((account_id.to_string(), 99999));
        }

        // Production validation logic (unchanged)
        // ... existing SSM/KMS code ...
    }
}
```

**Option B: Test-specific constructor (more realistic)**

```rust
// src/report_api_key.rs
impl ReportApiKey {
    #[cfg(test)]
    pub fn generate_test_value_for_account(account_id: &str) -> String {
        // Returns a specially formatted token that validate_value recognizes in tests
        format!("test_token_{}", account_id)
    }
}
```

**What This Test Validates**:

1. ✅ **Auth middleware executes**: `ReportApiKeyAuth::authenticate` runs
2. ✅ **Auth extension injected**: Middleware adds `Extension<ReportApiKeyAuth>` to request
3. ✅ **Account middleware executes**: `report_api_key_account` runs
4. ✅ **Account loaded from DB**: Middleware queries accounts database
5. ✅ **Account extension injected**: Middleware adds `Extension<Account>` to request
6. ✅ **Handler receives extensions**: `report::report` extracts both extensions
7. ✅ **End-to-end flow**: Full request → auth → account → handler → DB → response

**Why This is Better Than Test 2**:

| Aspect | Test 2 (Mock Auth) | Test 3 (Real Auth Middleware) |
|--------|-------------------|-------------------------------|
| Router | Test-specific | Production router |
| Auth Middleware | Skipped (Extension injected directly) | **Executed** |
| Account Middleware | Skipped | **Executed** |
| Coverage | Handler only | **Full request path** |
| Realism | Medium | **High** |
| Complexity | Low | Medium |

**Security Note**: The test bypass (`test_token_` prefix) is:
- ✅ Only compiled in test builds (`#[cfg(test)]`)
- ✅ Never included in release binaries
- ✅ Easy to audit (single if statement)
- ✅ Fails fast in production (no test_ prefix in real tokens)

**Alternative: Environment Variable Bypass** (if preferred):

```rust
// src/report_api_key.rs
impl ReportApiKey {
    pub async fn validate_value(value: &str) -> Result<(String, u32)> {
        // Test bypass via env var (alternative approach)
        if std::env::var("ARCHODEX_TEST_MODE").is_ok() {
            if let Some(account_id) = value.strip_prefix("test_token_") {
                return Ok((account_id.to_string(), 99999));
            }
        }

        // Production validation...
    }
}
```

This approach:
- ✅ Works in both test and release builds
- ✅ Controlled via environment variable
- ⚠️ Requires discipline (must never set in production)
- ⚠️ Slightly less secure than `#[cfg(test)]`

**Recommendation**: Use **Option A with `#[cfg(test)]`** for maximum security.

---

## 7. Authentication Bypass for Integration Testing (New Section)

### Problem Analysis

**Current Authentication Architecture**:
- `DashboardAuth::authenticate` middleware: Validates Cognito JWT tokens (src/auth.rs:88-163)
- `ReportApiKeyAuth::authenticate` middleware: Validates encrypted API keys via SSM/KMS (src/auth.rs:196-228)
- All protected routes wrapped with these middleware layers (src/router.rs:44-77)

**Testing Challenges**:
1. JWT tokens require valid Cognito user pool + JWKS endpoint
2. Report API keys require AWS SSM parameter store + KMS decryption
3. Both approaches require external AWS services unavailable in tests

### Solution: Test-Specific Constructors (Minimal Production Code Changes)

**Design Pattern**: Add `#[cfg(test)]`-gated constructors to auth types, allowing tests to bypass authentication without compromising production security.

####Required Production Code Modifications

**File**: `src/auth.rs`

```rust
impl DashboardAuth {
    // ... existing authenticate method ...

    #[cfg(test)]
    pub(crate) fn new_for_testing(principal: User) -> Self {
        Self { principal }
    }
}

impl ReportApiKeyAuth {
    // ... existing authenticate method ...

    #[cfg(test)]
    pub(crate) fn new_for_testing(account_id: String, key_id: u32) -> Self {
        Self { account_id, key_id }
    }
}
```

**File**: `src/account.rs` (if needed)

```rust
impl Account {
    #[cfg(test)]
    pub(crate) fn new_for_testing(id: String) -> Self {
        Self { id }
    }
}
```

**Security Guarantees**:
- ✅ `#[cfg(test)]` ensures code only compiled in test builds
- ✅ `pub(crate)` restricts visibility to crate (not public API)
- ✅ Never included in release binaries
- ✅ Production authentication paths unchanged

### Test Router Pattern

**File**: `tests/common/test_router.rs`

```rust
use axum::{Router, routing::{get, post}, Extension};
use crate::{auth::{DashboardAuth, ReportApiKeyAuth}, account::Account, user::User};

/// Creates test router bypassing auth middleware via Extension injection
pub fn create_test_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "Ok" }))
}

/// Creates authenticated test router with mock DashboardAuth
pub fn create_dashboard_authed_test_router(user_id: &str, account_id: &str) -> Router {
    let mock_auth = DashboardAuth::new_for_testing(User::new(user_id));
    let mock_account = Account::new_for_testing(account_id.to_string());

    Router::new()
        .route("/test/accounts", get(accounts::list_accounts))
        .layer(Extension(mock_auth))
        .layer(Extension(mock_account))
}

/// Creates authenticated test router with mock ReportApiKeyAuth
pub fn create_report_key_authed_test_router(account_id: &str) -> Router {
    let mock_auth = ReportApiKeyAuth::new_for_testing(account_id.to_string(), 12345);
    let mock_account = Account::new_for_testing(account_id.to_string());

    Router::new()
        .route("/test/report", post(report::report))
        .layer(Extension(mock_auth))
        .layer(Extension(mock_account))
}
```

### Integration Test Patterns

**Pattern 1: Unauthed endpoint** (e.g., /health):

```rust
#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_router();
    let response = app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

**Pattern 2: Dashboard-authed endpoint** (e.g., /accounts):

```rust
#[tokio::test]
async fn test_list_accounts() {
    let app = create_dashboard_authed_test_router("user123", "account456");
    let response = app.oneshot(Request::builder().uri("/test/accounts").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

**Pattern 3: Report-key-authed endpoint** (e.g., /report):

```rust
#[tokio::test]
async fn test_report_ingestion() {
    let (db, _account) = create_test_db_with_account("test_account").await;
    let app = create_report_key_authed_test_router("test_account");

    let report_json = create_test_report_request(3, 5);
    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/test/report")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&report_json).unwrap()))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

### Alternatives Considered and Rejected

**Alternative 1: Mock JWKS Server**
- ❌ Complex setup (requires HTTP server, JWT signing)
- ❌ Slow (network overhead)
- ❌ Tests authentication logic unnecessarily
- ❌ Fragile (version incompatibilities)

**Alternative 2: Mock AWS SSM/KMS**
- ❌ Requires moto or LocalStack
- ❌ Slow container startup
- ❌ Complex credential management
- ❌ Tests encryption unnecessarily

**Alternative 3: Feature Flags for Auth Bypass**
- ❌ Risk of accidentally deploying with auth disabled
- ❌ Production code polluted with test logic
- ❌ Difficult to audit security

### Recommendation Rationale

**Why Test-Specific Constructors Win**:
1. **Minimal Surface Area**: Only adds 2-3 small functions
2. **Compile-Time Safety**: `#[cfg(test)]` guarantees no production impact
3. **Fast Execution**: No network, no AWS, no containers
4. **Maintainable**: Clear pattern, easy to extend
5. **Secure**: Cannot accidentally deploy test code

**Implementation Permission Granted**: User has approved modifying production code (`src/auth.rs`, `src/account.rs`) to add `#[cfg(test)]`-gated constructors for testing purposes.

---

## 8. Documentation Resources

### SurrealDB Testing
- Official Docs: https://surrealdb.com/docs/surrealdb/embedding/rust
- In-Memory Guide: https://rust.code-maven.com/surrealdb-embedded-with-in-memory-database
- SDK API Reference: https://docs.rs/surrealdb/latest/surrealdb/engine/local/

### Rust Testing
- The Rust Book: https://doc.rust-lang.org/book/ch11-00-testing.html
- Tokio Testing: https://tokio.rs/tokio/topics/testing
- Axum Testing Example: https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs

### CI/CD
- GitHub Actions Rust: https://docs.github.com/en/actions/use-cases-and-examples/building-and-testing/building-and-testing-rust
- setup-rust-toolchain: https://github.com/actions-rust-lang/setup-rust-toolchain

---

## 8. Decisions and Rationale

### Decision 1: Use In-Memory SurrealDB as Primary Approach

**Rationale**:
- SurrealDB maintains API parity across backends (confirmed by fork's test suite)
- Zero infrastructure dependencies (aligns with Constitution's simplicity principle)
- Fast feedback loop (<30s test execution)
- Works perfectly in GitHub Actions
- Can add testcontainers later if needed (not over-engineering)

**Trade-offs Accepted**:
- Cannot test DynamoDB-specific behaviors (acceptable - fork maintains parity)
- Cannot test network failures (acceptable - not common failure mode for embedded use)

### Decision 2: Use Minimal Tooling (cargo test + tokio::test + oneshot)

**Rationale**:
- Leverages existing dependencies (no new libraries to learn)
- Fast setup time (aligns with startup speed priority)
- Easy to understand and maintain
- Can incrementally add libraries as test suite grows

**Trade-offs Accepted**:
- More verbose test setup initially (acceptable - 20-30 tests is manageable)
- Less sophisticated fixture management (can add `rstest` later if needed)

### Decision 3: Factory Functions for Test Data (Phase 1)

**Rationale**:
- Simplest pattern (lowest learning curve)
- Sufficient for small test suite (20-30 tests)
- Easy to migrate to builder pattern later if needed

**Trade-offs Accepted**:
- Less flexible than builders (acceptable for small suite)

### Decision 4: Standard GitHub Actions Workflow

**Rationale**:
- Industry standard for Rust projects
- Free for public repositories
- Automatic caching via `setup-rust-toolchain`
- ACT-compatible for local testing

**Trade-offs Accepted**:
- None - this is the optimal choice

---

## 9. Open Questions Resolved

### Q1: Testing Approach Selection
**Answer**: Hybrid approach with 90% in-memory tests, 10% optional server tests

### Q2: SurrealDB Testing Strategy
**Answer**: Use `kv-mem` (in-memory mode) - SurrealDB guarantees API parity across backends

### Q3: Framework Selection
**Answer**: Minimal tooling (cargo test + tokio::test + oneshot) with incremental additions

### Q4: Test Data Management
**Answer**: Factory functions initially, migrate to builders if test suite grows >50 tests

### Q5: CI Environment
**Answer**: GitHub Actions with standard Rust workflow

---

## 10. Next Steps (Phase 1 Implementation)

1. **Add minimal dev-dependencies** to Cargo.toml
2. **Create `tests/common/` structure** with helper modules
3. **Implement test database setup** using in-memory SurrealDB
4. **Write 2 example tests** (resource ingestion + API key)
5. **Create GitHub Actions workflow**
6. **Document testing patterns** in `tests/common/README.md`
7. **Verify CI execution** with passing tests

**Timeline**: 2-3 days (as per spec Phase 1)

---

## Conclusion

Research confirms that **SurrealDB in-memory mode with minimal Rust testing tooling** is the optimal approach for Archodex backend testing at current project stage. This approach:

- ✅ Aligns with Constitution's anti-over-engineering principle
- ✅ Provides fast feedback loop for TDD
- ✅ Requires zero infrastructure setup
- ✅ Works perfectly in GitHub Actions CI
- ✅ Maintains high production fidelity (SurrealDB API parity confirmed)
- ✅ Allows incremental complexity addition as test suite grows

All "NEEDS CLARIFICATION" items from Technical Context are now resolved and ready for Phase 1 implementation.
