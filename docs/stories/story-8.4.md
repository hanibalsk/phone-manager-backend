# Story 8.4: Path Correction Retrieval Endpoint

**Epic**: Epic 8 - Intelligent Path Detection
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** client application
**I want** to retrieve corrected paths for trips
**So that** I can display accurate route information to users

## Prerequisites

- Story 8.1 complete (path correction schema)
- Story 8.3 complete (automatic path correction)

## Acceptance Criteria

1. GET /api/v1/trips/:tripId/path endpoint
2. Returns original path, corrected path (if available), and status
3. Returns correction quality score when available
4. Returns 404 if trip or correction not found
5. Paths returned as coordinate arrays [lat, lon]

## Technical Notes

- Response includes both original and corrected paths
- Quality score indicates map-matching confidence
- Status indicates: PENDING, COMPLETED, FAILED, SKIPPED
- GeoJSON stored in database, converted to [lat, lon] arrays for response

## Implementation Tasks

- [x] Create GetPathCorrectionResponse DTO in domain crate (already exists as TripPathResponse)
- [x] Add get_trip_path endpoint to routes
- [x] Fetch correction from TripPathCorrectionRepository
- [x] Convert stored GeoJSON to response format
- [x] Return proper 404 for missing trip/correction
- [x] Add unit tests for GeoJSON parsing

---

## Dev Notes

- Endpoint is authenticated like other trip routes
- GeoJSON already stored in database as ST_AsGeoJSON output
- Coordinates converted from [lon, lat] to [lat, lon] for client convenience

---

## Dev Agent Record

### Debug Log
- Starting Story 8.4 implementation
- Added get_trip_path endpoint to trips routes
- Added route to app.rs
- Implemented parse_geojson_linestring helper

### Completion Notes
- GET /api/v1/trips/:tripId/path endpoint implemented
- Parses GeoJSON LineString from database
- Converts coordinates from [lon, lat] to [lat, lon] format
- Returns original_path, corrected_path, correction_status, correction_quality
- Returns 404 if trip not found or no path correction exists
- Comprehensive unit tests for GeoJSON parsing

---

## File List

- `crates/api/src/routes/trips.rs` - Added get_trip_path endpoint and parse_geojson_linestring helper
- `crates/api/src/app.rs` - Added /api/v1/trips/:trip_id/path route

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed - GET /api/v1/trips/:tripId/path endpoint |
