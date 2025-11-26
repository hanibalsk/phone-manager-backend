# Story 3.3: Request Idempotency Support

**Status**: Complete ✅

## Story

**As a** mobile app
**I want** uploads to be idempotent based on a key
**So that** network retries don't create duplicate location records

**Prerequisites**: Story 3.1 ✅, Story 3.2 ✅

## Acceptance Criteria

1. [x] Optional `Idempotency-Key` header accepted on location uploads
2. [x] Key stored with location record or in separate `idempotency_keys` table
3. [x] Duplicate key within 24 hours returns cached response (200 with same `processedCount`)
4. [x] Duplicate detection works for both single and batch uploads
5. [x] Keys expire/cleanup after 24 hours
6. [x] Returns same response status and body for idempotent requests

## Technical Notes

- Store key hash + response in database or Redis
- Use `ON CONFLICT (idempotency_key) DO NOTHING` for simple deduplication
- Consider TTL-based cleanup job

## Tasks/Subtasks

- [x] 1. Add idempotency key support
  - [x] 1.1 Extract idempotency key from header
  - [x] 1.2 Check for existing key before processing
  - [x] 1.3 Store key with response after successful processing
- [x] 2. Add cleanup for expired keys
  - [x] 2.1 Keys older than 24 hours should be deleted
- [x] 3. Write tests
  - [x] 3.1 Test duplicate key returns cached response
  - [x] 3.2 Test key expiration
- [x] 4. Run linting and formatting checks

## Dev Notes

- Can use simple hash storage in locations table or separate table
- Future optimization: Redis for distributed idempotency

## Dev Agent Record

### Debug Log

- Implemented idempotency_keys table with SHA-256 key hashing
- Extractor checks for existing key and returns cached response
- Keys automatically cleaned up after 24 hours via background job

### Completion Notes

Idempotency support fully implemented for both single and batch uploads. Keys hashed and stored with TTL-based cleanup.

## File List

### Modified Files

- `crates/api/src/routes/locations.rs` - idempotency key handling
- `crates/api/src/extractors/mod.rs` - idempotency key extractor
- `crates/persistence/src/migrations/` - idempotency_keys table

### New Files

- `crates/persistence/src/repositories/idempotency.rs` - idempotency repository

### Deleted Files

(None)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |
| 2025-11-26 | Implementation complete | Dev Agent |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes

---

## Senior Developer Review (AI)

### Reviewer: Martin Janci
### Date: 2025-11-26
### Outcome: ✅ Approve

### Summary
Idempotency support properly implemented with key hashing, response caching, and automatic TTL cleanup. Prevents duplicate uploads from network retries.

### Key Findings
- **[Info]** SHA-256 hashing for key storage
- **[Info]** 24-hour TTL with automatic cleanup
- **[Info]** Works for both single and batch uploads

### Acceptance Criteria Coverage
| AC | Status | Evidence |
|----|--------|----------|
| AC1 - Idempotency-Key header | ✅ | Header extractor |
| AC2 - Key stored | ✅ | idempotency_keys table |
| AC3 - Duplicate returns cached | ✅ | Response caching logic |
| AC4 - Works for single/batch | ✅ | Both handlers use same extractor |
| AC5 - 24h expiration | ✅ | TTL-based cleanup job |
| AC6 - Same response | ✅ | Cached response returned |

### Test Coverage and Gaps
- Idempotency key tests
- Expiration tests
- No gaps identified

### Architectural Alignment
- ✅ Clean separation of idempotency concern
- ✅ Background job for cleanup

### Security Notes
- Key hashing prevents key enumeration attacks

### Action Items
None - story approved for completion.

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Senior Developer Review notes appended | AI Reviewer |
