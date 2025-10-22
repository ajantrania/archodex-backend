# Feature Specification: Testing Framework Setup and Validation

**Feature Branch**: `002-specs-001-rate`
**Created**: 2025-10-14
**Status**: Draft
**Input**: User description: "specs/001-rate-limits-we/testing-framework-proposal.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Evaluate and Select Testing Approach (Priority: P1)

Development team needs to evaluate testing approaches (unit tests with mocks vs integration tests with database vs hybrid), research SurrealDB testing viability, and make informed selection based on project stage and maintenance burden.

**Why this priority**: Choosing pragmatic testing approach prevents over-engineering. Approach impacts setup complexity, CI requirements, and whether developers actually write tests. Better to have simple tests than complex tests that are avoided.

**Independent Test**: Can be fully tested by documenting evaluation criteria, researching SurrealDB testing options, comparing approaches against criteria (including Constitution principles), and getting stakeholder approval.

**Acceptance Scenarios**:

1. **Given** testing framework proposal document exists, **When** team reviews testing approaches (unit with mocks, integration with in-memory DB, integration with containers, hybrid), **Then** tradeoffs for each approach are clearly documented
2. **Given** SurrealDB is used in production, **When** team researches SurrealDB testing options (containerized, in-memory/embedded, DynamoDB-backed viability, mocking), **Then** each option's viability, complexity, and production-fidelity is documented
3. **Given** approaches are documented, **When** options are evaluated against project needs (maintenance burden per Constitution, setup complexity, CI compatibility, confidence level, debugging ease), **Then** each approach is scored or ranked
4. **Given** evaluation is complete, **When** team makes approach selection, **Then** decision includes clear rationale explaining why chosen approach best fits current project stage
5. **Given** approach is selected, **When** selection is presented to stakeholders, **Then** stakeholders understand tradeoffs (e.g., simplicity vs confidence) and approve decision before implementation begins

---

### User Story 2 - Setup Testing Framework Infrastructure (Priority: P1)

Development team needs to establish testing framework infrastructure by adding selected framework dependencies, creating reusable test helpers, and documenting testing patterns.

**Why this priority**: Foundation for all automated testing. Without proper framework setup, tests will be inconsistent, difficult to write, and unreliable.

**Independent Test**: Can be fully tested by adding dependencies to Cargo.toml, creating test helper modules, and verifying `cargo test` runs successfully with test database instances.

**Acceptance Scenarios**:

1. **Given** selected framework dependencies are identified, **When** dependencies are added to `[dev-dependencies]` in Cargo.toml, **Then** developer runs `cargo build --tests` and all dependencies resolve and compile successfully
2. **Given** `tests/common/` directory structure is created, **When** helper modules (db.rs, fixtures.rs) are implemented, **Then** modules expose reusable functions for database setup and test data generation
3. **Given** database setup helper is implemented, **When** test calls this function, **Then** test database instance starts and returns valid database connection
4. **Given** multiple tests use database helpers, **When** tests run in sequence, **Then** each test gets isolated database instance with no shared state
5. **Given** test helpers are documented in `tests/common/README.md`, **When** new developer reads documentation, **Then** they can write their first test following established patterns within 15 minutes

---

### User Story 3 - Validate Framework with Test Validation Approaches (Priority: P1)

Development team needs to validate chosen testing approach by implementing 3 distinct test validation approaches to verify the framework works and is maintainable: (1) unit tests for pure logic, (2) integration tests with mock authentication, and (3) integration tests with full authentication middleware.

**Why this priority**: Demonstrating multiple test validation approaches proves the framework is viable for different testing scenarios before using it for new development. Provides concrete examples showing the framework is actually maintainable across unit and integration testing patterns.

**Independent Test**: Can be fully tested by implementing 3 test validation approaches that pass consistently and are straightforward to write/debug.

**Acceptance Scenarios**:

