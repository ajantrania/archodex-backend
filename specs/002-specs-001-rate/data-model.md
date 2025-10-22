# Data Model: Testing Framework Setup and Validation

**Feature**: 002-specs-001-rate
**Date**: 2025-10-14
**Status**: Design Complete

---

## Overview

The testing framework is **infrastructure code** that does not introduce new production data models. This document describes the **test data structures** and **test helper patterns** used to validate existing Archodex features.

---

## Test Data Structures

### 1. Test Account

**Purpose**: Mock account record for testing multi-tenant features

**Structure**:
```rust
pub struct TestAccount {
    pub id: String,           // e.g., "test_account_001"
    pub name: String,         // e.g., "Test Account"
}
```

**Creation Pattern**:
```rust
pub fn create_test_account(id: &str, name: &str) -> TestAccount {
    TestAccount {
        id: id.to_string(),
        name: name.to_string(),
    }
}
```

**Usage**: Used in example tests to simulate account-scoped operations

---

### 2. Test Report

**Purpose**: Mock report data for validating resource ingestion

**Structure**:
```rust
pub struct TestReport {
    pub resources: Vec<TestResource>,
    pub events: Vec<TestEvent>,
}
```

**Builder Pattern**:
```rust
pub struct TestReportBuilder {
    num_resources: usize,
    num_events: usize,
}

impl TestReportBuilder {
    pub fn new() -> Self {
        Self {
            num_resources: 0,
            num_events: 0,
        }
    }

    pub fn with_resources(mut self, count: usize) -> Self {
        self.num_resources = count;
        self
    }

    pub fn with_events(mut self, count: usize) -> Self {
        self.num_events = count;
        self
    }

    pub fn build(self) -> TestReport {
        TestReport {
            resources: (0..self.num_resources)
                .map(|i| create_test_resource(&format!("res{}", i)))
                .collect(),
            events: (0..self.num_events)
                .map(|i| create_test_event(i))
                .collect(),
        }
    }
}
```

**Usage**: Example test 1 (resource ingestion)

---

### 3. Test Resource

**Purpose**: Mock resource data with unique ID and timestamps

**Structure**:
```rust
pub struct TestResource {
    pub id: String,                    // e.g., "res1", "res2"
    pub first_seen_at: DateTime<Utc>, // Timestamp
    pub last_seen_at: DateTime<Utc>,  // Timestamp
}
```

**Factory Function**:
```rust
pub fn create_test_resource(id: &str) -> TestResource {
    let now = Utc::now();
    TestResource {
        id: id.to_string(),
        first_seen_at: now,
        last_seen_at: now,
    }
}
```

**Validation Rules**:
- `id` must be unique within test report
- `first_seen_at` <= `last_seen_at`
- Timestamps use UTC timezone

---

### 4. Test Event

**Purpose**: Mock event data for testing event ingestion

**Structure**:
```rust
pub struct TestEvent {
    pub id: usize,                     // Sequential ID
    pub timestamp: DateTime<Utc>,      // Event timestamp
    pub resource_id: Option<String>,   // Optional resource reference
}
```

**Factory Function**:
```rust
pub fn create_test_event(id: usize) -> TestEvent {
    TestEvent {
        id,
        timestamp: Utc::now(),
        resource_id: None,
    }
}

pub fn create_test_event_for_resource(id: usize, resource_id: &str) -> TestEvent {
    TestEvent {
        id,
        timestamp: Utc::now(),
        resource_id: Some(resource_id.to_string()),
    }
}
```

---

### 5. Test API Key

**Purpose**: Mock ReportApiKey for testing encryption/decryption

**Structure**:
```rust
pub struct TestReportApiKey {
    pub id: i32,                       // Key ID
    pub account_id: String,            // Associated account
    pub created_at: DateTime<Utc>,     // Creation timestamp
    pub created_by: TestUser,          // Creator user
}
```

