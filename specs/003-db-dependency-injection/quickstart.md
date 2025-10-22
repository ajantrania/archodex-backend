# Quickstart: Database Dependency Injection for Testing

**Feature**: 003-db-dependency-injection
**Audience**: Developers writing integration tests for Archodex backend
**Time to Complete**: 5-10 minutes

## What You'll Learn

How to write integration tests that inject in-memory databases to validate complete request flows including database state verification.

## Prerequisites

- Rust development environment configured
- Archodex backend repository cloned
- Familiarity with `cargo test` and `tokio::test`

## Quick Example

### Before: Limited Testing

```rust
// tests/report_with_auth_test.rs (current limitation)
#[tokio::test]
async fn test_report_endpoint_rejects_invalid_token() {
    let app = archodex_backend::router::router();

    let response = app.oneshot(/* request */).await.unwrap();

    // Can only test HTTP status codes
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // ❌ Cannot verify database state (no access to DB)
}
```

### After: Full Integration Testing

```rust
// tests/report_integration_test.rs (with database injection via State)
#[tokio::test]
async fn test_report_creates_resources_in_database() {
    // 1. Create test databases
    let (accounts_db, test_resources_factory) = create_test_databases().await;

    // 2. Setup test account
    seed_test_account(&accounts_db, "test_acc_123").await;

    // 3. Create router with injected State (accounts_db + test factory)
    let app = create_test_router_with_state(accounts_db.clone(), test_resources_factory.clone());

    // 4. Execute request
    let response = app.oneshot(/* request */).await.unwrap();

    // 5. Verify HTTP response
    assert_eq!(response.status(), StatusCode::OK);

    // 6. ✅ Verify database state through test factory
    let resources_db = test_resources_factory.get_test_db("test_acc_123").await;
    let resources: Vec<Resource> = resources_db.select("resource").await.unwrap();
    assert_eq!(resources.len(), 3);
    assert_eq!(resources[0].resource_type, "AWS::DynamoDB::Table");
}
```

## Step-by-Step Guide

### Step 1: Import Test Helpers

```rust
// At the top of your test file
use archodex_backend::{AppState, AuthedAccount, DBConnection};
use surrealdb::{Surreal, engine::local::Db, engine::local::Mem};

mod common;
use common::providers::TestResourcesDbFactory;  // Test factory from tests/common/
```

### Step 2: Create Test Databases and Factory

```rust
async fn create_test_databases() -> (DBConnection, Arc<TestResourcesDbFactory>) {
    // Create in-memory accounts database
    let accounts_db = Surreal::new::<Mem>(()).await.unwrap();
    accounts_db.use_ns("test").use_db("accounts").await.unwrap();
    migrator::migrate_accounts_database(&accounts_db).await.unwrap();

    // Create test factory (manages in-memory resources databases per account)
    let test_factory = Arc::new(TestResourcesDbFactory::new());

    (
        DBConnection::Concurrent(accounts_db),
        test_factory,
    )
}
```

### Step 3: Seed Test Data

```rust
async fn seed_test_account(db: &DBConnection, account_id: &str) -> Account {
    let account = Account::new_for_testing(
        account_id.to_string(),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],  // salt
    );

    if let DBConnection::Concurrent(ref db) = db {
        let _: Option<Account> = db
            .create(("account", account_id))
            .content(&account)
            .await
            .unwrap();
    }

    account
}
```

### Step 4: Create Router with Injected State

```rust
fn create_test_router_with_state(
    accounts_db: DBConnection,
    resources_db_factory: Arc<TestResourcesDbFactory>,
) -> Router {
    let state = AppState {
        accounts_db,
        resources_db_factory,  // Inject test factory instead of production
    };

    archodex_backend::router::create_router_with_state(state)
}
```

### Step 5: Write Your Test

```rust
#[tokio::test]
async fn test_my_feature() {
    // Setup
    let (accounts_db, test_factory) = create_test_databases().await;
    let account = seed_test_account(&accounts_db, "test123").await;
    let app = create_test_router_with_state(accounts_db.clone(), test_factory.clone());

    // Execute
    let response = app
        .oneshot(Request::builder()
            .uri("/report")
            .method("POST")
            .header("authorization", create_test_auth_token(&account))
            .header("content-type", "application/json")
            .body(Body::from(/* request body */))
            .unwrap())
        .await
        .unwrap();

    // Verify HTTP
    assert_eq!(response.status(), StatusCode::OK);

    // Verify Database (via test factory)
    let resources_db = test_factory.get_test_db("test123").await;
    if let DBConnection::Concurrent(ref db) = resources_db {
        let count: Option<i64> = db
            .query("SELECT count() FROM resource GROUP ALL")
            .await
            .unwrap()
            .take(0)
            .unwrap();

        assert_eq!(count, Some(3));
    }
}
```

## Common Patterns

### Pattern 1: Testing Authentication Failure