1. **Given** unit test validation approach is implemented, **When** tests validate pure business logic (e.g., type conversions, data transformations) without external dependencies, **Then** tests pass quickly (<1 second) and demonstrate unit testing pattern works correctly
2. **Given** integration test with mock authentication is implemented, **When** test validates HTTP request/response flow with authentication bypassed via Extension injection, **Then** test passes and demonstrates integration testing with simplified auth works correctly
3. **Given** integration test with full authentication middleware is implemented, **When** test validates complete request flow including auth middleware → account loading → handler execution → database operations, **Then** test passes and demonstrates full-stack integration testing works correctly
4. **Given** all 3 test validation approaches are implemented, **When** developer runs `cargo test` locally, **Then** all tests pass within acceptable time for chosen approach (under 30 seconds total)
5. **Given** tests are committed to repository, **When** GitHub Actions CI runs via `cargo test`, **Then** tests execute successfully without excessive setup complexity
6. **Given** test validation approaches serve as documentation, **When** developer needs to write new test, **Then** they can reference appropriate test validation approach structure and patterns and write tests without getting blocked by setup complexity

---

### User Story 4 - Configure CI Integration for Automated Testing (Priority: P2)

Development team needs to configure GitHub Actions CI to automatically run tests on every push and block merges if tests fail. Local testing via ACT is also supported.

**Why this priority**: Automated CI testing prevents broken code from being merged. Essential for maintaining code quality but lower priority than establishing working framework locally first.

**Independent Test**: Can be fully tested by pushing code with passing/failing tests and verifying GitHub Actions correctly reports status.

**Acceptance Scenarios**:

1. **Given** GitHub Actions workflow is added to `.github/workflows/test.yml`, **When** code is pushed to any branch, **Then** workflow automatically triggers `cargo test` execution
2. **Given** GitHub Actions runs cargo test, **When** all tests pass, **Then** workflow succeeds and shows green status on pull request
3. **Given** GitHub Actions runs cargo test, **When** any test fails, **Then** workflow fails and blocks merge with clear error message
4. **Given** developer wants to test CI locally, **When** developer runs `act` locally, **Then** tests execute in local ACT environment matching GitHub Actions behavior
5. **Given** CI completes test run, **When** checking execution time, **Then** total workflow execution completes within 3-5 minutes depending on chosen approach

---

### Edge Cases

- **Framework evaluation with no clearly superior option**: Team MUST document risks of each choice and make pragmatic decision based on project stage within 4-hour time limit. Default to simplest viable option per Constitution's Development Stage Context.
- **Stakeholder disagreement on framework choice**: Team MUST present data-driven evaluation against defined criteria and facilitate informed discussion. Final decision prioritizes maintainability and simplicity.
- **Full integration tests too complex to maintain**: Team MUST pivot to simpler alternatives (in-memory database or targeted mocks). Framework complexity that prevents developer adoption violates Constitution principles.
- **SurrealDB container incompatible or too slow in CI**: Team MUST use in-memory SurrealDB mode (`kv-mem`) or evaluate mocking approach. CI execution time must remain under 5 minutes total.
- **In-memory SurrealDB behavioral differences from production**: Team MUST document known differences in tests/common/README.md and accept tradeoff for simplicity. Critical differences may require selective container-based tests.
- **Unit tests vs integration tests decision**: Team MUST evaluate based on maintenance burden (Constitution priority), debugging ease, and confidence level needed. Hybrid approach (unit tests for logic, integration tests for critical paths) is acceptable.
- **Selected testing approach proves too cumbersome**: Team MUST be prepared to pivot to simpler approach within 2-3 day validation phase. Early feedback and willingness to change prevents sunk cost fallacy.
- **Test setup complexity prevents developer adoption**: This violates Constitution Development Stage Context principle. Team MUST immediately simplify approach or provide better abstractions through helper functions.
- **CI environment has insufficient resources**: Tests MUST fail early with clear error message indicating resource constraints. Team evaluates lighter-weight approach or optimizes test execution (serial vs parallel).
- **Test flakiness due to database state or timing**: Framework MUST use deterministic test data and in-memory database isolation. Timing-based assertions are prohibited. Each test gets fresh database instance to prevent state contamination.

