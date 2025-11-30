# Story 8.5: On-Demand Path Correction

**Epic**: Epic 8 - Intelligent Path Detection
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** client application
**I want** to trigger path correction manually for a trip
**So that** I can re-process failed corrections or correct paths for older trips

## Prerequisites

- Story 8.1 complete (path correction schema)
- Story 8.2 complete (map-matching service)
- Story 8.3 complete (path correction service)

## Acceptance Criteria

1. POST /api/v1/trips/:tripId/correct-path endpoint
2. Re-triggers path correction for the specified trip
3. Returns status indicating correction queued/started
4. Can override existing FAILED or SKIPPED corrections
5. Returns 404 if trip not found
6. Returns 409 if correction is already PENDING or in-progress
7. Returns appropriate error if map-matching service unavailable

## Technical Notes

- Allows manual retry of failed corrections
- Can be used for trips that were completed before map-matching was enabled
- Uses existing PathCorrectionService
- Should validate trip exists and is in COMPLETED state

## Implementation Tasks

- [x] Create CorrectPathResponse DTO (already exists in domain)
- [x] Add trigger_path_correction endpoint to trips routes
- [x] Validate trip exists and is COMPLETED
- [x] Check if correction exists and handle status appropriately
- [x] Delete existing FAILED/SKIPPED correction if needed
- [x] Call PathCorrectionService.correct_trip_path
- [x] Return appropriate response status
- [x] Add unit tests

---

## Dev Notes

- Endpoint should be authenticated
- Consider rate limiting on-demand corrections
- Should work for trips that were completed before automatic correction was available

---

## Dev Agent Record

### Debug Log
- Starting Story 8.5 implementation
- Added trigger_path_correction endpoint to trips.rs
- Added POST /api/v1/trips/:trip_id/correct-path route to app.rs

### Completion Notes
- POST /api/v1/trips/:tripId/correct-path endpoint implemented
- Validates trip exists and is in COMPLETED state
- Returns 409 Conflict if correction is PENDING (in-progress)
- Allows retry for FAILED, SKIPPED, or COMPLETED corrections by deleting existing and re-running
- Returns CorrectPathResponse with status and message
- Handles map-matching service unavailable (returns SKIPPED status)
- Uses existing PathCorrectionService for correction workflow

---

## File List

- `crates/api/src/routes/trips.rs` - Added trigger_path_correction endpoint
- `crates/api/src/app.rs` - Added POST /api/v1/trips/:trip_id/correct-path route

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed - POST /api/v1/trips/:tripId/correct-path endpoint |
