# Data Model: Database Dependency Injection for Testing

**Feature**: 003-db-dependency-injection
**Date**: 2025-10-16
**Status**: N/A - No Data Model Changes

## Overview

This feature implements dependency injection for testing infrastructure. **There are no changes to the database schema, entities, or data models.**

## Rationale

The feature modifies only the _mechanism_ for obtaining database connections (injection vs global), not the data stored in those databases. All entities (Account, Resource, Event, etc.) remain unchanged.

## Implementation Details

### Modified Entities: Account (Internal Structure Only)

**Change**: Add optional test-only field to Account struct for holding injected database connection.

```rust
// src/account.rs
pub(crate) struct Account {
    // ... existing fields (unchanged) ...

    #[cfg(test)]
    #[serde(skip)]  // Not persisted to database
    injected_resources_db: Option<DBConnection>,
}
```

**Key Points:**
- Field only exists in test builds (`#[cfg(test)]`)
- Never serialized to database (`#[serde(skip)]`)
- Purely runtime state for dependency injection
- No database migrations needed
- No schema changes required

### No Database Schema Changes

**Accounts Database** (`archodex` namespace, `accounts` database):
- No changes to `account` table
- No changes to `has_access` relation
- No changes to `user` table

**Resources Database** (per-account namespace, `resources` database):
- No changes to `resource` table
- No changes to `event` table
- No changes to any relations

### No New Entities

This feature does not introduce any new entities that need to be persisted to the database.

### Test-Only Types

The feature introduces test-only wrapper types that are **not** entities:

```rust
#[cfg(test)]
pub(crate) struct TestAccountsDb(pub(crate) DBConnection);

#[cfg(test)]
pub(crate) struct TestResourcesDb(pub(crate) DBConnection);
```

These are:
- Runtime wrappers for dependency injection
- Never persisted to any database
- Only exist in test builds
- Used for Axum request extensions

## Conclusion

**No data-model.md content is applicable for this feature** because it modifies only the testing infrastructure, not the data model itself.

All database schemas, entities, relationships, and validation rules remain unchanged from the current implementation.
