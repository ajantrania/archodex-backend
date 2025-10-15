# Testing Framework Proposal: Account Plans & Rate Limiting

**Feature**: 001-rate-limits-we
**Date**: 2025-10-14
**Status**: Proposal for Review

---

## Executive Summary

**Recommendation**: Use **built-in Rust testing** (`cargo test`) with **testcontainers** for integration tests.

**Rationale**:
- No automated tests currently exist in the codebase
- Need lightweight, idiomatic Rust solution
- Must test SurrealDB interactions end-to-end
- Lambda deployment requires portable tests (no docker-compose in CI)

**Timeline**: 2-3 days before implementation begins
1. Day 1: Framework setup + 2 example tests for existing features
2. Day 2: Your review & approval
3. Day 3: Refine based on feedback, document patterns

---

## Testing Framework Options

### Option 1: Built-in Rust + testcontainers ⭐ **RECOMMENDED**

**Stack**:
- `cargo test` (built-in)
- `testcontainers` (Docker-based ephemeral databases)
- `axum-test` (HTTP testing helper)
- `rstest` (parameterized tests, fixtures)

**Pros**:
- ✅ Idiomatic Rust, zero learning curve for team
- ✅ Ephemeral SurrealDB containers (no shared state between tests)
- ✅ Works in CI (GitHub Actions, AWS CodeBuild)
- ✅ Fast setup (~5 min to add dependencies)
- ✅ Can test full request → database → response cycle

**Cons**:
- ⚠️ Requires Docker in CI (standard in most pipelines)
- ⚠️ Slightly slower than in-memory mocks (acceptable for integration tests)

**Example Test Structure**:
```rust
#[tokio::test]
async fn test_report_ingestion_with_resources() {
    let container = testcontainers::clients::Cli::default()
        .run(GenericImage::new("surrealdb/surrealdb", "latest"));

    let db = setup_test_db(&container).await;
    let app = setup_test_app(db).await;

    // Test report ingestion
    let response = app.post("/report")
        .json(&test_report())
        .send()
        .await;

    assert_eq!(response.status(), 200);

    // Verify resources were stored
    let resources = db.query("SELECT * FROM resource").await.unwrap();
    assert_eq!(resources.len(), 3);
}
```

---

### Option 2: nextest (Enhanced Test Runner)

**Stack**: Same as Option 1, but use `cargo nextest` instead of `cargo test`

**Pros**:
- ✅ Parallel test execution (faster CI)
- ✅ Better test output formatting
- ✅ JUnit XML output for CI integration

**Cons**:
- ⚠️ Extra tool to install/maintain
- ⚠️ Not necessary for small test suites initially

**Recommendation**: Start with Option 1, migrate to nextest if test suite grows >100 tests.

---

### Option 3: Integration Test Framework (e.g., cucumber-rust)

**Stack**: BDD-style tests with Gherkin syntax

**Pros**:
- ✅ Business-readable test scenarios
- ✅ Good for stakeholder demos

**Cons**:
- ❌ Overkill for backend API testing
- ❌ Slower iteration (extra abstraction layer)
- ❌ Team unfamiliarity

**Recommendation**: Not suitable for this project.

---

## Recommended Testing Strategy

### Test Pyramid

```
       ┌─────────────┐
       │  E2E Tests  │  ← 5% (manual, occasional)
       │   (manual)  │
       └─────────────┘
      ┌───────────────┐
      │ Integration   │  ← 70% (API → DB → Response)
      │    Tests      │
      └───────────────┘
    ┌───────────────────┐
    │   Unit Tests      │  ← 25% (pure logic, no I/O)
    └───────────────────┘
```

**For Rate Limiting Feature**:
- **Unit Tests** (25%): Counter logic, limit calculations, hour window math
- **Integration Tests** (70%): Full ingestion flow with rate limit enforcement
- **E2E Tests** (5%): Manual testing via Postman/curl before deployment

---

## Test Levels for Rate Limiting

### 1. Unit Tests (`src/rate_limits/counters.rs`)

