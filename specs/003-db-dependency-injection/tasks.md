# Tasks: Database Dependency Injection for Testing

**Input**: Design documents from `/specs/003-db-dependency-injection/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅ (N/A), contracts/ ✅ (N/A)

**Tests**: Not explicitly requested in spec.md - tests are included as part of implementation to validate the feature

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root
- All paths are relative to archodex-backend/

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Verify workspace is configured for Rust 2024 edition in Cargo.toml
- [X] T002 [P] Verify dev-dependencies include surrealdb with kv-mem feature for in-memory testing
- [X] T003 [P] Run `cargo clippy` and `cargo fmt` to establish baseline

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Create new `src/state.rs` file with AppState struct and ResourcesDbFactory trait
  - Define `AppState` with `accounts_db: DBConnection` and `resources_db_factory: Arc<dyn ResourcesDbFactory>`
  - Define `ResourcesDbFactory` trait with async `create_connection()` method
  - Implement `GlobalResourcesDbFactory` for production (uses existing global `resources_db()` function)
  - See research.md lines 66-106 for complete implementation

- [X] T005 [P] Create AuthedAccount wrapper type in `src/account.rs`
  - Define `AuthedAccount` struct with `account: Account` and `resources_db: DBConnection`
  - This separates authentication concern from domain Account type
  - No mutation of Account struct needed (keeping it as pure domain object)
  - See research.md lines 683-710 for rationale

- [X] T006 [P] Refactor `src/db.rs` to expose `create_production_state()` function
  - Initialize global connections once
  - Return `AppState` with accounts_db and `GlobalResourcesDbFactory`
  - Keep existing `accounts_db()` and `resources_db()` functions as implementation details

- [X] T007 Update `src/router.rs` to accept AppState
  - Create `create_router_with_state(state: AppState) -> Router` function (pub visibility for tests)
  - Modify existing `router()` to call `create_router_with_state(create_production_state())`
  - See research.md lines 829-850 for correct layer ordering

**Checkpoint**: Foundation ready - AppState and traits defined, production code can use global connections, test infrastructure ready for injection

---

## Phase 3: User Story 1 - Test Writer Can Inject Test Database Connections (Priority: P1) 🎯 MVP

**Goal**: Enable tests to inject in-memory databases so they can validate database state after operations

**Independent Test**: Write a test that creates an in-memory database, injects it into a test router, makes a request, and queries the injected database to verify data was persisted correctly

### Implementation for User Story 1

- [X] T008 [US1] Update middleware `report_api_key_account` in `src/db.rs` (lines 295-319)
  - Add `State(state): State<AppState>` parameter to function signature
  - Replace `accounts_db().await?` with `state.accounts_db` access
  - Use `state.resources_db_factory.create_connection()` instead of direct `resources_db()` call
  - Create `AuthedAccount` wrapper with account and resources_db
  - Insert `AuthedAccount` into request extensions (not Account)
  - See research.md lines 790-825 for complete implementation

- [X] T009 [US1] Update middleware `dashboard_auth_account` in `src/db.rs` (lines 266-294)
  - Add `State(state): State<AppState>` parameter to function signature
  - Replace `accounts_db().await?` with `state.accounts_db` access
  - Use `state.resources_db_factory.create_connection()` for resources DB
  - Create `AuthedAccount` wrapper with account and resources_db
  - Insert `AuthedAccount` into request extensions

- [X] T010 [US1] Update router middleware registration in `src/router.rs`
  - Change `middleware::from_fn(report_api_key_account)` to `middleware::from_fn_with_state(state.clone(), report_api_key_account)`
  - Change `middleware::from_fn(dashboard_auth_account)` to `middleware::from_fn_with_state(state.clone(), dashboard_auth_account)`
  - Ensure correct layer order: Auth middleware outermost (runs first), then account loading
  - Add `.with_state(state)` to Router
  - See research.md lines 657-673 for correct vs wrong layer order

- [X] T011 [US1] Update all handler functions to extract `Extension<AuthedAccount>` instead of `Extension<Account>`
  - Search for all occurrences of `Extension<Account>` in src/handlers/
  - Change to `Extension(authed): Extension<AuthedAccount>`
  - Access account via `authed.account` and resources DB via `authed.resources_db`
  - Approximately 10 handler functions affected (per plan.md line 24)
  - See research.md lines 875-888 for handler example

- [ ] T012 [P] [US1] Create `tests/common/providers.rs` with TestResourcesDbFactory implementation
  - Implement `TestResourcesDbFactory` struct that holds in-memory DBConnection
  - Implement `ResourcesDbFactory` trait for TestResourcesDbFactory
  - `create_connection()` returns clone of the test DB (ignores account_id/service_url)
  - Note: Must be in tests/ not src/ due to compilation unit boundaries (see research.md lines 717-743)

- [ ] T013 [P] [US1] Create test helper `create_test_router` in `tests/common/mod.rs`
  - Function signature: `create_test_router(accounts_db: DBConnection, resources_db: DBConnection) -> Router`
  - Create AppState with injected databases and TestResourcesDbFactory
  - Call `archodex_backend::router::create_router_with_state(state)`
  - Return Router for use in tests
  - See research.md lines 204-216 for implementation

- [ ] T014 [P] [US1] Create test helper functions in `tests/common/mod.rs`
  - `create_test_accounts_db() -> DBConnection` - creates in-memory accounts DB
  - `create_test_resources_db() -> DBConnection` - creates in-memory resources DB
  - `seed_test_account(db: &DBConnection, account_id: &str) -> Account` - seeds account data
  - See quickstart.md lines 80-116 for implementation examples

- [ ] T015 [US1] Write integration test in `tests/report_with_auth_test.rs`
  - Test: successful report ingestion with database validation
  - Create in-memory databases using test helpers
  - Seed test account
  - Create router with injected databases
  - POST to /report endpoint
  - Assert HTTP 200 response
  - Query injected resources_db to verify resources were created
  - This validates the entire dependency injection chain works end-to-end
  - See quickstart.md lines 37-62 for complete example

**Checkpoint**: At this point, User Story 1 should be fully functional - tests can inject databases and verify persistence

---

## Phase 4: User Story 2 - Production Code Uses Global Connections Without Changes (Priority: P1)

**Goal**: Ensure production code continues using global connection pooling with zero behavioral changes or performance impact

**Independent Test**: Run production application (both archodex-com and self-hosted) and verify all existing functionality works identically. Performance profiling should show no measurable difference.

### Implementation for User Story 2

- [X] T016 [US2] Validate production build with `cargo build --release`
  - Ensure compilation succeeds with all changes
  - Verify binary size is not significantly increased
  - Production code should monomorphize to zero-overhead trait calls

- [X] T017 [US2] Run existing test suite with `cargo test`
  - All existing tests should pass without modification
  - This validates backward compatibility
  - Fix any tests that break due to Extension<Account> → Extension<AuthedAccount> changes

- [ ] T018 [US2] Manual smoke test of production deployment **[REQUIRES MANUAL VERIFICATION]**
  - **STATUS**: Code changes complete, but manual testing required
  - **ACTION REQUIRED**: User must start local server and validate
  - Start local server with production configuration
  - Test account creation flow
  - Test report ingestion via /report endpoint
  - Test dashboard queries
  - Verify no performance regressions (response times within 5% of baseline per SC-003)
  - Verify connection pooling is still used (no per-request connection creation)
  - **IMPORTANT**: Use proper production build command (see project-notes.md)

**Checkpoint**: Production code validated - zero behavioral changes, all existing functionality works (PENDING MANUAL VERIFICATION)

---

## Phase 5: User Story 3 - Middleware Can Access Database Through Dependency Injection (Priority: P2)

**Goal**: Middleware functions support dependency injection for testing while maintaining production behavior

**Independent Test**: Write a test that creates a test router with middleware, injects test database, makes HTTP request, and verifies middleware correctly loaded account from injected database

### Implementation for User Story 3

- [ ] T019 [US3] Write test for authentication middleware with injected database in `tests/report_with_auth_test.rs`
  - Test: invalid API key rejected by middleware
  - Create test router with injected accounts database (no test account seeded)
  - POST to /report with invalid auth token
  - Assert HTTP 401 Unauthorized
  - Verifies middleware uses injected accounts_db

- [ ] T020 [US3] Write test for account loading middleware in `tests/report_with_auth_test.rs`
  - Test: valid auth token loads account from injected database
  - Seed account in test accounts_db
  - POST to /report with valid auth token
  - Handler should receive Extension<AuthedAccount> with correct account data
  - Can verify by checking response includes account-specific data

- [ ] T021 [US3] Write test for per-account resources database selection in `tests/report_with_auth_test.rs`
  - Test: middleware uses TestResourcesDbFactory to get resources DB
  - Create account with custom service_data_surrealdb_url (if archodex-com feature enabled)
  - Verify factory is called with correct account_id
  - Verify handler receives AuthedAccount with correct resources_db

**Checkpoint**: All middleware functions validated - support dependency injection for testing, work correctly in production

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, documentation, and cleanup

- [ ] T022 [P] Add documentation comments to AppState and ResourcesDbFactory in `src/state.rs`
  - Document purpose: "State-based dependency injection for testability"
  - Document GlobalResourcesDbFactory: "Production implementation using global connection pool"
  - Document trait method parameters and return values

- [ ] T023 [P] Add documentation comments to AuthedAccount in `src/account.rs`
  - Document purpose: "Wrapper for authenticated account with injected resources database"
  - Document that this separates authentication concern from domain Account type

- [ ] T024 [P] Update test helper documentation in `tests/common/mod.rs`
  - Add module-level doc comment explaining test infrastructure
  - Document each helper function with usage examples
  - Reference quickstart.md for detailed examples

- [ ] T025 Validate quickstart.md examples still work
  - Run each code example from quickstart.md
  - Verify all test patterns compile and pass
  - Update quickstart.md if any examples are out of date

- [ ] T026 Run full linting and formatting pass
  - `cargo fmt` on all modified files
  - `cargo clippy` and address any warnings
  - Ensure no new clippy warnings introduced

- [ ] T027 Final integration test run with `cargo test`
  - All tests should pass
  - Test suite should complete within performance targets (<30s per research.md line 22)
  - No flaky tests (run 5 times to verify)

- [ ] T028 Validate Constitution compliance
  - Verify all #[instrument] attributes preserved on production functions
  - Verify pub(crate) visibility maintained for Account type (security boundary)
  - Verify no #[cfg(test)] guards in production code paths
  - Run `cargo fmt` and `cargo clippy` as quality gates

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
  - T004 (AppState/traits) must complete before any other Phase 2 tasks
  - T005, T006, T007 can run in parallel after T004 completes
- **User Story 1 (Phase 3)**: Depends on Foundational completion
  - T008, T009 must complete before T010 (router registration needs updated middleware signatures)
  - T011 can run in parallel with T008-T010 (different handlers in different files)
  - T012, T013, T014 can run in parallel (different test files)
  - T015 depends on T008-T014 (needs all infrastructure in place)
- **User Story 2 (Phase 4)**: Depends on User Story 1 completion (validates the changes work in production)
  - T016-T018 must run sequentially (build → test → manual validation)
- **User Story 3 (Phase 5)**: Depends on User Story 1 completion (tests middleware specifically)
  - T019-T021 can run in parallel (different test cases)
- **Polish (Phase 6)**: Depends on all user stories being complete
  - T022-T024 can run in parallel (documentation in different files)
  - T025-T028 should run sequentially for validation

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - This is the MVP
- **User Story 2 (P1)**: Depends on User Story 1 - Validates production behavior unchanged
- **User Story 3 (P2)**: Depends on User Story 1 - Tests middleware specifically

### Within Each User Story

**User Story 1**:
- Middleware updates (T008-T009) before router registration (T010)
- All infrastructure (T008-T014) before integration test (T015)
- Handler updates (T011) can proceed in parallel with middleware

**User Story 2**:
- Build (T016) → Test (T017) → Manual validation (T018)

**User Story 3**:
- All tests (T019-T021) can run in parallel after US1 infrastructure complete

### Parallel Opportunities

**Phase 1 (Setup)**:
- T002 and T003 can run in parallel

**Phase 2 (Foundational)**:
- After T004 completes: T005, T006, T007 can run in parallel

**Phase 3 (User Story 1)**:
- T011 (handler updates) can run while T008-T010 (middleware) are in progress
- T012, T013, T014 (test helpers) can all run in parallel

**Phase 5 (User Story 3)**:
- T019, T020, T021 (middleware tests) can all run in parallel

**Phase 6 (Polish)**:
- T022, T023, T024 (documentation) can all run in parallel

---

## Parallel Example: User Story 1 Core Infrastructure

```bash
# After T004-T007 complete, launch these in parallel:

