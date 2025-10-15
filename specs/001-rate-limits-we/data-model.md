# Data Model: Account Plans & Rate Limiting

**Feature**: 001-rate-limits-we
**Date**: 2025-10-10
**Phase**: Phase 1 Design

This document defines the database schema for plan management and rate limiting.

## Overview

The rate limiting feature introduces two new tables in the **accounts database**:
1. **`plan`** - Stores account plan configurations with limits and metadata
2. **`usage`** - Maintains efficient resource/event counters per account

Additionally, the existing **report API key protobuf** is extended to embed plan limits.

---

## Database: Accounts (shared)

### Table: `plan`

Stores plan configurations for accounts. Each account can have multiple plans over time (historical tracking), with exactly one active plan at any moment. Plans define resource limits, event rate limits, and update frequency requirements.

**SurrealDB Table Definition:**

```surrealql
DEFINE TABLE IF NOT EXISTS plan SCHEMAFULL TYPE NORMAL;

-- Fields
DEFINE FIELD IF NOT EXISTS account_id ON TABLE plan TYPE record<account> READONLY;
DEFINE FIELD IF NOT EXISTS name ON TABLE plan TYPE string;
DEFINE FIELD IF NOT EXISTS max_resources ON TABLE plan TYPE option<int>;
DEFINE FIELD IF NOT EXISTS max_events_per_hour ON TABLE plan TYPE option<int>;
DEFINE FIELD IF NOT EXISTS update_frequency_seconds ON TABLE plan TYPE int
  ASSERT $value >= 60 AND $value <= 1200;
DEFINE FIELD IF NOT EXISTS start ON TABLE plan TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS end ON TABLE plan TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE plan TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS created_by ON TABLE plan TYPE string;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE plan TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS updated_by ON TABLE plan TYPE option<record<user>>;

-- Indexes
DEFINE INDEX IF NOT EXISTS plan_account_active_idx ON TABLE plan FIELDS account_id, start, end;
```

**Fields:**

| Field | Type | Nullable | Default | Assertions | Description |
|-------|------|----------|---------|------------|-------------|
| `account_id` | `record<account>` | No | - | READONLY | Foreign key to account table. Multiple plans per account (1:many for history). |
| `name` | `string` | Yes | `NONE` | - | Label like "Team", "Organization", "Custom". For identification only, NOT enforcement. |
| `max_resources` | `option<int>` | Yes | `NONE` | ≥ 0 if set | Maximum resources allowed. `NONE` = unlimited. |
| `max_events_per_hour` | `option<int>` | Yes | `NONE` | ≥ 0 if set | Maximum events per hour. `NONE` = unlimited. |
| `update_frequency_seconds` | `int` | No | - | 60 ≤ value ≤ 1200 | Minimum seconds between agent updates. Required field. |
| `start` | `datetime` | No | `time::now()` | - | Plan effective start time. |
| `end` | `option<datetime>` | Yes | `NONE` | - | Plan effective end time. `NONE` = currently active plan. |
| `created_at` | `datetime` | No | `time::now()` | - | Plan creation timestamp. |
| `created_by` | `string` | No | - | - | Identifier of who/what created the plan (e.g., user ID, "archodex-migrator", "system"). |
| `updated_at` | `option<datetime>` | Yes | `NONE` | - | Last update timestamp. |
| `updated_by` | `option<record<user>>` | Yes | `NONE` | - | User who last updated the plan. |

**Record ID Format:** Auto-generated (e.g., `plan:abc123def456`) since accounts can have multiple plan records

**Enforcement Semantics:**
- Limit enforcement is based on **field values** (`max_resources`, `max_events_per_hour`), NOT `name`
- `name` is purely for labeling (e.g., a "Team" plan could be customized to 2000 resources)
- `NONE` (null) values mean unlimited for that dimension
- `update_frequency_seconds` is always required (valid range: 60-1200)

