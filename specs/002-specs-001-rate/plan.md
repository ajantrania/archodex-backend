# Implementation Plan: Testing Framework Setup and Validation

**Branch**: `002-specs-001-rate` | **Date**: 2025-10-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-specs-001-rate/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

This feature establishes a testing framework for the Archodex backend to enable automated testing of existing features and prepare for testing the rate limiting feature (001-rate-limits-we). The primary requirement is to evaluate testing approaches (unit tests with mocks vs integration tests vs hybrid), research SurrealDB testing viability, select the most pragmatic approach for the project's current stage, and validate it with 2 example tests. The framework must balance maintainability, setup complexity, and confidence level while aligning with the Constitution's principle of avoiding over-engineering.

## Technical Context

**Language/Version**: Rust 2024 edition (workspace configured for edition 2024)
**Primary Dependencies**: axum 0.7.9, surrealdb 2.3.7, tokio 1.47.1, prost 0.13.5 (protobuf), aes-gcm 0.10.3
**Storage**: SurrealDB 2.3.7 (Archodex fork with DynamoDB backend for managed service, RocksDB for self-hosted)
**Testing**: NEEDS CLARIFICATION - Must evaluate cargo test + mocks vs testcontainers vs in-memory SurrealDB vs hybrid approach
**Target Platform**: AWS Lambda (managed service) and Linux/Docker (self-hosted), GitHub Actions CI
**Project Type**: Single backend project with workspace members (server, lambda, migrator)
**Performance Goals**: Test execution <60 seconds for integration tests, <30 seconds for unit tests, CI pipeline <5 minutes
**Constraints**: Must work in GitHub Actions without excessive infrastructure, maintainable by stealth-stage team (avoid over-engineering), Docker availability in CI
**Scale/Scope**: Initial framework with 2 example tests, expanding to ~20-30 tests for rate limiting feature, eventual target ~100-200 tests across all features

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify alignment with the Archodex Backend Constitution (.specify/memory/constitution.md):

**Core Principles:**
- [x] **Data Isolation & Multi-Tenancy**: N/A - Testing framework does not affect multi-tenancy. Test helpers will use isolated test accounts and databases.
- [x] **API-First Design**: N/A - Testing framework is internal development infrastructure, not exposed via APIs.
- [x] **Observability & Debugging**: Test helper functions will follow standard Rust conventions. No tracing needed for test utilities.
- [x] **Self-Hosted Parity**: Testing framework must support both managed and self-hosted deployment modes. Example tests will validate common features, not deployment-specific code.
- [x] **Graph Model Integrity**: N/A - Testing framework does not modify data models. Example tests will validate existing resource ingestion and API key features.

**Code Quality Gates:**
- [x] `cargo fmt` will be run after changes (test code follows same standards)
- [x] `cargo clippy` will pass after changes (includes test code)
- [x] Database schema changes include migrations via `migrator` workspace (N/A - no schema changes)

**Security & Compliance:**
- [x] Authentication/authorization checks for all new endpoints: N/A - no new endpoints
- [x] Data encryption requirements met: Example test validates API key encryption/decryption
- [x] Audit trail metadata: N/A - test data uses standard metadata patterns

**Development Stage Context Alignment:**
- [x] Avoids over-engineering: Testing approach evaluation explicitly considers project stage and maintenance burden per Constitution
- [x] Prioritizes simplicity: Framework selection criteria includes "ease of use" and "setup complexity"
- [x] Pragmatic approach: Spec allows pivoting to simpler approach if selected framework proves too cumbersome

**Pass/Fail**: **PASS** - Testing framework aligns with Constitution principles and explicitly prioritizes simplicity over industrial-scale robustness.

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
archodex-backend/
├── src/                          # Main library code (existing)
│   ├── account.rs
│   ├── report_api_key.rs         # Used in example test 2
│   ├── report.rs                 # Used in example test 1
│   ├── resource.rs               # Used in example test 1
│   └── ...
│
├── tests/                        # Integration tests (NEW - to be created)
│   ├── common/                   # Shared test helpers (NEW)
│   │   ├── mod.rs                # Re-exports helpers
│   │   ├── db.rs                 # Database setup functions
│   │   ├── fixtures.rs           # Test data builders
│   │   └── README.md             # Test patterns documentation
│   │
│   ├── report_ingestion_test.rs  # Example test 1 (NEW)
│   └── report_api_key_test.rs    # Example test 2 (NEW)
│
├── server/                       # Server workspace member (existing)
├── lambda/                       # Lambda workspace member (existing)
├── migrator/                     # Migrator workspace member (existing)
│
├── Cargo.toml                    # Workspace config (UPDATE [dev-dependencies])
└── .github/workflows/
    └── test.yml                  # CI configuration (NEW or UPDATE)
