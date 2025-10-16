# Review Checklist: Account Plans & Rate Limiting

**Feature**: 001-rate-limits-we
**Status**: Ready for Review
**Date**: 2025-10-10

All documents have been updated based on your feedback. Please review in the order listed below.

---

## ✅ Changes Made (Per Your Feedback)

### Major Issues - RESOLVED
- ✅ **DynamoDB backend**: Updated to specify DynamoDB for archodex.com, operator-choice for self-hosted
- ✅ **Per-account namespaces**: Acknowledged in v3 COUNT index research (still problematic for time-based counting)
- ✅ **Events with unknown resources**: Added FR-034 requirement to drop events referencing non-existent resources

### Medium Issues - RESOLVED
- ✅ **Testing framework**: Marked as TBD, acknowledged no automated tests currently exist
- ✅ **Plan management API**: Removed from scope, employees use direct database access

### Minor Issues - RESOLVED
- ✅ **AWS Lambda**: Added to target platform description
- ✅ **Protobuf file structure**: Consolidated into single `report_api_key.proto`
- ✅ **License isolation**: Clarified as comments-based, not strict module boundary

### Research Questions - ANSWERED
- ✅ **HyperLogLog**: Not recommended (worse in every dimension at your scale)
- ✅ **SurrealDB v3 COUNT index**: Partial compatibility (resources yes, hourly events no)
- ✅ **Self-hosted auth**: Single credential per account (multi-backend = abuse scenario)
- ✅ **Protobuf optional**: Simplified to "absent = unlimited" (cleaner semantics)

---

## 📋 Review Order

### 1. Core Documents (Start Here)

#### ✅ spec.md
**What changed**:
- Added FR-034: Events with unknown resources must be dropped
- Updated assumptions: Stand Alone Mode is agent-only, not backend plan type
- Backend plan tiers: Team, Organization, Custom (three, not four)

**Review focus**:
- [ ] FR-034 makes sense for graph consistency
- [ ] Three backend plan tiers are correct
- [ ] All functional requirements still accurate

---

#### ✅ plan.md
**What changed**:
- Technical context: DynamoDB backend, AWS Lambda, testing TBD
- Removed plan management API from scope
- Simplified protobuf file structure
- License isolation via comments (not strict module)
- Constitution check passed (post-design re-evaluation included)

**Review focus**:
- [ ] Technical context accurately reflects your architecture
- [ ] Project structure makes sense (src/rate_limits/ with comments)
- [ ] Constitution check conclusions reasonable
- [ ] Implementation approach (two phases) still correct

---

#### ✅ data-model.md
**What changed**:
- Simplified protobuf: PlanLimits required in AAD, optional fields = unlimited
- Removed "0 as sentinel" encoding (cleaner to use absent = unlimited)
- Updated plan configurations (removed Stand Alone Mode)
- All in single `report_api_key.proto` file

**Review focus**:
- [ ] Plan table schema correct
- [ ] Counter table schema makes sense
- [ ] Protobuf simplified encoding (absent = unlimited) acceptable
- [ ] Database migration approach sound

---

#### ✅ research.md
**What changed**:
- Added Section 4: HyperLogLog analysis (NOT RECOMMENDED)
- Added Section 5: SurrealDB v3 COUNT index (PARTIAL COMPATIBILITY)
- Added Section 6: Self-hosted auth summary (single credential per account)
- Per-account namespace acknowledged in v3 assessment

**Review focus**:
- [ ] HLL analysis makes sense (rejection justified)
- [ ] v3 COUNT index assessment updated for per-account namespaces
- [ ] Hybrid approach (v3 for resources, counter for events) reasonable future path
- [ ] Self-hosted auth summary matches your single-backend understanding

---

### 2. Contracts & APIs

#### ✅ contracts/plan-fetch.yaml
**What changed**:
- Nothing (self-hosted plan fetch API still valid for Phase 2)

**Review focus**:
- [ ] API contract makes sense for self-hosted backends
- [ ] Request/response schemas appropriate
- [ ] Error handling comprehensive