**Plan History & Active Plan Query:**
- Active plan: `WHERE account_id = account:{id} AND start <= time::now() AND (end IS NONE OR end > time::now())`
- Historical plans have `end` set to when they were replaced
- Only one active plan per account at any time (enforced via application logic)
- Plan changes create new record with current `start`, set old record's `end = time::now()`

**Default Backend Plan Configurations:**

| Plan Name | max_resources | max_events_per_hour | update_frequency_seconds | Typical Use Case |
|-----------|---------------|---------------------|--------------------------|------------------|
| Team | 500 | 1000 | 1200 (20 min) | Free tier, self-hosted or managed |
| Organization | 5000 | 10000 | 60 (1 min) | Paid tier, frequent updates |
| Custom | `NONE` (unlimited) | `NONE` (unlimited) | 60 (1 min) | Paid tier, no limits |

*Note: These are default templates. Actual limits are customizable per account.*

**Stand Alone Mode (Not in Backend):**
- Stand Alone Mode is NOT a backend plan type
- It is an agent-only mode where agents operate without any backend (no archodex.com account)
- Limits are hardcoded in the agent: 50 resources, 100 events/hour
- The backend has no knowledge of or involvement with Stand Alone Mode

**Audit Trail:**
- `created_by`, `created_at` - Track plan record creation
- `updated_by`, `updated_at` - Track modifications to plan records
- `start`, `end` - Track plan effective periods for complete history
- Satisfies FR-018 audit requirements

**Plan Lifecycle & Auto-Creation:**
- Plans are **auto-created** during account creation (default: Team plan)
- Employees update plans by creating new records and ending old ones
- No manual plan creation by employees - only modifications

---

### Table: `usage`

Maintains efficient resource and event counters per account. Avoids slow COUNT() queries by using atomic UPSERT operations.

**SurrealDB Table Definition:**

```surrealql
DEFINE TABLE IF NOT EXISTS usage SCHEMAFULL TYPE NORMAL;

-- Fields
DEFINE FIELD IF NOT EXISTS account_id ON TABLE usage TYPE record<account> READONLY;
DEFINE FIELD IF NOT EXISTS resource_count ON TABLE usage TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS event_hour_window ON TABLE usage TYPE option<string>;
DEFINE FIELD IF NOT EXISTS event_count_this_hour ON TABLE usage TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS last_updated_at ON TABLE usage TYPE option<datetime>;

-- Indexes
DEFINE INDEX IF NOT EXISTS usage_account_hour_idx ON TABLE usage FIELDS account_id, event_hour_window UNIQUE;
```

**Fields:**

| Field | Type | Nullable | Default | Description |
|-------|------|----------|---------|-------------|
| `account_id` | `record<account>` | No | - | Foreign key to account. READONLY to prevent updates. |
| `resource_count` | `int` | No | 0 | Current total resource count for the account (stored in resource counter record). |
| `event_hour_window` | `option<string>` | Yes | `NONE` | Hour window for event tracking (format: "YYYY-MM-DDTHH"). |
| `event_count_this_hour` | `int` | No | 0 | Event count for this specific hour window. |
| `last_updated_at` | `option<datetime>` | Yes | `NONE` | Timestamp of last counter update. |

**Record ID Format:**
- **Resource counter**: `usage:{account_id}:resources` (e.g., `usage:1234567890:resources`)
- **Event counter**: `usage:{account_id}:{hour}` (e.g., `usage:1234567890:2025-10-10T14`)

**Hour Window Format:** `"YYYY-MM-DDTHH"` (e.g., `"2025-10-10T14"`)
- Represents clock hour boundaries (e.g., 14:00:00 - 14:59:59)
- Used to identify specific hour slots for event tracking
- **Hour derived from `last_seen_at` timestamp** (most recent event occurrence time, not report ingestion time)

**Counter Types:**

1. **Resource Counter** (one per account):
   - ID: `usage:{account_id}:resources`
   - Tracks total resource count across all time
   - `event_hour_window` is `NONE` for resource counters

