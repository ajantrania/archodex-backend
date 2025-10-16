# Research: Axum Middleware Database Dependency Injection

**Feature**: 003-db-dependency-injection
**Date**: 2025-10-16
**Status**: Research Complete

---

## Executive Summary

This research determines how Axum middleware (`dashboard_auth_account` and `report_api_key_account` in `src/db.rs`) can support dependency injection for database connections while maintaining backward compatibility with production code. The investigation covers Axum extension patterns, dependency injection strategies, and middleware architecture.

**Key Decision: Use Request Extensions to Carry Optional Injected Database Connections**

**Rationale**:
- Middleware can check for injected database connections before falling back to global connections
- Zero production impact—production code path never checks for test extensions
- Maintains existing middleware signatures and behavior
- Enables test isolation without #[cfg(test)] guards in production paths
- Aligns with Rust's explicit dependency injection philosophy

---

## 1. Current Middleware Architecture

### Middleware Flow

Production `/report` endpoint has **two middleware layers**:

```rust
// src/router.rs:74-77
Router::new()
    .route("/report", post(report::report))
    .layer(middleware::from_fn(report_api_key_account))      // Layer 2: Loads Account
    .layer(middleware::from_fn(ReportApiKeyAuth::authenticate)); // Layer 1: Auth
```

**Request Flow**:
1. `ReportApiKeyAuth::authenticate` → Validates auth header → Injects `Extension<ReportApiKeyAuth>`
2. `report_api_key_account` → Loads account from DB → Injects `Extension<Account>`
3. `report::report` handler → Extracts `Extension<Account>` → Calls `account.resources_db()`

### Current Middleware Implementation

**File**: `src/db.rs:295-319`

```rust
pub(crate) async fn report_api_key_account(
    Extension(auth): Extension<ReportApiKeyAuth>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let account = accounts_db()              // ← Uses global connection
        .await?
        .get_account_by_id(auth.account_id().to_owned())
        .await?
        .check_first_real_error()?
        .take::<Option<Account>>(0)
        .context("Failed to get account record")?;

    let Some(account) = account else {
        not_found!("Account not found");
    };

    auth.validate_account_access(&*(account.resources_db().await?))
        .await?;

    req.extensions_mut().insert(account);    // ← Injects Account into request

    Ok(next.run(req).await)
}
```

### Current Account Implementation

**File**: `src/account.rs:147-161`

```rust
pub(crate) async fn resources_db(&self) -> anyhow::Result<DBConnection> {
    #[cfg(not(feature = "archodex-com"))]
    let service_data_surrealdb_url = Env::surrealdb_url();
    #[cfg(feature = "archodex-com")]
    let Some(service_data_surrealdb_url) = &self.service_data_surrealdb_url else {
        bail!("No service data SurrealDB URL configured for account {}", self.id);
    };

    resources_db(service_data_surrealdb_url, &self.id).await  // ← Uses global connection
}
```

**Key Constraint**: `Account` is stored in request extensions and extracted by handlers. The Account itself must carry any injected database connections.

---

## 2. Axum Extension Pattern Research

### How Request Extensions Work

**Pattern**: Axum's `Extension` system allows middleware to attach arbitrary data to requests:

```rust
// In middleware
req.extensions_mut().insert(some_data);

// In handler
Extension(some_data): Extension<SomeType>
```

**Requirements**:
- Data must implement `Clone` trait
- Each type can only be inserted once per request (keyed by type)
- Extraction fails with 500 error if type not found

### State vs Extensions

| Aspect | State | Extensions |
|--------|-------|------------|
| Scope | Global per router | Per-request |
| Timing | Set at router creation | Set during request processing |
| Use Case | Shared resources (pools) | Request-specific data |
| Type Safety | Compile-time | Runtime (500 on missing) |

**Conclusion**: Extensions are the correct choice for passing request-specific data like injected database connections.

---

## 3. Dependency Injection Patterns in Rust

### Pattern 1: Constructor Injection (Standard Approach)

**Description**: Pass dependencies explicitly through constructors or factory methods.

```rust
struct Service {
    db: Arc<Database>,
}

impl Service {
    fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}
```

**Pros**: Explicit, type-safe, no hidden dependencies
**Cons**: Requires refactoring existing code

### Pattern 2: Optional Injection with Fallback

**Description**: Accept optional dependencies and fall back to global state if not provided.