## Requirements *(mandatory)*

### Functional Requirements

**Testing Approach Evaluation** (✅ Completed in plan stage):
- **FR-001**: Team evaluated multiple testing approach options (unit tests with mocks, integration tests with in-memory DB, integration tests with containers, hybrid approach)
- **FR-002**: Evaluation assessed each option against defined criteria (SurrealDB integration capability, CI compatibility, developer learning curve, test execution speed, maintenance overhead)
- **FR-003**: Testing approach selection is documented with clear rationale explaining tradeoffs and why chosen option best fits project needs (see research.md)
- **FR-004**: Testing approach selection was approved by stakeholders before implementation phase

**Testing Infrastructure Implementation**:
- **FR-005**: Selected testing approach (in-memory SurrealDB with minimal tooling) is pragmatic for current project stage and minimizes maintenance burden
- **FR-006**: SurrealDB testing strategy uses in-memory mode (`kv-mem`) for simplicity and speed (documented in research.md)
- **FR-007**: Selected testing approach is maintainable at current project stage (minimal setup, simple CI, straightforward debugging per Constitution principles)
- **FR-008**: Testing framework MUST work in GitHub Actions CI without excessive infrastructure requirements
- **FR-009**: Selected testing tooling dependencies MUST be added to `[dev-dependencies]` section of Cargo.toml
- **FR-010**: Test organization MUST follow Rust conventions: unit tests in `#[cfg(test)] mod tests` within production code files, integration tests in `tests/` directory, shared test helpers in `tests/common/` module

**Test Helpers and Reusability**:
- **FR-012**: Test helpers MUST provide reusable functions appropriate to chosen testing approach (database setup, test data builders)
- **FR-013**: Test helpers MUST provide fixture generators for common test data (reports, resources, events, accounts)
- **FR-014**: Test helpers MUST reduce boilerplate code by at least 70% compared to inline test setup (per SC-015)

**Test Validation Approaches** (Framework validation via 3 distinct approaches):
- **FR-015**: Framework MUST demonstrate viability by implementing 3 test validation approaches as specified in FR-015.1, FR-015.2, and FR-015.3 below
- **FR-015.1**: Test validation approach 1 (unit) MUST validate pure business logic without external dependencies (e.g., type conversions, data transformations) using inline `#[cfg(test)]` tests in production code files
- **FR-015.2**: Test validation approach 2 (integration with mock auth) MUST validate HTTP request/response flow with authentication bypassed via test router or Extension injection
- **FR-015.3**: Test validation approach 3 (integration with full auth) MUST validate complete request flow including auth middleware → account loading → handler execution → database operations, with explicit assertions for: (1) HTTP 200 status code, (2) account record exists in database with correct ID, (3) report resources stored in database with correct count matching request, (4) events stored in database with valid timestamps
- **FR-019**: All test validation approaches MUST use test helpers (where applicable) to demonstrate reusability and reduce boilerplate
- **FR-020**: All test validation approaches MUST be straightforward to write, run, and debug without excessive setup steps

**Documentation**:
- **FR-021**: Framework documentation MUST be created in `tests/common/README.md` including: (1) overview of testing infrastructure, (2) database setup helper functions with examples, (3) test data fixture patterns with examples, (4) test router patterns for auth bypass, (5) three test validation approach patterns (unit, integration mock, integration full), (6) troubleshooting section with common errors and debugging tips
- **FR-022**: Documentation MUST explain how to write new tests following established patterns, enabling new developers to write and run their first test within 15 minutes
- **FR-023**: Documentation MUST include concrete code examples for database setup, test data generation, and assertions appropriate to each test validation approach

**CI Integration**:
- **FR-024**: GitHub Actions workflow MUST run `cargo test` automatically on every push to any branch
- **FR-025**: GitHub Actions MUST block pull request merges if any tests fail
- **FR-026**: GitHub Actions setup MUST be straightforward with minimal infrastructure requirements (complexity acceptable for project stage)
- **FR-027**: Tests SHOULD be runnable locally via ACT to match GitHub Actions environment (desirable but not mandatory)