2. **Event Counters** (multiple per account, one per hour):
   - ID: `usage:{account_id}:{hour}`
   - Tracks event count for specific hour window
   - `event_hour_window` set to hour string (e.g., "2025-10-10T14")
   - `resource_count` is 0 for event counters

**Counter Update Semantics:**

**Resource Counter (Only Count NEW Resources):**
```surrealql
-- During resource ingestion, use RETURN BEFORE to detect new vs updated resources
INSERT INTO resource (id, first_seen_at, last_seen_at)
VALUES (...)
ON DUPLICATE KEY UPDATE last_seen_at = $input.last_seen_at
RETURN BEFORE;

-- After query execution, count results where BEFORE is null (new inserts)
-- Then atomically increment counter by new resource count only
UPSERT usage:{account_id}:resources SET
  resource_count = (resource_count ?? 0) + {new_resource_count},
  last_updated_at = time::now();
```

**Key Insight:** Using `RETURN BEFORE`:
- If `BEFORE` is `null` → new resource was created → count it
- If `BEFORE` has data → existing resource was updated → don't count
- This ensures resource_count = unique resources, not total upserts

**Event Counter with Hour Segregation (Multi-Hour Support):**
```rust
async fn check_and_increment_event_counters(
    account_id: &str,
    events: Vec<Event>,
    plan: &Plan,
) -> Result<Vec<Event>> {
    let db = accounts_db().await?;
    let mut accepted_events = Vec::new();
    let mut hour_limit_errors = Vec::new();

    // Step 1: Segregate events by hour from last_seen_at timestamps
    let mut events_by_hour: HashMap<String, Vec<Event>> = HashMap::new();
    for event in events {
        let hour = event.last_seen_at.format("%Y-%m-%dT%H").to_string();
        events_by_hour.entry(hour).or_default().push(event);
    }

    // Step 2: For each hour segment, fetch current count, check limit, update counter
    for (hour, events_in_hour) in events_by_hour {
        // Fetch counter for this specific account + hour combination
        let counter: Option<Counter> = db
            .query("SELECT * FROM usage WHERE account_id = $account_id AND event_hour_window = $hour LIMIT 1")
            .bind(("account_id", format!("account:{}", account_id)))
            .bind(("hour", hour.clone()))
            .await?
            .take(0)?;

        // Determine current count for this specific hour
        let current_count = counter.map(|c| c.event_count_this_hour).unwrap_or(0);

        // Check if adding these events would exceed limit for THIS hour
        if let Some(max) = plan.max_events_per_hour {
            if current_count + events_in_hour.len() as i64 > max {
                // Drop events for THIS hour only, continue processing other hours
                hour_limit_errors.push(format!(
                    "Event limit exceeded for hour {}: {}/{} events",
                    hour, current_count, max
                ));
                continue;  // Skip to next hour
            }
        }

        // Step 3: Update counter for this hour (limits not exceeded)
        let counter_id = format!("usage:{}:{}", account_id, hour);
        db.query(
            "UPSERT $counter_id SET
                account_id = $account_id,
                event_hour_window = $hour,
                event_count_this_hour = (event_count_this_hour ?? 0) + $count,
                resource_count = 0,
                last_updated_at = time::now();"
        )
        .bind(("counter_id", counter_id))
        .bind(("account_id", format!("account:{}", account_id)))
        .bind(("hour", hour))
        .bind(("count", events_in_hour.len() as i64))
        .await?;

        // Add accepted events for this hour
        accepted_events.extend(events_in_hour);
    }

    // If some hours were rejected but others succeeded, return partial success
    if !hour_limit_errors.is_empty() {
        if accepted_events.is_empty() {
            // All hours rejected
            return Err(RateLimitError::AllEventsRejected(hour_limit_errors));
        } else {
            // Partial rejection - log warnings but proceed
            for error in hour_limit_errors {
                warn!("{}", error);
            }
        }
    }

    Ok(accepted_events)
}
```

