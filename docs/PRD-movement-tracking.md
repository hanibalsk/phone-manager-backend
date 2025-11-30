# Product Requirements Document: Movement Tracking & Intelligent Path Detection

**Project:** Phone Manager Backend Extension
**Version:** 1.0
**Date:** 2025-11-30
**Author:** Martin Janci
**Status:** Draft

---

## 1. Description

Movement Tracking & Intelligent Path Detection is a backend feature extension for the Phone Manager application. This extension enables comprehensive trip lifecycle management, movement event logging with sensor telemetry, enhanced location tracking with transportation mode context, and map-snapping path correction capabilities.

The system receives movement data from Android clients that perform local trip detection using activity recognition, Bluetooth car detection, and Android Auto integration. The backend stores, processes, and serves this movement data while providing intelligent path correction through external map-matching services.

### Key Capabilities

- **Trip Management**: Full lifecycle tracking of user trips from start to completion
- **Movement Events**: Detailed event logging with sensor telemetry and confidence metrics
- **Transportation Modes**: Classification support for STATIONARY, WALKING, RUNNING, CYCLING, IN_VEHICLE, and UNKNOWN modes
- **Path Correction**: Map-snapping integration to align GPS traces to road/path networks
- **Offline Support**: Idempotent operations supporting client-side storage with sync on connectivity

---

## 2. Deployment Intent

- **Type:** Production SaaS/Application
- **Scale:** Multi-tenant backend serving mobile clients
- **Environment:** Cloud-hosted PostgreSQL with PostGIS extension
- **Integration:** REST API consumed by Android application

---

## 3. Goals

### Primary Goals

| ID | Goal | Success Metric |
|----|------|----------------|
| G1 | Enable trip tracking from client detection | 100% trip data persistence from client submissions |
| G2 | Store movement events with full telemetry | Complete sensor data capture per event |
| G3 | Provide map-snapped path corrections | Path accuracy improvement >80% vs raw GPS |
| G4 | Support offline-first client architecture | Zero data loss with idempotent operations |
| G5 | Maintain existing system performance | <200ms p95 response time for all endpoints |

### Secondary Goals

| ID | Goal | Success Metric |
|----|------|----------------|
| G6 | Enable trip history analysis | Query support for date ranges and filters |
| G7 | Support multiple detection sources | Handle all client detection mechanisms |
| G8 | Provide trip statistics | Aggregate distance, duration, mode breakdown |

---

## 4. Context

### Project Type

**Brownfield Extension** - This PRD extends the existing Phone Manager backend which already provides:

- Device registration and management (Epic 1)
- Location tracking with batch upload (Epic 2)
- Group management for family/friends sharing (Epic 3)
- Geofencing and proximity alerts (Epic 4)

### Existing Infrastructure

| Component | Status | Integration Point |
|-----------|--------|-------------------|
| Device Authentication | Implemented | API key via X-API-Key header |
| Location Storage | Implemented | PostgreSQL with coordinate validation |
| Group Membership | Implemented | Device-to-group relationships |
| Geofences | Implemented | Per-device circular regions |
| Proximity Alerts | Implemented | Device-to-device distance monitoring |

### Technology Stack

- **Language:** Rust 1.83+ (Edition 2024)
- **Framework:** Axum 0.8 with Tokio async runtime
- **Database:** PostgreSQL 16 with SQLx + PostGIS extension
- **New Tables:** movement_events, trips, frequent_locations, common_routes

### Client Architecture

The Android client (developed separately) implements:
- Local trip detection state machine
- Activity recognition integration
- Bluetooth car and Android Auto detection
- Offline storage with sync status tracking
- Background location collection during trips

---

## 5. Functional Requirements

### Trip Management

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01 | System shall create trips with client-generated localTripId for idempotency | Must Have |
| FR-02 | System shall update trip state (ACTIVE ’ COMPLETED/CANCELLED) | Must Have |
| FR-03 | System shall store trip metadata: start/end timestamps, detection source, transportation mode | Must Have |
| FR-04 | System shall calculate and store trip statistics on completion | Must Have |
| FR-05 | System shall support trip retrieval by device with pagination | Must Have |
| FR-06 | System shall support trip retrieval by date range | Should Have |

### Movement Events

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-07 | System shall store movement events with sensor telemetry (accuracy, speed, bearing, altitude) | Must Have |
| FR-08 | System shall associate movement events with trips via tripId | Must Have |
| FR-09 | System shall store transportation mode and confidence per event | Must Have |
| FR-10 | System shall support batch movement event upload (max 100/batch) | Must Have |
| FR-11 | System shall validate event timestamps are within reasonable bounds | Must Have |

### Location Enhancement

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-12 | System shall store transportation mode with location uploads | Must Have |
| FR-13 | System shall store detection source with location uploads | Must Have |
| FR-14 | System shall support optional tripId association for locations | Should Have |

