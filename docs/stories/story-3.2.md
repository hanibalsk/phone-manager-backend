# Story 3.2: Batch Location Upload API

**Status**: Complete ✅

## Story

**As a** mobile app
**I want** to upload multiple locations at once
**So that** I can efficiently sync when coming back online

**Prerequisites**: Story 3.1 ✅

## Acceptance Criteria

1. [x] `POST /api/v1/locations/batch` accepts JSON: `{"deviceId": "<uuid>", "locations": [<location-objects>]}`
2. [x] Validates: 1-50 locations per batch, max 1MB payload
3. [x] Each location validated same as single upload
4. [x] Returns 400 if batch validation fails with details
5. [x] Returns 404 if device not registered
6. [x] Returns 200 with: `{"success": true, "processedCount": <count>}`
7. [x] Request timeout: 30 seconds
8. [x] All locations inserted in single transaction (atomic)

## Technical Notes

- Use SQLx batch insert: `INSERT INTO locations (...) VALUES ($1,$2,$3), ($4,$5,$6), ...`
- Transaction ensures all-or-nothing semantics
- Consider using `COPY` for larger batches (future optimization)

## Tasks/Subtasks

- [x] 1. Add batch insert method to repository
  - [x] 1.1 Implement `insert_locations_batch` method
  - [x] 1.2 Use transaction for atomicity
- [x] 2. Implement upload_batch handler
  - [x] 2.1 Validate batch size limits
  - [x] 2.2 Verify device exists
  - [x] 2.3 Process all locations in transaction
  - [x] 2.4 Update device last_seen_at
- [x] 3. Write tests
  - [x] 3.1 Test batch size validation
  - [x] 3.2 Test transaction atomicity
  - [x] 3.3 Test error responses
- [x] 4. Run linting and formatting checks

## Dev Notes

- BatchUploadRequest already defined in domain models
- Maximum 50 locations per batch (from config)
- All-or-nothing semantics via transaction

## Dev Agent Record

### Debug Log

- Implemented insert_locations_batch with transaction wrapper
- Handler validates batch size (1-50 locations)
- All locations inserted atomically
- Device last_seen_at updated to most recent location timestamp

### Completion Notes

Batch upload fully functional with atomic transactions and configurable batch size limits.

## File List

### Modified Files

- `crates/api/src/routes/locations.rs` - upload_batch handler
- `crates/persistence/src/repositories/location.rs` - batch insert method

### New Files

(None)

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
Batch location upload properly implemented with atomic transactions and configurable batch limits. Efficient for mobile sync scenarios.

### Key Findings
- **[Info]** Transaction ensures all-or-nothing semantics
- **[Info]** Batch size limit (50) prevents resource exhaustion
- **[Info]** Efficient batch INSERT query

### Acceptance Criteria Coverage
| AC | Status | Evidence |
|----|--------|----------|
| AC1 - POST batch endpoint | ✅ | BatchUploadRequest struct |
| AC2 - 1-50 locations, 1MB limit | ✅ | Validation + body limit |
| AC3 - Per-location validation | ✅ | Validator on each location |
| AC4 - 400 for batch errors | ✅ | ApiError::Validation |
| AC5 - 404 for unregistered device | ✅ | Device existence check |
| AC6 - 200 with processedCount | ✅ | Returns locations.len() |
| AC7 - 30s timeout | ✅ | Tower timeout layer |
| AC8 - Atomic transaction | ✅ | SQLx transaction |

### Test Coverage and Gaps
- Batch size validation tested
- Transaction rollback tested
- No gaps identified

### Architectural Alignment
- ✅ Follows repository pattern
- ✅ Transaction management at repository layer

### Security Notes
- Batch size limits prevent DoS via large payloads

### Action Items
None - story approved for completion.

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Senior Developer Review notes appended | AI Reviewer |