**Hour Segregation Logic (Handles Multi-Hour Requests):**
- **Step 1**: Segregate events by hour extracted from `last_seen_at` (occurrence time)
- **Step 2**: For each hour:
  - Query for counter record with matching account_id AND event_hour_window
  - If counter exists, use current count; otherwise start from 0
  - Check if adding events exceeds limit for THIS hour only
  - If exceeded: drop events for this hour, continue processing other hours
  - If within limit: UPSERT counter for this account-hour combination
- **Step 3**: Return accepted events (may be partial if some hours exceeded limits)

**Key Benefits:**
- Each hour has its own counter record (supports multi-hour tracking)
- Direct query by account_id + hour using composite unique index
- No single-record bottleneck - different hours tracked independently
- Handles events spanning multiple hour blocks in single request
- Each hour segment checked and enforced independently
- Events from one hour exceeding limits don't affect other hours
- Partial batch handling: accept valid hours, reject over-limit hours

**Performance Characteristics:**
- Indexed lookup on (account_id, event_hour_window): O(log n) - meets <5ms requirement (SC-009)
- Atomic UPSERT: No race conditions
- Auto-creates on first increment (no initialization needed)
- Multiple concurrent hour updates don't conflict

---

## Modified Entity: Account

The existing `account` table gains a relationship to the `plan` table.

**New Relationship:**
- Each account has **one active plan** at any time (1:many relationship for history)
- Relationship expressed via foreign key in `plan` table
- Multiple plan records per account allowed (historical tracking)

**No schema changes to account table** - relationship is expressed via foreign key in `plan` table.

**Query Pattern:**
```surrealql
-- Get account with active plan
SELECT *,
  (SELECT * FROM plan
   WHERE account_id = $parent.id
   AND start <= time::now()
   AND (end IS NONE OR end > time::now())
   LIMIT 1)[0] AS active_plan
FROM account:{account_id};

-- Get account with plan history
SELECT *,
  (SELECT * FROM plan
   WHERE account_id = $parent.id
   ORDER BY start DESC) AS plan_history
FROM account:{account_id};
```

---

## Modified Entity: Report API Key

The report API key protobuf is extended to include plan limits in the **outer plaintext message** for agent-side limiting, while also including them in **encrypted contents** for backend tamper verification.

**Updated Protobuf Schema:**

```protobuf
syntax = "proto3";

package archodex.report_api_key;

message ReportApiKey {
  uint32 version = 1; // Always 1
  optional string endpoint = 2;
  bytes account_salt = 3;
  bytes nonce = 4;
  bytes encrypted_contents = 5;
  PlanLimits plan_limits = 6; // NEW: Plan limits in plaintext (for agent-side limiting)
}

message ReportApiKeyEncryptedContents {
  fixed64 account_id = 1;
  PlanLimits plan_limits = 2; // NEW: Plan limits encrypted (for backend tamper detection)
}

message ReportApiKeyEncryptedAAD {
  fixed32 key_id = 1;
  optional string endpoint = 2;
  bytes account_salt = 3;
}

// NEW: Plan limits transmitted to agents
message PlanLimits {
  optional uint64 max_resources = 1;       // Absent = unlimited
  optional uint64 max_events_per_hour = 2; // Absent = unlimited
  uint32 update_frequency_seconds = 3;     // Required, range 60-1200
}
```

**Field Semantics (Simplified - No Backwards Compatibility Needed):**

| Field | Presence | Value | Meaning |
|-------|----------|-------|---------|
| `plan_limits` (outer) | Always present | - | All new keys include plan limits in plaintext |
| `plan_limits` (inner) | Always present | - | Encrypted copy for backend verification |
| `max_resources` | Absent | - | Unlimited resources (Custom plan) |
| `max_resources` | Present | > 0 | Specific limit (e.g., 500 for Team) |
| `max_events_per_hour` | Absent | - | Unlimited events (Custom plan) |
| `max_events_per_hour` | Present | > 0 | Specific limit (e.g., 1000 for Team) |
| `update_frequency_seconds` | Always present | 60-1200 | Required update frequency |

