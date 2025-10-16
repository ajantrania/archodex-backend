# Quickstart: Testing Framework Setup and Validation

**Feature**: 002-specs-001-rate
**Date**: 2025-10-14
**Branch**: 002-specs-001-rate

---

## Overview

This quickstart guide shows how to use the Archodex backend testing framework to write and run tests. The framework uses **SurrealDB in-memory mode** for fast, isolated testing with zero infrastructure dependencies.

---

## Prerequisites

- Rust 2024 edition installed
- Archodex backend repository cloned
- On branch `002-specs-001-rate`

---

## Quick Start (5 Minutes)

### 1. Run Existing Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_report_ingests_resources_correctly
```

**Expected Output**:
```
running 2 tests
test report_ingestion_test::test_report_ingests_resources_correctly ... ok
test report_api_key_test::test_api_key_roundtrip ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

### 2. Write Your First Test

**File**: `tests/my_first_test.rs`

```rust
mod common;

use common::{create_test_db_with_migrations, create_test_account};

#[tokio::test]
async fn test_create_account() {
    // Setup: Create in-memory database with migrations
    let db = create_test_db_with_migrations().await;

    // Execute: Create test account
    let account = create_test_account("my_test", "My Test Account");

    db.query(format!(
        "CREATE account:{} CONTENT {{ name: '{}' }}",
        account.id, account.name
    ))
    .await
    .unwrap();

    // Verify: Check account was created
    let result: Option<Account> = db.select(("account", &account.id))
        .await
        .unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "My Test Account");
}
```

**Run it**:
```bash
cargo test test_create_account
```

---

### 3. Use Test Helpers

The framework provides helpers in `tests/common/`:

```rust
mod common;

use common::{
    create_test_db_with_account,
    create_test_report,
    create_test_resources,
};

#[tokio::test]
async fn test_with_helpers() {
    // Get database + account in one call
    let (db, account) = create_test_db_with_account("test123").await;

    // Generate test report with 5 resources, 0 events
    let report = create_test_report(5, 0);

    // Your test logic here...
}
```

---

## Common Testing Patterns

### Pattern 1: Database Test

```rust
#[tokio::test]
async fn test_database_operation() {
    let db = create_test_db_with_migrations().await;

    // Setup
    db.query("CREATE something").await.unwrap();

    // Execute
    let result = db.select("something").await.unwrap();

    // Verify
    assert!(!result.is_empty());
}
```

### Pattern 2: Test with Test Data

```rust
#[tokio::test]
async fn test_with_generated_data() {
    let (db, account) = create_test_db_with_account("test_acc").await;

    // Generate 10 test resources
    let resources = create_test_resources(10);

    // Use in test
    for resource in resources {
        db.create("resource").content(resource).await.unwrap();
    }

    // Verify
    let count: Vec<Resource> = db.select("resource").await.unwrap();
    assert_eq!(count.len(), 10);
}
```

### Pattern 3: Test Builder Pattern

```rust
#[tokio::test]
async fn test_with_builder() {
    let (db, account) = create_test_db_with_account("test_acc").await;

    // Use builder for complex test data
    let report = TestReportBuilder::new()
        .with_resources(20)
        .with_events(100)
        .build();

    // Test with report...
}
```

---

## Available Test Helpers

### Database Helpers (`tests/common/db.rs`)

```rust
// Create empty in-memory database
let db = create_test_db().await;

// Create database with migrations applied
let db = create_test_db_with_migrations().await;

// Create database + test account
let (db, account) = create_test_db_with_account("my_account").await;
```

### Fixture Helpers (`tests/common/fixtures.rs`)

```rust
// Account fixtures
let account = create_test_account("id", "name");

// Report fixtures
let report = create_test_report(num_resources, num_events);
let builder = create_test_report_builder();

// Resource fixtures
let resource = create_test_resource("res1");
let resources = create_test_resources(10);

// Event fixtures
let event = create_test_event(1);
let events = create_test_events(100);

// API key fixtures
let key = create_test_api_key(12345, "account_id");
let salt = create_test_account_salt();

// User fixtures
let user = create_test_user("user123");
```

---

## Running Tests in CI

Tests automatically run in GitHub Actions on every push:

```yaml
# .github/workflows/test.yml (already configured)
- name: Run tests
  run: cargo test --all-features --verbose
```

**Local CI Testing** (with ACT):
```bash
# Install ACT
brew install act  # macOS
# or: cargo install act

# Run CI locally
act
```

---

## Debugging Failed Tests

### Show Test Output