# Parallel Group 1: Handler updates (different files)
Task T011: "Update handler functions to use Extension<AuthedAccount>"
  - src/handlers/report.rs
  - src/handlers/dashboard.rs
  - src/handlers/[other handlers]

# Parallel Group 2: Test infrastructure (different files)
Task T012: "Create tests/common/providers.rs with TestResourcesDbFactory"
Task T013: "Create test helper create_test_router in tests/common/mod.rs"
Task T014: "Create test helper functions in tests/common/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003) - ~30 minutes
2. Complete Phase 2: Foundational (T004-T007) - ~3-4 hours
3. Complete Phase 3: User Story 1 (T008-T015) - ~4-5 hours
4. **STOP and VALIDATE**: Run T015 integration test to verify end-to-end flow
5. Total MVP time: ~8-10 hours (1-1.5 days)

**MVP Deliverable**: Tests can inject in-memory databases and verify database state after operations

### Full Feature (All User Stories)

1. Complete MVP (User Stories 1) → ~8-10 hours
2. Add User Story 2 (T016-T018) - Production validation → ~2-3 hours
3. Add User Story 3 (T019-T021) - Middleware testing → ~2-3 hours
4. Polish (T022-T028) - Documentation and validation → ~2-3 hours
5. **Total time: ~14-18 hours (2-2.5 days)**