**Simplified Unlimited Encoding:**
- **Unlimited**: Field is absent (not set in protobuf)
- **Limited**: Field is present with value > 0
- **No sentinel value needed**: Cleaner semantics using protobuf's optional feature

**Security Properties:**
- Plan limits in **outer message** (plaintext) - agents can read for client-side limiting
- Plan limits in **encrypted contents** - backend verifies no tampering by comparing outer vs decrypted
- If outer `plan_limits` ≠ encrypted `plan_limits` → reject as tampered
- Backend **always** fetches authoritative limits from `plan` table (ignores key values for enforcement)
- Key limits are for agent-side optimization only, not security boundary

**Backward Compatibility:**
- Version field stays at 1
- Optional fields allow old agents to ignore new data
- Old keys (without `plan_limits`) continue to work
- Gradual migration via key rotation

---

## Entity Relationships

```
┌─────────────────┐
│     user        │
└────────┬────────┘
         │
         │ created_by / updated_by
         │
         ▼
┌─────────────────┐      1:many      ┌─────────────────┐
│    account      │◄─────────────────┤      plan       │
└────────┬────────┘   account_id     │  (w/ history)   │
         │                           └─────────────────┘
         │ 1:many                              │
         │ (resource + events)                 │ defines limits
         ▼                                     │
┌─────────────────┐                           ▼
│      usage      │                  ┌─────────────────┐
│  (multiple      │                  │ report_api_key  │
│   records)      │                  │  (protobuf)     │
└─────────────────┘                  └─────────────────┘
         │                                     │
         │                                     │ contains
         │                                     │ (plaintext +
         │                                     │  encrypted)
         │                                     ▼
         │                           ┌─────────────────┐
         │                           │  PlanLimits     │
         └───────────────────────────┤  (protobuf)     │
           enforced against          └─────────────────┘
```

---

## State Transitions

### Plan Lifecycle

```
┌─────────────┐
│ Account     │  New account created → auto-create plan
│  Created    │  - Default Team plan (500 resources, 1000 events/hr)
└──────┬──────┘  - start = time::now(), end = NONE
       │          - created_by = "system" or user ID
       │
       │ Employee updates plan (upgrade/downgrade)
       ▼
┌─────────────┐
│  Plan       │  1. Create new plan record with new limits
│  Updated    │     - start = time::now(), end = NONE
│             │  2. Set old plan record: end = time::now()
└──────┬──────┘  3. New API keys fetch active plan limits
       │
       │ (Plans are never deleted, retained for history)
       ▼
┌─────────────┐
│   Active    │  Active plan: WHERE start <= NOW() AND
│  (ongoing)  │               (end IS NONE OR end > NOW())
└─────────────┘  - Self-hosted backends fetch current plan
                 - Historical plans available for auditing
```

### Counter Lifecycle

```
┌─────────────┐
│ Uninitialized│ Counter doesn't exist yet
│   (NONE)    │
└──────┬──────┘
       │ First resource/event ingested
       ▼
┌─────────────┐
│ Initialized │  UPSERT creates counter with initial values
│ (count > 0) │  - Resource: usage:{account_id}:resources
└──────┬──────┘  - Event: usage:{account_id}:{hour}
       │
       │ Subsequent ingestion
       ▼
┌─────────────┐
│ Incrementing│  Counters increase via UPSERT +=
│             │  - resource_count += NEW resources only (RETURN BEFORE = null)
└──────┬──────┘  - event_count for specific hours tracked independently
       │
       │ New hour events arrive
       ▼
┌─────────────┐
│  New Hour   │  New counter record created for new hour
│   Counter   │  - usage:{account_id}:{new_hour}
└──────┬──────┘  - event_count_this_hour starts at event count
       │
       └──────────────┐ Loop back to incrementing
```