```bash
# Show println! output
cargo test -- --nocapture

# Show specific test output
cargo test test_name -- --nocapture --exact
```

### Run Single Test

```bash
# Run one specific test
cargo test test_report_ingests_resources_correctly --exact
```

### Run Tests Serially (No Parallelism)

```bash
# Useful for debugging race conditions
cargo test -- --test-threads=1
```

---

## Performance Tips

### Fast Tests (In-Memory)

**DO THIS**:
```rust
#[tokio::test]
async fn fast_test() {
    let db = create_test_db().await; // In-memory, <10ms
    // Test runs in milliseconds
}
```

**AVOID**:
```rust
// Don't use containers unless testing server-specific features
let container = SurrealDb::default().start().await; // +2-5 seconds
```

### Parallel Test Execution

Tests run in parallel by default. To ensure isolation:

```rust
// ✅ Good: Each test creates its own DB
#[tokio::test]
async fn test_1() {
    let db = create_test_db().await; // Isolated
}

#[tokio::test]
async fn test_2() {
    let db = create_test_db().await; // Isolated
}
```

---

## Example Tests

### Example 1: Unit Test (PrincipalChainIdPart)

**Location**: `src/principal_chain.rs` (inline `#[cfg(test)]` module)

**What it tests**: Validates type conversions (TryFrom/From) for SurrealDB serialization

**Run it**:
```bash
cargo test test_principal_chain_id_part_round_trip
```

**Why this test**: Tests pure logic with no external dependencies (no DB, no AWS, no auth). Perfect idiomatic Rust unit test.

### Example 2: Integration Test (Health Check)

**Location**: `tests/health_check_test.rs`

**What it tests**: Validates HTTP endpoint handling (auth bypassed for simplicity)

**Run it**:
```bash
cargo test test_health_endpoint
```

**Why this test**: Demonstrates integration testing pattern with test routers that bypass authentication.

### Example 3: Integration Test (Report with Full Auth)

**Location**: `tests/report_with_auth_test.rs`

**What it tests**: Validates COMPLETE authentication middleware chain + report ingestion

**Run it**:
```bash
cargo test test_report_with_full_auth
cargo test test_report_endpoint_rejects_invalid_auth
```

**Why this test**: Uses production router to test actual auth middleware execution, extension injection, and full request flow.

---

## Best Practices

### ✅ Do

- Create fresh database for each test (`create_test_db()`)
- Use test helpers from `tests/common/`
- Keep tests fast (<5 seconds per test)
- Use descriptive test names (`test_account_creation_with_valid_data`)
- Test one thing per test function

### ❌ Don't

- Share database instances between tests
- Use production credentials or real AWS resources
- Write tests that depend on execution order
- Test against deployed backends (only local/ephemeral DBs)
- Skip cleanup (in-memory DB cleans up automatically)

---

## Troubleshooting

### "Cannot find module `common`"

**Solution**: Make sure `tests/common/mod.rs` exists (NOT `tests/common.rs`)

```bash
# Should have this structure:
tests/
├── common/
│   └── mod.rs
└── my_test.rs
```

### "Database connection failed"

**Solution**: Using in-memory mode requires no connection. Check you're using `create_test_db()`:

```rust
// ✅ Correct
let db = create_test_db().await;

// ❌ Wrong - requires running SurrealDB server
let db = Surreal::new::<Ws>("localhost:8000").await;
```

### "Test timeout"

**Solution**: Tests should complete quickly (<5s). If timing out:
- Check for infinite loops
- Verify async/await usage is correct
- Ensure `#[tokio::test]` is used for async tests

---

## Next Steps

1. **Read Example Tests**: Start with `tests/report_ingestion_test.rs`
2. **Write Tests for Your Code**: Use patterns from examples
3. **Run Tests Frequently**: Use `cargo test` during development
4. **Check CI**: Verify tests pass in GitHub Actions

For more detailed documentation, see `tests/common/README.md`.

---

## Quick Reference Card

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name --exact

# Show test output
cargo test -- --nocapture

# Run serially
cargo test -- --test-threads=1

# Format code
cargo fmt

# Run linter
cargo clippy

# Full CI check
cargo test && cargo clippy && cargo fmt --check
```

---

## Summary

The Archodex testing framework provides:
- ✅ **Fast in-memory database** testing (SurrealDB `kv-mem`)
- ✅ **Test helpers** for common setup patterns
- ✅ **Zero infrastructure** requirements (no Docker needed)
- ✅ **Automatic cleanup** (no manual teardown)
- ✅ **CI integration** (GitHub Actions)

Start writing tests today with `cargo test`!