```rust
#[tokio::test]
async fn test_invalid_auth_rejected() {
    let (accounts_db, test_factory) = create_test_databases().await;
    let app = create_test_router_with_state(accounts_db, test_factory);

    let response = app
        .oneshot(Request::builder()
            .uri("/report")
            .method("POST")
            .header("authorization", "invalid_token")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### Pattern 2: Testing Resource Creation

```rust
#[tokio::test]
async fn test_resources_created_correctly() {
    let (accounts_db, test_factory) = create_test_databases().await;
    seed_test_account(&accounts_db, "acc123").await;
    let app = create_test_router_with_state(accounts_db, test_factory.clone());

    // Send request with 3 resources
    let response = app.oneshot(/* request */).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify all 3 resources were created via test factory
    let resources_db = test_factory.get_test_db("acc123").await;
    if let DBConnection::Concurrent(ref db) = resources_db {
        let resources: Vec<Resource> = db.select("resource").await.unwrap();
        assert_eq!(resources.len(), 3);

        // Verify specific resource attributes
        assert_eq!(resources[0].id, "arn:aws:dynamodb:us-east-1:123456789012:table/MyTable");
        assert_eq!(resources[0].resource_type, "AWS::DynamoDB::Table");
    }
}
```

### Pattern 3: Testing with Existing Data

```rust
#[tokio::test]
async fn test_update_existing_resource() {
    let (accounts_db, test_factory) = create_test_databases().await;
    seed_test_account(&accounts_db, "acc123").await;

    // Pre-populate database with existing resource via test factory
    let resources_db = test_factory.get_test_db("acc123").await;
    if let DBConnection::Concurrent(ref db) = resources_db {
        let existing_resource = Resource {
            id: "arn:aws:dynamodb:us-east-1:123456789012:table/MyTable".to_string(),
            resource_type: "AWS::DynamoDB::Table".to_string(),
            // ... other fields
        };
        let _: Option<Resource> = db.create("resource").content(existing_resource).await.unwrap();
    }

    let app = create_test_router_with_state(accounts_db, test_factory.clone());

    // Send update request
    let response = app.oneshot(/* request */).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify resource was updated (not duplicated)
    let resources_db = test_factory.get_test_db("acc123").await;
    if let DBConnection::Concurrent(ref db) = resources_db {
        let resources: Vec<Resource> = db.select("resource").await.unwrap();
        assert_eq!(resources.len(), 1);  // Still only 1 resource
    }
}
```

## Helper Functions Reference

The test suite provides these helper functions in `tests/common/`:

```rust
// Database creation
pub async fn create_test_accounts_db() -> DBConnection
pub async fn create_test_accounts_db_with_migrations() -> DBConnection
pub async fn create_test_accounts_db_with_account(account_id: &str) -> (DBConnection, Account)

// Test factory (tests/common/providers.rs)
pub struct TestResourcesDbFactory {
    // Manages in-memory databases per account
}
impl TestResourcesDbFactory {
    pub fn new() -> Self
    pub async fn get_test_db(&self, account_id: &str) -> DBConnection
}

// Router creation with State
pub fn create_test_router_with_state(
    accounts_db: DBConnection,
    resources_db_factory: Arc<TestResourcesDbFactory>,
) -> Router

// Test data creation
pub fn create_test_auth_token(account: &Account) -> String
pub fn create_test_report_request() -> ReportRequest
```

## Troubleshooting

### Error: "Extension of type `AuthedAccount` was not found"

**Cause**: Middleware didn't inject AuthedAccount because authentication failed or account doesn't exist in the test database.

**Solution**: Ensure you've seeded the test account before making requests:
```rust
seed_test_account(&accounts_db, "test123").await;
```

Also verify you're using the correct auth token for the seeded account.

### Error: "Cannot query database - no connection"

**Cause**: Trying to query `DBConnection` in non-Concurrent variant (RocksDB guard), or test factory hasn't created the database for the account yet.

**Solution**: Always use `if let DBConnection::Concurrent(ref db)` to unwrap, and ensure you've retrieved the database from the test factory:
```rust
let resources_db = test_factory.get_test_db("account_id").await;
if let DBConnection::Concurrent(ref db) = resources_db {
    let resources = db.select("resource").await.unwrap();
}
```

### Test Hanging or Timing Out

**Cause**: Test database migrations not applied, causing queries to fail silently.

**Solution**: Always call migration functions after creating databases:
```rust
migrator::migrate_accounts_database(&accounts_db).await.unwrap();
migrator::migrate_account_resources_database(&resources_db).await.unwrap();
```

### Tests Pass Individually But Fail in Parallel

**Cause**: Tests are sharing global state (environment variables, global connections).

**Solution**: Ensure each test creates its own databases - the helper functions already do this correctly. If you're modifying environment variables, use `#[serial]` from the `serial_test` crate.

## Performance Tips

### Reuse Migration Helpers

Instead of manually applying migrations, use the provided helpers:

```rust
// ❌ Slow: Manual migration
let db = create_test_db().await;
db.query("DEFINE TABLE resource ...").await.unwrap();
db.query("DEFINE FIELD ...").await.unwrap();
// ... 20 more queries

// ✅ Fast: Use helper
let db = create_test_db_with_migrations().await;
```

### Parallel Test Execution

Tests using injected databases are automatically isolated and can run in parallel:

```bash
# Default: Runs tests in parallel
cargo test

# Explicit parallelism
cargo test --test-threads=8
```

### Cleanup is Automatic

No need to manually clean up databases - Rust's RAII handles it:

```rust
#[tokio::test]
async fn test_something() {
    let (accounts_db, resources_db) = create_test_databases().await;

    // ... test code ...

    // No cleanup needed - databases freed when they go out of scope
}
```

## Next Steps

- **Read research.md**: Understand the architecture and design decisions
- **Read plan.md**: See the full implementation plan
- **Write your first test**: Use the examples above as templates
- **Extend test helpers**: Add domain-specific helpers in `tests/common/`

## Getting Help

If you encounter issues:

1. Check `tests/report_with_auth_test.rs` for example tests
2. Review `tests/common/` for available helper functions
3. Consult `research.md` for architectural details
4. Ask in #backend-dev Slack channel

## Summary

**You now know how to:**
- ✅ Create in-memory test databases
- ✅ Inject databases into test routers
- ✅ Seed test data before requests
- ✅ Verify HTTP responses AND database state
- ✅ Write fully isolated integration tests

**Happy testing!** 🎉