**Resource Counting Logic:**
- Use `RETURN BEFORE` on INSERT ... ON DUPLICATE KEY UPDATE
- If `BEFORE` is null → NEW resource → increment counter
- If `BEFORE` has data → UPDATED resource → don't increment
- Ensures resource_count = unique resources, not total upserts
- Single counter record: `usage:{account_id}:resources`

**Event Counting Logic (Per-Hour Records):**
- Extract hour from `last_seen_at` field: `event.last_seen_at.format("%Y-%m-%dT%H")`
- Query for counter: `SELECT * FROM usage WHERE account_id = account:{id} AND event_hour_window = {hour}`
- If exists: increment `event_count_this_hour`
- If not exists: UPSERT creates new record `usage:{account_id}:{hour}`
- **Uses event occurrence time**, not backend ingestion time
- Each hour has independent counter record

### API Key Generation with Limits

```
┌─────────────┐
│ User Request│  Dashboard or API call to create key
│ Create Key  │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Fetch Plan  │  Backend queries active plan for account
│   Limits    │  SELECT * FROM plan WHERE account_id = {id}
└──────┬──────┘  AND start <= NOW() AND (end IS NONE OR end > NOW())
       │
       ▼
┌─────────────┐
│ Encode      │  1. Build PlanLimits protobuf message
│ Plan Limits │  2. Add to outer message (plaintext, agent-readable)
└──────┬──────┘  3. Add to encrypted contents (tamper detection)
       │
       ▼
┌─────────────┐
│  Encrypt    │  AES-GCM encryption (account_id + plan_limits)
│ Contents    │  - Outer plan_limits: agents read for client-side limiting
└──────┬──────┘  - Inner plan_limits: backend verifies no tampering
       │
       ▼
┌─────────────┐
│Return Key to│  Key contains plan limits in two locations
│    User     │  - Agent reads outer plaintext limits
└─────────────┘  - Backend verifies outer = decrypted inner
```

---

## Validation Rules

### Plan Table

1. **account_id** must reference valid account (enforced by `record<account>` type)
2. **update_frequency_seconds** must be between 60 and 1200 (ASSERT clause)
3. **max_resources** if set, must be ≥ 0
4. **max_events_per_hour** if set, must be ≥ 0
5. **created_by** must be a non-empty string (user ID or system identifier like "archodex-migrator")
6. **updated_by** if set, must reference valid user
7. Only one active plan per account (enforced by application logic via start/end)

### Account Counter Table

1. **account_id** must reference valid account
2. **resource_count** must be ≥ 0
3. **event_count_this_hour** must be ≥ 0
4. **event_hour_window** if set, must match format "YYYY-MM-DDTHH"
5. Unique constraint on (account_id, event_hour_window) - one record per account-hour combination

### Report API Key Protobuf

1. **version** must be 1
2. **account_salt** must be 16 bytes
3. **nonce** must be 12 bytes (AES-GCM requirement)
4. **plan_limits.update_frequency_seconds** if present, must be 60-1200
5. **plan_limits.max_resources** if present and > 0, represents limit
6. **plan_limits.max_events_per_hour** if present and > 0, represents limit

---

## Migration Notes

### Creating Plan Table

Migration will be added to `migrator` workspace:

```rust
// migrator/src/migrations/m20251010_create_plan_table.rs

pub async fn up(db: &Surreal<Any>) -> Result<()> {
    // Create plan table
    db.query(include_str!("m20251010_create_plan_table.surql"))
        .await?
        .check()?;

    Ok(())
}
```

### Creating Account Counter Table

Same migration file will create both tables:

```surrealql
-- m20251010_create_plan_table.surql

-- Plan table
DEFINE TABLE IF NOT EXISTS plan SCHEMAFULL TYPE NORMAL;
-- ... (field definitions as above)

-- Account counter table
DEFINE TABLE IF NOT EXISTS usage SCHEMAFULL TYPE NORMAL;
-- ... (field definitions as above)
```

