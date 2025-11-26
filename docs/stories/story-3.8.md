# Story 3.8: Database Query Performance Optimization

**Status**: Not Started

## Story

**As a** backend system
**I want** all queries to meet performance targets
**So that** API response times stay under 200ms

**Prerequisites**: Story 3.5

## Acceptance Criteria

1. [ ] Group device listing query: <50ms for 20 devices
2. [ ] Single location insert: <10ms
3. [ ] Batch location insert (50 locations): <100ms
4. [ ] Device registration query: <20ms
5. [ ] All queries use prepared statements (SQLx compile-time checks)
6. [ ] EXPLAIN ANALYZE shows index usage for all queries
7. [ ] Connection pool sized appropriately (20-100 connections)

## Technical Notes

- Review all migrations for proper indexing
- Use `EXPLAIN ANALYZE` to validate query plans
- Add covering indexes where needed
- Monitor query latency via metrics

## Tasks/Subtasks

- [ ] 1. Review and optimize indexes
  - [ ] 1.1 Verify all queries use indexes
  - [ ] 1.2 Add missing indexes if needed
- [ ] 2. Validate query performance
  - [ ] 2.1 Test all critical queries under load
  - [ ] 2.2 Document performance benchmarks
- [ ] 3. Optimize connection pool
  - [ ] 3.1 Configure min/max connections appropriately
- [ ] 4. Run linting and formatting checks

## Dev Notes

- SQLx provides compile-time query checking
- Index on (device_id, captured_at DESC) for location lookups
- Index on (group_id, active) for device listings

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