**Framework Quality**:
- **FR-028**: Framework MUST provide clear error messages when setup fails (e.g., missing dependencies, configuration errors)
- **FR-029**: Tests MUST be deterministic and produce identical results across 10 consecutive executions
- **FR-030**: Framework MUST support chosen testing approach (unit tests, integration tests, or hybrid) effectively
- **FR-031**: If approach uses test databases or resources, cleanup MUST occur automatically (tests never deploy against real backends)

### Key Entities *(include if feature involves data)*

- **Framework Evaluation Document**: Analysis comparing framework options against criteria with selection rationale
- **Test Database Instance**: Isolated SurrealDB instance for each test (implementation depends on selected framework)
- **Test Account**: Mock account record with ID and name used for validating existing features
- **Test Report**: Generated report containing configurable number of resources and events for testing ingestion
- **Test Resource**: Mock resource with unique ID, first_seen_at, and last_seen_at timestamps
- **Test API Key**: Generated ReportApiKey with encrypted protobuf payload for testing crypto operations
- **Test Helper Module**: Reusable code in `tests/common/` providing database setup and fixture generation functions
- **Test Documentation**: README and examples in `tests/common/` explaining patterns and usage
- **Framework Dependencies**: Crates added to `[dev-dependencies]` for selected testing framework

## Success Criteria *(mandatory)*

### Framework Evaluation and Selection

- **SC-001**: Team evaluates at least 3 framework options from proposal document within 4 hours
- **SC-002**: Evaluation criteria (SurrealDB integration, CI compatibility, learning curve, test speed, maintenance) are applied to each option
- **SC-003**: Framework selection is documented with clear rationale and tradeoff analysis
- **SC-004**: Stakeholders review and approve framework selection before implementation begins

### Framework Setup and Viability

- **SC-005**: Selected framework dependencies are successfully added to Cargo.toml and compile without errors via `cargo build --tests`
- **SC-006**: Test helper modules (`tests/common/`) are created with reusable functions appropriate to chosen approach within 2 hours
- **SC-007**: Test setup (database, mocks, or data builders) is straightforward for developers to use and understand
- **SC-008**: Tests have appropriate isolation based on chosen approach (no shared state for integration tests, clean mocks for unit tests)
- **SC-009**: 3 test validation approaches (unit, integration with mock auth, integration with full auth) are implemented and pass consistently
- **SC-010**: Testing framework setup (evaluation + dependencies + helpers + 3 test validation approaches) completes within 2-3 days total
- **SC-011**: Developers can write and run new tests without excessive setup complexity (aligns with Constitution principles)

### Test Execution Performance

| Scenario | Target Time | Rationale |
|----------|-------------|-----------|
| Single unit test | < 1 second | Fast feedback for TDD workflow |
| Single integration test | < 5 seconds | Acceptable for database operations (in-memory) |
| Full local test suite | < 30 seconds | Current stage (3 test validation approaches: ~7-8 tests total), may increase to 60s as suite grows |
| Full CI pipeline | < 3-5 minutes | From push to result, including checkout, build, test, clippy |

- **SC-012**: Developer can run `cargo test` locally and all tests pass within target times above
- **SC-013**: Individual tests run quickly enough to encourage frequent execution per target times
- **SC-014**: CI pipeline runs all tests automatically and completes within target time

### Framework Quality and Usability

- **SC-015**: Test helper functions reduce boilerplate code by at least 70% compared to inline test setup
- **SC-016**: Test isolation is appropriate for chosen approach (clean state for each test)
- **SC-017**: Framework provides clear error messages when setup fails (not silent failures or hangs)
- **SC-018**: Test resource cleanup occurs automatically if resources are created (containers, connections, temporary files)
- **SC-019**: Tests are deterministic and produce identical results across 10 consecutive runs
- **SC-020**: Debugging failing tests is straightforward without excessive complexity