**What to test**:
- ✅ Hour window string formatting (`"2025-10-10T14"`)
- ✅ Counter ID generation (`usage:{account_id}:{hour}`)
- ✅ Event segregation by hour (split events into hourly buckets)
- ✅ Limit calculation logic (current + new <= max)

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hour_window_formatting() {
        let timestamp = Utc.with_ymd_and_hms(2025, 10, 10, 14, 30, 0).unwrap();
        let hour = format_hour_window(timestamp);
        assert_eq!(hour, "2025-10-10T14");
    }

    #[test]
    fn test_segregate_events_by_hour() {
        let events = vec![
            event_at_time(Utc.with_ymd_and_hms(2025, 10, 10, 14, 30, 0).unwrap()),
            event_at_time(Utc.with_ymd_and_hms(2025, 10, 10, 14, 45, 0).unwrap()),
            event_at_time(Utc.with_ymd_and_hms(2025, 10, 10, 15, 10, 0).unwrap()),
        ];

        let segregated = segregate_events_by_hour(events);

        assert_eq!(segregated.len(), 2); // 2 hours
        assert_eq!(segregated.get("2025-10-10T14").unwrap().len(), 2);
        assert_eq!(segregated.get("2025-10-10T15").unwrap().len(), 1);
    }
}
```

**No Database Required**: Pure Rust logic tests.

---

### 2. Integration Tests (`tests/rate_limiting_integration.rs`)

**What to test**:
- ✅ End-to-end ingestion with plan enforcement
- ✅ Resource counter increments only for NEW resources (RETURN BEFORE logic)
- ✅ Event counter increments per-hour correctly
- ✅ Rate limit rejection returns 429 with correct error message
- ✅ Multi-hour events processed independently

**Example**:
```rust
use testcontainers::clients::Cli;
use testcontainers::GenericImage;

