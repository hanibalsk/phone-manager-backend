# Story 3.9: Materialized View Refresh for Group Stats

**Status**: Not Started

## Story

**As a** backend system
**I want** efficient group statistics via materialized views
**So that** group queries remain fast as data grows

**Prerequisites**: Story 3.7 (background jobs)

## Acceptance Criteria

1. [ ] `group_member_counts` materialized view refreshed hourly
2. [ ] View provides: group_id, member_count, last_activity
3. [ ] Refresh completes in <1 minute for 10K groups
4. [ ] Refresh runs as background job (non-blocking)
5. [ ] View used for group size validation queries (future optimization)

## Technical Notes

- Created in migration 005
- `REFRESH MATERIALIZED VIEW CONCURRENTLY group_member_counts`
- Requires UNIQUE index on group_id

## Tasks/Subtasks

- [ ] 1. Create materialized view refresh job
  - [ ] 1.1 Implement refresh job
  - [ ] 1.2 Use CONCURRENTLY for non-blocking refresh
- [ ] 2. Register with scheduler
  - [ ] 2.1 Run hourly via job scheduler
  - [ ] 2.2 Log refresh results
- [ ] 3. Write tests
  - [ ] 3.1 Test view refresh
- [ ] 4. Run linting and formatting checks

## Dev Notes

- View created in database migrations
- CONCURRENTLY allows reads during refresh
- Requires unique index on view

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
