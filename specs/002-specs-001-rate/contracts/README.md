# API Contracts: Testing Framework Setup and Validation

**Feature**: 002-specs-001-rate
**Date**: 2025-10-14
**Status**: N/A

---

## Overview

This feature is **infrastructure code** (testing framework) and does not expose any HTTP APIs or external contracts.

**No API contracts are defined for this feature.**

---

## Testing Framework Interfaces

While there are no HTTP API contracts, the testing framework provides **internal testing interfaces** for use by other features:

### Test Helper API (Rust)

**Module**: `tests/common`

#### Database Setup Functions

```rust
/// Creates in-memory SurrealDB instance for testing
pub async fn create_test_db() -> Surreal<Db>

/// Creates test database with migrations applied
pub async fn create_test_db_with_migrations() -> Surreal<Db>

/// Creates test database with sample account
pub async fn create_test_db_with_account(account_id: &str) -> (Surreal<Db>, TestAccount)
```

**Usage**:
```rust
#[tokio::test]
async fn test_something() {
    let db = create_test_db().await;
    // Use db in test
}
```

---

#### Test Data Fixtures

```rust
// Account fixtures
pub fn create_test_account(id: &str, name: &str) -> TestAccount

// Report fixtures
pub fn create_test_report(num_resources: usize, num_events: usize) -> TestReport
pub fn create_test_report_builder() -> TestReportBuilder

// Resource fixtures
pub fn create_test_resource(id: &str) -> TestResource
pub fn create_test_resources(count: usize) -> Vec<TestResource>

// Event fixtures
pub fn create_test_event(id: usize) -> TestEvent
pub fn create_test_events(count: usize) -> Vec<TestEvent>

// API key fixtures
pub fn create_test_api_key(id: i32, account_id: &str) -> TestReportApiKey
pub fn create_test_account_salt() -> Vec<u8>

// User fixtures
pub fn create_test_user(id: &str) -> TestUser
```

**Usage**:
```rust
#[tokio::test]
async fn test_ingestion() {
    let report = create_test_report(10, 5); // 10 resources, 5 events
    // Use report in test
}
```

---

## Future Features Using This Framework

Other features (e.g., 001-rate-limits-we) will use these test helpers to write their own tests. This framework provides the foundation but does not expose HTTP endpoints.

---

## Validation Tests

The testing framework itself is validated by 2 example tests:

### Example Test 1: Resource Ingestion
- **File**: `tests/report_ingestion_test.rs`
- **Purpose**: Validates that resource ingestion works correctly
- **Validates**: Existing `ingest_report()` functionality

### Example Test 2: API Key Generation
- **File**: `tests/report_api_key_test.rs`
- **Purpose**: Validates API key encryption/decryption
- **Validates**: Existing `ReportApiKey` crypto operations

---

## Conclusion

This feature has **no HTTP API contracts** because it is internal testing infrastructure. The "contracts" are the Rust test helper functions documented above, which other features will use to write their tests.