```rust
struct Service {
    db: Option<Arc<Database>>,
}

impl Service {
    fn get_db(&self) -> Arc<Database> {
        self.db.clone().unwrap_or_else(|| global_db())
    }
}
```

**Pros**: Backward compatible, gradual migration path
**Cons**: Still has global state dependency

### Pattern 3: Request-Scoped Injection via Extensions (Selected)

**Description**: Use Axum request extensions to carry optional injected dependencies.

```rust
// Test code injects database connection
req.extensions_mut().insert(TestDatabaseConnection(db));

// Middleware/handler checks for injection before using global
let db = req.extensions()
    .get::<TestDatabaseConnection>()
    .map(|conn| conn.0.clone())
    .unwrap_or_else(|| global_db());
```

**Pros**:
- Zero production impact (production never checks for test extensions)
- No refactoring of existing code paths
- Clear separation of test vs production behavior
- Works with existing middleware signatures

**Cons**:
- Runtime checking (minimal overhead)
- Requires coordination between test setup and middleware

---

## 4. Decision: How Middleware Should Handle Injected vs Global Connections

### Selected Approach: Extension-Based Optional Database Injection

**Strategy**: Use request extensions to optionally carry injected database connections. Middleware and handlers check for these test-specific extensions before falling back to global connections.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Test Setup                                                   │
│ • Creates in-memory database                                 │
│ • Wraps in TestAccountsDb/TestResourcesDb extension type    │
│ • Injects via layer(Extension(test_db))                     │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Middleware: report_api_key_account / dashboard_auth_account │
│ 1. Extract Extension<ReportApiKeyAuth>                      │
│ 2. Check req.extensions().get::<TestAccountsDb>()          │
│ 3. If found: Use injected connection                        │
│ 4. If not found: Use accounts_db() (production path)       │
│ 5. Load Account from database                               │
│ 6. Create Account with optional injected resources DB       │
│ 7. Insert Extension(account) into request                   │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Handler: report::report                                      │
│ 1. Extract Extension(account)                               │
│ 2. Call account.resources_db()                              │
│ 3. Account checks for injected DB before using global       │
└─────────────────────────────────────────────────────────────┘
```

### Implementation Sketch

**Step 1: Define Test-Only Extension Types**

```rust
// src/db.rs (or tests/common/mod.rs with pub(crate) re-export)

/// Test-only extension type for injecting accounts database connection
/// Only used in test code, never in production
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct TestAccountsDb(pub(crate) DBConnection);

/// Test-only extension type for injecting resources database connection
/// Only used in test code, never in production
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct TestResourcesDb(pub(crate) DBConnection);
```

**Step 2: Modify Middleware to Check for Injected Connections**

```rust
// src/db.rs:295-319 (modified)

pub(crate) async fn report_api_key_account(
    Extension(auth): Extension<ReportApiKeyAuth>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    // Check for test-injected accounts database
    #[cfg(test)]
    let accounts_db = if let Some(TestAccountsDb(db)) = req.extensions().get::<TestAccountsDb>() {
        db.clone()
    } else {
        accounts_db().await?
    };

    #[cfg(not(test))]
    let accounts_db = accounts_db().await?;

    let account = accounts_db
        .get_account_by_id(auth.account_id().to_owned())
        .await?
        .check_first_real_error()?
        .take::<Option<Account>>(0)
        .context("Failed to get account record")?;

    let Some(mut account) = account else {
        not_found!("Account not found");
    };

    // Check for test-injected resources database
    #[cfg(test)]
    if let Some(TestResourcesDb(db)) = req.extensions().get::<TestResourcesDb>() {
        account.inject_resources_db(db.clone());
    }

    auth.validate_account_access(&*(account.resources_db().await?))
        .await?;

    req.extensions_mut().insert(account);

    Ok(next.run(req).await)
}
```

**Step 3: Modify Account to Support Injected Resources DB**

```rust
// src/account.rs:15-35 (modified struct)

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Account {
    #[serde(deserialize_with = "surrealdb_deserializers::string::deserialize")]
    id: String,
    #[cfg(feature = "archodex-com")]
    endpoint: String,
    #[cfg(feature = "archodex-com")]
    service_data_surrealdb_url: Option<String>,
    #[serde(deserialize_with = "surrealdb_deserializers::bytes::deserialize")]
    salt: Vec<u8>,
    #[cfg(not(feature = "archodex-com"))]
    #[serde(default, deserialize_with = "...")]
    api_private_key: Option<Vec<u8>>,
    created_at: Option<DateTime<Utc>>,
    created_by: Option<User>,
    deleted_at: Option<DateTime<Utc>>,
    deleted_by: Option<User>,

    // NEW: Test-only field for injected database connection
    #[cfg(test)]
    #[serde(skip)]
    injected_resources_db: Option<DBConnection>,
}
```

```rust
// src/account.rs:147-161 (modified method)

