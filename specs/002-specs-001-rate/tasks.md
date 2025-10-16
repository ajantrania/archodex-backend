# Tasks: Testing Framework Setup and Validation

**Branch**: `002-specs-001-rate`
**Input**: Design documents from `/specs/002-specs-001-rate/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅

**Tests**: This feature establishes a testing framework - the "tests" validate the framework itself through 3 example tests (revised from 2).

**Organization**: Tasks are grouped by user story to enable independent implementation and validation of each testing approach.

---

## ⚠️ IMPORTANT: User Approval Required

**Phase 6 (CI Integration and Documentation) requires explicit user approval before starting.**

- **Phases 1-5** (T001-T038): Can be implemented without approval - establishes functional testing framework
- **Phase 6** (T039-T057): **STOP and await user approval** - adds CI automation and comprehensive documentation
- **Phase 7** (T058-T066): **STOP and await user approval** - final polish and validation

**The testing framework is FULLY FUNCTIONAL after Phase 5 completes.** Phase 6 and 7 are optional enhancements that require user confirmation before proceeding.

---

## 🎯 CRITICAL: Test Quality Requirements

**ALL tests MUST be meaningful and test actual Archodex business logic.**

### Quality Standards:

1. **Test Real Business Logic**: Every test must validate actual Archodex backend functionality, not just toy examples
2. **No Trivial/Meaningless Tests**: If a test becomes trivial or meaningless during implementation, STOP and flag the issue
3. **Flag Blockers Immediately**: If implementation encounters issues that would require reducing test quality or meaningfulness, STOP and report:
   - What business logic was intended to be tested
   - What blocker was encountered
   - Why the test cannot be implemented meaningfully
   - Request guidance before proceeding

### Examples of Meaningful vs Meaningless Tests:

**✅ MEANINGFUL** (Keep these):
- Testing PrincipalChainIdPart conversion logic (real production code for SurrealDB serialization)
- Testing report ingestion with actual resource/event processing
- Testing authentication middleware with actual auth flow
- Testing API key encryption/decryption with real crypto logic

**❌ MEANINGLESS** (STOP and flag):
- Testing "1 + 1 = 2" just to have a passing test
- Testing a mock function that returns hardcoded values
- Testing trivial getters/setters with no business logic
- Creating fake business logic just to have something to test

### Implementation Guidance:

If during implementation you discover:
- ❌ The proposed test file/function doesn't exist or isn't suitable
- ❌ The business logic is too tightly coupled to external services
- ❌ The test would require significant refactoring to be meaningful
- ❌ The test can only validate trivial/toy behavior

**Then STOP immediately and:**
1. Document what you were trying to test
2. Explain the blocker encountered
3. Propose alternatives if available
4. Request user guidance before proceeding

**Do NOT:**
- Write trivial tests just to check off the task
- Create fake business logic just to have something to test
- Reduce test scope to meaninglessness to avoid blockers
- Skip reporting issues and move on to next task

---

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Exact file paths included in descriptions

## Path Conventions
- Single Rust project: `src/`, `tests/` at repository root
- Workspace members: `server/`, `lambda/`, `migrator/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic dependencies for testing framework

- [ ] T001 [P] [Setup] Add minimal dev-dependencies to `Cargo.toml` (`tower = { version = "0.5", features = ["util"] }` for oneshot)
- [ ] T002 [P] [Setup] Create `tests/` directory structure at project root
- [ ] T003 [P] [Setup] Create `tests/common/` subdirectory for shared test helpers

**Checkpoint**: Basic project structure ready for test helper implementation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core test infrastructure that MUST be complete before ANY validation tests can be written

**⚠️ CRITICAL**: No example test work can begin until this phase is complete