### Path Correction

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-15 | System shall provide map-snapped coordinates for trip paths | Should Have |
| FR-16 | System shall store both original and corrected coordinates | Must Have |
| FR-17 | System shall indicate correction confidence/quality | Should Have |
| FR-18 | System shall support on-demand path correction requests | Could Have |

---

## 6. Non-Functional Requirements

### Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | API response time (p95) | <200ms |
| NFR-02 | Batch upload processing | <500ms for 100 events |
| NFR-03 | Path correction latency | <2s for trips <1000 points |
| NFR-04 | Concurrent connections | 10,000+ |

### Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-05 | System availability | 99.9% uptime |
| NFR-06 | Data durability | Zero data loss with idempotent operations |
| NFR-07 | Graceful degradation | Function without map-snapping service |

### Scalability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-08 | Movement events per device | 100,000+ lifetime |
| NFR-09 | Active trips per device | 1 concurrent |
| NFR-10 | Historical trips per device | Unlimited (with pagination) |

### Security

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-11 | Authentication | API key with SHA-256 hashing |
| NFR-12 | Authorization | Device ownership validation |
| NFR-13 | Data isolation | Group-based access control |
| NFR-14 | Rate limiting | 100 requests/minute per API key |

---

## 7. User Journeys

### Journey 1: Automatic Trip Recording

**Actor:** Mobile app user with trip detection enabled

**Flow:**
1. User gets in car; Android detects Bluetooth connection to car audio
2. Client creates local trip record with ACTIVE state
3. Client begins collecting location and movement events
4. Client periodically syncs events to backend via batch upload
5. Backend stores events with tripId association
6. User arrives at destination; activity recognition detects STATIONARY
7. Client marks trip as COMPLETED with end timestamp
8. Client sends trip completion to backend
9. Backend calculates trip statistics (distance, duration, mode breakdown)
10. Backend triggers path correction for the trip
11. User can view trip in history with corrected path on map

**Success Criteria:**
- Complete trip data persisted within 30 seconds of completion
- Path correction applied within 2 minutes
- All movement events associated with trip

### Journey 2: Offline Trip with Delayed Sync

**Actor:** Mobile app user traveling through areas with poor connectivity

**Flow:**
1. User starts trip in area with connectivity
2. Client creates trip and syncs initial state to backend
3. User enters area without connectivity
4. Client continues collecting movement events locally
5. Client queues events for sync with PENDING status
6. User returns to connectivity
7. Client performs batch sync of queued events
8. Backend processes events idempotently (handles duplicates)
9. Trip data is complete on backend
10. User views complete trip history

**Success Criteria:**
- Zero data loss during offline period
- Idempotent sync handles retry scenarios
- Trip timeline is complete and accurate

### Journey 3: Trip History Review

**Actor:** User reviewing past trips

**Flow:**
1. User opens trip history in app
2. Client requests trips for device with pagination
3. Backend returns trip list with summary statistics
4. User selects specific trip for details
5. Client requests trip with movement events
6. Backend returns trip with original and corrected paths
7. User views trip on map with path visualization
8. User can toggle between raw GPS and map-snapped view
9. User reviews trip statistics and mode breakdown

**Success Criteria:**
- Trip list loads within 200ms
- Trip details with events load within 500ms
- Both raw and corrected paths available

---

## 8. UX Principles

| ID | Principle | Implementation |
|----|-----------|----------------|
| UX-01 | Seamless Background Operation | Trip detection and data collection require no user intervention |
| UX-02 | Offline Resilience | Full functionality without connectivity; transparent sync |
| UX-03 | Data Accuracy | Map-snapped paths provide realistic route visualization |
| UX-04 | Quick Access | Trip history loads immediately with pagination |
| UX-05 | Transparency | Show both raw and corrected paths for user trust |
| UX-06 | Battery Efficiency | Backend optimized for batch operations reducing sync frequency |
| UX-07 | Privacy Control | Clear data export and deletion capabilities |
| UX-08 | Consistent Experience | Same trip data visible across reinstalls via backend storage |
| UX-09 | Graceful Degradation | Show raw paths when correction unavailable |
| UX-10 | Progressive Enhancement | Basic trips work immediately; corrections enhance over time |

---

## 9. Epic Overview

### Epic 5: Movement Events API
Core movement event storage and retrieval with sensor telemetry support.

| Story | Description | Priority |
|-------|-------------|----------|
| 5.1 | Create movement event endpoint with validation | Must Have |
| 5.2 | Batch movement event upload (max 100) | Must Have |
| 5.3 | Movement event retrieval by device | Must Have |
| 5.4 | Movement event retrieval by trip | Must Have |
| 5.5 | Movement event database schema with PostGIS | Must Have |
| 5.6 | Transportation mode and confidence storage | Must Have |

### Epic 6: Trip Lifecycle Management
Complete trip creation, state management, and statistics calculation.

| Story | Description | Priority |
|-------|-------------|----------|
| 6.1 | Create trip endpoint with localTripId idempotency | Must Have |
| 6.2 | Update trip state (ACTIVE/COMPLETED/CANCELLED) | Must Have |
| 6.3 | Trip completion with statistics calculation | Must Have |
| 6.4 | Trip retrieval by device with pagination | Must Have |
| 6.5 | Trip retrieval by date range | Should Have |
| 6.6 | Trip database schema and indexes | Must Have |

