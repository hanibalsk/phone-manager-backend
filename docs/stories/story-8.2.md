# Story 8.2: Map-Matching Service Integration

**Epic**: Epic 8 - Intelligent Path Detection
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** backend system
**I want** to integrate with a map-matching service
**So that** I can correct GPS traces to road networks

## Prerequisites

- Story 8.1 complete (path correction schema)

## Acceptance Criteria

1. Service client for OSRM Match API or Valhalla Trace API
2. Configuration: `PM__MAP_MATCHING__PROVIDER` (osrm|valhalla), `PM__MAP_MATCHING__URL`, `PM__MAP_MATCHING__TIMEOUT_MS`
3. Accepts array of coordinates, returns snapped coordinates
4. Handles service timeouts gracefully (30s default)
5. Extracts confidence/quality metric from response
6. Rate limiting to map-matching service (configurable)
7. Circuit breaker for service failures (fails open)

## Technical Notes

- Use reqwest for HTTP client
- OSRM: GET /match/v1/driving/{coordinates}
- Valhalla: POST /trace_route with trace_options
- Consider retry with exponential backoff

## Implementation Tasks

- [x] Add map-matching configuration to config.rs
- [x] Create MapMatchingProvider enum (Osrm, Valhalla)
- [x] Create MapMatchingClient trait
- [x] Implement OsrmClient for OSRM Match API
- [ ] Implement ValhhallaClient for Valhalla Trace API (optional - OSRM first)
- [x] Add reqwest dependency to Cargo.toml
- [x] Create MapMatchingRequest and MapMatchingResponse types
- [x] Handle timeouts with configurable duration
- [x] Extract confidence metric from service response
- [x] Add basic rate limiting (requests per minute)
- [x] Implement circuit breaker for service failures
- [x] Add tests for service client

---

## Dev Notes

- Focus on OSRM first as it's simpler (GET with polyline)
- OSRM Match endpoint: GET /match/v1/{profile}/{coordinates}?overview=full
- Circuit breaker opens after consecutive failures, fails open (returns error)
- Rate limiting prevents overwhelming external service

---

## Dev Agent Record

### Debug Log
- Starting Story 8.2 implementation

### Completion Notes
- Implemented OSRM Match API client with full support
- MapMatchingClient struct with match_coordinates() method
- Token bucket rate limiter (requests per minute)
- Circuit breaker with Open/Closed/HalfOpen states
- MapMatchingConfig with all configurable options
- Comprehensive error handling (MapMatchingError enum)
- Unit tests for rate limiter, circuit breaker, and client

---

## File List

- `Cargo.toml` - Added reqwest workspace dependency
- `crates/api/Cargo.toml` - Added reqwest.workspace = true
- `config/default.toml` - Added [map_matching] configuration section
- `crates/api/src/config.rs` - Added MapMatchingConfig struct
- `crates/api/src/lib.rs` - Added services module
- `crates/api/src/services/mod.rs` - Created services module
- `crates/api/src/services/map_matching.rs` - MapMatchingClient implementation

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed - MapMatchingClient with OSRM support |
