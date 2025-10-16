# Feature Specification: Account Plans & Rate Limiting

**Feature Branch**: `001-rate-limits-we`
**Created**: 2025-10-10
**Status**: Draft
**Input**: User description: "Rate Limits - We are going to add plans & rate limiting to our backend. We will want to be flexible for the future, but are currently thinking of 4 segments right now. The primary limits are max number of resources allowed to be tracked and max events per hour tracked. After max resources are hit, new resources (and events involving those new resources) will be dropped. After max events are hit, new events will be dropped.

The 4 plan segments:

Stand Alone Mode: This is a always free mode where the archodex-agent won't transmit any data - just log locally.
This means no self hosting or archodex-com hosted. Max Resources: 50. Max Events/hour 100

Team Mode: This is also a free tier plan. It can either be self hosted or archodex.com managed. Max Resources - 500. Max Events per hour: 1000. Agent to report into the backend at least every 20 minutes.

Organization: This is a paid plan. It can be either self hosted or archodex.com managed. Max resources: 5,000+. Max Events/hour: 10,000. Report into BE every minute

Custom: This is a paid plan. It can be either self hosted or archodex.com managed. Max resources: unlimited. Max Events/hour: unlimited. Report into BE every minute.

For now, plans will be created/updated by Archodex.com employees. We will defer building self-service planning. Plans can be something mutated, though we should track when/who changed it.

Let's keep things as simple as possible for now.

For self hosted instances, they will need to talk to archodex.com to get their plan limits - it should not be owned by it's own local table (or else users will be able to simply change the limits).


All code related to this should be in a folder as much as possible, as this will be governed by the restriction in the fair use license."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Resource Limit Enforcement (Priority: P1)

An account with a plan configured for 500 max resources is ingesting resource data from agents. When the account reaches 500 resources, the backend drops any new resources and their associated events while continuing to accept events for existing resources.

**Why this priority**: Core limiting behavior - prevents unlimited resource growth and ensures plans are enforced. Without this, rate limiting has no effect.

**Independent Test**: Create an account with a specific plan, ingest resources until the limit is reached, then verify new resources are rejected while existing resources continue to receive events.

**Acceptance Scenarios**:

1. **Given** an account with max_resources=500 and 499 resources tracked, **When** agent reports a new resource, **Then** the resource is accepted and count becomes 500
2. **Given** an account with max_resources=500 and 500 resources tracked (at limit), **When** agent reports a new resource, **Then** the new resource is dropped and count remains 500
3. **Given** an account with max_resources=500 and 500 resources tracked (at limit), **When** agent reports an event for an existing resource, **Then** the event is accepted and recorded
4. **Given** an account with max_resources=500 and 500 resources tracked (at limit), **When** agent reports an event involving a new resource, **Then** both the new resource and the event are dropped
5. **Given** an account with max_resources=null (unlimited resources), **When** agent reports any number of resources, **Then** all resources are accepted without limit

---

### User Story 2 - Events Per Hour Limit Enforcement (Priority: P1)

An account with a plan configured for 1000 max events per hour is ingesting event data from agents. When the account reaches 1000 events in the current hour, the backend drops any new events until the hour window resets.

**Why this priority**: Core limiting behavior - prevents unlimited event ingestion and protects backend from being overwhelmed. Critical for resource management.

**Independent Test**: Create an account with a specific plan, ingest events until the hourly limit is reached, verify new events are rejected, then verify limit resets after the hour boundary.

**Acceptance Scenarios**:

1. **Given** an account with max_events_per_hour=1000 and 999 events in current hour, **When** agent reports a new event, **Then** the event is accepted and count becomes 1000
2. **Given** an account with max_events_per_hour=1000 and 1000 events in current hour (at limit), **When** agent reports a new event, **Then** the event is dropped
3. **Given** an account that hit the event limit in the previous hour, **When** a new hour begins, **Then** the event counter resets to 0 and new events are accepted
4. **Given** an account with max_events_per_hour=null (unlimited events), **When** agent reports any number of events per hour, **Then** all events are accepted without limit
5. **Given** an account with max_events_per_hour=10000 and 9999 events in current hour, **When** agent reports 2 events, **Then** the first event is accepted (count 10000) and the second is dropped

