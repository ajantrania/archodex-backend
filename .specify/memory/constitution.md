# Archodex Backend Constitution

## Core Principles

### I. Data Isolation & Multi-Tenancy
Every account MUST have isolated data storage. Account resources are stored in dedicated SurrealDB databases with separate connection strings. Cross-account data leakage is strictly prohibited. Self-hosted instances MUST maintain the same isolation guarantees as the managed service. Access controls MUST be validated at both the API layer and database layer.

**Rationale**: As a platform handling sensitive cloud infrastructure telemetry, tenant isolation is non-negotiable for security and compliance.

### II. API-First Design
All functionality MUST be exposed through well-defined HTTP APIs. API contracts MUST be versioned and backward compatible. Breaking changes require MAJOR version increments and migration paths. API endpoints MUST support both human-readable and machine-readable formats (JSON).

**Rationale**: Archodex serves both dashboard UI and agent clients. Clear API boundaries enable independent evolution of frontend, backend, and agent components.

### III. Observability & Debugging
All new and updated production functions MUST use the `#[instrument]` tracing attribute for structured logging. Confidential and PII information MUST only be logged at `trace` level. Database operations, authentication flows, and tenant provisioning MUST be traceable.

**Test code exception**: Test helper functions and test fixtures MAY omit `#[instrument]` to avoid over-engineering test infrastructure. Complex test setup functions that would benefit from debugging visibility SHOULD use `#[instrument]`, but simple factory functions and fixtures need not.

**Rationale**: Deep visibility into system behavior is essential for debugging multi-tenant operations. Logs are internal-only (Archodex employees for managed service, self-hosted operators for their deployments). Test code prioritizes simplicity and clarity over exhaustive instrumentation.

### IV. Self-Hosted Parity
Features developed for the managed archodex.com service MUST work in self-hosted deployments unless explicitly scoped as managed-only (e.g., AWS account provisioning). Configuration MUST use feature flags and environment variables to differentiate deployment modes. Documentation MUST clearly indicate managed-only versus universal features.

**Rationale**: Archodex's value proposition includes self-hosted deployment. Feature parity maintains trust and reduces maintenance burden.

### V. Graph Model Integrity
Archodex's core value is graph modeling of resources and events. Resource IDs encode hierarchical relationships (e.g., AWS Partition → Account → Region → Table). The `contains` relation MUST respect single-parent constraints. Event relations MUST preserve principal chains for provenance tracking. All data model changes MUST update `DATAMODEL.md` once finalized (after changes/test/iteration cycles).

**Rationale**: Accurate graph topology is foundational to Archodex's value. Corrupted relationships break queries and visualization. Documentation sync ensures team alignment.

## Security & Compliance

### Authentication & Authorization
- All API endpoints MUST validate JWT tokens from AWS Cognito (managed) or equivalent (self-hosted)
- Report API keys MUST be validated for account association and revocation state
- User access to accounts MUST be verified via the `has_access` relation in the accounts database
- Self-hosted deployments MUST implement equivalent authentication mechanisms

### Data Protection
- Customer resource data MUST be encrypted at rest and in transit
- API key secrets MUST use AWS KMS (managed) or equivalent encryption (self-hosted)
- Salts for secret value hashing MUST be unique per account and stored securely
- Soft deletes MUST be used for accounts (via `deleted_at` timestamp)

### Audit & Compliance
- All resource mutations MUST record `created_by`/`created_at` and `deleted_by`/`deleted_at` metadata
- API key creation and revocation MUST be auditable
- Event ingestion MUST preserve principal chains for provenance tracking

## Development Workflow

### Code Quality Gates
- `cargo fmt` MUST be run after all changes (formatting enforced)
- `cargo clippy` MUST pass after all changes (linting enforced)
- Database schema changes require migrations via the `migrator` workspace member
- Data model changes MUST update `DATAMODEL.md` documentation

## Governance

### Development Stage Context
Archodex is a stealth-stage startup. Code quality and correctness are essential, but speed and simplicity matter more than industrial-scale robustness. Avoid over-engineering for scale not yet needed (e.g., AWS-level disaster recovery, complex blast radius mitigation). Build for clarity and maintainability first; optimize and harden as the product matures.

### License Considerations
Archodex releases under the Fair Core License (FCL-1.0-MIT). When implementing features, consider whether functionality should be license-restricted to prevent competitive abuse. Key license prohibitions: redistributing source code in competing products, bypassing license key enforcement. Features involving usage limits, API restrictions, and license enforcement MUST be designed to prevent user modification or bypass.

### Compliance Verification
All feature specifications and implementation plans MUST include a Constitution Check section verifying alignment with:
- Data isolation requirements (Principle I)
- API design standards (Principle II)
- Observability requirements (Principle III)
- Self-hosted parity (Principle IV)
- Graph model integrity (Principle V)
- Applicable security and compliance requirements

Complexity that violates constitutional principles MUST be justified in the plan's Complexity Tracking section.