**Factory Function**:
```rust
pub fn create_test_api_key(id: i32, account_id: &str) -> TestReportApiKey {
    TestReportApiKey {
        id,
        account_id: account_id.to_string(),
        created_at: Utc::now(),
        created_by: create_test_user("test_user"),
    }
}
```

**Usage**: Example test 2 (API key generation and validation)

---

### 6. Test User

**Purpose**: Mock user data for authentication context

**Structure**:
```rust
pub struct TestUser {
    pub id: String,    // e.g., "user123"
}
```

**Factory Function**:
```rust
pub fn create_test_user(id: &str) -> TestUser {
    TestUser {
        id: id.to_string(),
    }
}
```

---

## Test Helper Modules

### Module: `tests/common/db.rs`

**Purpose**: Database setup and connection helpers

**Functions**:
```rust
/// Creates in-memory SurrealDB instance for testing
pub async fn create_test_db() -> Surreal<Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    db
}

/// Creates test database with migrations applied
pub async fn create_test_db_with_migrations() -> Surreal<Db> {
    let db = create_test_db().await;
    migrator::migrate_accounts_database(&db).await.unwrap();
    db
}

/// Creates test database with sample account
pub async fn create_test_db_with_account(account_id: &str) -> (Surreal<Db>, TestAccount) {
    let db = create_test_db_with_migrations().await;
    let account = create_test_account(account_id, "Test Account");

    db.query(format!("CREATE account:{} CONTENT {{ name: '{}' }}", account.id, account.name))
        .await
        .unwrap();

    (db, account)
}
```

---

### Module: `tests/common/fixtures.rs`

**Purpose**: Test data generation and fixture builders

**Functions**:
```rust
// Account fixtures
pub fn create_test_account(id: &str, name: &str) -> TestAccount { /* ... */ }

// Report fixtures
pub fn create_test_report(num_resources: usize, num_events: usize) -> TestReport { /* ... */ }
pub fn create_test_report_builder() -> TestReportBuilder { /* ... */ }

// Resource fixtures
pub fn create_test_resource(id: &str) -> TestResource { /* ... */ }
pub fn create_test_resources(count: usize) -> Vec<TestResource> {
    (0..count)
        .map(|i| create_test_resource(&format!("res{}", i)))
        .collect()
}

// Event fixtures
pub fn create_test_event(id: usize) -> TestEvent { /* ... */ }
pub fn create_test_events(count: usize) -> Vec<TestEvent> {
    (0..count)
        .map(create_test_event)
        .collect()
}

// API key fixtures
pub fn create_test_api_key(id: i32, account_id: &str) -> TestReportApiKey { /* ... */ }
pub fn create_test_account_salt() -> Vec<u8> {
    rand::thread_rng().gen::<[u8; 16]>().to_vec()
}

// User fixtures
pub fn create_test_user(id: &str) -> TestUser { /* ... */ }
```

---

### Module: `tests/common/mod.rs`

**Purpose**: Re-exports all test helpers

**Structure**:
```rust
mod db;
mod fixtures;

pub use db::*;
pub use fixtures::*;
```

---

## Test Database Schema

### No New Schema Required

The testing framework validates **existing** Archodex data models:
- `account` table (existing)
- `resource` table (existing)
- `event` table (existing)
- `report_api_key` table (existing)

**Migrations**: Use existing `migrator` workspace member to set up test databases.

---

## State Transitions

### Test Lifecycle State Machine

```
┌─────────────┐
│   Setup     │  create_test_db(), create_test_account()
└─────┬───────┘
      │
      ▼
┌─────────────┐
│   Execute   │  Run test operations (ingest_report, generate_key, etc.)
└─────┬───────┘
      │
      ▼
┌─────────────┐
│   Verify    │  Assert expected outcomes
└─────┬───────┘
      │
      ▼
┌─────────────┐
│  Cleanup    │  Automatic (in-memory DB destroyed on scope exit)
└─────────────┘
```