### Documentation and Knowledge Transfer

- **SC-021**: `tests/common/README.md` documentation is created with patterns, examples, and usage instructions covering all 3 test validation approaches
- **SC-022**: Documentation enables new developer to write their first test following established patterns within 15 minutes (measured by having developer read README and write passing test)
- **SC-023**: Test validation approaches serve as reference implementations demonstrating helper usage and best practices for unit and integration testing patterns

### CI Integration

- **SC-024**: CI configuration automatically runs `cargo test` on every push to any branch
- **SC-025**: CI blocks pull request merges when any test fails
- **SC-026**: CI setup is maintainable and doesn't require excessive infrastructure or configuration for chosen approach

## Assumptions *(optional)*

- Developers are familiar with Rust testing conventions (`#[test]`, `#[tokio::test]`, `cargo test`)
- CI environment is GitHub Actions (ACT compatibility desirable but not required)
- Tests never run against deployed production or staging backends (only local ephemeral databases or mocks with point-in-time restoration each run)
- Test execution times per performance table above are acceptable for developer workflow
- Factory functions are preferred over JSON fixtures for test data generation (more flexible and type-safe)
- Existing codebase has business logic suitable for test validation approaches (type conversions for unit tests, HTTP endpoints for integration tests)
- All tests must be deterministic and repeatable (no flaky tests due to timing or network issues)
- Test data uses realistic values but can be simplified for readability (e.g., predictable IDs like "res1", "res2")
- Framework validation with 3 test validation approaches is sufficient proof of viability before broader adoption
- Rate limiting feature tests will be written separately in feature 001-rate-limits-we using testing framework established here
- Team has capacity to spend 4-8 hours evaluating testing approaches and SurrealDB options before implementation
- Framework evaluation document (testing-framework-proposal.md) contains sufficient detail for informed decision-making
- Archodex SurrealDB fork (DynamoDB-backed) may or may not be testable depending on research outcomes
- In-memory SurrealDB (`kv-mem`) is selected approach for simplicity, accepting potential behavioral differences from production

## Dependencies *(optional)*

**Pre-Selection** (for evaluation):
- **testing-framework-proposal.md**: Document comparing testing approaches and framework options
- **Stakeholder availability**: For reviewing and approving approach/framework selection
- **Archodex SurrealDB fork**: Located at https://github.com/Archodex/surrealdb (local: `/Users/ajantrania/code/archodex/surrealdb`) - DynamoDB-backed version used in production

**Post-Selection** (implementation dependencies for selected approach):
- **tower crate**: For `tower::ServiceExt::oneshot()` used in integration test request/response testing (already in workspace dependencies)
- **GitHub Actions**: CI environment (ACT for local testing is optional)
- **SurrealDB in-memory mode**: `Surreal::new::<Mem>()` for isolated test database instances
- **Existing business logic for unit tests**: Type conversion logic (e.g., PrincipalChainIdPart) in production code
- **Existing HTTP endpoints for integration tests**: Health endpoint and report endpoint with authentication middleware

## Out of Scope *(optional)*

- **Writing tests for rate limiting feature**: Tests for plans, counters, and rate limit enforcement will be written in feature 001-rate-limits-we
- **Testing against deployed backends**: Tests never run against real production or staging environments (only local/ephemeral test databases or mocks)
- **End-to-end tests with external systems**: Manual testing with Postman/curl only
- **Performance benchmark tests**: Handled separately with criterion crate in `benches/` directory if needed
- **Load testing or stress testing**: Not part of automated test suite
- **BDD-style tests with Gherkin syntax**: cucumber-rust rejected as overkill for API testing
- **Enhanced test runner like nextest**: Deferred until test suite grows beyond 100 tests
- **Test coverage enforcement tooling**: No minimum coverage percentage required for framework validation
- **UI/frontend testing**: Backend API testing only
- **Security penetration testing**: Separate security audit process
- **Testing of deployment infrastructure**: Lambda and AWS resources not in scope