Each story adds value and can be demonstrated independently.

### Incremental Delivery

1. **Checkpoint 1**: After Phase 2 (Foundational) → Foundation ready
2. **Checkpoint 2**: After Phase 3 (US1) → Test infrastructure working (MVP!)
3. **Checkpoint 3**: After Phase 4 (US2) → Production validated
4. **Checkpoint 4**: After Phase 5 (US3) → Middleware fully tested
5. **Checkpoint 5**: After Phase 6 (Polish) → Feature complete

### Parallel Team Strategy

With multiple developers:

1. **Dev A**: Phase 2 (Foundational) - ~3-4 hours
2. Once Phase 2 complete, split work:
   - **Dev A**: T008-T010 (middleware updates) - ~2 hours
   - **Dev B**: T011 (handler updates) - ~2 hours
   - **Dev C**: T012-T014 (test helpers) - ~2 hours
3. **Dev A**: T015 (integration test) - ~1 hour
4. **Dev B**: Phase 4 (US2 validation) - ~2-3 hours
5. **Dev C**: Phase 5 (US3 middleware tests) - ~2-3 hours
6. **All**: Phase 6 (Polish) together - ~2 hours

**Parallel completion time: ~10-12 hours**

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- **Critical**: Layer order in router registration (research.md lines 657-673) - Auth must run before account loading
- **Critical**: TestResourcesDbFactory must be in tests/common/, not src/ (research.md lines 717-743)
- **Critical**: AuthedAccount wrapper approach preferred over mutating Account (research.md lines 683-710)
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