#[tokio::test]
async fn test_resource_limit_enforcement() {
    // Setup: Ephemeral SurrealDB container
    let docker = Cli::default();
    let container = docker.run(GenericImage::new("surrealdb/surrealdb", "v2.3.7"));
    let db_url = format!("ws://localhost:{}", container.get_host_port_ipv4(8000));

    let db = connect_test_db(&db_url).await;

    // Create test account with Team plan (500 resources)
    db.query("
        CREATE account:test_account CONTENT { name: 'Test Account' };
        CREATE plan CONTENT {
            account_id: account:test_account,
            name: 'Team',
            max_resources: 500,
            max_events_per_hour: 1000,
            update_frequency_seconds: 1200,
            start: time::now(),
            end: NONE,
            created_by: 'test-system'
        };
    ").await.unwrap();

    // Ingest 500 resources (should succeed)
    let report_500 = generate_test_report(500, 0);
    let result = ingest_report("test_account", report_500, &db).await;
    assert!(result.is_ok());

    // Verify counter
    let counter: Counter = db.query("SELECT * FROM usage:test_account:resources")
        .await.unwrap().take(0).unwrap();
    assert_eq!(counter.resource_count, 500);

    // Ingest 1 more resource (should fail)
    let report_1 = generate_test_report(1, 0);
    let result = ingest_report("test_account", report_1, &db).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Resource limit exceeded"));
    assert!(err.to_string().contains("500/500"));
}

#[tokio::test]
async fn test_multi_hour_event_enforcement() {
    let docker = Cli::default();
    let container = docker.run(GenericImage::new("surrealdb/surrealdb", "v2.3.7"));
    let db_url = format!("ws://localhost:{}", container.get_host_port_ipv4(8000));
    let db = connect_test_db(&db_url).await;

    // Create account with Team plan (1000 events/hour)
    setup_test_account_with_plan(&db, "test_account", 500, 1000).await;

    // Ingest 1500 events: 900 in hour 14:00, 600 in hour 15:00
    let events = vec![
        generate_events_at_hour("2025-10-10T14", 900),
        generate_events_at_hour("2025-10-10T15", 600),
    ].concat();

    let report = generate_report_with_events(events);
    let result = ingest_report("test_account", report, &db).await;

    // Should succeed: both hours under limit
    assert!(result.is_ok());

    // Verify counters
    let counter_14: Counter = db.query("
        SELECT * FROM usage
        WHERE account_id = account:test_account
        AND event_hour_window = '2025-10-10T14'
    ").await.unwrap().take(0).unwrap();
    assert_eq!(counter_14.event_count_this_hour, 900);

    let counter_15: Counter = db.query("
        SELECT * FROM usage
        WHERE account_id = account:test_account
        AND event_hour_window = '2025-10-10T15'
    ").await.unwrap().take(0).unwrap();
    assert_eq!(counter_15.event_count_this_hour, 600);

    // Ingest 200 more events in hour 14:00 (should fail for that hour)
    let events = generate_events_at_hour("2025-10-10T14", 200);
    let report = generate_report_with_events(events);
    let result = ingest_report("test_account", report, &db).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Event limit exceeded for hour 2025-10-10T14"));
}
```

**Database Required**: Uses testcontainers for ephemeral SurrealDB.

---

### 3. Example Tests for Existing Features (Before Rate Limiting)

To validate the testing framework before implementing rate limiting, we'll write 2 tests for **existing features**:

#### Test 1: Basic Resource Ingestion (Integration)

**File**: `tests/report_ingestion_test.rs`

```rust
use testcontainers::clients::Cli;
use testcontainers::GenericImage;

#[tokio::test]
async fn test_report_ingests_resources_correctly() {
    // Setup ephemeral database
    let docker = Cli::default();
    let container = docker.run(GenericImage::new("surrealdb/surrealdb", "v2.3.7"));
    let db_url = format!("ws://localhost:{}", container.get_host_port_ipv4(8000));
    let db = connect_test_db(&db_url).await;

    // Create test account
    db.query("CREATE account:test_acc CONTENT { name: 'Test' }").await.unwrap();

    // Build test report with 3 resources
    let report = Report {
        resources: vec![
            Resource { id: "res1".into(), first_seen_at: Utc::now(), last_seen_at: Utc::now() },
            Resource { id: "res2".into(), first_seen_at: Utc::now(), last_seen_at: Utc::now() },
            Resource { id: "res3".into(), first_seen_at: Utc::now(), last_seen_at: Utc::now() },
        ],
        events: vec![],
    };

    // Ingest report
    let result = ingest_report("test_acc", report, &db).await;
    assert!(result.is_ok());

    // Verify resources were stored
    let resources: Vec<Resource> = db.query("SELECT * FROM resource ORDER BY id")
        .await.unwrap().take(0).unwrap();

    assert_eq!(resources.len(), 3);
    assert_eq!(resources[0].id, "res1");
    assert_eq!(resources[1].id, "res2");
    assert_eq!(resources[2].id, "res3");
}
```

**What this validates**:
- ✅ testcontainers works correctly
- ✅ Can connect to ephemeral SurrealDB
- ✅ Can run queries and verify results
- ✅ Test isolation (each test gets fresh DB)

---

#### Test 2: Report API Key Generation (Unit + Integration)

**File**: `tests/report_api_key_test.rs`

```rust
#[tokio::test]
async fn test_report_api_key_roundtrip() {
    // Setup
    let account_id = "1234567890";
    let account_salt = rand::thread_rng().gen::<[u8; 16]>().to_vec();

    // Generate API key
    let api_key = ReportApiKey {
        id: 12345,
        account_id: account_id.parse().unwrap(),
        created_at: Utc::now(),
        created_by: User { id: "user123".into() },
    };

    let key_string = api_key.generate_value(account_id, account_salt.clone()).await.unwrap();

    // Validate format
    assert!(key_string.starts_with("archodex_"));

    // Decode and verify
    let decoded = ReportApiKey::decode_and_validate(&key_string).await.unwrap();
    assert_eq!(decoded.account_id, account_id);
    assert_eq!(decoded.key_id, 12345);
}

#[test]
fn test_report_api_key_tamper_detection() {
    // Generate valid key
    let key_string = generate_test_api_key();

    // Tamper with protobuf bytes (modify one byte)
    let mut tampered = key_string.clone();
    let bytes = tampered.as_bytes_mut();
    bytes[20] ^= 0xFF; // Flip bits in middle of key

    // Validation should fail
    let result = ReportApiKey::decode_and_validate(&tampered).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("decryption failed"));
}
```

**What this validates**:
- ✅ AES-GCM encryption/decryption works
- ✅ Protobuf encoding/decoding works
- ✅ Tamper detection catches modifications
- ✅ Can test crypto logic without database

---

## Dependencies to Add

**Cargo.toml** (`[dev-dependencies]` section):

```toml
[dev-dependencies]
testcontainers = "0.23"
rstest = "0.22"
tokio-test = "0.4"
axum-test = "14.0"
```

**Estimated setup time**: 5-10 minutes

---

## Test Organization

```
archodex-backend/
├── src/
│   └── rate_limits/
│       ├── mod.rs
│       ├── plan.rs
│       ├── counters.rs       # Unit tests at bottom: #[cfg(test)] mod tests
│       ├── enforcement.rs    # Unit tests at bottom
│       └── plan_fetch.rs
│
└── tests/                    # Integration tests (separate crate)
    ├── common/               # Shared test helpers
    │   ├── mod.rs
    │   ├── db.rs             # setup_test_db(), connect_test_db()
    │   └── fixtures.rs       # generate_test_report(), test data builders
    │
    ├── report_ingestion_test.rs      # Existing feature (example)
    ├── report_api_key_test.rs        # Existing feature (example)
    │
    └── rate_limiting/                # New tests for rate limiting feature
        ├── mod.rs
        ├── resource_limits_test.rs   # Resource counter & enforcement
        ├── event_limits_test.rs      # Event counter & multi-hour
        └── plan_management_test.rs   # Plan CRUD operations
```

**Conventions**:
- Unit tests: `#[cfg(test)] mod tests` at bottom of source files
- Integration tests: Separate `tests/` directory (Rust convention)
- Test helpers: `tests/common/` module (shared fixtures, DB setup)

---

## Timeline & Approval Process

### Phase 1: Framework Setup + Example Tests (2 days)

**Day 1 (4-6 hours)**:
1. Add `testcontainers` and `rstest` to `Cargo.toml`
2. Create `tests/common/` with DB setup helpers
3. Write Test 1: Basic resource ingestion (integration)
4. Write Test 2: Report API key roundtrip (unit + integration)
5. Verify `cargo test` passes on local machine

**Day 2 (2-3 hours)**:
6. Document test patterns in `tests/common/README.md`
7. Add CI configuration (GitHub Actions or AWS CodeBuild)
8. Run tests in CI to verify Docker works
9. Present to you for review

### Phase 2: Your Review (1 day)

**You review**:
- [ ] Test readability (are tests easy to understand?)
- [ ] Test helpers (are fixtures reusable?)
- [ ] Test execution (run `cargo test` locally)
- [ ] CI integration (tests run automatically on push?)

**Feedback cycle**:
- If approved → proceed to Phase 3
- If changes needed → iterate and re-submit

### Phase 3: Adopt for Rate Limiting (ongoing)

**During rate limiting implementation**:
- Use approved test patterns for all new code
- Aim for 70% integration test coverage, 25% unit test coverage
- Write tests **before** implementing each component (TDD optional, but encouraged)

---

## Success Criteria

**Before starting rate limiting implementation, we must have**:
1. ✅ 2 example tests passing (`cargo test` succeeds)
2. ✅ testcontainers working in CI
3. ✅ Test helper functions documented
4. ✅ Your approval on test style/approach

**During rate limiting implementation**:
- ✅ Every new module has unit tests (counters, enforcement, plan logic)
- ✅ Every API endpoint has integration test (report ingestion with limits)
- ✅ CI fails if tests fail (block merges)

---

## Open Questions for Discussion

1. **Test Data Management**: Should we use fixtures (JSON files) or builder functions for test reports?
   - Recommendation: **Builder functions** (more flexible, type-safe)

2. **CI Environment**: GitHub Actions or AWS CodeBuild?
   - Recommendation: **GitHub Actions** (faster feedback, free for public repos)

3. **Test Database**: Ephemeral containers vs persistent test DB?
   - Recommendation: **Ephemeral containers** (perfect isolation, no cleanup needed)

4. **Coverage Targets**: Should we enforce minimum coverage (e.g., 80%)?
   - Recommendation: **No hard requirement initially**, focus on critical paths first

5. **Performance Tests**: Should we add benchmark tests (SC-008, SC-009 requirements)?
   - Recommendation: **Yes, but as separate `benches/` tests** using `criterion` crate

---

## Next Steps

1. **Your decision**: Approve testing approach or request changes
2. **My work**: Implement Phase 1 (framework + 2 example tests)
3. **Your review**: Run tests locally, verify CI works
4. **Approval**: Green light to use this methodology for rate limiting feature

Once approved, testing framework becomes the standard for all future backend development.

---

## Appendix: Alternative Frameworks Considered

| Framework | Pros | Cons | Verdict |
|-----------|------|------|---------|
| **cargo test + testcontainers** | Idiomatic, simple, ephemeral DBs | Requires Docker | ✅ **RECOMMENDED** |
| nextest | Faster parallel execution | Extra tool, not needed yet | ⏸️ Revisit later |
| cucumber-rust | BDD-style, business-readable | Overkill for API tests | ❌ Not suitable |
| mockall | Fast unit tests with mocks | Can't test DB interactions | 🔧 Use for specific cases only |
| sqlx + postgres | Well-established Rust SQL testing | We use SurrealDB, not Postgres | ❌ Not applicable |

---

**Ready to proceed?** Please review and let me know:
1. Approve testing approach as-is
2. Request specific changes
3. Discuss open questions above