---

### User Story 3 - Plan Limit Retrieval for Self-Hosted Instances (Priority: P2)

A self-hosted Archodex backend instance needs to retrieve plan limits for its account from the global archodex.com service to enforce limits locally without storing mutable limit data in its own database.

**Why this priority**: Prevents self-hosted users from bypassing limits by modifying local database values. Essential for license enforcement but can be implemented after core limiting logic.

**Independent Test**: Run a self-hosted backend, configure it with account credentials, verify it fetches plan limits from archodex.com and applies them to local ingestion.

**Acceptance Scenarios**:

1. **Given** a self-hosted backend configured for account 123456789, **When** the backend starts up, **Then** it calls archodex.com API to fetch plan limits for account 123456789
2. **Given** a self-hosted backend with plan limits cached, **When** limits are updated on archodex.com by an employee, **Then** the self-hosted backend refreshes limits within 5 minutes
3. **Given** a self-hosted backend cannot reach archodex.com, **When** ingestion occurs, **Then** the backend uses last-known cached limits and logs a warning
4. **Given** a self-hosted backend has never successfully fetched limits, **When** ingestion occurs, **Then** the backend rejects all ingestion and logs an error

---

### User Story 4 - Transmit Plan Limits to Agents (Priority: P2)

Agents receive their account's plan limits as part of the report API key so they can implement client-side rate limiting and understand their constraints before transmitting data to the backend.

**Why this priority**: Enables agents to self-throttle and avoid wasted transmission of data that will be dropped by backend limits. Prevents the agent from logging data that would be dropped, removing ability of customer to try to bypass limits. Improves user experience by providing visibility into limits. Required before agents can implement any client-side limiting logic.

**Independent Test**: Create a report API key for an account with specific plan limits, decode the API key on agent side, verify limits are present and match the account's plan.

**Acceptance Scenarios**:

1. **Given** an account with plan limits (max_resources=500, max_events_per_hour=1000, update_frequency=1200s), **When** a report API key is generated for that account, **Then** the API key contains limits with max_resources=500, max_events_per_hour=1000, update_frequency=1200 seconds
2. **Given** an account with unlimited plan limits (max_resources=null, max_events_per_hour=null), **When** a report API key is generated, **Then** the API key contains limits with max_resources and max_events_per_hour set to null
3. **Given** an agent receives a report API key with embedded limits, **When** the agent decodes the key, **Then** the limits are cryptographically verified and cannot have been tampered with
4. **Given** an account's plan limits are updated (e.g., max_resources increased from 500 to 2000), **When** a new report API key is issued, **Then** the new key reflects the updated limits

---

### User Story 5 - Plan Management by Archodex Employees (Priority: P3)

An Archodex employee needs to create a new plan for an account or modify the limits/type of an existing plan, with all changes tracked for audit purposes.

**Why this priority**: Enables plan administration but not required for MVP enforcement. Can be done via direct database manipulation initially if needed.

**Independent Test**: Employee creates or updates a plan via admin interface (or direct DB), verify changes are recorded with audit metadata (who/when), verify affected account sees new limits.

**Acceptance Scenarios**:

1. **Given** employee is authenticated as Archodex admin, **When** they create a new plan for account 123456789 with name="Team", max_resources=500, max_events_per_hour=1000, **Then** the plan is created with those limits and records created_by and created_at
2. **Given** an account has a plan with max_resources=500, **When** employee increases the limit to max_resources=2000 for that specific customer, **Then** the plan limits are updated and record updated_by and updated_at
3. **Given** an employee updates a plan's limits, **When** a self-hosted backend next fetches limits, **Then** it receives the updated limits
4. **Given** multiple plan changes over time, **When** employee views plan history, **Then** they can see all mutations with timestamps and who made each change

