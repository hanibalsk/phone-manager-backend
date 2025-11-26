# Story 3.6: Location Retention Policy Enforcement

**Status**: Not Started

## Story

**As a** privacy-conscious system
**I want** locations older than 30 days automatically deleted
**So that** user data doesn't accumulate indefinitely

**Prerequisites**: Story 3.1, Story 3.7 (background jobs)

## Acceptance Criteria

1. [ ] Background job runs hourly to delete old locations
2. [ ] Deletes locations where `created_at < NOW() - INTERVAL '30 days'`
3. [ ] Job logs count of deleted records
4. [ ] Job completes in <5 minutes for 1M+ location records
5. [ ] Uses database function `cleanup_old_locations(retention_days)` from migrations
6. [ ] Retention period configurable via `PM__LIMITS__LOCATION_RETENTION_DAYS`

## Technical Notes

- Use `tokio::time::interval` for scheduling
- DELETE in batches (e.g., 10K rows at a time) to avoid long locks
- Add index on `created_at` for efficient cleanup

## Tasks/Subtasks

- [ ] 1. Create cleanup job
  - [ ] 1.1 Implement location cleanup job
  - [ ] 1.2 Use configurable retention period
  - [ ] 1.3 Delete in batches for performance
- [ ] 2. Register with scheduler
  - [ ] 2.1 Run hourly via job scheduler
  - [ ] 2.2 Log results
- [ ] 3. Write tests
  - [ ] 3.1 Test cleanup deletes old locations
  - [ ] 3.2 Test retention period configuration
- [ ] 4. Run linting and formatting checks

## Dev Notes

- Requires Story 3.7 (Background Job Scheduler)
- Batch deletion prevents long-running transactions
- Log deleted count for monitoring

## Dev Agent Record

### Debug Log

(Implementation notes will be added here)

### Completion Notes

(To be filled upon completion)

## File List

### Modified Files

(To be filled)

### New Files

(To be filled)

### Deleted Files

(None expected)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Code formatted with rustfmt
- [ ] Story file updated with completion notes
