# Story UGM-2.3: Migration Metrics

**Status**: Complete ✅

## Story

**As a** system operator,
**I want** to monitor migration success and failure rates,
**So that** I can detect issues and track feature adoption.

**Epic**: UGM-2: Group Migration
**Prerequisites**: Story UGM-2.2: Group Migration Endpoint

## Acceptance Criteria

1. [x] Given the Prometheus metrics endpoint exists, when a migration succeeds, then the `migration_total{status="success"}` counter is incremented
2. [x] Given a migration fails, when the failure is recorded, then the `migration_total{status="failure"}` counter is incremented
3. [x] Given a migration operation runs, when it completes (success or failure), then the `migration_duration_seconds` histogram records the duration
4. [x] Given the `/metrics` endpoint is called, when migration metrics are requested, then all migration counters and histograms are included in the response

## Technical Notes

- Metrics use the existing `metrics` crate with `metrics-exporter-prometheus`
- Three new metrics:
  - `migration_total{status, reason?}` - Counter for total migrations (success/failure)
  - `migration_duration_seconds` - Histogram for migration duration
  - `migration_devices_total` - Counter for total devices migrated
- Failure reasons include: `validation_error`, `already_migrated`, `no_devices`, `not_device_owner`, `group_name_exists`, `transaction_start_error`, `create_group_error`, `create_membership_error`, `update_devices_error`, `audit_log_error`, `commit_error`

## Tasks/Subtasks

- [x] 1. Add record_migration_success function to metrics middleware
- [x] 2. Add record_migration_failure function to metrics middleware
- [x] 3. Add timing to migrate_registration_group handler
- [x] 4. Record success metrics on successful migration
- [x] 5. Record failure metrics on all error paths

## File List

### Files Created

- `docs/stories/story-UGM-2.3.md` - This story file

### Files Modified

- `crates/api/src/middleware/metrics.rs` - Added migration metrics functions
- `crates/api/src/routes/groups.rs` - Added metrics recording to migration endpoint

## Implementation Details

### Metrics Functions

```rust
// Record successful migration
pub fn record_migration_success(duration_secs: f64, devices_migrated: i32);

// Record failed migration
pub fn record_migration_failure(duration_secs: f64, reason: &str);
```

### Integration Points

The migration endpoint (`migrate_registration_group`) now:
1. Records start time at the beginning
2. Records failure metrics on each error path with specific reason
3. Records success metrics after successful transaction commit

### Prometheus Output

When scraped, the `/metrics` endpoint includes:
- `migration_total{status="success"}` - Successful migrations
- `migration_total{status="failure",reason="..."}` - Failed migrations by reason
- `migration_duration_seconds` - Duration histogram with default buckets
- `migration_devices_total` - Total devices migrated across all migrations

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass (unit tests in workspace)
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Code passes clippy
- [x] Story file updated with completion notes

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created and implemented | Dev Agent |
