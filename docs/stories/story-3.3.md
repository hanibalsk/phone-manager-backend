# Story 3.3: Request Idempotency Support

**Status**: Not Started

## Story

**As a** mobile app
**I want** uploads to be idempotent based on a key
**So that** network retries don't create duplicate location records

**Prerequisites**: Story 3.1, Story 3.2

## Acceptance Criteria

1. [ ] Optional `Idempotency-Key` header accepted on location uploads
2. [ ] Key stored with location record or in separate `idempotency_keys` table
3. [ ] Duplicate key within 24 hours returns cached response (200 with same `processedCount`)
4. [ ] Duplicate detection works for both single and batch uploads
5. [ ] Keys expire/cleanup after 24 hours
6. [ ] Returns same response status and body for idempotent requests

## Technical Notes

- Store key hash + response in database or Redis
- Use `ON CONFLICT (idempotency_key) DO NOTHING` for simple deduplication
- Consider TTL-based cleanup job

## Tasks/Subtasks

- [ ] 1. Add idempotency key support
  - [ ] 1.1 Extract idempotency key from header
  - [ ] 1.2 Check for existing key before processing
  - [ ] 1.3 Store key with response after successful processing
- [ ] 2. Add cleanup for expired keys
  - [ ] 2.1 Keys older than 24 hours should be deleted
- [ ] 3. Write tests
  - [ ] 3.1 Test duplicate key returns cached response
  - [ ] 3.2 Test key expiration
- [ ] 4. Run linting and formatting checks

## Dev Notes

- Can use simple hash storage in locations table or separate table
- Future optimization: Redis for distributed idempotency

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