**Isolation**: Each test creates fresh database instance (no shared state).

---

## Data Relationships (Test Data Only)

```
TestAccount
    │
    ├──< TestReport
    │       ├──< TestResource (many)
    │       └──< TestEvent (many)
    │
    └──< TestReportApiKey
            └──< TestUser (created_by)
```

**Note**: These relationships mirror production data models but exist only in test scope.

---

## Validation Rules (Test Data)

### TestReport Validation
- ✅ Must have at least 0 resources (empty reports allowed for edge case testing)
- ✅ Must have at least 0 events (empty reports allowed)
- ✅ Resource IDs must be unique within report

### TestResource Validation
- ✅ `id` must be non-empty string
- ✅ `first_seen_at` <= `last_seen_at`
- ✅ Timestamps must be valid UTC DateTime

### TestEvent Validation
- ✅ `id` must be unique within report
- ✅ `timestamp` must be valid UTC DateTime
- ✅ `resource_id` (if present) should reference existing resource (soft validation)

### TestReportApiKey Validation
- ✅ `id` must be positive integer
- ✅ `account_id` must reference existing test account
- ✅ `created_at` must be valid UTC DateTime
- ✅ Generated key string must start with "archodex_"
- ✅ Encrypted payload must be decryptable with correct salt

---

## Performance Characteristics (Test Data)

### Test Data Generation Performance

| Operation | Count | Expected Time |
|-----------|-------|---------------|
| `create_test_account()` | 1 | <1μs |
| `create_test_resource()` | 1 | <1μs |
| `create_test_resources()` | 100 | <100μs |
| `create_test_events()` | 1000 | <1ms |
| `create_test_report()` | 100 resources + 1000 events | <2ms |
| `create_test_db()` | 1 instance | <10ms |
| `create_test_db_with_migrations()` | 1 instance | <50ms |

**Rationale**: In-memory operations are extremely fast, suitable for TDD workflow.

---

## Example Usage Patterns

### Pattern 1: Simple Test with Database

```rust
#[tokio::test]
async fn test_create_account() {
    let db = create_test_db_with_migrations().await;

    let account = create_test_account("test123", "Test Account");

    db.query(format!("CREATE account:{} CONTENT {{ name: '{}' }}", account.id, account.name))
        .await
        .unwrap();

    let result: Option<TestAccount> = db.select(("account", &account.id))
        .await
        .unwrap();

    assert!(result.is_some());
}
```

### Pattern 2: Test with Generated Report

```rust
#[tokio::test]
async fn test_ingest_report() {
    let (db, account) = create_test_db_with_account("test_acc").await;

    let report = create_test_report(3, 0); // 3 resources, 0 events

    let result = ingest_report(&account.id, report, &db).await;

    assert!(result.is_ok());

    let resources: Vec<Resource> = db.select("resource").await.unwrap();
    assert_eq!(resources.len(), 3);
}
```

### Pattern 3: Test with Builder Pattern

```rust
#[tokio::test]
async fn test_complex_report() {
    let (db, account) = create_test_db_with_account("test_acc").await;

    let report = TestReportBuilder::new()
        .with_resources(10)
        .with_events(50)
        .build();

    let result = ingest_report(&account.id, report, &db).await;

    assert!(result.is_ok());
}
```

---

## Documentation Location

**Primary Documentation**: `tests/common/README.md`

**Contents**:
- How to use test helpers
- Example test patterns
- Database setup guidelines
- Test data generation best practices

---

## Conclusion

The testing framework introduces **no new production data models**. All data structures defined in this document are **test helpers and fixtures** designed to validate existing Archodex features. The design prioritizes:

- ✅ Simplicity (factory functions over complex builders initially)
- ✅ Speed (in-memory generation, <1ms per fixture)
- ✅ Isolation (fresh database per test)
- ✅ Maintainability (clear helper functions in `tests/common/`)

This aligns with the Constitution's principle of avoiding over-engineering at the current project stage.
