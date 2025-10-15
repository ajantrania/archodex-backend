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

### User Story 3 - Validate Framework with Existing Feature Tests (Priority: P1)

Development team needs to validate chosen testing approach by writing 2 example tests for existing features (resource ingestion and API key generation) to verify the approach works and is maintainable.

**Why this priority**: Writing tests for existing features proves the chosen approach is viable before using it for new development. Provides concrete examples showing whether approach is actually maintainable.

**Independent Test**: Can be fully tested by implementing 2 complete example tests that pass consistently and are straightforward to write/debug.

**Acceptance Scenarios**:

1. **Given** resource ingestion test is written using chosen approach, **When** test validates ingestion behavior (method depends on approach: verify database state, verify mocked calls, or verify function outputs), **Then** test passes and clearly demonstrates ingestion works correctly
2. **Given** API key generation test is written, **When** test generates key with encryption and protobuf encoding, **Then** test validates key can be decoded and tamper detection works correctly
3. **Given** both example tests are implemented, **When** developer runs `cargo test` locally, **Then** both tests pass within acceptable time for chosen approach (under 60 seconds)
4. **Given** tests are committed to repository, **When** GitHub Actions CI runs via `cargo test`, **Then** tests execute successfully without excessive setup complexity
5. **Given** example tests serve as documentation, **When** developer needs to write new test, **Then** they can reference example test structure and patterns and write tests without getting blocked by setup complexity

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

- What happens when framework evaluation identifies no clearly superior option (should document risks of each choice and make pragmatic decision based on project stage)?
- How does team handle stakeholder disagreement on framework choice (should present data-driven evaluation and allow informed discussion)?
- What happens when full integration tests with real SurrealDB are too complex to maintain (should evaluate simpler alternatives like in-memory database or mocks)?
- What happens when SurrealDB container (DynamoDB-backed) is incompatible with testcontainers or too slow in CI (should evaluate in-memory SurrealDB or mocking approach)?
- What happens when in-memory SurrealDB has different behavior than production DynamoDB-backed version (should document known differences and accept tradeoff for simplicity)?
- How does team decide between unit tests with mocks vs integration tests with database (should evaluate based on maintenance burden, debugging ease, and confidence level needed)?
- What happens when selected testing approach proves too cumbersome after implementation (should be prepared to pivot to simpler approach)?
- What happens when test setup is so complex that developers avoid writing tests (violates Constitution principles - should choose simpler approach)?
- What happens when CI environment has insufficient resources for selected testing approach (should fail early with clear error or choose lighter-weight approach)?
- How does system handle test flakiness due to database state or timing issues (should choose deterministic approach or accept tradeoff)?

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
- **FR-010**: Test organization MUST follow Rust conventions with unit tests in `#[cfg(test)] mod tests` and integration tests in `tests/` directory (if integration tests are chosen)

**Test Helpers and Reusability**:
- **FR-011**: Test helpers MUST be organized in `tests/common/` module for shared fixtures and utilities (if applicable to chosen approach)
- **FR-012**: Test helpers MUST provide reusable functions appropriate to chosen testing approach (database setup, mocks, or test data builders)
- **FR-013**: Test helpers MUST provide fixture generators for common test data (reports, resources, events, accounts)
- **FR-014**: Test helpers MUST reduce boilerplate code compared to inline test setup

**Framework Validation**:
- **FR-015**: Chosen approach MUST demonstrate viability by implementing 2 example tests for existing features
- **FR-016**: Example test 1 MUST validate resource ingestion behavior appropriate to chosen approach (e.g., function logic with mocked DB, or end-to-end with test database)
- **FR-017**: Example test 2 MUST validate Report API key generation logic (encryption, protobuf encoding, tamper detection) - can use mocked or real crypto functions
- **FR-018**: Example tests MUST use test helpers to demonstrate reusability and reduce boilerplate
- **FR-019**: Example tests MUST be straightforward to write, run, and debug without excessive setup steps

**Documentation**:
- **FR-020**: Framework documentation MUST be created in `tests/common/README.md` with patterns and examples
- **FR-021**: Documentation MUST explain how to write new tests using established patterns
- **FR-022**: Documentation MUST include examples appropriate to chosen approach (database setup, mock setup, or test data builders) and assertions

**CI Integration**:
- **FR-023**: GitHub Actions workflow MUST run `cargo test` automatically on every push to any branch
- **FR-024**: GitHub Actions MUST block pull request merges if any tests fail
- **FR-025**: GitHub Actions setup MUST be straightforward with minimal infrastructure requirements (complexity acceptable for project stage)
- **FR-030**: Tests SHOULD be runnable locally via ACT to match GitHub Actions environment (desirable but not mandatory)

**Framework Quality**:
- **FR-026**: Framework MUST provide clear error messages when setup fails (e.g., missing dependencies, configuration errors)
- **FR-027**: Tests MUST be deterministic and produce consistent results across multiple runs
- **FR-028**: Framework MUST support chosen testing approach (unit tests, integration tests, or hybrid) effectively
- **FR-029**: If approach uses test databases or resources, cleanup MUST occur automatically (tests never deploy against real backends)

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

- **SC-005**: Selected framework dependencies are successfully added to Cargo.toml and compile without errors
- **SC-006**: Test helper modules (`tests/common/`) are created with reusable functions appropriate to chosen approach within 2 hours
- **SC-007**: Test setup (database, mocks, or data builders) is straightforward for developers to use and understand
- **SC-008**: Tests have appropriate isolation based on chosen approach (no shared state for integration tests, clean mocks for unit tests)
- **SC-009**: 2 example tests (resource ingestion + API key) are implemented and pass consistently using chosen approach
- **SC-010**: Testing framework setup (evaluation + dependencies + helpers + examples) completes within 2-3 days total
- **SC-011**: Developers can write and run new tests without excessive setup complexity (aligns with Constitution principles)