### Default Plans for Existing Accounts

**Auto-creation during account creation:**
- All new accounts automatically get a default Team plan
- Plan created in same transaction as account creation
- start = account.created_at, end = NONE

**Backfill migration for existing accounts:**
- Create plan records for all existing accounts without plans
- Use Team plan defaults (500 resources, 1000 events/hour, 1200s update frequency)
- start = time::now(), end = NONE
- created_by = system user

**Migration Query:**
```surrealql
-- Backfill plans for accounts without active plans
FOR $account IN (SELECT id FROM account) {
  LET $existing_plan = (SELECT * FROM plan
                        WHERE account_id = $account.id
                        AND end IS NONE
                        LIMIT 1);
  IF count($existing_plan) = 0 THEN
    CREATE plan CONTENT {
      account_id: $account.id,
      name: "Team",
      max_resources: 500,
      max_events_per_hour: 1000,
      update_frequency_seconds: 1200,
      start: time::now(),
      end: NONE,
      created_by: "archodex-migrator"
    };
  END;
};
```

---

## Performance Considerations

### Counter Query Performance

**Resource count check:**
```surrealql
SELECT resource_count FROM usage:{account_id}:resources;
```
- Direct ID lookup: O(1)
- Expected time: <1ms
- Meets SC-009 requirement (<5ms)

**Event count check for specific hour:**
```surrealql
SELECT event_count_this_hour FROM usage
WHERE account_id = account:{account_id}
AND event_hour_window = "2025-10-10T14"
LIMIT 1;
```
- Indexed lookup on (account_id, event_hour_window): O(log n)
- Expected time: <2ms
- Meets SC-009 requirement

### Plan Lookup Performance

```surrealql
SELECT * FROM plan
WHERE account_id = account:{account_id}
AND start <= time::now()
AND (end IS NONE OR end > time::now())
LIMIT 1;
```
- Composite index on `(account_id, start, end)`: O(log n) lookup
- Expected time: <2ms (with proper indexing)
- Can be cached in memory per account connection for duration of ingestion transaction

### Rate Limit Enforcement Overhead

Total overhead per ingestion request:
1. Plan lookup: ~2ms (cached after first lookup in transaction)
2. Counter read (resource + N event hours): ~3ms
3. Counter write (resource + N event hours): ~4ms
4. **Total: ~9ms** ✅ Meets SC-008 requirement (<10ms)

Note: Overhead only applies when limits exist. Unlimited plans (Custom) skip counter checks.

---

## Summary

**New Tables:**
- `plan` - Account plan configurations with history (accounts DB, 1:many per account)
- `usage` - Efficient resource/event counters (accounts DB, multiple records per account)

**Modified Entities:**
- `account` - Gains 1:many relationship to `plan` (historical tracking)
- `report_api_key` - Protobuf extended with `PlanLimits` in both plaintext and encrypted sections

**Key Design Decisions:**
- Plan history via `start`/`end` fields (1:many relationship, query for active plan)
- Counters in separate table to avoid COUNT() queries (SurrealDB 2.x compatibility)
- **Multiple counter records per account**: One resource counter (`usage:{account_id}:resources`) + one per hour for events (`usage:{account_id}:{hour}`)
- Composite unique index on (account_id, event_hour_window) for efficient multi-hour tracking
- Plan limits in API key outer message (agents read) + encrypted contents (backend verifies)
- RETURN BEFORE to count only NEW resources (not updates)
- Event hour attribution using `last_seen_at` (occurrence time, not report time)
- Hour segregation with per-hour limit enforcement and partial batch handling
- Backward compatible protobuf extension (version stays at 1)

**Performance Targets:**
- ✅ Counter queries <5ms (SC-009)
- ✅ Rate limit enforcement overhead <10ms (SC-008)
- ✅ Indexed lookups (no table scans)