- [ ] T004 [Foundational] Create `tests/common/mod.rs` with module re-exports for `db`, `fixtures`, and `test_router`
- [ ] T005 [Foundational] Implement `tests/common/db.rs` with `create_test_db()` function (in-memory SurrealDB using `Surreal::new::<Mem>()`)
- [ ] T006 [Foundational] Implement `create_test_db_with_migrations()` in `tests/common/db.rs` (applies migrations via `migrator::migrate_accounts_database`)
- [ ] T007 [Foundational] Implement `create_test_db_with_account(account_id: &str)` in `tests/common/db.rs` (returns tuple of DB and TestAccount)
- [ ] T008 [P] [Foundational] Implement `create_test_account(id: &str, name: &str)` factory function in `tests/common/fixtures.rs`
- [ ] T009 [P] [Foundational] Implement `create_test_resource(id: &str)` factory function in `tests/common/fixtures.rs`
- [ ] T010 [P] [Foundational] Implement `create_test_resources(count: usize)` factory function in `tests/common/fixtures.rs`
- [ ] T011 [P] [Foundational] Implement `create_test_event(id: usize)` factory function in `tests/common/fixtures.rs`
- [ ] T012 [P] [Foundational] Implement `create_test_events(count: usize)` factory function in `tests/common/fixtures.rs`
- [ ] T013 [P] [Foundational] Implement `TestReportBuilder` struct with builder pattern in `tests/common/fixtures.rs` (methods: `new()`, `with_resources(count)`, `with_events(count)`, `build()`)
- [ ] T014 [P] [Foundational] Implement `create_test_report(num_resources: usize, num_events: usize)` factory function in `tests/common/fixtures.rs`

**Checkpoint**: Foundation ready - validation tests can now be written

---

## Phase 3: User Story 1 - Unit Test Example (Priority: P1) 🎯 MVP

**Goal**: Demonstrate unit testing pattern with pure logic test (no external dependencies)

**Independent Test**: Run `cargo test test_principal_chain_id_part_round_trip` - should pass and complete in <1ms

**Validation Approach**: Pure logic tests for type conversions (TryFrom/From traits)

**🎯 TEST QUALITY REQUIREMENT**: These tests MUST validate real PrincipalChainIdPart business logic used in Archodex production code. If src/principal_chain.rs doesn't exist or doesn't have meaningful conversion logic, STOP and flag the issue instead of writing trivial tests.

### Implementation for User Story 1

- [ ] T015 [US1] Add `#[cfg(test)] mod tests` section at bottom of `src/principal_chain.rs`
- [ ] T016 [US1] Implement `test_principal_chain_id_part_round_trip()` unit test in `src/principal_chain.rs` (validates TryFrom/From traits for SurrealDB Object conversion with event field)
- [ ] T017 [US1] Implement `test_principal_chain_id_part_without_event()` unit test in `src/principal_chain.rs` (validates conversion with None event)
- [ ] T018 [US1] Implement `test_principal_chain_id_part_invalid_object_missing_id()` unit test in `src/principal_chain.rs` (validates error handling for missing required fields)
- [ ] T019 [US1] Implement `test_principal_chain_id_part_invalid_event_type()` unit test in `src/principal_chain.rs` (validates error handling for invalid field types)
- [ ] T020 [US1] Run `cargo test principal_chain` to verify all unit tests pass with execution time <1 second

**Checkpoint**: Unit testing pattern validated - pure logic tests working without external dependencies

---

## Phase 4: User Story 2 - Integration Test with Auth Bypass (Priority: P1)

**Goal**: Demonstrate integration testing pattern with test router that bypasses authentication

**Independent Test**: Run `cargo test test_health_endpoint` - should pass with HTTP request → handler flow

**Validation Approach**: HTTP layer testing with mock auth (Extension injection)

**🎯 TEST QUALITY REQUIREMENT**: This test should validate a real HTTP endpoint if possible. If only trivial endpoints exist (like a hardcoded "Ok" response), consider testing a more meaningful endpoint with actual business logic. If no suitable endpoints exist, STOP and flag the issue.

### Test Router Infrastructure for User Story 2

- [ ] T021 [US2] Create `tests/common/test_router.rs` module for test-specific routers
- [ ] T022 [US2] Implement `create_test_router()` function in `tests/common/test_router.rs` (simple router with `/health` endpoint, no auth middleware)
- [ ] T023 [US2] Update `tests/common/mod.rs` to re-export test_router module

### Integration Test Implementation for User Story 2

