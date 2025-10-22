# Implementation Plan: Database Dependency Injection for Testing

**Branch**: `003-db-dependency-injection` | **Date**: 2025-10-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-db-dependency-injection/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Enable integration testing of database interactions by implementing State-based dependency injection for SurrealDB connections using Axum's State extractor. Tests will be able to inject in-memory database connections via AppState while production code continues using global connection pooling with zero behavioral changes. This unblocks comprehensive integration testing (T036 from 002-specs-001-rate) while maintaining the security boundaries established in 002-specs-001-rate/research.md (pub(crate) visibility for internal types).

**Architecture**: Uses Axum State with `ResourcesDbFactory` trait for explicit dependency injection. Middleware receives `State(state): State<AppState>` making dependencies visible in function signatures. Production uses `GlobalResourcesDbFactory` with global connections; tests use `TestResourcesDbFactory` with in-memory databases.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: axum 0.7.9, surrealdb 2.3.7, tokio 1.47.1
**Storage**: SurrealDB (RocksDB for self-hosted, DynamoDB backend for archodex-com managed service)
**Testing**: cargo test with in-memory SurrealDB (kv-mem feature enabled in dev-dependencies)
**Target Platform**: Linux server (both AWS Lambda for managed service and standalone for self-hosted)
**Project Type**: Single backend project
**Performance Goals**: Zero overhead for production (trait generics/monomorphization eliminates runtime cost), test suite <30s (per 002-specs-001-rate/research.md)
**Constraints**: Maintain pub(crate) visibility for Account type (security boundary per 002-specs-001-rate/research.md), no #[cfg(test)] guards in production code paths (violates Rust idioms), maintain existing connection pooling behavior, explicit dependencies in function signatures (State-based DI)
**Scale/Scope**: New AppState struct with ResourcesDbFactory trait (~80 lines), 2 middleware functions updated to use State (dashboard_auth_account, report_api_key_account), new AuthedAccount wrapper type, router initialization changes, test infrastructure (test factory in tests/common/), ~10 existing handler functions change from Extension<Account> to Extension<AuthedAccount>

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify alignment with the Archodex Backend Constitution (.specify/memory/constitution.md):

**Core Principles:**
- [x] **Data Isolation & Multi-Tenancy**: Yes - feature only affects testing infrastructure, does not change account isolation logic. Production database connections remain tenant-isolated as before.
- [x] **API-First Design**: N/A - this is internal testing infrastructure, no HTTP API changes.
- [x] **Observability & Debugging**: Existing #[instrument] attributes preserved on all production functions. Test helpers MAY omit #[instrument] per Constitution Principle III exception for test code.
- [x] **Self-Hosted Parity**: Yes - works identically for both managed (archodex-com feature) and self-hosted deployments. Both use same dependency injection mechanism for testing.
- [x] **Graph Model Integrity**: N/A - no data model changes, only dependency injection architecture for testing.

**Code Quality Gates:**
- [x] `cargo fmt` will be run after changes
- [x] `cargo clippy` will pass after changes
- [x] Database schema changes include migrations via `migrator` workspace - N/A, no schema changes

**Security & Compliance:**
- [x] Authentication/authorization checks for all new endpoints - N/A, no new endpoints
- [x] Data encryption requirements met - N/A, no changes to encryption
- [x] Audit trail metadata (`created_by`, `created_at`, etc.) included where applicable - N/A, no data mutations affected

**Pass/Fail**: PASS - All applicable checks satisfied. This is purely testing infrastructure that maintains all existing security boundaries and production behavior.

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── state.rs            # NEW: AppState and ResourcesDbFactory trait
├── account.rs          # Modified: Add AuthedAccount wrapper (separate from Account domain type)
├── db.rs              # Modified: Update middleware to use State<AppState> and create AuthedAccount
├── router.rs          # Modified: Initialize with AppState, update layer ordering
└── handlers/          # Modified: Extract Extension<AuthedAccount> instead of Extension<Account>

tests/
├── common/
│   ├── providers.rs   # NEW: TestResourcesDbFactory implementation
│   └── mod.rs         # Modified: Export test factory and helpers
└── report_with_auth_test.rs  # Modified: Add successful ingestion tests with DB validation
```

**Structure Decision**: Single project structure. Changes implement State-based DI:
1. **New state.rs**: AppState struct with `accounts_db: DBConnection` and `resources_db_factory: Arc<dyn ResourcesDbFactory>`, plus trait definitions
2. **New AuthedAccount wrapper**: Separates authentication concern from domain Account type
3. **Middleware updates**: Add `State(state): State<AppState>` parameter, use factory to create connections, insert AuthedAccount
4. **Router initialization**: Create AppState with GlobalResourcesDbFactory for production
5. **Layer ordering fix**: Auth middleware runs first (outermost layer)
6. **Handler updates**: Change from `Extension<Account>` to `Extension<AuthedAccount>`
7. **Test infrastructure**: TestResourcesDbFactory in tests/common/providers.rs (not behind #[cfg(test)] in src/)

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

No violations detected. Constitution Check passed - see section above for details.