## Risks *(optional)*

- **Framework selection paralysis**: Team may struggle to choose between framework options with similar tradeoffs. Mitigation: Set 4-hour time limit for evaluation and use data-driven scoring against clear criteria.
- **Selected framework infrastructure unavailable in CI**: Chosen framework may require infrastructure not available in CI environment. Mitigation: Verify CI compatibility as part of evaluation criteria before final approval.
- **Test execution time**: Integration tests may be slower than desired depending on framework choice. Mitigation: Include test speed as evaluation criterion and monitor times during validation.
- **Resource usage**: Multiple concurrent tests may exhaust system resources depending on framework. Mitigation: Evaluate isolation approach as part of framework selection and run tests serially initially if needed.
- **SurrealDB version mismatch**: Test database version may drift from production version. Mitigation: Pin database version in test code regardless of framework choice.
- **Test data complexity**: Fixture generation may become complex for realistic scenarios. Mitigation: Start with simple factory functions for 3 test validation approaches, refactor if patterns emerge.
- **Flaky tests**: Network or timing issues may cause intermittent failures. Mitigation: Use deterministic test data and avoid time-based assertions.
- **Framework proves inadequate during validation**: Selected framework may not meet needs after implementation. Mitigation: Keep 2-3 day validation phase focused, get quick feedback, and be prepared to pivot if necessary.
- **Existing features may lack testability**: Business logic or endpoints may be difficult to test. Mitigation: Refactor for testability if needed as part of framework setup, or add `#[cfg(test)]` constructors for test-only bypass.

## Timeline *(optional)*

### Phase 0: Testing Approach Evaluation and Selection (0.5-1 day)

**Evaluation (4-8 hours)**:
1. Review testing-framework-proposal.md document with testing approaches
2. **Research SurrealDB testing options** (2-4 hours):
   - Investigate if DynamoDB-backed SurrealDB works with testcontainers
   - Research SurrealDB in-memory/embedded mode availability and viability
   - Document behavioral differences between in-memory and production
   - Evaluate mocking feasibility for database layer
3. **Score testing approaches** against evaluation criteria:
   - **Maintenance burden** (per Constitution - setup complexity, debugging ease)
   - **Project stage fit** (pragmatic for early-stage vs over-engineered)
   - **CI compatibility** (infrastructure requirements, execution time)
   - **Confidence level** (how much coverage do we actually need?)
   - **Developer adoption** (will developers actually write tests?)
4. Document tradeoffs and selection rationale (e.g., "chose in-memory for simplicity, accepting behavioral differences")
5. Present to stakeholders for approval before proceeding to implementation

### Phase 1-5: Framework Implementation and Validation (2-3 days)

**Phase 1 - Setup** (15-30 minutes):
1. Add minimal dev-dependencies to Cargo.toml (tower for oneshot)
2. Create `tests/` and `tests/common/` directory structure

**Phase 2 - Foundational Infrastructure** (2-3 hours):
3. Implement core test helper modules (`tests/common/mod.rs`, `db.rs`, `fixtures.rs`)
4. Create database setup functions (in-memory SurrealDB with migrations)
5. Create test data factory functions (accounts, resources, events, reports)

**Phase 3 - Test Validation Approach 1: Unit Tests** (1-2 hours):
6. Implement unit tests for business logic (e.g., PrincipalChainIdPart type conversions)
7. Add `#[cfg(test)] mod tests` in production code files
8. Verify unit tests pass quickly (<1 second execution)

**Phase 4 - Test Validation Approach 2: Integration with Mock Auth** (1-2 hours):
9. Create test router infrastructure (`tests/common/test_router.rs`)
10. Implement integration test with authentication bypassed (health endpoint)
11. Verify integration test passes (<5 seconds execution)