---

## Risk Assessment

### High Risk Items

1. **Layer ordering** (T010): Getting middleware layer order wrong will break authentication flow
   - Mitigation: Follow research.md lines 657-673 carefully, write test to validate auth runs first

2. **Handler signature changes** (T011): Breaking all handler signatures could cause compilation errors
   - Mitigation: Use compiler to find all occurrences, update systematically

### Medium Risk Items

1. **Test infrastructure visibility** (T012): TestResourcesDbFactory in wrong location won't be visible to tests
   - Mitigation: Place in tests/common/ per research.md guidance

2. **Production performance** (T016-T018): Trait dispatch or State overhead could impact performance
   - Mitigation: Rust monomorphization eliminates runtime cost, validate with profiling

### Low Risk Items

1. **Documentation** (T022-T024): Low impact, can be fixed easily
2. **Test helpers** (T013-T014): Isolated to test code, no production impact

---

## Success Criteria Validation

After completing all tasks, validate these success criteria from spec.md:

- **SC-001**: Test can create Account with injected database, call report ingestion, verify 100% of stored resources ✅ T015
- **SC-002**: Tests execute within performance targets (unit <1s, integration <5s, full suite <30s) ✅ T027
- **SC-003**: Production code shows zero performance regression (within 5% of baseline) ✅ T018
- **SC-004**: Test isolation complete - 100 consecutive parallel runs produce identical results ✅ T027
- **SC-005**: Existing production functionality works identically ✅ T017, T018
- **SC-006**: New integration tests can be written within 20 minutes following documented patterns ✅ T024, T025