---

### Edge Cases

- What happens when an account is exactly at the resource limit (e.g., 500/500) and an agent reports both new resources and events for existing resources in the same report batch?
- What happens when the hourly event limit is reached mid-request (e.g., 990 events in hour, request contains 20 events)? (Resolved: Accept entire request, allowing temporary overage)
- What happens when plan limits are changed while active ingestion is happening (e.g., max_resources reduced from 5000 to 500)?
- What happens when a self-hosted backend's connection to archodex.com is intermittent (succeeds sometimes, fails other times)?
- What happens when an account's limits are reduced and the account already exceeds the new limits (e.g., 3000 resources when max_resources is changed to 500)?
- How should the system efficiently count resources and events in SurrealDB given that COUNT() queries are slow before v3.0?
- What happens in Stand Alone Mode - does the backend need to exist at all, or is this purely agent-side behavior?
- What happens when archodex.com is completely unreachable for an extended period (days/weeks) for a self-hosted instance? (Resolved: 3-day cache expiration)
- How should agents interpret unlimited limits in the API key (null values, zero, maximum integer value)?
- When plan limits are updated, agents will get the new limits by requesting a new report API key (no automatic re-issuance)
- How should self-hosted operators configure their preferred plan fetch interval (environment variable, config file, database setting)?
- How should self-hosted backends authenticate to archodex.com when fetching plan limits? (TBD: evaluate during implementation)
- Where should efficient counter values be stored for SurrealDB 2.x? (TBD: evaluate tradeoffs and measure COUNT() baseline)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST enforce a maximum number of resources per account based on their specific plan limit
- **FR-002**: System MUST enforce a maximum number of events per hour per account based on their specific plan limit
- **FR-003**: System MUST drop new resources (and events involving those new resources) when the account has reached its resource limit
- **FR-004**: System MUST drop new events when the account has reached its hourly event limit
- **FR-005**: System MUST continue accepting events for existing resources even when the resource limit is reached
- **FR-006**: System MUST reset the hourly event counter at the beginning of each clock hour
- **FR-007**: System MUST support configurable plan limits with a name field for labeling (e.g., "Team", "Organization", "Custom")
- **FR-008**: Plans MAY have a max_resources limit (if null, unlimited resources are allowed)
- **FR-009**: Plans MAY have a max_events_per_hour limit (if null, unlimited events are allowed)
- **FR-010**: Plans MUST have a configurable update_frequency_seconds value specifying minimum seconds between agent updates
- **FR-011**: Limit enforcement MUST be based on actual plan field values (max_resources, max_events_per_hour), NOT on the plan's name
- **FR-012**: Employees MUST be able to customize limits for individual accounts (e.g., increase max_resources for a specific "Team" plan customer)
- **FR-013**: Update frequency MUST be configurable per account, with minimum 60 seconds and maximum 1200 seconds (20 minutes)
- **FR-017**: Plans MUST only be created and updated by Archodex employees (no self-service)
- **FR-018**: System MUST record created_by, created_at, updated_by, and updated_at metadata for all plan mutations
- **FR-019**: Self-hosted backend instances MUST fetch plan limits from archodex.com API (MUST NOT store mutable limits in local database)
- **FR-020**: Self-hosted backend instances MUST cache plan limits locally to handle temporary archodex.com unavailability
- **FR-021**: Self-hosted backend instances MUST refresh plan limits periodically (at least every 60 minutes) to detect updates
- **FR-025**: Self-hosted backend instances MUST reject ingestion if cached plan limits are older than 3 days (72 hours)
- **FR-026**: When an account plan is downgraded, existing resources beyond the new limit MUST remain tracked but new resources MUST be rejected
- **FR-027**: When processing a request, if the request started processing before the hourly limit was reached, all data in that request MUST be accepted even if this temporarily exceeds the limit
- **FR-031**: When resources or events are dropped due to limit violations, the backend MUST send an explicit rejection response to the agent indicating the limit was hit and which data was dropped
- **FR-032**: All limit enforcement operations MUST emit structured logs using #[instrument] tracing attribute
- **FR-033**: Limit enforcement MAY emit metrics (e.g., drop counts per account) if existing metrics infrastructure supports it without requiring new systems
- **FR-034**: When an event references a resource (in principal chain or as target) that does not exist in the backend (because it was dropped due to resource limits), the event MUST be dropped to maintain graph consistency
- **FR-022**: System MUST isolate all rate limiting code in a dedicated directory/module to support Fair Core License restrictions
- **FR-023**: System MUST include plan limit information (max_resources, max_events_per_hour, update_frequency) in the report API key transmitted to agents
- **FR-024**: Plan limit information in report API keys MUST be cryptographically protected to prevent tampering (via authenticated encryption or AAD)
- **FR-028**: System MUST provide an efficient mechanism to track resource counts per account that performs acceptably with SurrealDB 2.x (before COUNT index improvements in v3.0)
- **FR-029**: System MUST provide an efficient mechanism to track event counts per hour per account that performs acceptably with SurrealDB 2.x
- **FR-030**: Self-hosted backends SHOULD allow operators to configure how often to fetch plan limit updates from archodex.com (plan_fetch_interval), defaulting to 60 minutes, minimum 5 minutes, maximum 1 day