- [ ] T024 [US2] Create `tests/health_check_integration_test.rs` integration test file
- [ ] T025 [US2] Implement `test_health_endpoint()` integration test in `tests/health_check_integration_test.rs` (uses `tower::ServiceExt::oneshot()` to test HTTP request/response)
- [ ] T026 [US2] Run `cargo test test_health_endpoint` to verify integration test passes with execution time <5 seconds

**Checkpoint**: Integration testing pattern validated - HTTP layer tests working with test routers

---

## Phase 5: User Story 3 - Integration Test with Full Auth Middleware (Priority: P1) 🎯 COMPREHENSIVE

**Goal**: Demonstrate full authentication middleware testing using production router with test-specific auth token validation

**Independent Test**: Run `cargo test report_with_auth` - should pass with complete auth → account → handler → DB flow

**Validation Approach**: Full middleware chain testing with production router

**DESIGN DECISION**: Use production router with `#[cfg(test)]`-gated auth bypass for test tokens

**🎯 TEST QUALITY REQUIREMENT**: These tests MUST validate real authentication middleware and report ingestion business logic. If the /report endpoint doesn't exist, or if auth middleware can't be meaningfully tested, or if report processing is trivial, STOP and flag the issue. Do not create fake endpoints or fake business logic just to have something to test.

### Production Code Changes for User Story 3 (Test-Only Constructors)