**Phase 5 - Test Validation Approach 3: Integration with Full Auth** (2-3 hours):
12. Add `#[cfg(test)]` constructors to production code for test-only auth bypass
13. Implement integration tests with full authentication middleware (report endpoint)
14. Verify full-stack integration tests pass (<10 seconds execution)
15. **STOP: Await user approval before proceeding to Phase 6 (CI/Documentation)**

**Phase 6 - CI Integration and Documentation** (3-4 hours) - **REQUIRES USER APPROVAL**:
16. Create or update GitHub Actions workflow (`.github/workflows/test.yml`)
17. Configure workflow to run cargo test, clippy, and fmt checks
18. Verify CI blocks merges when tests fail
19. Create comprehensive `tests/common/README.md` documentation

**Phase 7 - Polish and Validation** (1-2 hours) - **REQUIRES USER APPROVAL**:
20. Run cargo fmt and clippy on all test code
21. Validate determinism (10 consecutive test runs)
22. Verify test isolation (parallel vs serial execution)
23. Update CLAUDE.md with testing framework information

### Phase 6-7: Review and Approval (flexible timeline)

**Review criteria**:
- Framework setup is straightforward and well-documented
- All 3 test validation approaches are readable and maintainable
- Test helpers reduce boilerplate effectively
- Local test execution works smoothly (`cargo test` completes in <30 seconds)
- CI integration functions correctly (Phases 6-7 only)
- Framework is ready for broader adoption

**Outcomes**:
- If approved after Phase 5: Framework is functional and ready for use (CI/docs are optional enhancements)
- If approved after Phase 7: Framework is production-ready with full CI automation and comprehensive documentation
- If changes needed: Iterate based on feedback and re-submit
- If rejected: Evaluate alternative testing approaches

### Post-Implementation: Adoption (outside scope of this feature)

Testing framework established here will be used for writing tests in other features:
- Feature 001-rate-limits-we will use this testing framework for plan and rate limit tests
- Future backend features will follow established test validation approach patterns
- Framework may be enhanced based on real-world usage feedback

## Open Questions for Discussion *(optional)*

**To be resolved during framework evaluation (Phase 0)**:

1. **Testing Approach**: What level of testing is pragmatic for current project stage?
   - Option A: Unit tests with mocked database (simplest, fastest, may miss integration issues)
   - Option B: Integration tests with in-memory SurrealDB (moderate complexity, may differ from production)
   - Option C: Integration tests with containerized SurrealDB (most realistic, highest complexity)
   - Option D: Hybrid approach (unit tests for logic, selective integration tests for critical paths)
   - **Resolution approach**: Evaluate based on maintenance burden, setup complexity, CI requirements, and confidence level needed

2. **SurrealDB Testing Strategy**: How do we test database interactions?
   - Can we run Archodex's DynamoDB-backed SurrealDB fork in testcontainers? (matches production but likely complex/slow)
   - Is in-memory SurrealDB available and sufficient? (simpler but may have behavioral differences from DDB-backed version)
   - Should we use SurrealDB's embedded/memory mode if available? (need to investigate viability)
   - **Should we mock database interactions entirely?** (simplest approach, fastest tests, reduces infrastructure complexity - viable first-class option)
   - **Resolution approach**: Research SurrealDB testing options (including Archodex fork testability), evaluate complexity vs confidence tradeoff, document decision rationale

3. **Framework Selection**: Which testing framework/tools best support chosen approach?
   - cargo test + testcontainers (if integration tests with containers)
   - cargo test + in-memory database (if in-memory approach)
   - cargo test + mockall/similar (if mocking approach)
   - nextest for enhanced test runner (optional addition)
   - **Resolution approach**: Select based on chosen testing approach from questions 1 & 2

**Decisions made with reasonable defaults**:

4. **Test Data Management**: Use builder functions (more flexible, type-safe) ✓
5. **CI Environment**: Use GitHub Actions (faster feedback, free for public repos) or AWS CodeBuild ✓
6. **Coverage Targets**: No hard requirement initially, focus on critical paths ✓
7. **Performance Tests**: Separate `benches/` tests with criterion crate if needed ✓