---

## Acceptance Criteria for Each User Story

### User Story 1 Acceptance
1. ✅ Test creates in-memory SurrealDB connection
2. ✅ Test creates Account with injected connection
3. ✅ Account uses injected connection instead of global state
4. ✅ Test queries injected database to verify resources stored correctly
5. ✅ Multiple tests run in parallel with isolated databases

### User Story 2 Acceptance
1. ✅ Production code uses existing global connection pool
2. ✅ No measurable performance overhead from DI architecture
3. ✅ Database errors handled identically to previous implementation

### User Story 3 Acceptance
1. ✅ Middleware loads accounts from injected test database
2. ✅ Account extension injected into request with correct database connection
3. ✅ Production middleware uses global connection pool

---

## Quick Reference: Key Files Modified

| File | Purpose | User Story |
|------|---------|-----------|
| `src/state.rs` | NEW: AppState, traits, factories | US1 (Foundation) |
| `src/account.rs` | NEW: AuthedAccount wrapper | US1 (Foundation) |
| `src/db.rs` | Modified: Middleware with State | US1, US3 |
| `src/router.rs` | Modified: State initialization, layer order | US1, US2 |
| `src/handlers/*.rs` | Modified: Extract AuthedAccount | US1 |
| `tests/common/providers.rs` | NEW: TestResourcesDbFactory | US1 |
| `tests/common/mod.rs` | NEW: Test helpers | US1 |
| `tests/report_with_auth_test.rs` | Modified: Integration tests | US1, US3 |

---

## Estimated Effort Summary

| Phase | Tasks | Estimated Time | Critical Path |
|-------|-------|---------------|---------------|
| Phase 1: Setup | T001-T003 | 30 min | No |
| Phase 2: Foundational | T004-T007 | 3-4 hours | **YES** |
| Phase 3: User Story 1 | T008-T015 | 4-5 hours | **YES** |
| Phase 4: User Story 2 | T016-T018 | 2-3 hours | YES |
| Phase 5: User Story 3 | T019-T021 | 2-3 hours | No |
| Phase 6: Polish | T022-T028 | 2-3 hours | No |
| **TOTAL** | **28 tasks** | **14-18 hours** | - |

**Critical Path**: Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2) → Phase 6 (validation)

**Minimum MVP**: Phase 1 + Phase 2 + Phase 3 = ~8-10 hours

**Parallel Opportunities**: Up to 30% time savings with 3 developers (~10-12 hours total)