- [ ] T027 [P] [US3] Add `#[cfg(test)] pub(crate) fn new_for_testing(principal: User) -> Self` constructor to `DashboardAuth` in `src/auth.rs`
- [ ] T028 [P] [US3] Add `#[cfg(test)] pub(crate) fn new_for_testing(account_id: String, key_id: u32) -> Self` constructor to `ReportApiKeyAuth` in `src/auth.rs`
- [ ] T029 [P] [US3] Add `#[cfg(test)]` bypass logic to `ReportApiKey::validate_value()` in `src/report_api_key.rs` (recognizes "test_token_{account_id}" format in test builds only, returns `Ok((account_id, 99999))`)
- [ ] T030 [P] [US3] Add `#[cfg(test)] pub(crate) fn new_for_testing(id: String) -> Self` constructor to `Account` in `src/account.rs` (if Account struct doesn't already have suitable constructor)

### Test Helpers for User Story 3

- [ ] T031 [P] [US3] Implement `setup_test_env()` function in `tests/common/mod.rs` (sets required environment variables like `ARCHODEX_DOMAIN` for test mode)
- [ ] T032 [P] [US3] Implement `get_test_accounts_db()` async function in `tests/common/db.rs` (returns in-memory accounts database instance, separate from per-test DBs)
- [ ] T033 [P] [US3] Implement `create_test_auth_token(account_id: &str)` function in `tests/common/fixtures.rs` (generates "test_token_{account_id}" format string)
- [ ] T034 [P] [US3] Implement `create_test_report_request(num_resources: usize, num_events: usize)` in `tests/common/fixtures.rs` (returns JSON-serializable report request matching API contract)

### Integration Test Implementation for User Story 3

- [ ] T035 [US3] Create `tests/report_with_auth_test.rs` integration test file
- [ ] T036 [US3] Implement `test_report_endpoint_with_full_auth_middleware()` in `tests/report_with_auth_test.rs` (uses production router `crate::router::router()`, validates auth middleware execution, account loading, and report ingestion with assertions: (1) HTTP 200 status code, (2) account record exists in database with correct ID, (3) report resources count in database matches request count, (4) events stored in database with valid timestamps)
- [ ] T037 [US3] Implement `test_report_endpoint_rejects_invalid_auth()` in `tests/report_with_auth_test.rs` (validates auth rejection for missing/invalid tokens, expects UNAUTHORIZED status)
- [ ] T038 [US3] Run `cargo test report_with_auth` to verify auth middleware tests pass (should execute full auth → account → handler → DB path)

**Checkpoint**: Full authentication middleware testing validated - complete request flow working with production router

---

## 🛑 STOP HERE - AWAIT USER CONFIRMATION BEFORE PROCEEDING TO PHASE 6

**After Phase 5 completes**, STOP implementation and validate before proceeding.

**What Phase 5 delivers**:
- Test directory structure (`tests/`, `tests/common/`)
- Test helper modules (`db.rs`, `fixtures.rs`, `test_router.rs`)
- 3 example test approaches validated (unit, integration mock auth, integration full auth)
- Framework validated locally with `cargo test`
- All test helpers reduce boilerplate effectively
- Testing framework is FUNCTIONAL and ready to use

**Before continuing to Phase 6**:
1. Run `cargo test` locally to verify all tests pass (<30 seconds)
2. Review all 3 example test approaches to ensure they meet requirements
3. Confirm framework is usable and maintainable
4. **⚠️ GET EXPLICIT APPROVAL FROM USER to proceed with CI automation (Phase 6) ⚠️**

**What's NOT in Phase 6 scope**: Phase 6 only adds CI automation and documentation. The framework itself is already complete and functional after Phase 5.

**DO NOT START PHASE 6 WITHOUT EXPLICIT USER APPROVAL**

---

## Phase 6: User Story 4 - CI Integration and Documentation (Priority: P2)

**⚠️⚠️⚠️ DO NOT START THIS PHASE WITHOUT EXPLICIT USER APPROVAL ⚠️⚠️⚠️**

**Goal**: Configure GitHub Actions CI for automated testing and provide developer documentation

**Independent Test**: Push code and verify GitHub Actions workflow runs tests successfully

### CI Configuration for User Story 4

- [ ] T039 [US4] Create `.github/workflows/test.yml` CI configuration file (or update if exists)
- [ ] T040 [US4] Configure CI workflow with name "Test Suite" to trigger on push to main branch and on all pull requests
- [ ] T041 [US4] Add checkout step using `actions/checkout@v4`
- [ ] T042 [US4] Add Rust toolchain setup using `actions-rust-lang/setup-rust-toolchain@v1` with stable toolchain and clippy component
- [ ] T043 [US4] Add test execution step: `cargo test --all-features --verbose`
- [ ] T044 [US4] Add clippy check step: `cargo clippy -- -D warnings`
- [ ] T045 [US4] Add format check step: `cargo fmt -- --check`

### CI Validation for User Story 4

- [ ] T046 [US4] Push code to branch and verify GitHub Actions workflow automatically triggers
- [ ] T047 [US4] Verify workflow succeeds when all tests pass (green status on PR)
- [ ] T048 [US4] Create intentional test failure (modify a test to assert false), push, and verify workflow fails with clear error message
- [ ] T049 [US4] Fix test, push, and verify workflow succeeds again (demonstrates CI correctly enforces quality)
- [ ] T050 [US4] Verify total CI workflow execution completes within 3-5 minutes from push to result

### Documentation for User Story 4

- [ ] T051 [P] [US4] Create comprehensive `tests/common/README.md` documentation with testing patterns, helper function usage, and examples
- [ ] T052 [P] [US4] Document database setup helpers in README: `create_test_db()`, `create_test_db_with_migrations()`, `create_test_db_with_account()`
- [ ] T053 [P] [US4] Document test data fixtures in README: account, resource, event, report, API key factory functions
- [ ] T054 [P] [US4] Document test router patterns in README: `create_test_router()`, auth bypass strategy with `#[cfg(test)]` constructors
- [ ] T055 [P] [US4] Add example test patterns to README: Pattern 1 (unit test inline), Pattern 2 (integration with mock auth), Pattern 3 (integration with full auth)
- [ ] T056 [P] [US4] Add troubleshooting section to README: common errors ("Cannot find module common", "Database connection failed", "Test timeout"), debugging tips (--nocapture, --exact, --test-threads=1)
- [ ] T057 [US4] Verify documentation meets acceptance criteria: new developer can write first test following patterns within 15 minutes

**Checkpoint**: CI integration complete - automated testing on every push, comprehensive documentation available

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, code quality, and framework refinement

- [ ] T058 [P] [Polish] Run `cargo fmt` to format all test code (both integration tests and inline unit tests)
- [ ] T059 [P] [Polish] Run `cargo clippy -- -D warnings` to ensure no warnings in test code
- [ ] T060 [Polish] Run full test suite with `cargo test --verbose` and verify execution time <30 seconds for all tests
- [ ] T061 [Polish] Verify all test helpers work correctly by reviewing test helper usage in all 3 example tests
- [ ] T062 [Polish] Validate quickstart.md instructions by following guide step-by-step and writing a new test (simulate new developer experience)
- [ ] T063 [Polish] (Optional) Run ACT locally (`act`) to verify CI workflow works in local environment (requires Docker, skip if not available)
- [ ] T064 [Polish] Update `CLAUDE.md` with testing framework information: add "cargo test" to Commands section, add testing technologies to Active Technologies
- [ ] T065 [Polish] Run determinism validation: execute `cargo test` 10 consecutive times and confirm identical results (no flaky tests)
- [ ] T066 [Polish] Verify test isolation: run `cargo test -- --test-threads=1` (serial) and `cargo test` (parallel), confirm identical results

**Checkpoint**: Framework is production-ready and ready for use in feature 001-rate-limits-we

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all validation tests
- **User Stories 1-3 (Phase 3-5)**: All depend on Foundational phase completion
  - US1 (Unit Test): Can start immediately after Foundational
  - US2 (Integration Test with Mock Auth): Can start immediately after Foundational
  - US3 (Integration Test with Full Auth): Can start immediately after Foundational
  - All three validation approaches can proceed in parallel if multiple developers available
- **User Story 4 (Phase 6)**: Depends on completion of US1, US2, US3 (needs working tests to validate)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1 - Unit Test)**: Depends on Foundational (Phase 2) only - no dependencies on other stories
- **User Story 2 (P1 - Integration Mock Auth)**: Depends on Foundational (Phase 2) only - no dependencies on other stories
- **User Story 3 (P1 - Integration Full Auth)**: Depends on Foundational (Phase 2) only - no dependencies on other stories
- **User Story 4 (P2 - CI/Docs)**: Depends on US1, US2, US3 completion (needs working tests to validate in CI)

