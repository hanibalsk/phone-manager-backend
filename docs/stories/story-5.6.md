# Story 5.6: Transportation Mode Enum and Validation

**Epic**: Epic 5 - Movement Events API
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** backend system
**I want** strict validation of transportation modes and detection sources
**So that** only valid values enter the database

## Prerequisites

- Story 5.1 (Movement Event Database Schema)

## Acceptance Criteria

1. TransportationMode enum: STATIONARY, WALKING, RUNNING, CYCLING, IN_VEHICLE, UNKNOWN
2. DetectionSource enum: ACTIVITY_RECOGNITION, BLUETOOTH_CAR, ANDROID_AUTO, MULTIPLE, NONE
3. Enums implement Serialize/Deserialize with exact string matching
4. Invalid mode/source returns 400 with error message
5. Database VARCHAR fields validated against enum values
6. Unit tests cover all enum variants and invalid values

## Technical Notes

- Use serde rename_all = "SCREAMING_SNAKE_CASE" for API
- Implement FromStr and Display traits for enums
- Store as VARCHAR in database for flexibility

## Implementation Notes

This story was implemented as part of Story 5.2. The enums are defined in:
- `crates/domain/src/models/movement_event.rs`

Both enums include:
- Serialize/Deserialize with SCREAMING_SNAKE_CASE
- FromStr implementation for parsing
- Display implementation
- as_str() method for database storage
- Comprehensive unit tests for all variants and error cases
