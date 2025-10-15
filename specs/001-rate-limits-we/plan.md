# Implementation Plan: Account Plans & Rate Limiting

**Branch**: `001-rate-limits-we` | **Date**: 2025-10-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-rate-limits-we/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement account-based rate limiting for resources and events with three backend plan tiers (Team, Organization, Custom). Enforcement will be based on configurable limits (max_resources, max_events_per_hour, update_frequency_seconds) stored in a new `plan` table. Self-hosted backends will fetch limits from archodex.com to prevent local bypass. Agents will receive plan limits embedded in their report API keys via authenticated encryption.

**Note on Stand Alone Mode**: This is an agent-only mode where agents log locally without any backend communication (no archodex.com account needed). Stand Alone Mode is hardcoded into the agent with limits of 50 resources/100 events per hour, and is NOT managed by the backend. The backend only handles the three plan tiers above.

**Implementation approach**: Two-phase rollout starting with archodex.com managed service (Phase 1 MVP), then extending to self-hosted plan fetching (Phase 2). Core challenge is efficient resource/event counting in SurrealDB 2.x without relying on COUNT() queries.

## Technical Context

**Language/Version**: Rust 2024 edition (workspace configured for edition 2024)
**Primary Dependencies**: axum 0.7, surrealdb 2.3.7, aes-gcm 0.10.3, prost 0.13.5 (protobuf), tokio 1.47
**Storage**: SurrealDB 2.x with DynamoDB backend for archodex.com (custom fork at github.com/Archodex/surrealdb), operator-choice backend for self-hosted (RocksDB, TiKV, etc.)
**Testing**: TBD - testing framework design needed as part of this feature (no automated tests currently implemented)
**Target Platform**: AWS Lambda (archodex.com managed service), Linux server (self-hosted Docker deployments)
**Project Type**: Single server project (Rust workspace with server, lambda, migrator members)
**Performance Goals**: <10ms latency overhead for rate limit checks during event ingestion (SC-008), <5ms for counter queries with up to 10k resources (SC-009)
**Constraints**: Must work with SurrealDB 2.x (COUNT() queries slow before v3.0), DynamoDB backend uses table-level transaction locks (minimize transaction scope), code isolation via comments (not strict module boundary)
**Scale/Scope**: Support accounts from Team (500 resources) to unlimited (Custom plan), handle up to 10k events/hour for Organization tier

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify alignment with the Archodex Backend Constitution (.specify/memory/constitution.md):

**Core Principles:**
- [x] **Data Isolation & Multi-Tenancy**: Plan table will be in accounts DB (shared) with account_id foreign key. Limit enforcement happens per-account in resources DBs. No cross-account data leakage - each account's limits only affect their own ingestion.
- [x] **API-First Design**: Plan management via direct database manipulation by employees (out of scope for API). Report API continues with existing endpoint but with extended protobuf. Self-hosted plan fetching will add new endpoint on archodex.com.
- [x] **Observability & Debugging**: All rate limiting functions will use `#[instrument]` attribute per constitution. Limit violations, drops, and plan fetches will be logged. No PII in rate limiting (only account IDs, resource counts, event counts).
- [x] **Self-Hosted Parity**: Phase 1 (MVP) is managed-only, Phase 2 extends to self-hosted. Feature flag differentiation via existing `archodex-com` feature. Self-hosted will fetch limits from archodex.com API to maintain parity.
- [x] **Graph Model Integrity**: Rate limiting does not modify resource/event graph relationships. New `plan` table is administrative metadata. Will update `DATAMODEL.md` after implementation with plan table schema and counter mechanism.

**Code Quality Gates:**
- [x] `cargo fmt` will be run after changes
- [x] `cargo clippy` will pass after changes
- [x] Database schema changes include migrations via `migrator` workspace - plan table creation and any counter storage migration

**Security & Compliance:**
- [x] Authentication/authorization checks for all new endpoints - plan limits in API key use existing authenticated encryption
- [x] Data encryption requirements met - plan limits in report API keys protected via AES-GCM AAD (FR-024)
- [x] Audit trail metadata (`created_by`, `created_at`, etc.) included - plan table has created_by, created_at, updated_by, updated_at per FR-018

**License Considerations (FCL-1.0-MIT):**
- [x] All rate limiting code will be isolated in dedicated module (`src/rate_limits/` or similar) per FR-022 to support license restrictions
- [x] Self-hosted plan fetching from archodex.com prevents local limit bypass, enforcing license compliance per spec requirements

**Pass/Fail**: **PASS** - All constitutional requirements met. No violations requiring justification.

---

## Constitution Check (Post-Design Re-evaluation)

*Re-evaluated after Phase 1 design completion*

