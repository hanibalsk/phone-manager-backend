# Story 7.2: Enhanced Location Upload with Context

**Epic**: Epic 7 - Enhanced Location Context
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** mobile app
**I want** to include transportation mode in location uploads
**So that** location data has movement context

## Prerequisites

- Story 7.1 complete (location schema migration)

## Acceptance Criteria

1. `POST /api/v1/locations` enhanced to accept: `"transportation_mode": "<mode-optional>", "detection_source": "<source-optional>", "tripId": "<uuid-optional>"`
2. Existing payload structure unchanged (new fields optional)
3. Validates transportationMode and detectionSource same as movement events
4. Returns 404 if tripId provided but trip doesn't exist
5. Returns 404 if tripId's trip doesn't belong to deviceId
6. Response unchanged: `{"success": true, "processedCount": 1}`
7. Batch upload also supports new fields

## Technical Notes

- Maintain backward compatibility - all new fields optional
- Validate tripId belongs to same device for security
- Reuse TransportationMode and DetectionSource validation from movement events

## Implementation Tasks

- [x] Add transportation_mode, detection_source, trip_id to UploadLocationRequest
- [x] Add transportation_mode, detection_source, trip_id to LocationData (batch)
- [x] Add validation for transportation_mode and detection_source
- [x] Update LocationInput struct to include context fields
- [x] Add trip validation (exists and belongs to device)
- [x] Update upload_location handler to process new fields
- [x] Update upload_batch handler to process new fields
- [x] Add tests for new fields

---

## Dev Notes

- TransportationMode and DetectionSource types already exist in movement_event.rs
- TripRepository already exists for trip validation
- All new fields are optional for backward compatibility

---

## Dev Agent Record

### Debug Log
- Starting Story 7.2 implementation
- Added context fields to UploadLocationRequest and LocationData
- Updated LocationInput struct with context fields
- Updated insert_location and insert_locations_batch SQL
- Added trip validation in upload_location handler
- Added batch trip validation in upload_batch handler
- Updated all test fixtures across domain, persistence, and api crates
- All tests passing, clippy clean

### Completion Notes
Story 7.2 completed successfully. Enhanced location upload endpoints to support context fields:
- transportation_mode: Optional TransportationMode enum (e.g., WALKING, IN_VEHICLE)
- detection_source: Optional DetectionSource enum (e.g., ACTIVITY_RECOGNITION)
- trip_id: Optional UUID with validation (trip must exist and belong to device)

Both single and batch upload endpoints support the new fields while maintaining backward compatibility.

---

## File List

- `crates/domain/src/models/location.rs` - Added context fields to UploadLocationRequest and LocationData
- `crates/persistence/src/repositories/location.rs` - Added context fields to LocationInput, updated INSERT queries
- `crates/api/src/routes/locations.rs` - Added trip validation and context field processing

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed |
| 2025-11-30 | Senior Developer Review: APPROVED |

---

## Senior Developer Review (AI)

**Reviewer**: Martin Janci
**Date**: 2025-11-30
**Outcome**: ✅ **APPROVED**

### Summary

Story 7.2 enhances location upload endpoints with context fields while maintaining backward compatibility. All 7 acceptance criteria are met.

### Acceptance Criteria Coverage

| AC | Status | Evidence |
|----|--------|----------|
| POST accepts context fields | ✅ | `UploadLocationRequest:70-76` |
| Existing payload unchanged | ✅ | All fields are `Option<>` |
| Validates transportationMode/detectionSource | ✅ | Reuses movement_event enums |
| Returns 404 if tripId doesn't exist | ✅ | Trip validation in handler |
| Returns 404 if trip doesn't belong to device | ✅ | Device ownership check |
| Response unchanged | ✅ | Same `{success, processedCount}` |
| Batch supports new fields | ✅ | `LocationData` has context fields |

### Key Strengths

- Reuses TransportationMode/DetectionSource from movement_event.rs
- Trip ownership validation for security
- Both single and batch uploads support context fields
- Comprehensive test fixtures updated

### Action Items

None required.
