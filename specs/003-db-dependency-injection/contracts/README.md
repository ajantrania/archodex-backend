# API Contracts: Database Dependency Injection for Testing

**Feature**: 003-db-dependency-injection
**Date**: 2025-10-16
**Status**: N/A - No API Contract Changes

## Overview

This feature implements dependency injection for testing infrastructure. **There are no changes to HTTP APIs, endpoints, or contracts.**

## Rationale

The feature modifies only internal testing infrastructure (how tests inject database connections into handlers). All existing HTTP endpoints, request/response formats, and API contracts remain completely unchanged.

## API Impact Analysis

### No New Endpoints

This feature does not add any new HTTP endpoints.

### No Modified Endpoints

All existing endpoints maintain identical contracts:
- `/report` (POST) - unchanged
- `/dashboard/{account_id}/...` (various) - unchanged
- All other endpoints - unchanged

### No Request/Response Changes

**Request Formats**: All unchanged
**Response Formats**: All unchanged
**Error Responses**: All unchanged
**Headers**: All unchanged
**Authentication**: All unchanged

## Internal Changes (Not Part of API Contract)

The following changes are internal implementation details and do not affect API contracts:

### Middleware Internal Behavior (Test Mode Only)

In test builds, middleware checks for injected database connections before falling back to global connections. This is purely internal and has zero impact on:
- Request validation
- Response format
- Error handling
- Status codes
- Headers

### Test Helper Functions

New test-only functions for creating routers with injected databases:

```rust
// tests/common/db.rs (test-only, not part of public API)
pub fn create_test_router_with_injected_dbs(
    accounts_db: DBConnection,
    resources_db: DBConnection,
) -> Router
```

**Visibility**: Test code only, never exposed via HTTP

## Conclusion

**No API contracts are affected by this feature.** The contracts directory remains empty because all API endpoints maintain their existing contracts.

If future features introduce new endpoints or modify existing ones, those contracts will be documented here. This feature is purely internal testing infrastructure.
