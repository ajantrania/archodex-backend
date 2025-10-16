# Feature Specification: Database Dependency Injection for Testing

**Feature Branch**: `003-db-dependency-injection`
**Created**: 2025-10-16
**Status**: Draft
**Input**: User description: "DB Dependency Injection - integration testing (002-specs-001-rate/tasks.md - T036) is blocked becuase it isn't possible to keep the security boundaries we want (see 002-specs-001-rate/research.md) while switching out the DB connection for an in-memory DB to enable testing. Some details logged in the test - archodex-backend/tests/report_with_auth_test.rs. Researching the idiomatic rust way to do this, ti seems we want explicit dependency injection as Using global state with #[cfg(test)] guards goes against Rust's philosophy of explicit dependencies and can cause test isolation issues. Rust's idiomatic way to inject a DB (or any side-effecting dependency) is explicit DI—pass it in via constructor/state, often behind a trait, using generics or a trait object. We need to modify our code base to do this to enable testing w/o affecting the current behavior of archodex-com or the self-hosted version of Archodex."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Test Writer Can Inject Test Database Connections (Priority: P1)

A developer writing integration tests for the report ingestion endpoint needs to validate that reports are correctly stored in the database. Currently, the test cannot verify database state because `Account::resources_db()` uses global static connections that cannot be replaced with in-memory test databases.

**Why this priority**: This is the foundational blocker preventing comprehensive integration testing. Without dependency injection, tests can only validate HTTP status codes, not actual business logic or data persistence.

**Independent Test**: Can be fully tested by writing a test that creates an in-memory database, injects it into an Account, calls report ingestion logic, and verifies the resources were stored correctly. Success means the test can query the injected database and assert on the stored data.

**Acceptance Scenarios**:

1. **Given** a test creates an in-memory SurrealDB connection, **When** the test creates an Account with that connection injected, **Then** the Account uses the injected connection instead of global state
2. **Given** an Account with an injected test database, **When** report ingestion logic stores resources, **Then** the test can query the injected database to verify resources were stored with correct IDs and timestamps
3. **Given** multiple tests running in parallel, **When** each test injects its own database connection, **Then** tests remain isolated with no shared state between them

---

### User Story 2 - Production Code Uses Global Connections Without Changes (Priority: P1)

Production code (archodex-com and self-hosted deployments) must continue using the current global database connection architecture without any behavioral changes, performance regressions, or increased complexity.

**Why this priority**: Production stability is non-negotiable. The refactor must be zero-impact to production behavior—same connection pooling, same performance, same error handling. Any production impact would be a critical failure.

**Independent Test**: Can be tested by running the production application (both archodex-com and self-hosted configurations) and verifying all existing functionality works identically: account creation, report ingestion, dashboard queries, etc. Performance profiling should show no measurable difference.

**Acceptance Scenarios**:

1. **Given** production code is deployed, **When** handlers are invoked, **Then** database connections use the existing global connection pool (no new connections created per request)
2. **Given** the codebase is built in release mode, **When** examining binary size and performance, **Then** there is no measurable overhead from the dependency injection architecture
3. **Given** existing production error handling, **When** database errors occur, **Then** errors are handled identically to the previous implementation

---

### User Story 3 - Middleware Can Access Database Through Dependency Injection (Priority: P2)

Middleware functions like `report_api_key_account` (src/db.rs:296) and `dashboard_auth_account` (src/db.rs:266) need to load accounts from the database. These middleware must support dependency injection for testing while maintaining current production behavior.

**Why this priority**: Middleware authentication and account loading are critical paths that need testing. Without DI support in middleware, we can't test complete request flows (authentication → account loading → handler → database).

**Independent Test**: Can be tested by writing a test that creates a test router with middleware, injects test database connections, makes HTTP requests, and verifies middleware correctly loaded accounts from the injected database.

**Acceptance Scenarios**:

1. **Given** a test router with authentication middleware, **When** the test injects a test database connection, **Then** middleware loads accounts from the injected database
2. **Given** middleware receives a request, **When** account loading succeeds, **Then** the Account extension is injected into the request with the correct database connection for downstream handlers
3. **Given** production middleware, **When** processing requests, **Then** middleware uses global connection pool (no per-request database connection creation)

---

### Edge Cases

- What happens when a test injects a database connection but the Account tries to call a method that creates a new connection (e.g., `resources_db()` with different URL)?
- How does the system handle mixed scenarios where some code uses injected connections and other code uses global connections in the same request path?
- What happens if middleware injects an Account with a test database, but handler code tries to access global database state?
- How does connection cleanup work for injected test databases to prevent resource leaks in test suites?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow tests to inject custom database connections via AuthedAccount wrapper (authentication wrapper that holds Account and injected resources DB connection)
- **FR-002**: System MUST preserve existing global connection pooling behavior for production code (archodex-com and self-hosted)
- **FR-003**: `Account::resources_db()` method MUST return the injected connection when present, otherwise fall back to global connection
- **FR-004**: Middleware functions (`dashboard_auth_account`, `report_api_key_account`) MUST support loading accounts with injected database connections for testing
- **FR-005**: System MUST maintain test isolation—each test gets independent database connections with no shared state
- **FR-006**: Production code MUST NOT include test-specific connection handling logic in release builds (use `#[cfg(test)]` appropriately)
- **FR-007**: System MUST support injection for both accounts database and resources database connections
- **FR-008**: Axum extension pattern MUST work correctly with injected database connections (handlers extract `Extension<AuthedAccount>` which wraps Account with injected DB)
- **FR-009**: Test helper functions MUST provide ergonomic APIs for creating accounts with injected connections (e.g., `Account::new_for_testing_with_db()`)
- **FR-010**: System MUST handle connection lifetimes correctly—injected connections live as long as the test, global connections remain static

### Key Entities *(include if feature involves data)*

- **Account**: Core entity that needs database access. Currently uses global static connections via `resources_db()` method. Will be extended to support optional injected connections.
- **DBConnection**: Enum wrapping SurrealDB connections (Nonconcurrent for RocksDB, Concurrent for other backends). Represents the connection that needs to be injectable.
- **Test Database Context**: New test-only entity (lifetime-bound) that holds in-memory database connections for injection into Accounts during testing.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Test code can create an Account with injected database, call report ingestion, and query the injected database to verify 100% of stored resources match the request
- **SC-002**: Tests using injected databases execute within same performance targets as specified in 002-specs-001-rate research.md (unit tests <1s, integration tests <5s, full suite <30s)
- **SC-003**: Production code shows no measurable performance regression—API response times remain within 5% of baseline measurements (within 5% is considered zero impact given measurement variance)
- **SC-004**: Test isolation is complete—100 consecutive test runs with parallel execution produce identical, deterministic results with no flaky tests
- **SC-005**: Existing production functionality works identically—all manual testing scenarios (account creation, report ingestion, dashboard queries) complete successfully with no behavioral changes
- **SC-006**: Developer experience is smooth—new integration tests can be written following documented patterns within 20 minutes, including dependency injection setup