### Within Each User Story

- **User Story 1 (Unit Tests)**: Tasks T015-T020 must be sequential (tests build on each other in same file)
- **User Story 2 (Integration Mock)**: T021-T023 (setup) sequential, then T024-T026 (tests) sequential
- **User Story 3 (Integration Full Auth)**:
  - T027-T030 (prod code changes) can be parallel (different files, all `#[cfg(test)]` constructors)
  - T031-T034 (test helpers) can be parallel (different helper functions in different modules)
  - T035-T038 (tests) must be sequential (same file, tests share setup patterns)
- **User Story 4 (CI/Docs)**:
  - T039-T045 (CI config) must be sequential (single workflow file)
  - T046-T050 (CI validation) must be sequential (depends on workflow being deployed)
  - T051-T057 (docs) can be parallel (different sections of README)
  - CI group and Docs group can be parallel with each other

### Parallel Opportunities

**Phase 1 (Setup)**: All tasks T001-T003 can run in parallel (different directories/files)

**Phase 2 (Foundational)**:
- T004 must complete first (module structure required for others)
- T005-T007 must be sequential (each builds on previous in same file)
- T008-T014 can all run in parallel (different fixture functions, marked [P])

**Phase 3 (US1)**: All tasks sequential (same file, incremental test additions)

**Phase 4 (US2)**: T021-T023 sequential (test router setup), then T024-T026 sequential (test implementation)

**Phase 5 (US3)**:
- T027-T030 all parallel (different files, marked [P])
- T031-T034 all parallel (different helper functions, marked [P])
- T035-T038 sequential (same test file)

**Phase 6 (US4)**:
- T039-T045 sequential (CI workflow)
- T046-T050 sequential (CI validation)
- T051-T057 all parallel (documentation sections, marked [P])
- CI work and Docs work can be parallel to each other

**Phase 7 (Polish)**:
- T058-T059 can run in parallel (different quality checks, marked [P])
- T060-T066 should be sequential (validation builds on previous validations)

---

## Parallel Example: Foundational Phase

