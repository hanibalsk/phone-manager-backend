# Story 3.2: Batch Location Upload API

**Status**: Not Started

## Story

**As a** mobile app
**I want** to upload multiple locations at once
**So that** I can efficiently sync when coming back online

**Prerequisites**: Story 3.1

## Acceptance Criteria

1. [ ] `POST /api/v1/locations/batch` accepts JSON: `{"deviceId": "<uuid>", "locations": [<location-objects>]}`
2. [ ] Validates: 1-50 locations per batch, max 1MB payload
3. [ ] Each location validated same as single upload
4. [ ] Returns 400 if batch validation fails with details
5. [ ] Returns 404 if device not registered
6. [ ] Returns 200 with: `{"success": true, "processedCount": <count>}`
7. [ ] Request timeout: 30 seconds
8. [ ] All locations inserted in single transaction (atomic)

## Technical Notes

- Use SQLx batch insert: `INSERT INTO locations (...) VALUES ($1,$2,$3), ($4,$5,$6), ...`
- Transaction ensures all-or-nothing semantics
- Consider using `COPY` for larger batches (future optimization)

## Tasks/Subtasks

- [ ] 1. Add batch insert method to repository
  - [ ] 1.1 Implement `insert_locations_batch` method
  - [ ] 1.2 Use transaction for atomicity
- [ ] 2. Implement upload_batch handler
  - [ ] 2.1 Validate batch size limits
  - [ ] 2.2 Verify device exists
  - [ ] 2.3 Process all locations in transaction
  - [ ] 2.4 Update device last_seen_at
- [ ] 3. Write tests
  - [ ] 3.1 Test batch size validation
  - [ ] 3.2 Test transaction atomicity
  - [ ] 3.3 Test error responses
- [ ] 4. Run linting and formatting checks

## Dev Notes

- BatchUploadRequest already defined in domain models
- Maximum 50 locations per batch (from config)
- All-or-nothing semantics via transaction

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
