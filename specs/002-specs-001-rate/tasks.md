# Tasks: Testing Framework Setup and Validation

**Input**: Design documents from `/specs/002-specs-001-rate/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Tests are included in this feature as the feature IS establishing the testing framework.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

---

## 📋 Current Status

**Completed**:
- ✅ Phase 3 (User Story 1): Research and testing approach selection - **COMPLETED in plan stage** (T004-T009)

**Next Steps**:
- 🔄 Phase 1: Setup directory structure (T001-T003)
- 🔄 Phase 4: Implement framework infrastructure (T010-T017)
- 🔄 Phase 5: Write example tests (T018-T030)

**🛑 STOP POINT**: After Phase 5 completion, STOP and await confirmation before proceeding to Phase 6 (CI Integration)

---

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Project Structure)

**Purpose**: Create basic directory structure for testing framework

- [ ] T001 Create `tests/` directory at repository root
- [ ] T002 Create `tests/common/` directory for shared test helpers
- [ ] T003 Create `tests/common/mod.rs` for module re-exports

**Checkpoint**: Basic test directory structure is in place

---

## Phase 2: Foundational (No blocking prerequisites for this feature)

**Purpose**: This feature establishes the testing framework itself, so no foundational infrastructure is required beyond Phase 1.

**Note**: Phase 1 setup is sufficient to begin User Story implementation.

**Checkpoint**: Foundation ready - user story implementation can begin

---

## Phase 3: User Story 1 - Evaluate and Select Testing Approach (Priority: P1)

**Goal**: Evaluate testing approaches (unit tests with mocks vs integration tests with database vs hybrid), research SurrealDB testing viability, and make informed selection based on project stage and maintenance burden.

**Independent Test**: Can be fully tested by documenting evaluation criteria, researching SurrealDB testing options, comparing approaches against criteria (including Constitution principles), and getting stakeholder approval.

### Research and Evaluation for User Story 1

- [x] T004 [US1] Review testing-framework-proposal.md document (if exists) or create evaluation document in `specs/002-specs-001-rate/research.md` ✅ **COMPLETED in plan stage**
- [x] T005 [US1] Research SurrealDB testing options: DynamoDB-backed testability, in-memory SurrealDB (`kv-mem`), containerized SurrealDB, mocking database layer ✅ **COMPLETED in plan stage**
- [x] T006 [US1] Evaluate testing approach options: unit tests with mocks, integration tests with in-memory DB, integration tests with containers, hybrid approach ✅ **COMPLETED in plan stage**
- [x] T007 [US1] Score approaches against evaluation criteria: maintenance burden (per Constitution), setup complexity, CI compatibility, confidence level, debugging ease ✅ **COMPLETED in plan stage**
- [x] T008 [US1] Document tradeoffs and selection rationale in `specs/002-specs-001-rate/research.md` ✅ **COMPLETED in plan stage**
- [x] T009 [US1] Present evaluation results to stakeholders for approval before proceeding to implementation ✅ **COMPLETED in plan stage**

**Decision from Research**: Use **SurrealDB in-memory mode (`kv-mem`)** with minimal tooling (cargo test + tokio::test + oneshot)

**Checkpoint**: ✅ Testing approach selected and approved - ready to implement framework

---

## Phase 4: User Story 2 - Setup Testing Framework Infrastructure (Priority: P1)

**Goal**: Establish testing framework infrastructure by adding selected framework dependencies, creating reusable test helpers, and documenting testing patterns.

**Independent Test**: Can be fully tested by adding dependencies to Cargo.toml, creating test helper modules, and verifying `cargo test` runs successfully with test database instances.

### Dependencies for User Story 2

- [ ] T010 [US2] Add `tower = { version = "0.5", features = ["util"] }` to `[dev-dependencies]` in root `Cargo.toml` for `oneshot()` HTTP testing pattern
- [ ] T011 [US2] Verify `cargo build --tests` compiles successfully with new dependencies

### Test Helper Modules for User Story 2

- [ ] T012 [P] [US2] Implement `tests/common/db.rs` with database setup functions: `create_test_db()`, `create_test_db_with_migrations()`, `create_test_db_with_account()`
- [ ] T013 [P] [US2] Implement `tests/common/fixtures.rs` with test data factory functions: `create_test_account()`, `create_test_report()`, `create_test_resources()`, `create_test_events()`, `create_test_api_key()`, `create_test_user()`
- [ ] T014 [US2] Update `tests/common/mod.rs` to re-export all helpers from `db` and `fixtures` modules

### Documentation for User Story 2

- [ ] T015 [US2] Create `tests/common/README.md` with testing patterns, examples, and usage instructions
- [ ] T015.5 [US2] Verify documentation meets requirements: includes patterns for writing tests, provides clear examples of helper usage (database setup and fixtures), enables new developer to write first test within 15 minutes

### Verification for User Story 2

- [ ] T016 [US2] Verify `cargo test` runs (even with no tests yet) and helper modules compile successfully
- [ ] T017 [US2] Verify test helpers provide isolation - each test gets its own in-memory database instance
- [ ] T017.5 [US2] Test error handling scenarios: verify framework provides clear error messages when setup fails (e.g., simulate missing dependency, database connection failure)

**Checkpoint**: Testing framework infrastructure is complete and documented - ready to write example tests

---

## Phase 5: User Story 3 - Validate Framework with Existing Feature Tests (Priority: P1)

**Goal**: Validate chosen testing approach by writing 2 example tests for existing features (resource ingestion and API key generation) to verify the approach works and is maintainable.

**Independent Test**: Can be fully tested by implementing 2 complete example tests that pass consistently and are straightforward to write/debug.

### Example Test 1: Resource Ingestion

- [ ] T018 [US3] Create `tests/report_ingestion_test.rs` file
- [ ] T019 [US3] Implement `test_report_ingests_resources_correctly()` test using in-memory database
- [ ] T020 [US3] Test should: create test DB with migrations, create test account, generate test report with 3 resources, ingest report, verify 3 resources stored in database
- [ ] T021 [US3] Verify test uses helpers from `tests/common/` to demonstrate reusability
- [ ] T022 [US3] Run `cargo test test_report_ingests_resources_correctly` and verify it passes

### Example Test 2: API Key Generation

- [ ] T023 [P] [US3] Create `tests/report_api_key_test.rs` file
- [ ] T024 [P] [US3] Implement `test_api_key_roundtrip()` test for encryption/decryption
- [ ] T025 [P] [US3] Test should: generate ReportApiKey with account_id and salt, encrypt to key string, verify format starts with "archodex_", decode and validate decryption works
- [ ] T026 [P] [US3] Implement `test_tamper_detection()` test to verify tampered keys are rejected
- [ ] T027 [US3] Run `cargo test report_api_key` and verify both tests pass

### Framework Validation for User Story 3

- [ ] T028 [US3] Run `cargo test` locally and verify both example tests pass within 30 seconds
- [ ] T029 [US3] Verify tests are straightforward to write, run, and debug without excessive setup complexity
- [ ] T030 [US3] Verify example tests serve as clear documentation for future test writers
- [ ] T030.5 [US3] Verify test determinism: run full test suite 10 consecutive times and confirm identical results (no flaky tests, no timing issues)

**Checkpoint**: Example tests validate that the testing framework is viable and maintainable

---

## 🛑 STOP HERE - AWAIT USER CONFIRMATION BEFORE PROCEEDING TO PHASE 6

**Once Phase 5 is complete**, STOP and validate before proceeding.

**What Phase 5 should deliver**:
- Test directory structure (`tests/`, `tests/common/`)
- Test helper modules (`db.rs`, `fixtures.rs`)
- Documentation (`tests/common/README.md`)
- 2 working example tests (resource ingestion + API key)
- Framework validated locally with `cargo test`

**Before continuing to Phase 6**:
1. Run `cargo test` locally to verify all tests pass
2. Review example tests to ensure they meet requirements
3. Confirm framework is usable and maintainable
4. **Get explicit approval from USER to proceed with CI automation (Phase 6)**

**What's next (Phase 6)**: CI Integration - requires USER confirmation to proceed

---

## Phase 6: User Story 4 - Configure CI Integration for Automated Testing (Priority: P2)

**⚠️ DO NOT START THIS PHASE WITHOUT EXPLICIT APPROVAL ⚠️**

**Goal**: Configure GitHub Actions CI to automatically run tests on every push and block merges if tests fail. Local testing via ACT is also supported.

**Independent Test**: Can be fully tested by pushing code with passing/failing tests and verifying GitHub Actions correctly reports status.

### CI Configuration for User Story 4

- [ ] T031 [US4] Create `.github/workflows/test.yml` workflow file (or update if exists)
- [ ] T032 [US4] Configure workflow to trigger on push to all branches and on pull requests
- [ ] T033 [US4] Add step to checkout code (`actions/checkout@v4`)
- [ ] T034 [US4] Add step to setup Rust toolchain (`actions-rust-lang/setup-rust-toolchain@v1`) with stable toolchain and clippy component
- [ ] T035 [US4] Add step to run tests: `cargo test --all-features --verbose`
- [ ] T036 [US4] Add step to run clippy: `cargo clippy -- -D warnings`
- [ ] T037 [US4] Add step to check formatting: `cargo fmt -- --check`

### CI Validation for User Story 4

- [ ] T038 [US4] Push code to branch and verify GitHub Actions workflow automatically triggers
- [ ] T039 [US4] Verify workflow succeeds when all tests pass (green status on PR)
- [ ] T040 [US4] Create intentional test failure, push, and verify workflow fails and blocks merge with clear error message
- [ ] T041 [US4] Fix test, push, and verify workflow succeeds again
- [ ] T042 [US4] (Optional) Test CI locally using `act` to verify ACT compatibility

### Performance Verification for User Story 4

- [ ] T043 [US4] Verify total CI workflow execution completes within 3-5 minutes from push to result

**Checkpoint**: CI integration is complete and automatically enforces test quality

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final documentation and validation across all user stories

- [ ] T044 [P] Update `specs/002-specs-001-rate/quickstart.md` with final testing patterns and examples (if not already complete)
- [ ] T045 [P] Verify all 4 user stories work independently as documented
- [ ] T046 Run `cargo fmt` to format all test code
- [ ] T047 Run `cargo clippy` to verify no linter warnings in test code
- [ ] T048 Final end-to-end validation: run `cargo test` locally, verify all tests pass, verify CI passes on push
- [ ] T049 (Optional) Add builder pattern to `tests/common/fixtures.rs` if factory functions prove insufficient (defer unless needed)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: N/A - Phase 1 setup is sufficient for this feature
- **User Story 1 (Phase 3)**: Depends on Phase 1 completion - Research and selection
- **User Story 2 (Phase 4)**: Depends on Phase 1 completion AND User Story 1 approval - Framework implementation
- **User Story 3 (Phase 5)**: Depends on User Story 2 completion - Example tests validate framework
- **User Story 4 (Phase 6)**: Depends on User Story 3 completion - CI needs working tests to validate
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Research and evaluation - MUST complete first to make informed decision
- **User Story 2 (P1)**: Framework implementation - MUST complete after US1 approval
- **User Story 3 (P1)**: Example tests - MUST complete after US2 to validate framework works
- **User Story 4 (P2)**: CI integration - Should complete after US3 to have working tests

### Within Each User Story

- **US1**: Research tasks are sequential (T004 → T005 → T006 → T007 → T008 → T009)
- **US2**: Dependencies added first (T010-T011), then helpers can be written in parallel (T012, T013 are [P]), then documentation (T015), then verification (T016-T017)
- **US3**: Test 1 tasks sequential (T018-T022), Test 2 tasks can start in parallel (T023-T027 marked [P]), validation at end (T028-T030)
- **US4**: CI configuration tasks sequential (T031-T037), then validation tasks (T038-T043)

### Parallel Opportunities

- Within US2: T012 and T013 can run in parallel (different files)
- Within US3: Test 1 (T018-T022) and Test 2 (T023-T027) can be developed in parallel by different developers
- Within US4: No parallel opportunities (sequential CI setup and validation)
- Phase 7: T044 and T045 can run in parallel

---

## Parallel Example: User Story 2

```bash
# After dependencies are added (T010-T011), launch helpers in parallel:
Task T012: "Implement tests/common/db.rs with database setup functions"
Task T013: "Implement tests/common/fixtures.rs with test data factory functions"
```

## Parallel Example: User Story 3

```bash
# Example Test 1 and Example Test 2 can be developed concurrently:
Task T018-T022: "Create and implement resource ingestion test"
Task T023-T027: "Create and implement API key generation tests"
```

---

## Implementation Strategy

### MVP First (Minimal Viable Framework)

1. Complete Phase 1: Setup (T001-T003)
2. ✅ Complete Phase 3: User Story 1 - Research and Selection (T004-T009) **COMPLETED in plan stage**
3. Complete Phase 4: User Story 2 - Framework Infrastructure (T010-T017)
4. Complete Phase 5: User Story 3 - Example Tests (T018-T030)
5. **🛑 STOP and VALIDATE**: Run `cargo test` locally, verify framework is usable
6. **⚠️ AWAIT CONFIRMATION**: Get explicit approval before proceeding to Phase 6
7. Complete Phase 6: User Story 4 - CI Integration (T031-T043) **[Only after approval]**
8. Complete Phase 7: Polish (T044-T049) **[Only after approval]**

### Incremental Delivery

1. Complete Setup (Phase 1) → Directory structure ready
2. Complete US1 (Phase 3) → Testing approach decided and approved
3. Complete US2 (Phase 4) → Framework infrastructure in place
4. Complete US3 (Phase 5) → Framework validated with working examples → **Framework ready for use!**
5. Complete US4 (Phase 6) → CI automation ensures quality → **Production-ready testing framework!**
6. Complete Polish (Phase 7) → Final refinements

### Timeline Estimate

- **Phase 1**: 15 minutes (directory creation)
- **Phase 3 (US1)**: ✅ COMPLETED in plan stage (research and evaluation)
- **Phase 4 (US2)**: 4-6 hours (framework implementation)
- **Phase 5 (US3)**: 4-6 hours (example tests)
- **🛑 STOP POINT**: Validation and approval checkpoint
- **Phase 6 (US4)**: 2-3 hours (CI configuration) **[After approval]**
- **Phase 7**: 1-2 hours (polish and validation) **[After approval]**
- **Remaining work**: ~10-14 hours (Phases 1, 4, 5 only)

---

## Notes

- [P] tasks = different files, no dependencies, can run in parallel
- [Story] label maps task to specific user story (US1, US2, US3, US4)
- Each user story should be independently completable and testable
- Tests are integral to this feature (the feature IS the testing framework)
- Commit after each task or logical group
- Stop at any checkpoint to validate independently
- US1 decision gates all subsequent work - ensure stakeholder approval before proceeding
- Framework uses in-memory SurrealDB (`kv-mem`) per research.md decision
- No Docker or external infrastructure required (aligns with Constitution anti-over-engineering principle)
- Framework should be straightforward to use (15-minute learning curve per success criteria)