**Core Principles:**
- [x] **Data Isolation & Multi-Tenancy**: ✅ CONFIRMED - Plan table in accounts DB with account_id FK. Counter table in accounts DB for cross-tenant access. No data leakage - counters and limits are per-account only.
- [x] **API-First Design**: ✅ CONFIRMED - Plan management exposed via REST API (POST/GET/PATCH /admin/accounts/{id}/plan). Self-hosted plan fetch via GET /v1/self-hosted/plan-limits. Protobuf extension is backward compatible (optional fields, version=1).
- [x] **Observability & Debugging**: ✅ CONFIRMED - All rate limit functions use #[instrument] per design. Limit violations, counter updates, plan fetches all logged. No PII (only account IDs, counts, timestamps).
- [x] **Self-Hosted Parity**: ✅ CONFIRMED - Phase 1 is managed-only, Phase 2 adds self-hosted via plan fetch API. Feature flag differentiation confirmed (archodex-com feature). **WIP NOTE**: Self-hosted authentication design (credentials, API) is deferred to Phase 2 implementation (see self-hosted-auth-summary.md marked as WIP).
- [x] **Graph Model Integrity**: ✅ CONFIRMED - Rate limiting is purely administrative metadata. No changes to resource/event graph relationships. New tables (plan, account_counter) are isolated. DATAMODEL.md will be updated post-implementation.

**Code Quality Gates:**
- [x] cargo fmt - Will be run per quickstart deployment checklist
- [x] cargo clippy - Will be run per quickstart deployment checklist
- [x] Database migrations - Migration file m20251010_create_plan_table.rs defined in quickstart

**Security & Compliance:**
- [x] Authentication - Plan management requires admin Cognito auth. Self-hosted plan fetch uses AES-GCM authenticated credential (AAD-based, tamper-proof).
- [x] Data encryption - Plan limits in report API keys protected via AAD in AES-GCM (FR-024 satisfied). Limits are authenticated but not encrypted (agents can read, backend verifies).
- [x] Audit trail - Plan table has created_by, created_at, updated_by, updated_at per design (FR-018 satisfied).

**License Considerations:**
- [x] License isolation - All rate limiting in src/rate_limits/ module per design (FR-022 satisfied).
- [x] Self-hosted limit bypass prevention - Self-hosted backends fetch limits from archodex.com API, cannot modify locally (FR-019 satisfied).

**Design Quality:**
- [x] Performance targets met - Counter mechanism <5ms (SC-009), total overhead <10ms (SC-008) per research analysis.
- [x] Backward compatibility - Protobuf extension uses optional fields, version stays at 1. Old agents/backends continue to work.
- [x] Simplicity - Three backend plan tiers (Team, Organization, Custom). Stand Alone Mode is agent-only with no backend complexity.

**Post-Design Pass/Fail**: **PASS** ✅

All constitutional requirements validated against completed design artifacts. No violations. Ready to proceed to task generation (/speckit.tasks).

## Project Structure

### Documentation (this feature)

```
specs/001-rate-limits-we/
├── spec.md              # Feature specification (input)
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── plan-management.yaml     # OpenAPI spec for employee plan admin
│   └── plan-fetch.yaml          # OpenAPI spec for self-hosted plan fetching (Phase 2)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

This is a single Rust workspace project with the following structure:

```
src/
├── rate_limits/              # NEW: Rate limiting code (marked with comments for FCL)
│   ├── mod.rs                # Module exports and shared types
│   ├── plan.rs               # Plan model, queries, CRUD operations
│   ├── counters.rs           # Resource/event counting mechanism
│   ├── enforcement.rs        # Limit enforcement logic during ingestion
│   └── plan_fetch.rs         # Self-hosted plan fetching (Phase 2)
├── report_api_key.proto      # MODIFIED: Extended with PlanLimits message in AAD
├── report_api_key.rs         # MODIFIED: Generate/validate with plan limits
├── report.rs                 # MODIFIED: Call rate limit enforcement before ingestion
├── account.rs                # MODIFIED: Relationship to plan table
├── router.rs                 # MODIFIED: Add plan management endpoints
├── auth.rs                   # May need admin auth helpers for plan management
├── db.rs                     # Connection utilities (unchanged)
├── env.rs                    # May add plan fetch config for self-hosted
├── lib.rs                    # MODIFIED: Declare rate_limits module
└── [other existing files unchanged]

migrator/
└── src/
    ├── lib.rs                # MODIFIED: Add plan table migration
    └── migrations/
        └── m20251010_create_plan_table.rs  # NEW: Plan table schema

tests/
└── [integration tests for rate limiting - to be defined in tasks.md]
```

**Structure Decision**: Single project structure (Rust workspace). All rate limiting code isolated in `src/rate_limits/` module per Fair Core License requirements. Database migrations via existing `migrator` workspace member. Integration tests will be added to validate limit enforcement end-to-end.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

**No violations** - Constitution Check passed without requiring justification.