#### ❌ contracts/plan-management.yaml
**Status**: REMOVED (out of scope per your feedback)

---

### 3. Implementation Guide

#### ✅ quickstart.md
**What changed**:
- Step 8: Removed plan management API endpoints, replaced with direct database access
- Removed admin API implementation sections
- Updated key files reference (router.rs now only for Phase 2)

**Review focus**:
- [ ] Phase 1 steps make sense (database, counters, enforcement, protobuf)
- [ ] Direct database access approach for plan management acceptable
- [ ] Phase 2 steps clear (self-hosted plan fetching)
- [ ] Implementation checklist comprehensive

---

### 4. Self-Hosted Authentication Design (NEW)

#### ✅ self-hosted-auth-summary.md (NEW)
**What it contains**:
- Complete design for single-credential-per-account approach
- Credential structure (AES-GCM authenticated)
- Database schema for credentials
- API endpoints (auto-generate, regenerate, fetch)
- Abuse detection (optional, non-blocking)
- Security analysis
- Code examples
- Implementation timeline (2-3 weeks for Phase 2)

**Review focus**:
- [ ] Single credential per account approach acceptable
- [ ] Auto-generation during account creation makes sense
- [ ] Regeneration flow (invalidates old) reasonable
- [ ] Abuse detection optional and non-blocking per requirements
- [ ] Security analysis matches threat model
- [ ] Implementation timeline realistic

---

## 🎯 Key Decisions to Confirm

Before proceeding to `/speckit.tasks`, please confirm:

1. **Counter Mechanism**:
   - [ ] Exact counter table (not HyperLogLog) approved
   - [ ] Account counter in accounts DB (not per-account namespace) acceptable
   - [ ] DynamoDB table-level transaction locks considered

2. **Protobuf Encoding**:
   - [ ] "Absent = unlimited" encoding cleaner than "0 = unlimited"
   - [ ] Breaking change acceptable (no active customers)
   - [ ] All in single `report_api_key.proto` file

3. **Self-Hosted Auth**:
   - [ ] Single credential per account (not multi-backend feature)
   - [ ] Auto-generate during account creation
   - [ ] Regeneration invalidates previous credential
   - [ ] Abuse detection optional and non-blocking

4. **Plan Management**:
   - [ ] Direct database access by employees (no admin API)
   - [ ] Example SurrealQL queries in quickstart sufficient

5. **Testing Strategy**:
   - [ ] Defer testing framework design to implementation phase
   - [ ] Or: Should I create a testing strategy document now?

6. **Events with Unknown Resources**:
   - [ ] FR-034 requirement makes sense
   - [ ] Implementation: Validate all resource IDs in principal chains exist before ingesting events

---

## 📝 Questions or Concerns?

If any of the above needs clarification or revision:
1. Note which document and section
2. Describe the issue or question
3. I'll update and we'll iterate

Once all documents are approved, we proceed to `/speckit.tasks` for task generation.

---

## 📊 Document Status Summary

| Document | Status | Changes | Review Priority |
|----------|--------|---------|-----------------|
| spec.md | ✅ Updated | FR-034, 3 plan tiers | **High** |
| plan.md | ✅ Updated | DynamoDB, Lambda, scope | **High** |
| data-model.md | ✅ Updated | Protobuf simplified | **High** |
| research.md | ✅ Updated | HLL, v3, auth sections | Medium |
| contracts/plan-fetch.yaml | ✅ No Change | Self-hosted API | Low |
| quickstart.md | ✅ Updated | Removed plan mgmt API | Medium |
| self-hosted-auth-summary.md | ✅ NEW | Complete auth design | **High** |
| ~~plan-management.yaml~~ | ❌ Removed | Out of scope | N/A |

---

## ✨ Ready to Proceed?

Once you've reviewed and approved (or requested changes), respond with:
- "Approved - proceed to tasks" (if everything looks good)
- Or: Specific feedback on documents that need revision

I'll then either:
- Make requested changes and present for re-review
- Proceed to `/speckit.tasks` to generate implementation task list