### Test Execution Performance

| Scenario | Target Time | Rationale |
|----------|-------------|-----------|
| Single unit test | < 1 second | Fast feedback for TDD workflow |
| Single integration test | < 5 seconds | Acceptable for database operations (in-memory) |
| Full local test suite | < 30 seconds | Current stage (2 example tests), may increase to 60s as suite grows |
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

- **SC-021**: `tests/common/README.md` documentation is created with patterns, examples, and usage instructions
- **SC-022**: Documentation enables new developer to write their first test following patterns within 15 minutes
- **SC-023**: Example tests serve as reference implementation demonstrating helper usage and best practices

### CI Integration

- **SC-024**: CI configuration automatically runs `cargo test` on every push to any branch
- **SC-025**: CI blocks pull request merges when any test fails
- **SC-026**: CI setup is maintainable and doesn't require excessive infrastructure or configuration for chosen approach

## Assumptions *(optional)*

- Developers are familiar with Rust testing conventions (`#[test]`, `#[tokio::test]`, `cargo test`)
- CI environment is GitHub Actions (ACT compatibility desirable but not required)
- Tests never run against deployed production or staging backends (only local ephemeral databases or mocks with point-in-time restoration each run)
- Test execution times per performance table above are acceptable for developer workflow
- Builder functions are preferred over JSON fixtures for test data generation (more flexible and type-safe)
- Existing codebase has report ingestion and API key generation features that can be used for example tests
- All tests must be deterministic and repeatable (no flaky tests due to timing or network issues)
- Test data uses realistic values but can be simplified for readability (e.g., predictable IDs like "res1", "res2")
- Framework validation with 2 example tests is sufficient proof of viability before broader adoption
- Rate limiting feature tests will be written separately in feature 001-rate-limits-we using selected approach
- Team has capacity to spend 4-8 hours evaluating testing approaches and SurrealDB options before implementation
- Framework evaluation document (testing-framework-proposal.md) contains sufficient detail for informed decision-making
- Archodex SurrealDB fork (DynamoDB-backed) may or may not be testable depending on research outcomes
- Mocking SurrealDB interactions is a viable and acceptable option if it reduces complexity significantly

## Dependencies *(optional)*

**Pre-Selection** (for evaluation):
- **testing-framework-proposal.md**: Document comparing testing approaches and framework options
- **Stakeholder availability**: For reviewing and approving approach/framework selection
- **Archodex SurrealDB fork**: Located at https://github.com/Archodex/surrealdb (local: `/Users/ajantrania/code/archodex/surrealdb`) - DynamoDB-backed version used in production

**Post-Selection** (implementation dependencies vary by chosen approach):
- **Potential option - mockall or similar**: Mocking library for database layer (if mocking approach chosen) - viable first-class option
- **Potential option - testcontainers**: Library for ephemeral Docker containers (if containerized integration tests chosen)
- **Potential option - rstest**: Parameterized testing and test fixtures (if selected)
- **Potential option - tokio-test**: Testing utilities for async Tokio code (likely needed regardless)
- **Potential option - axum-test**: HTTP request/response testing for Axum framework (if selected)
- **GitHub Actions**: CI environment (ACT for local testing)
- **SurrealDB version/mode**: Depends on approach (Archodex fork with DDB, in-memory, containerized, or mocked)
- **Existing report ingestion feature**: Required for example test 1
- **Existing API key generation feature**: Required for example test 2

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
- **Test data complexity**: Fixture generation may become complex for realistic scenarios. Mitigation: Start with simple builder functions for 2 example tests, refactor if patterns emerge.
- **Flaky tests**: Network or timing issues may cause intermittent failures. Mitigation: Use deterministic test data and avoid time-based assertions.
- **Framework proves inadequate during validation**: Selected framework may not meet needs after implementation. Mitigation: Keep 2-3 day validation phase focused, get quick feedback, and be prepared to pivot if necessary.
- **Existing features may lack testability**: Report ingestion or API key features may be difficult to test. Mitigation: Refactor for testability if needed as part of framework setup.

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

### Phase 1: Framework Setup + Validation (2-3 days)

**Day 1 (4-6 hours)**:
1. Add selected framework dependencies to `[dev-dependencies]` in Cargo.toml
2. Create `tests/common/` directory structure
3. Implement `tests/common/db.rs` with database setup helpers (names depend on framework choice)
4. Implement `tests/common/fixtures.rs` with test data builders
5. Write integration test for basic resource ingestion (3 resources)
6. Write integration test for Report API key generation with crypto validation
7. Verify `cargo test` passes locally with both tests green

**Day 2 (2-3 hours)**:
8. Create `tests/common/README.md` documenting patterns, helpers, and examples
9. Add or update CI configuration file (`.github/workflows/test.yml` or AWS CodeBuild config)
10. Push to repository and verify CI runs tests successfully with selected framework
11. Verify CI blocks merges when tests fail (test with intentional failure)
12. Present completed framework for final approval

### Phase 2: Review and Approval (flexible timeline)

**Review criteria**:
- Framework setup is straightforward and well-documented
- Example tests are readable and maintainable
- Test helpers reduce boilerplate effectively
- Local test execution works smoothly (`cargo test`)
- CI integration functions correctly
- Framework is ready for broader adoption

**Outcomes**:
- If approved: Framework becomes standard for all future testing (including feature 001-rate-limits-we)
- If changes needed: Iterate based on feedback and re-submit
- If rejected: Evaluate alternative testing approaches

### Phase 3: Adoption (outside scope of this feature)

Framework will be used for writing tests in other features:
- Feature 001-rate-limits-we will use this framework for plan and rate limit tests
- Future backend features will follow established patterns
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