pub(crate) async fn resources_db(&self) -> anyhow::Result<DBConnection> {
    // Check for injected connection first (test path)
    #[cfg(test)]
    if let Some(ref db) = self.injected_resources_db {
        return Ok(db.clone());
    }

    // Production path (unchanged)
    #[cfg(not(feature = "archodex-com"))]
    let service_data_surrealdb_url = Env::surrealdb_url();
    #[cfg(feature = "archodex-com")]
    let Some(service_data_surrealdb_url) = &self.service_data_surrealdb_url else {
        bail!("No service data SurrealDB URL configured for account {}", self.id);
    };

    resources_db(service_data_surrealdb_url, &self.id).await
}

/// Injects a test database connection into this account
/// Only available in test builds
#[cfg(test)]
pub(crate) fn inject_resources_db(&mut self, db: DBConnection) {
    self.injected_resources_db = Some(db);
}
```

**Step 4: Test Setup Helper**

```rust
// tests/common/db.rs

use archodex_backend::db::{TestAccountsDb, TestResourcesDb};

pub async fn create_test_router_with_injected_dbs() -> Router {
    // Create in-memory test databases
    let accounts_db = create_test_accounts_db().await;
    let resources_db = create_test_resources_db().await;

    // Use production router with injected database extensions
    archodex_backend::router::router()
        .layer(Extension(TestAccountsDb(accounts_db)))
        .layer(Extension(TestResourcesDb(resources_db)))
}
```

### Production Path Verification

**Production Build** (`cargo build --release`):
- `#[cfg(test)]` blocks are completely removed by compiler
- No `TestAccountsDb` or `TestResourcesDb` types exist
- No runtime checks for test extensions
- Identical performance to current implementation
- **Zero overhead, zero risk**

**Test Build** (`cargo test`):
- `#[cfg(test)]` blocks are included
- Middleware checks `req.extensions()` for test database types
- If found: uses injected connection
- If not found: falls back to production path (global connection)
- **Backward compatible with existing tests that don't inject databases**

---

## 5. Alternatives Considered

### Alternative 1: Trait-Based Dependency Injection

**Approach**: Define a trait for database access and use generics/trait objects.

```rust
trait DatabaseProvider {
    async fn accounts_db(&self) -> Result<DBConnection>;
    async fn resources_db(&self, account_id: &str) -> Result<DBConnection>;
}

struct GlobalDatabaseProvider;
struct TestDatabaseProvider { /* ... */ }

pub(crate) async fn report_api_key_account<P: DatabaseProvider>(
    db_provider: Extension<P>,
    // ...
) -> Result<Response> {
    let db = db_provider.accounts_db().await?;
    // ...
}
```

**Pros**:
- Clean separation of concerns
- More "Rust idiomatic" for new code
- Compile-time polymorphism (zero-cost with generics)

**Cons**:
- **Major refactoring required**: All middleware signatures change
- **Breaking change**: Existing handler code needs updates
- **Complex migration**: Requires changing router setup, middleware registration, etc.
- **Over-engineering**: Too heavy for the problem (testing support only)
- **Violates requirement**: Must maintain existing middleware signatures

**Verdict**: ❌ **Rejected** - Requires too much refactoring, doesn't meet backward compatibility requirement

---

### Alternative 2: Axum State for Database Injection

**Approach**: Use `State` extractor to pass database connections.

```rust
#[derive(Clone)]
struct AppState {
    accounts_db: Option<DBConnection>,
    resources_db: Option<DBConnection>,
}

pub(crate) async fn report_api_key_account(
    State(state): State<AppState>,
    Extension(auth): Extension<ReportApiKeyAuth>,
    // ...
) -> Result<Response> {
    let db = state.accounts_db
        .clone()
        .unwrap_or_else(|| accounts_db().await?);
    // ...
}
```