```

**Structure Decision**:

This is a **single Rust workspace project** with multiple workspace members (server, lambda, migrator). Testing framework follows standard Rust conventions:

- **Unit tests**: Inline in source files using `#[cfg(test)] mod tests` (for pure logic)
- **Integration tests**: Separate `tests/` directory (for API-to-DB flows)
- **Test helpers**: `tests/common/` module for shared fixtures and utilities
- **Dependencies**: Added to workspace `[dev-dependencies]` in root `Cargo.toml`

The chosen approach (to be determined in Phase 0) will determine which test organization strategy is used:
- **Option A (mocking)**: Primarily unit tests in `src/` files, minimal `tests/` directory
- **Option B (integration)**: Primarily integration tests in `tests/` directory with testcontainers or in-memory DB
- **Option C (hybrid)**: Mix of both approaches

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

**No violations** - Constitution Check passed. Testing framework design aligns with all constitutional principles.

---

## Post-Design Constitution Check

*Re-evaluation after Phase 1 design completion*

**Date**: 2025-10-14

### Review of Design Artifacts

Design phase generated:
- `research.md` - SurrealDB testing strategy and framework selection
- `data-model.md` - Test data structures and helper patterns
- `contracts/README.md` - No API contracts (testing infrastructure)
- `quickstart.md` - Developer onboarding guide

### Constitution Alignment Review

**Core Principles:**
- [x] **Data Isolation & Multi-Tenancy**: ✅ Confirmed - Each test creates isolated in-memory database, no cross-test contamination
- [x] **API-First Design**: ✅ N/A - Testing infrastructure does not expose APIs
- [x] **Observability & Debugging**: ✅ Confirmed - Test helpers use standard Rust patterns, clear error messages
- [x] **Self-Hosted Parity**: ✅ Confirmed - Tests work identically in managed and self-hosted modes (in-memory SurrealDB)
- [x] **Graph Model Integrity**: ✅ N/A - No data model changes

**Code Quality Gates:**
- [x] `cargo fmt` will be run after changes: ✅ Confirmed in CI workflow
- [x] `cargo clippy` will pass after changes: ✅ Confirmed in CI workflow
- [x] Database schema changes: ✅ N/A - No schema changes

**Security & Compliance:**
- [x] Authentication/authorization: ✅ N/A - No new endpoints
- [x] Data encryption: ✅ Example test validates API key encryption
- [x] Audit trail: ✅ N/A - Test data only

**Development Stage Context Alignment:**
- [x] **Avoids over-engineering**: ✅ **CONFIRMED** - Selected minimal tooling (cargo test + tokio::test + oneshot), rejected complex alternatives
- [x] **Prioritizes simplicity**: ✅ **CONFIRMED** - Factory functions over complex builders, in-memory DB over containers (initially)
- [x] **Pragmatic approach**: ✅ **CONFIRMED** - Research explicitly evaluated maintenance burden, chose fastest/simplest option

### Design Decision Review

**Key Decisions Made:**
1. **Use SurrealDB in-memory mode (`kv-mem`)** instead of testcontainers
   - **Rationale**: Zero infrastructure, fast execution, aligns with anti-over-engineering
   - **Constitution Alignment**: ✅ Development Stage Context (simplicity over robustness)

2. **Use minimal tooling** (cargo test + tokio::test + oneshot)
   - **Rationale**: Leverages existing dependencies, no learning curve
   - **Constitution Alignment**: ✅ Development Stage Context (avoid premature complexity)

3. **Factory functions for test data** instead of complex fixtures
   - **Rationale**: Sufficient for small test suite, easy to understand
   - **Constitution Alignment**: ✅ Development Stage Context (build for clarity first)

### Verification: No Over-Engineering

**Evidence of Pragmatic Design:**
- ✅ Research evaluated 4 approaches, chose simplest viable option
- ✅ Rejected testcontainers initially (can add later if needed)
- ✅ Rejected mocking approach (would require major refactoring)
- ✅ Rejected DynamoDB/LocalStack testing (too complex for current stage)
- ✅ Selected zero-dependency approach (uses existing tooling)

**Complexity Metrics:**
- New dependencies: **0** (tower already in workspace for oneshot)
- Infrastructure requirements: **0** (no Docker, no services)
- Setup time: **<10 minutes** (add helpers, write 2 tests)
- Test execution time: **<30 seconds** (in-memory operations)

### Final Verdict

**Pass/Fail**: ✅ **PASS**

**Summary**: Post-design review confirms that all design decisions align with Constitutional principles, especially the Development Stage Context requirement to avoid over-engineering. The testing framework prioritizes:
- Simplicity over sophistication
- Speed over robustness
- Minimal dependencies over feature-rich tooling
- Incremental complexity over upfront investment

The design explicitly rejects more complex approaches (testcontainers, mocking, LocalStack) in favor of the simplest viable solution (in-memory SurrealDB), demonstrating proper application of constitutional principles.