```bash
# After T004 completes (module structure), launch all fixture functions in parallel:
Task T008: "Implement create_test_account() in tests/common/fixtures.rs"
Task T009: "Implement create_test_resource() in tests/common/fixtures.rs"
Task T010: "Implement create_test_resources() in tests/common/fixtures.rs"
Task T011: "Implement create_test_event() in tests/common/fixtures.rs"
Task T012: "Implement create_test_events() in tests/common/fixtures.rs"
Task T013: "Implement TestReportBuilder in tests/common/fixtures.rs"
Task T014: "Implement create_test_report() in tests/common/fixtures.rs"
```

## Parallel Example: User Story 3 (Full Auth Testing)

```bash
# Launch all production code changes in parallel:
Task T027: "Add #[cfg(test)] constructor to DashboardAuth in src/auth.rs"
Task T028: "Add #[cfg(test)] constructor to ReportApiKeyAuth in src/auth.rs"
Task T029: "Add #[cfg(test)] bypass to ReportApiKey::validate_value in src/report_api_key.rs"
Task T030: "Add #[cfg(test)] constructor to Account in src/account.rs"

# Then launch all test helpers in parallel:
Task T031: "Implement setup_test_env() in tests/common/mod.rs"
Task T032: "Implement get_test_accounts_db() in tests/common/db.rs"
Task T033: "Implement create_test_auth_token() in tests/common/fixtures.rs"
Task T034: "Implement create_test_report_request() in tests/common/fixtures.rs"
```

---

## Implementation Strategy

### MVP First (US1 + US2 + US3 Only)

1. Complete Phase 1: Setup → Basic structure ready (15 minutes)
2. Complete Phase 2: Foundational → Test helpers available (2-3 hours)
3. Complete Phase 3: User Story 1 → Unit testing validated (1-2 hours)
4. Complete Phase 4: User Story 2 → Integration testing (mock auth) validated (1-2 hours)
5. Complete Phase 5: User Story 3 → Integration testing (full auth) validated (2-3 hours)
6. **🛑 STOP and VALIDATE**: Run `cargo test` - all 3 validation approaches working (<30 seconds execution)
7. **🛑 AWAIT USER APPROVAL**: Do NOT proceed to Phase 6 without explicit user confirmation
8. Framework ready for use in feature 001-rate-limits-we after Phase 5

### Full Delivery (Including CI and Docs)

**⚠️ Phase 6 and Phase 7 require explicit user approval before starting**

1. Complete Phases 1-5 per MVP strategy above → Framework validated locally (6-9 hours)
2. **🛑 STOP - Get user approval before proceeding**
3. Complete Phase 6: User Story 4 → CI automation + documentation (3-4 hours) **[Only after user approval]**
4. Complete Phase 7: Polish → Production-ready framework (1-2 hours) **[Only after user approval]**
5. **Total without Phase 6/7: 6-9 hours** (Phases 1-5 only)
6. **Total with Phase 6/7: 10-15 hours** (2-3 days as per spec timeline, only if approved)

### Incremental Value Delivery

1. **After Phase 2**: Test helpers available → Can write tests manually without examples
2. **After Phase 3**: Unit testing pattern validated → Usable for pure logic tests
3. **After Phase 4**: Integration testing (mock auth) validated → Usable for API tests without real auth
4. **After Phase 5**: Integration testing (full auth) validated → Usable for complete flow tests → **✅ FRAMEWORK FUNCTIONAL & READY TO USE**
5. **🛑 STOP POINT**: Get explicit user approval before proceeding to Phase 6
6. **After Phase 6** (optional): CI automation + documentation → Production-ready framework **[Requires user approval]**
7. **After Phase 7** (optional): Polished and validated → Ready for broader adoption **[Requires user approval]**

### Parallel Team Strategy

With 2 developers:

**Sprint 1 (Setup + Foundational)**: Both developers work together (4-6 hours)
- Developer A: Phase 1 Setup (T001-T003)
- Developer B: Phase 2 module structure (T004-T007)
- Both: Phase 2 fixtures in parallel (T008-T014)

**Sprint 2 (Validation Tests)**: Parallel development (4-6 hours)
- Developer A: US1 (T015-T020) + US2 (T021-T026)
- Developer B: US3 (T027-T038)

**🛑 STOP and get user approval before Sprint 3**