### Key Entities

- **Plan**: Defines resource and event limits for an account. Attributes include name (label like "Team", "Organization", "Custom"), max_resources (nullable - null means unlimited), max_events_per_hour (nullable - null means unlimited), update_frequency_seconds (minimum seconds between agent updates, range: 60-1200), created_by, created_at, updated_by, updated_at. Associated with one account. Limits are customizable per account - name is for identification only, not enforcement.

- **Account**: Existing entity that now has a relationship to Plan. Each account has exactly one active plan.

- **Resource Count**: Tracks current number of resources for an account. Updated during ingestion. Used to enforce resource limits. MUST be efficiently queryable in SurrealDB 2.x without relying on COUNT() queries (potential implementation: maintain a counter field updated during resource upserts).

- **Event Count Window**: Tracks number of events ingested in the current hour for an account. Resets each clock hour. Used to enforce event rate limits. MUST be efficiently queryable in SurrealDB 2.x without relying on COUNT() queries (potential implementation: maintain hourly counter fields).

- **Report API Key**: Existing entity that now includes embedded plan limit information (max_resources, max_events_per_hour, update_frequency). Limits are cryptographically protected within the key structure to prevent tampering.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Accounts with max_resources=500 cannot add new resources regardless of agent reporting behavior
- **SC-002**: Accounts with max_events_per_hour=1000 cannot ingest more than 1000 events in any given clock hour
- **SC-003**: Accounts with max_resources=null and max_events_per_hour=null can ingest unlimited resources and events without rejection
- **SC-004**: Self-hosted backends successfully fetch and apply plan limits from archodex.com within 30 seconds of startup
- **SC-005**: When plan limits are updated by an employee, self-hosted backends apply new limits within 60 minutes without restart
- **SC-006**: Limit violations result in graceful request handling - the specific request that triggers the limit is accepted in full (allowing temporary overage), and subsequent requests are rejected until limits reset (partial batch handling prevents aggressive rejection while maintaining enforcement)
- **SC-007**: All plan mutations are auditable with complete who/when metadata
- **SC-008**: Rate limiting enforcement adds less than 10ms latency to event ingestion processing
- **SC-010**: Agents receive explicit rejection responses when data is dropped due to limits (no silent drops)
- **SC-009**: Resource and event count queries complete in under 5ms for accounts with up to 10,000 resources (SurrealDB 2.x compatibility)

## Assumptions