**Pros**:
- Type-safe at router construction time
- Clear dependency declaration
- More performant than runtime extension checks

**Cons**:
- **Breaking change**: All middleware must add `State` parameter
- **Global per router**: Can't have different databases per request (kills test isolation)
- **Requires router refactoring**: All route registration must pass state
- **Doesn't support per-request injection**: Tests can't inject different DBs for different requests

**Verdict**: ❌ **Rejected** - Doesn't support per-request test isolation, requires too much refactoring

---

### Alternative 3: Global Test Mode Flag

**Approach**: Use a global flag to switch between test and production database providers.

```rust
static TEST_MODE: AtomicBool = AtomicBool::new(false);
static TEST_DB: OnceCell<Mutex<DBConnection>> = OnceCell::new();

pub(crate) async fn accounts_db() -> Result<DBConnection> {
    if TEST_MODE.load(Ordering::Relaxed) {
        Ok(TEST_DB.get().unwrap().lock().await.clone())
    } else {
        // Production path
    }
}
```

**Pros**:
- No middleware changes required
- Simple implementation

**Cons**:
- **Violates Rust philosophy**: Hidden global state is an anti-pattern
- **Test isolation impossible**: All tests share same global database
- **Race conditions**: Parallel tests interfere with each other
- **Fragile**: Forgetting to reset flag causes test pollution
- **Violates requirement**: Spec explicitly requires "test isolation—each test gets independent database connections"

**Verdict**: ❌ **Rejected** - Violates Rust idioms and test isolation requirements

---

### Alternative 4: Conditional Compilation with Test Middleware

**Approach**: Create separate middleware implementations for testing.

```rust
#[cfg(not(test))]
pub(crate) async fn report_api_key_account(/* ... */) {
    let db = accounts_db().await?;  // Production
    // ...
}

#[cfg(test)]
pub(crate) async fn report_api_key_account(/* ... */) {
    let db = /* get from somewhere */;  // Test
    // ...
}
```

**Cons**:
- **Code duplication**: Maintaining two versions of same middleware
- **Drift risk**: Easy to forget updating both versions
- **Still need injection mechanism**: Doesn't solve how tests provide the database

**Verdict**: ❌ **Rejected** - Code duplication, maintenance burden

---

## 6. Why Extension-Based Approach Wins

### Comparison Matrix

| Criterion | Extension-Based | Trait-Based | State-Based | Global Flag |
|-----------|----------------|-------------|-------------|-------------|
| Zero production impact | ✅ `#[cfg(test)]` only | ❌ Changes signatures | ❌ Adds State param | ⚠️ Runtime check |
| Backward compatible | ✅ No breaking changes | ❌ All middleware changes | ❌ Router changes | ✅ No changes |
| Test isolation | ✅ Per-request injection | ✅ Per-request possible | ❌ Global per router | ❌ Global shared |
| Maintenance burden | ✅ Minimal | ❌ High (refactoring) | ⚠️ Medium | ✅ Low |
| Rust idiomatic | ✅ Yes (explicit DI) | ✅ Yes (trait based) | ✅ Yes (State) | ❌ No (global state) |
| Compile-time safety | ⚠️ Runtime extension check | ✅ Compile-time | ✅ Compile-time | ❌ Runtime check |
| Migration complexity | ✅ Minimal | ❌ High | ⚠️ Medium | ✅ None |

**Winner**: **Extension-Based Approach**

### Key Benefits

1. **Zero Production Impact**: All test-specific code is behind `#[cfg(test)]`, completely removed from release builds
2. **Minimal Changes**: Only touches middleware logic, no signature changes
3. **Perfect Test Isolation**: Each test can inject independent database connections via router layers
4. **Backward Compatible**: Existing tests continue working (fall back to global connections if no injection)
5. **Clear Intent**: Test database extension types (`TestAccountsDb`, `TestResourcesDb`) make injection explicit
6. **Maintainable**: Small, localized changes to 2 middleware functions and Account struct
7. **Rust Idiomatic**: Explicit dependency injection via request extensions, no hidden global state

---

## 7. Implementation Roadmap

### Phase 1: Core Infrastructure (Minimal Changes)

**Files Modified**:
- `src/db.rs`: Add test extension types, modify 2 middleware functions
- `src/account.rs`: Add optional injected DB field, modify `resources_db()` method