**Sprint 3 (CI + Docs + Polish)**: Both developers work together (4-6 hours) **[ONLY AFTER USER APPROVAL]**
- Developer A: CI configuration and validation (T039-T050)
- Developer B: Documentation (T051-T057)
- Both: Polish and final validation (T058-T066)

---

## Notes

- **[P] tasks**: Different files, no dependencies, can run in parallel
- **[Story] label**: Maps task to specific user story (US1=Unit, US2=Integration Mock, US3=Integration Full, US4=CI/Docs)
- **Each user story validates a different testing approach**: unit, integration with mock auth, integration with full auth
- **Tests are the FEATURE ITSELF** (not optional) - they validate the framework works
- **Foundational phase is CRITICAL** - all example tests depend on test helpers being available
- **`#[cfg(test)]` constructors ensure production security** - test-only code never compiled in release builds
- **Framework follows Constitution principle**: simplicity over sophistication (in-memory DB, minimal tooling)
- **Security-safe auth bypass**: Test token format "test_token_{account_id}" only recognized in `#[cfg(test)]` builds
- **Stop at any checkpoint** to validate user story independently
- **Commit after each task** or logical group of related tasks
- **Avoid**: complex test fixtures initially, over-engineering, production code changes beyond `#[cfg(test)]` constructors

### 🎯 Test Quality Enforcement

- **ALL tests must be meaningful** - test real Archodex business logic, not toy examples
- **If a test becomes trivial during implementation**: STOP immediately and flag the issue to user
- **If blockers prevent meaningful testing**: STOP, document the blocker, propose alternatives, request guidance
- **Never write meaningless tests just to complete a task** - quality over completion
- **Examples are in research.md** - use them as reference for what "meaningful" looks like

---

## Validation Checklist

After completing all tasks, verify:

- ✅ `cargo test` passes with all tests completing in <30 seconds
- ✅ Unit tests (US1): 4-5 tests in `src/principal_chain.rs` passing, <1 second execution
- ✅ Integration tests (US2): 1 test in `tests/health_check_integration_test.rs` passing, <5 seconds
- ✅ Integration tests (US3): 2 tests in `tests/report_with_auth_test.rs` passing (full auth flow + rejection), <10 seconds
- ✅ Test helpers in `tests/common/` reduce boilerplate by 70%+ (verify by comparing test code with/without helpers)
- ✅ Test isolation verified: each test gets fresh in-memory database, no shared state
- ✅ Determinism verified: 10 consecutive runs produce identical results
- ✅ `cargo clippy -- -D warnings` passes with no warnings in test code
- ✅ `cargo fmt --check` passes (all test code formatted)
- ✅ GitHub Actions CI workflow runs successfully (<5 minutes total)
- ✅ CI blocks merges when tests fail (validated with intentional failure)
- ✅ Documentation in `tests/common/README.md` enables new developer to write test in 15 minutes
- ✅ Production code changes are security-safe (all test helpers marked `#[cfg(test)]`)
- ✅ Framework is ready for use in feature 001-rate-limits-we

---

## Success Metrics (from spec.md)

### Framework Setup and Viability

- **SC-005**: ✅ Dependencies compile without errors (`cargo build --tests`)
- **SC-006**: ✅ Test helpers created within 2 hours (Phase 2 Foundational tasks T004-T014)
- **SC-007**: ✅ Test setup is straightforward (simple factory functions, in-memory DB, no Docker)
- **SC-009**: ✅ 3 example tests implemented and pass (US1: 4-5 unit tests, US2: 1 integration test, US3: 2 integration tests)
- **SC-010**: ✅ Framework setup completes within 2-3 days (10-15 hours estimated)
- **SC-011**: ✅ Developers can write and run tests without excessive complexity

### Test Execution Performance

- **SC-012**: ✅ `cargo test` passes locally within <30 seconds (all 7-8 tests)
- **SC-013**: ✅ Individual tests run quickly per targets:
  - Unit tests: <1 second
  - Integration test (mock auth): <5 seconds
  - Integration tests (full auth): <10 seconds combined
- **SC-014**: ✅ CI pipeline completes within 3-5 minutes (test + clippy + fmt)

### Framework Quality and Usability