- **Stand Alone Mode is NOT a backend plan type** - it is a hardcoded agent-only mode where agents log locally without transmitting to any backend (no archodex.com account needed). The backend has no knowledge of Stand Alone Mode.
- Backend plan types: Team, Organization, Custom (three tiers, not four)
- Plan names like "Team", "Organization", "Custom" are labels for customer identification and defaults, but actual enforcement uses the numeric limit fields
- Default backend plan configurations: Team (500 resources, 1000 events/hour, 1200s update), Organization (5000 resources, 10000 events/hour, 60s update), Custom (null/null/60s)
- These defaults can be customized per-account - a "Team" plan customer could be given 2000 resources if needed
- Hourly event windows align with clock hours (e.g., 2:00:00 PM to 2:59:59 PM) rather than rolling 60-minute windows
- When a plan is changed, new limits apply immediately (no grace period for accounts exceeding new limits)
- Plan limit updates on archodex.com are eventually consistent to self-hosted backends (default 60-minute refresh interval)
- Agents will get updated plan limits by requesting a new report API key from the backend (keys are not automatically re-issued when plans change)
- Self-hosted backends can function with cached limits for up to 3 days (72 hours) when archodex.com is unreachable; after 3 days, cached limits expire and ingestion is rejected until connection is restored
- Resource and event counting is exact (no probabilistic data structures like HyperLogLog)
- Plan changes are rare enough that we don't need to optimize for high-frequency plan mutations

## Clarifications

### Session 2025-10-10

- Q: How does the self-hosted backend authenticate to archodex.com when fetching plan limits? → A: TBD - evaluate between account ID + shared secret (Option A) vs OAuth/JWT token (Option B) during implementation
- Q: When the backend drops resources or events due to limit violations, should the agent be notified that data was dropped? → A: Backend sends explicit rejection response to agent indicating limit hit and data dropped
- Q: For the efficient resource/event counting mechanism needed for SurrealDB 2.x, where should these counter values be stored? → A: TBD - evaluate tradeoffs between separate counter table vs embedded account fields, and measure COUNT() performance baseline before deciding
- Q: When a limit is hit during a request, what happens to data in that specific request? → A: All data in the specific request that triggered the limit will be accepted (allowing temporary overage to avoid partial batch rejection)
- Q: When limits are enforced and data is dropped, what level of observability is required? → A: Structured logs required (with #[instrument] tracing); metrics/alerts optional if existing infrastructure supports it easily - do not create new observability systems for this feature

### Implementation Sequencing

**Phase 1 (MVP)**: Archodex-hosted implementation only
- Build core rate limiting for archodex.com managed backends
- Plan management, limit enforcement, API key transmission
- Agent notification on drops
- Focus on getting managed service working end-to-end

**Phase 2**: Self-hosted extension
- Add plan limit fetching from archodex.com for self-hosted backends
- Implement caching and expiration logic
- Add configurable fetch intervals for operators
- Extend existing rate limiting to work with fetched limits

**Rationale**: Control complexity by validating core limiting logic on managed service before adding distributed fetch/cache complexity for self-hosted deployments.

## Decisions

### Cached Limit Expiration for Self-Hosted
Self-hosted backends will trust cached plan limits for **3 days (72 hours)** when archodex.com is unreachable. After 3 days, cached limits expire and the backend will reject ingestion until connection to archodex.com is restored. This balances availability (handles multi-day outages) with license enforcement (prevents indefinite disconnected operation).

### Plan Limit Reduction Behavior
When an account's limits are reduced below current usage (e.g., 3000 resources when max_resources is changed to 500), the system will use **graceful degradation**: existing resources remain tracked but new resources are dropped. This allows existing work to continue without disruption while enforcing the new limit on growth. Resources will naturally decrease over time as infrastructure changes.

### Partial Batch Handling
When an agent sends a batch of events that would partially exceed the hourly limit, the system will **honor request atomicity**: all data in the specific request that triggers the limit will be accepted. This means if a request arrives when the account is at 980/1000 events and contains 50 events, all 50 events are accepted (bringing total to 1030), and the next request will be rejected. This treats agent requests as logical units and avoids partial data loss within a single request.