**Changes**:
```rust
// src/db.rs
#[cfg(test)]
pub(crate) struct TestAccountsDb(pub(crate) DBConnection);
#[cfg(test)]
pub(crate) struct TestResourcesDb(pub(crate) DBConnection);

// Modify dashboard_auth_account and report_api_key_account to check for injected DBs

// src/account.rs
#[cfg(test)]
injected_resources_db: Option<DBConnection>,

#[cfg(test)]
pub(crate) fn inject_resources_db(&mut self, db: DBConnection) { /* ... */ }

pub(crate) async fn resources_db(&self) -> anyhow::Result<DBConnection> {
    #[cfg(test)]
    if let Some(ref db) = self.injected_resources_db {
        return Ok(db.clone());
    }
    // Existing production code...
}
```

**Effort**: 2-3 hours
**Risk**: Very low (all changes behind `#[cfg(test)]`)

### Phase 2: Test Helpers

**Files Created**:
- `tests/common/db.rs`: Database creation helpers
- `tests/common/router.rs`: Router creation with injected DBs

**Functions**:
```rust
pub async fn create_test_accounts_db() -> DBConnection { /* ... */ }
pub async fn create_test_resources_db() -> DBConnection { /* ... */ }
pub fn create_test_router_with_injected_dbs(
    accounts_db: DBConnection,
    resources_db: DBConnection,
) -> Router { /* ... */ }
```

**Effort**: 2-3 hours
**Risk**: Very low (test-only code)

### Phase 3: Example Test Implementation

**File**: `tests/report_with_auth_test.rs`

**Test**:
```rust
#[tokio::test]
async fn test_report_ingestion_with_database_validation() {
    let accounts_db = create_test_accounts_db().await;
    let resources_db = create_test_resources_db().await;

    // Setup: Create test account in accounts DB
    let account = create_test_account(&accounts_db, "test_account_123").await;

    // Setup: Create test auth
    let auth_token = create_test_auth_token(&account);

    // Create router with injected databases
    let app = archodex_backend::router::router()
        .layer(Extension(TestAccountsDb(accounts_db)))
        .layer(Extension(TestResourcesDb(resources_db.clone())));

    // Execute: POST /report
    let response = app.oneshot(/* ... */).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify: Query injected resources DB
    let resources: Vec<Resource> = resources_db
        .select("resource")
        .await
        .unwrap();
    assert_eq!(resources.len(), 3);
}
```

**Effort**: 3-4 hours
**Risk**: Low (validates entire approach)

---

## 8. Extension Type Design Considerations

### Why Separate Extension Types?

**Question**: Why `TestAccountsDb` and `TestResourcesDb` instead of reusing `DBConnection` directly?

**Answer**: Type-safety and explicit intent.

```rust
// ❌ Bad: Ambiguous which database this is
req.extensions_mut().insert(db_connection);

// ✅ Good: Crystal clear this is the test accounts database
req.extensions_mut().insert(TestAccountsDb(db_connection));
```

**Benefits**:
- **Type Safety**: Can't accidentally mix up accounts DB and resources DB
- **Explicit Intent**: Code clearly shows "this is a test database injection"
- **Future-Proof**: Can add methods or metadata to wrapper types if needed
- **Search-Friendly**: Easy to find all test database injection points

### Wrapper Type Pattern

```rust
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct TestAccountsDb(pub(crate) DBConnection);

#[cfg(test)]
impl TestAccountsDb {
    /// Creates a test accounts database with standard schema
    pub async fn new() -> anyhow::Result<Self> {
        let db = create_in_memory_db().await?;
        migrate_accounts_database(&db).await?;
        Ok(Self(db))
    }
}
```

**Future Extensions** (if needed):
```rust
#[cfg(test)]
impl TestAccountsDb {
    /// Seeds the database with a test account
    pub async fn with_account(&self, id: &str) -> anyhow::Result<Account> {
        // ...
    }

    /// Returns the underlying connection for direct queries
    pub fn connection(&self) -> &DBConnection {
        &self.0
    }
}
```

---

## 9. How Extension Carrying Works in Detail

### Question: How Does Account Extension Carry Injected DB?

**Current Flow**:
```
Middleware → Creates Account → Inserts into request → Handler extracts Account
```