- **SC-015**: ✅ Test helpers reduce boilerplate by 70%+ (factory functions, DB setup helpers)
- **SC-016**: ✅ Test isolation appropriate (in-memory DB per test, no shared state)
- **SC-017**: ✅ Framework provides clear error messages when setup fails
- **SC-019**: ✅ Tests are deterministic (10 consecutive runs produce identical results)
- **SC-020**: ✅ Debugging failing tests is straightforward (--nocapture, --exact flags documented)

### Documentation and Knowledge Transfer

- **SC-021**: ✅ `tests/common/README.md` created with patterns, examples, and usage instructions
- **SC-022**: ✅ Documentation enables new developer to write first test in 15 minutes
- **SC-023**: ✅ Example tests serve as reference implementation (3 different approaches documented)

### CI Integration

- **SC-024**: ✅ CI runs `cargo test` on every push to any branch
- **SC-025**: ✅ CI blocks merges when tests fail (validated with intentional failure)
- **SC-026**: ✅ CI setup is maintainable (standard GitHub Actions workflow, no Docker required)

---

## Timeline Estimate

**Total: 2-3 days (10-15 hours)** per spec Phase 1 timeline

### Detailed Breakdown:

- **Phase 1 (Setup)**: 15-30 minutes
- **Phase 2 (Foundational)**: 2-3 hours (including test helpers and fixtures)
- **Phase 3 (US1 - Unit Tests)**: 1-2 hours (4-5 unit tests in principal_chain.rs)
- **Phase 4 (US2 - Integration Mock)**: 1-2 hours (test router + 1 health check test)
- **Phase 5 (US3 - Integration Full Auth)**: 2-3 hours (prod code changes + helpers + 2 tests)
- **Phase 6 (US4 - CI/Docs)**: 3-4 hours (GitHub Actions + comprehensive README)
- **Phase 7 (Polish)**: 1-2 hours (fmt, clippy, validation, CLAUDE.md update)

### Day-by-Day Schedule:

**Day 1 (6-8 hours)**:
- Morning: Phase 1 + Phase 2 (Setup + Foundational) → 2.5-3.5 hours
- Afternoon: Phase 3 + Phase 4 (US1 + US2) → 2-4 hours
- Evening: Start Phase 5 (US3) → 1-2 hours

**Day 2 (4-6 hours)**:
- Morning: Complete Phase 5 (US3) → 1-2 hours
- Afternoon: Phase 6 (US4 - CI + Docs) → 3-4 hours

**Day 3 (Buffer, 0-2 hours)**:
- Phase 7 (Polish + Final Validation) → 1-2 hours
- Buffer for unexpected issues or review feedback

---

## Feature Readiness Gates

### Gate 1: Foundation Ready (After Phase 2)
- ✅ Directory structure created
- ✅ Test helpers available
- ✅ `cargo test` compiles (even with no tests)
- **Decision**: Proceed to validation tests

### Gate 2: Framework Validated Locally (After Phase 5)
- ✅ All 3 testing approaches working (unit, integration mock, integration full)
- ✅ `cargo test` passes in <30 seconds
- ✅ Test helpers reduce boilerplate effectively
- **Decision**: Proceed to CI integration OR stop here if framework is sufficient for immediate needs

### Gate 3: Production Ready (After Phase 7)
- ✅ CI automation working
- ✅ Documentation complete
- ✅ All quality checks passing
- ✅ Framework validated and polished
- **Decision**: Framework ready for use in feature 001-rate-limits-we

---

## Next Steps After Feature Completion

Once this testing framework feature is complete:

1. **Use framework in feature 001-rate-limits-we**: Write tests for rate limiting plans, counters, and enforcement
2. **Iterate based on feedback**: If factory functions prove insufficient, add builder pattern helpers
3. **Expand test coverage**: Write tests for other existing features (resource ingestion, API key management)
4. **Monitor test execution time**: If test suite grows beyond 30 seconds, optimize or parallelize
5. **Consider testcontainers**: If in-memory DB differences from production become problematic, add Layer 2 tests
6. **Update Constitution**: Add testing requirements to Code Quality Gates section if appropriate
