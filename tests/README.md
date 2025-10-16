# Archodex Backend Testing Framework

This directory contains the testing framework for the Archodex backend, implemented as part of feature `002-specs-001-rate`.

## Overview

The testing framework uses **SurrealDB in-memory mode** (`kv-mem`) for fast, isolated testing with zero infrastructure dependencies. This approach aligns with the project's Constitution principle of avoiding over-engineering.

## Structure

```
tests/
├── common/              # Shared test helpers
│   ├── mod.rs          # Module re-exports and setup utilities
│   ├── db.rs           # Database setup functions
│   ├── fixtures.rs     # Test data factories
│   └── test_router.rs  # Test router helpers
└── health_check_integration_test.rs  # Example integration test
```

## Quick Start

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run specific test
cargo test test_principal_chain_id_part_round_trip

# Run with output
cargo test -- --nocapture
```

### Writing a Unit Test

Unit tests go inline in source files using `#[cfg(test)] mod tests`:

```rust
// In src/your_module.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Your test code here
        assert_eq!(1 + 1, 2);
    }
}
```

**Example**: See `src/principal_chain.rs:228` for real unit tests validating type conversions.

### Writing an Integration Test

Integration tests go in separate files in the `tests/` directory:

```rust
// tests/your_integration_test.rs
mod common;

use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[tokio::test]
async fn test_your_endpoint() {
    let app = common::create_test_router();

    let response = app
        .oneshot(Request::builder().uri("/endpoint").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

**Example**: See `tests/health_check_integration_test.rs` for a complete integration test.

## Test Helpers

### Database Helpers (`common::db`)

```rust
use common::*;

// Create empty in-memory database
let db = create_test_db().await;

// Create database with migrations (schema)
let db = create_test_db_with_migrations().await;

// Create database + test account
let (db, account) = create_test_db_with_account("test_account_123").await;

// Get shared accounts database
let accounts_db = get_test_accounts_db().await;
```

### Test Data Fixtures (`common::fixtures`)

```rust
use common::*;

// Create test account
let account = create_test_account("acc_001", "Test Account");

// Create test user
let user = create_test_user("user_123");

// Create test auth token (for auth bypass in tests)
let token = create_test_auth_token("acc_001");  // Returns: "test_token_acc_001"

// Generate random salt
let salt = create_test_account_salt();
```

### Test Router Helpers (`common::test_router`)

```rust
use common::*;

// Create simple test router (no auth)
let app = create_test_router();
```

## Test Patterns

### Pattern 1: Pure Logic (Unit Test)

Test business logic without external dependencies.

```rust
#[test]
fn test_conversion_logic() {
    let resource_id = ResourceId::from_parts(vec![
        ("partition", "aws"),
        ("account", "123456789012"),
    ]);

    let value: surrealdb::sql::Value = resource_id.into();
    // Assert on value...
}
```

**When to use**: Testing pure functions, type conversions, validation logic.

### Pattern 2: Integration Test with Mock Auth

Test HTTP endpoints with authentication bypassed.

```rust
#[tokio::test]
async fn test_endpoint_logic() {
    let app = create_test_router();  // No auth middleware

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

**When to use**: Testing handler logic in isolation from auth concerns.

### Pattern 3: Integration Test with Full Auth (Coming in Phase 5)

Test complete request flow including authentication middleware.

**When to use**: Testing end-to-end flows with authentication.

## Performance Characteristics

- **Unit tests**: <1ms per test
- **Integration tests (in-memory)**: <50ms per test
- **Full test suite**: <30 seconds target

## Design Decisions

### Why In-Memory SurrealDB?

1. **Zero infrastructure**: No Docker, no containers, no setup
2. **Fast**: Microsecond-level operations
3. **Isolated**: Each test gets fresh database
4. **API parity**: SurrealDB maintains identical API across all backends (mem, rocksdb, dynamodb)

### Why Not Testcontainers?

- Adds 2-5 seconds per test for container startup
- Requires Docker in development and CI
- Unnecessary complexity for current project stage
- Can add later if needed (Layer 2 tests)

### Why Not Mocking?

- Would require introducing trait abstraction layer (major refactor)
- Archodex code uses SurrealDB-specific types extensively
- Risk of mock/real implementation divergence
- In-memory mode provides better production fidelity

## Test Quality Requirements

⚠️ **IMPORTANT**: All tests MUST be meaningful and test actual Archodex business logic.

### ✅ Good Tests
- Test real production code (PrincipalChainIdPart conversions)
- Validate business rules (resource ID validation)
- Test error handling (missing fields, invalid types)
- Test integration flows (HTTP → Handler → DB)

### ❌ Bad Tests
- Testing "1 + 1 = 2" just to have a passing test
- Testing mock functions that return hardcoded values
- Testing trivial getters/setters with no logic
- Creating fake business logic just to have something to test

**If you encounter a test that would be meaningless, STOP and report the issue rather than writing it.**

## Current Test Coverage

### Unit Tests (src/principal_chain.rs)
- ✅ `test_principal_chain_id_part_round_trip` - TryFrom/From round-trip
- ✅ `test_principal_chain_id_part_without_event` - Optional field handling
- ✅ `test_principal_chain_id_part_invalid_object_missing_id` - Error handling
- ✅ `test_principal_chain_id_part_invalid_event_type` - Type validation

### Integration Tests
- ✅ `test_health_endpoint` - Simple HTTP endpoint testing

## Next Steps

### Phase 5: Full Auth Middleware Testing (In Progress)
- Add `#[cfg(test)]` constructors to auth types
- Test complete authentication flow
- Test report ingestion with auth

### Phase 6: CI Integration (Requires Approval)
- GitHub Actions workflow
- Automated testing on every push
- Clippy and formatting checks

### Phase 7: Polish (Requires Approval)
- Format all code
- Verify test isolation and determinism
- Update CLAUDE.md

## Troubleshooting

### "Cannot find module `common`"
- Ensure `tests/common/mod.rs` exists (NOT `tests/common.rs`)

### "kv-mem feature not enabled"
- Check `Cargo.toml` has `surrealdb = { version = "= 2.3.7", features = ["rustls", "kv-mem"] }` in `[dev-dependencies]`

### Tests timing out
- Ensure `#[tokio::test]` is used for async tests
- Check for infinite loops or blocking operations

## Resources

- **Feature Spec**: `specs/002-specs-001-rate/spec.md`
- **Implementation Plan**: `specs/002-specs-001-rate/plan.md`
- **Task Breakdown**: `specs/002-specs-001-rate/tasks.md`
- **Research**: `specs/002-specs-001-rate/research.md`
- **Quickstart Guide**: `specs/002-specs-001-rate/quickstart.md`

---

**Status**: Framework operational and ready for use ✅

**Last Updated**: 2025-10-15