### Epic 7: Enhanced Location Context
Extend existing location tracking with transportation and trip context.

| Story | Description | Priority |
|-------|-------------|----------|
| 7.1 | Add transportation mode to location uploads | Must Have |
| 7.2 | Add detection source to location uploads | Must Have |
| 7.3 | Optional tripId association for locations | Should Have |
| 7.4 | Location schema migration for new fields | Must Have |

### Epic 8: Intelligent Path Detection
Map-snapping integration and path correction capabilities.

| Story | Description | Priority |
|-------|-------------|----------|
| 8.1 | Map-snapping service integration (OSRM/Valhalla) | Should Have |
| 8.2 | Path correction storage (original + corrected) | Should Have |
| 8.3 | Automatic path correction on trip completion | Should Have |
| 8.4 | On-demand path correction endpoint | Could Have |
| 8.5 | Correction quality/confidence metrics | Could Have |
| 8.6 | Fallback handling when service unavailable | Should Have |

---

## 10. Out of Scope

| Item | Rationale |
|------|-----------|
| Android client implementation | Developed by parallel agent in separate project |
| iOS client | Android-first strategy |
| Real-time trip sharing | Future enhancement |
| Trip predictions/suggestions | Future ML enhancement |
| Carbon footprint calculation | Future enhancement |
| Social trip sharing | Future enhancement |
| Voice commands | Future enhancement |
| Wearable integration | Future enhancement |

---

## 11. Assumptions and Dependencies

### Assumptions

| ID | Assumption |
|----|------------|
| A1 | Client performs trip detection; backend is storage-focused |
| A2 | PostGIS extension available in production PostgreSQL |
| A3 | External map-snapping service (OSRM/Valhalla) available or deployable |
| A4 | Client handles GPS permission and battery optimization |
| A5 | Existing API key authentication sufficient for new endpoints |
| A6 | Device ownership validation patterns from existing codebase apply |

### Dependencies

| ID | Dependency | Type | Risk |
|----|------------|------|------|
| D1 | PostgreSQL PostGIS extension | Infrastructure | Low |
| D2 | OSRM or Valhalla map-matching service | External Service | Medium |
| D3 | Android client trip detection | External Team | Medium |
| D4 | Existing device/location infrastructure | Internal | Low |

### Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Map-snapping service latency | User experience | Async processing, queue-based correction |
| Large trip data volumes | Storage costs | Data retention policies, archival strategy |
| Client-server time sync | Data accuracy | Server-side timestamp validation |
| Battery drain concerns | User adoption | Optimized batch sync, adaptive collection |

---

## 12. Next Steps

1. **Architecture Design**: Route to 3-solutioning workflow for architecture.md
   - PostGIS integration patterns
   - Map-snapping service selection (OSRM vs Valhalla vs Google Roads API)
   - Background job processing for path correction
   - Event-driven architecture considerations

2. **Epic Detailing**: Expand epic stories in epics.md with:
   - Acceptance criteria
   - Technical specifications
   - Test scenarios
   - Story point estimates

3. **API Design Review**: Validate BACKEND_API_SPEC.md alignment with PRD requirements

4. **Client Coordination**: Sync with Android team on:
   - API contract finalization
   - Error handling patterns
   - Sync protocol details

---

## Appendix A: Data Models Reference

### MovementEvent
```
- id: UUID
- deviceId: UUID
- tripId: UUID (optional)
- timestamp: i64 (milliseconds)
- latitude: f64
- longitude: f64
- accuracy: f32 (meters)
- speed: f32 (m/s, optional)
- bearing: f32 (degrees, optional)
- altitude: f64 (meters, optional)
- transportationMode: enum
- confidence: f32 (0.0-1.0)
- detectionSource: enum
- createdAt: timestamp
```

### Trip
```
- id: UUID
- deviceId: UUID
- localTripId: string (client-generated)
- state: enum (ACTIVE, COMPLETED, CANCELLED)
- startTimestamp: i64
- endTimestamp: i64 (optional)
- startLatitude: f64
- startLongitude: f64
- endLatitude: f64 (optional)
- endLongitude: f64 (optional)
- transportationMode: enum (primary)
- detectionSource: enum
- distanceMeters: f64 (optional)
- durationSeconds: i64 (optional)
- createdAt: timestamp
- updatedAt: timestamp
```

### Transportation Modes
- STATIONARY
- WALKING
- RUNNING
- CYCLING
- IN_VEHICLE
- UNKNOWN

### Detection Sources
- ACTIVITY_RECOGNITION
- BLUETOOTH_CAR
- ANDROID_AUTO
- MULTIPLE
- NONE

---

*Document generated as part of BMAD workflow. Reference: BACKEND_API_SPEC.md, ANDROID_APP_SPEC.md*