**With Injection**:
```
Test Layer → Injects TestResourcesDb extension
     ↓
Middleware → Extracts TestResourcesDb from request.extensions()
     ↓
Middleware → Loads Account from DB
     ↓
Middleware → Calls account.inject_resources_db(test_db)
     ↓
Middleware → Inserts Account (with injected DB) into request
     ↓
Handler → Extracts Extension(account)
     ↓
Handler → Calls account.resources_db() → Returns injected DB
```

### Key Insight: Account is Modified Before Insertion

```rust
// In middleware (after loading account from DB):
let mut account = accounts_db
    .get_account_by_id(auth.account_id().to_owned())
    .await?
    // ... error handling ...

// INJECT the test database into the account
#[cfg(test)]
if let Some(TestResourcesDb(db)) = req.extensions().get::<TestResourcesDb>() {
    account.inject_resources_db(db.clone());  // ← Modifies account
}

// NOW insert the modified account into request
req.extensions_mut().insert(account);  // ← Account carries injected DB
```

**Handler receives Account with injected DB already inside it**:
```rust
pub(crate) async fn report(
    Extension(account): Extension<Account>,  // ← Already has injected DB
    Json(req): Json<Request>,
) -> Result<()> {
    let db = account.resources_db().await?;  // ← Returns injected DB
    // ...
}
```

---

## 10. Production Safety Verification

### Compile-Time Guarantees

**Production Build** (`cargo build --release`):
```rust
// This code:
#[cfg(test)]
pub(crate) struct TestAccountsDb(pub(crate) DBConnection);

#[cfg(test)]
if let Some(TestResourcesDb(db)) = req.extensions().get::<TestResourcesDb>() {
    account.inject_resources_db(db.clone());
}

// Becomes this (completely removed):
// [empty]
```

**Verification Steps**:
1. Compile release binary: `cargo build --release --verbose`
2. Examine binary symbols: No `TestAccountsDb` or `TestResourcesDb` symbols
3. Disassemble middleware: No extension checking code path
4. Performance test: Identical latency to current implementation

### Test-Only Visibility

```rust
#[cfg(test)]
pub(crate) struct TestAccountsDb(pub(crate) DBConnection);
//           ^^^^^^^^^^^
//           Only visible within crate, only in test builds
```

**Cannot be used in**:
- Production builds (removed by compiler)
- External crates (pub(crate) restriction)
- Accidental misuse (type doesn't exist in release mode)

---

## 11. Backward Compatibility Analysis

### Existing Tests Continue Working

**Scenario**: Test that uses production router without injecting databases.

```rust
#[tokio::test]
async fn existing_test() {
    let app = archodex_backend::router::router();  // No injection

    let response = app.oneshot(/* ... */).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

**Behavior**:
1. Middleware checks `req.extensions().get::<TestAccountsDb>()`
2. Returns `None` (no injection)
3. Falls back to `accounts_db().await?` (production path)
4. **Identical behavior to before changes**

### New Tests Can Inject Databases

**Scenario**: New test that injects in-memory databases.

```rust
#[tokio::test]
async fn new_test_with_injection() {
    let accounts_db = create_test_accounts_db().await;
    let resources_db = create_test_resources_db().await;

    let app = archodex_backend::router::router()
        .layer(Extension(TestAccountsDb(accounts_db)))
        .layer(Extension(TestResourcesDb(resources_db.clone())));

    // Test can now query resources_db to verify data
}
```

**Behavior**:
1. Middleware checks `req.extensions().get::<TestAccountsDb>()`
2. Returns `Some(test_db)` (found injection)
3. Uses injected database instead of global connection
4. **New capability, zero impact on existing tests**

---

## 12. Open Questions Resolved

### Q1: How do tests inject both accounts DB and resources DB?

**Answer**: Via router layer extensions before middleware runs.

```rust
Router::new()
    .route("/report", post(report::report))
    .layer(Extension(TestAccountsDb(accounts_db)))    // Injected first
    .layer(Extension(TestResourcesDb(resources_db)))  // Injected first
    .layer(middleware::from_fn(report_api_key_account))  // Runs after, can extract
```

### Q2: What happens if only accounts DB is injected but not resources DB?

**Answer**: Middleware will use injected accounts DB, but Account.resources_db() will fall back to global connection.

**Behavior**:
- Middleware loads account from injected accounts DB ✅
- Account has no injected resources DB
- `account.resources_db()` returns global connection ⚠️
- **This is fine for tests that only need to control accounts DB**

### Q3: Can middleware signature remain unchanged?

**Answer**: Yes, completely unchanged.

```rust
// Before and after: IDENTICAL SIGNATURE
pub(crate) async fn report_api_key_account(
    Extension(auth): Extension<ReportApiKeyAuth>,
    mut req: Request,
    next: Next,
) -> Result<Response>
```

**All changes are internal to the function body.**

### Q4: Does this work with both dashboard_auth_account and report_api_key_account?

**Answer**: Yes, identical pattern for both.

Both middleware:
1. Check for `TestAccountsDb` extension
2. Use injected DB or fall back to global
3. Load Account from database
4. Check for `TestResourcesDb` extension
5. Inject into Account if found
6. Insert Account into request extensions

### Q5: How does DBConnection cloning work?

**Answer**: `DBConnection` is an enum that wraps `Surreal<Any>`, which implements `Clone`.

```rust
// From src/db.rs:152-168
pub(crate) enum DBConnection {
    #[cfg(feature = "rocksdb")]
    Nonconcurrent(tokio::sync::MappedMutexGuard<'static, Surreal<Any>>),
    Concurrent(Surreal<Any>),  // ← Surreal<Any> implements Clone
}
```

**For in-memory test databases**:
- Uses `Surreal<Mem>` (memory backend)
- Implements `Clone` via Arc internally
- Multiple clones share same underlying in-memory database
- **Perfect for tests: all clones see same data**

---

## 13. Rationale Summary

### Why This Approach is Optimal

**1. Zero Production Impact**
- All test-specific code behind `#[cfg(test)]`
- Compiler removes completely from release builds
- No runtime overhead, no risk

**2. Minimal Changes**
- Two middleware functions: ~20 lines added
- One Account method: ~5 lines added
- No signature changes, no breaking changes

**3. Maximum Flexibility**
- Tests can inject both databases
- Tests can inject only one database
- Tests can inject nothing (existing behavior)
- Per-request isolation via extensions

**4. Rust Idiomatic**
- Explicit dependency injection (no hidden globals)
- Type-safe wrapper types for clarity
- Uses Axum's extension pattern correctly
- Aligns with Rust philosophy of explicit over implicit

**5. Maintainable**
- Clear separation of test vs production code
- Easy to understand and audit
- Search-friendly (`#[cfg(test)]` + `Test*` types)
- Future-proof (can extend without breaking changes)

**6. Testable**
- Enables complete integration testing
- Database state verification
- Test isolation (parallel execution safe)
- No test pollution or race conditions

---

## 14. Next Steps

### Implementation Tasks

**Task 1**: Add test extension types to `src/db.rs`
- `TestAccountsDb` and `TestResourcesDb` wrapper types
- Compile guard with `#[cfg(test)]`
- Derive `Clone` for extension compatibility

**Task 2**: Modify middleware functions
- `dashboard_auth_account`: Check for injected accounts DB
- `report_api_key_account`: Check for injected accounts DB and resources DB
- Fall back to global connections if not found

**Task 3**: Extend Account struct
- Add `injected_resources_db: Option<DBConnection>` field (test-only)
- Implement `inject_resources_db()` method (test-only)
- Modify `resources_db()` to check for injection before using global

**Task 4**: Create test helpers
- `create_test_accounts_db()` - In-memory accounts database
- `create_test_resources_db()` - In-memory resources database
- `create_test_router_with_dbs()` - Router with injected databases

**Task 5**: Write example test
- Full integration test with database validation
- Verify injection works end-to-end
- Document pattern for future tests

**Estimated Effort**: 1-2 days total

---

## Conclusion

**Decision**: Use request extensions to carry optional injected database connections. Middleware checks for test-specific extension types (`TestAccountsDb`, `TestResourcesDb`) before falling back to global connections.

**Key Properties**:
- ✅ Zero production impact (`#[cfg(test)]` compilation guards)
- ✅ Backward compatible (existing middleware signatures unchanged)
- ✅ Test isolation (per-request injection via extensions)
- ✅ Minimal maintenance burden (~50 lines of code)
- ✅ Rust idiomatic (explicit dependency injection)
- ✅ Future-proof (can extend without breaking changes)

**This approach unblocks comprehensive integration testing (T036 from 002-specs-001-rate) while maintaining all existing production behavior and security boundaries.**
