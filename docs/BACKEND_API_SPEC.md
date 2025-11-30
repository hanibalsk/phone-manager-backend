# Backend API Specification - Movement Tracking & Intelligent Path Detection

**Version:** 1.0.0
**Date:** 2025-11-30
**Status:** Draft

---

## Table of Contents

1. [Overview](#1-overview)
2. [Authentication](#2-authentication)
3. [Data Models](#3-data-models)
4. [API Endpoints](#4-api-endpoints)
5. [Backend Processing Requirements](#5-backend-processing-requirements)
6. [Database Schema](#6-database-schema)
7. [Error Handling](#7-error-handling)
8. [Rate Limiting](#8-rate-limiting)

---

## 1. Overview

The backend must support:
- Movement event ingestion with full sensor telemetry
- Trip lifecycle management (create, update, complete)
- Location data with transportation mode context
- Path correction (map-snapping) and returning corrected coordinates to the app

### Base URL
```
https://api.example.com/api/v1
```

### Supported Content Types
- Request: `application/json`
- Response: `application/json`

---

## 2. Authentication

All endpoints require authentication via API key.

### Headers
```
X-API-Key: {apiKey}
X-Device-ID: {deviceId}
Content-Type: application/json
```

### Error Response (401 Unauthorized)
```json
{
  "error": "UNAUTHORIZED",
  "message": "Invalid or missing API key"
}
```

---

## 3. Data Models

### 3.1 MovementEvent

Movement events are sent from the app when the user's transportation mode changes.

```json
{
  "eventId": "550e8400-e29b-41d4-a716-446655440000",
  "deviceId": "device-uuid-123",
  "timestamp": "2025-11-30T10:30:00.000Z",
  "previousMode": "STATIONARY",
  "newMode": "IN_VEHICLE",
  "detectionSource": {
    "primary": "BLUETOOTH_CAR",
    "contributing": ["ACTIVITY_RECOGNITION", "BLUETOOTH_CAR"]
  },
  "confidence": 0.85,
  "detectionLatencyMs": 250,
  "location": {
    "latitude": 48.1234567,
    "longitude": 17.5678901,
    "accuracy": 10.5,
    "speed": 5.2
  },
  "deviceState": {
    "batteryLevel": 75,
    "batteryCharging": false,
    "networkType": "WIFI",
    "networkStrength": -65
  },
  "telemetry": {
    "accelerometer": {
      "magnitude": 9.81,
      "variance": 0.15,
      "peakFrequency": 2.1
    },
    "gyroscope": {
      "magnitude": 0.05
    },
    "stepCount": 1234,
    "significantMotion": true,
    "activityRecognition": {
      "type": "WALKING",
      "confidence": 92
    }
  },
  "tripId": "trip-uuid-456"
}
```

#### Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `eventId` | UUID | Yes | Client-generated unique identifier |
| `deviceId` | string | Yes | Device identifier |
| `timestamp` | ISO 8601 | Yes | Event timestamp |
| `previousMode` | enum | Yes | Previous transportation mode |
| `newMode` | enum | Yes | New transportation mode |
| `detectionSource` | object | Yes | How the mode was detected |
| `confidence` | float | Yes | Detection confidence (0.0-1.0) |
| `detectionLatencyMs` | integer | Yes | Time from sensor event to detection |
| `location` | object | No | Location at time of event |
| `deviceState` | object | No | Device state at time of event |
| `telemetry` | object | No | Sensor telemetry data |
| `tripId` | UUID | No | Associated trip ID if within a trip |

#### Transportation Mode Enum
```
STATIONARY | WALKING | RUNNING | CYCLING | IN_VEHICLE | UNKNOWN
```

#### Detection Source Enum
```
ACTIVITY_RECOGNITION | BLUETOOTH_CAR | ANDROID_AUTO | MULTIPLE | NONE
```

---

### 3.2 Trip

Trips represent journeys from one location to another.

```json
{
  "tripId": "server-generated-uuid",
  "localTripId": "client-generated-uuid",
  "deviceId": "device-uuid-123",
  "startTime": "2025-11-30T10:00:00.000Z",
  "endTime": "2025-11-30T10:45:00.000Z",
  "status": "COMPLETED",
  "startLocation": {
    "latitude": 48.1234567,
    "longitude": 17.5678901
  },
  "endLocation": {
    "latitude": 48.2345678,
    "longitude": 17.6789012
  },
  "statistics": {
    "distanceMeters": 5432.5,
    "durationSeconds": 2700,
    "locationCount": 54,
    "movementEventCount": 8
  },
  "modes": {
    "dominant": "IN_VEHICLE",
    "breakdown": {
      "IN_VEHICLE": 2400,
      "WALKING": 300
    }
  },
  "triggers": {
    "start": "MODE_CHANGE",
    "end": "STATIONARY"
  },
  "createdAt": "2025-11-30T10:00:00.000Z",
  "updatedAt": "2025-11-30T10:45:00.000Z"
}
```

#### Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tripId` | UUID | Response only | Server-assigned trip ID |
| `localTripId` | UUID | Yes | Client-generated ID for idempotency |
| `deviceId` | string | Yes | Device identifier |
| `startTime` | ISO 8601 | Yes | Trip start timestamp |
| `endTime` | ISO 8601 | No | Trip end timestamp (null if active) |
| `status` | enum | Yes | Trip status |
| `startLocation` | object | No | Starting coordinates |
| `endLocation` | object | No | Ending coordinates |
| `statistics` | object | No | Trip statistics |
| `modes` | object | No | Transportation mode breakdown |
| `triggers` | object | No | What triggered start/end |

#### Trip Status Enum
```
ACTIVE | COMPLETED | CANCELLED
```

#### Trip Trigger Enum
```
MODE_CHANGE | TIME | DISTANCE | STATIONARY | MANUAL
```

---

### 3.3 LocationPayload (Enhanced)

Location data with transportation mode context.

```json
{
  "deviceId": "device-uuid-123",
  "timestamp": "2025-11-30T10:30:00.000Z",
  "latitude": 48.1234567,
  "longitude": 17.5678901,
  "accuracy": 10.5,
  "altitude": 150.0,
  "bearing": 45.5,
  "speed": 12.3,
  "provider": "fused",
  "batteryLevel": 75,
  "networkType": "WIFI",
  "transportationMode": "IN_VEHICLE",
  "detectionSource": "BLUETOOTH_CAR",
  "modeConfidence": 0.92,
  "tripId": "trip-uuid-456"
}
```

#### New Fields (Added to existing payload)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `transportationMode` | enum | No | Current transportation mode |
| `detectionSource` | enum | No | How mode was detected |
| `modeConfidence` | float | No | Mode detection confidence (0.0-1.0) |
| `tripId` | UUID | No | Associated trip ID |

---

### 3.4 LocationCorrection

Corrections sent from backend to app.

```json
{
  "originalTimestamp": 1701340200000,
  "correctedLatitude": 48.1235000,
  "correctedLongitude": 17.5679000,
  "correctionSource": "ROAD_SNAP",
  "correctionConfidence": 0.95,
  "roadName": "Main Street",
  "roadType": "primary"
}
```

#### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `originalTimestamp` | long | Unix timestamp (ms) of original location |
| `correctedLatitude` | double | Corrected latitude coordinate |
| `correctedLongitude` | double | Corrected longitude coordinate |
| `correctionSource` | enum | Algorithm used for correction |
| `correctionConfidence` | float | Confidence in correction (0.0-1.0) |
| `roadName` | string | Name of the road (if available) |
| `roadType` | string | Type of road (primary, secondary, etc.) |

#### Correction Source Enum
```
ROAD_SNAP | INTERPOLATION | MANUAL | ALGORITHM
```

---

## 4. API Endpoints

### 4.1 Movement Events API

#### Create Movement Event
```http
POST /movement-events
```

**Request Body:** `MovementEvent` object

**Response (201 Created):**
```json
{
  "eventId": "550e8400-e29b-41d4-a716-446655440000",
  "processedAt": "2025-11-30T10:30:01.000Z"
}
```

---

#### Create Movement Events (Batch)
```http
POST /movement-events/batch
```

**Request Body:**
```json
{
  "events": [
    { /* MovementEvent */ },
    { /* MovementEvent */ }
  ]
}
```

**Constraints:**
- Maximum 100 events per batch

**Response (200 OK):**
```json
{
  "processedCount": 50,
  "failedCount": 2,
  "errors": [
    {
      "eventId": "uuid",
      "error": "INVALID_TIMESTAMP",
      "message": "Timestamp is in the future"
    }
  ]
}
```

---

#### List Movement Events
```http
GET /movement-events?deviceId={deviceId}&from={timestamp}&to={timestamp}&limit={limit}&offset={offset}
```

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `deviceId` | string | Yes | - | Device identifier |
| `from` | ISO 8601 | No | 7 days ago | Start timestamp |
| `to` | ISO 8601 | No | now | End timestamp |
| `limit` | integer | No | 50 | Max results (1-100) |
| `offset` | integer | No | 0 | Pagination offset |

**Response (200 OK):**
```json
{
  "events": [ /* MovementEvent[] */ ],
  "pagination": {
    "total": 150,
    "limit": 50,
    "offset": 0,
    "hasMore": true
  }
}
```

---

#### Get Movement Event
```http
GET /movement-events/{eventId}
```

**Response (200 OK):** `MovementEvent` object

---

### 4.2 Trips API

#### Create Trip
```http
POST /trips
```

**Request Body:**
```json
{
  "localTripId": "client-uuid",
  "deviceId": "device-uuid",
  "startTime": "2025-11-30T10:00:00.000Z",
  "status": "ACTIVE",
  "startLocation": {
    "latitude": 48.1234567,
    "longitude": 17.5678901
  },
  "modes": {
    "dominant": "IN_VEHICLE"
  },
  "triggers": {
    "start": "MODE_CHANGE"
  }
}
```

**Response (201 Created):**
```json
{
  "tripId": "server-uuid",
  "localTripId": "client-uuid",
  "createdAt": "2025-11-30T10:00:00.000Z"
}
```

**Notes:**
- `localTripId` ensures idempotency - retrying with same `localTripId` returns existing trip

---

#### Update Trip
```http
PATCH /trips/{tripId}
```

**Request Body (partial update):**
```json
{
  "endTime": "2025-11-30T10:45:00.000Z",
  "status": "COMPLETED",
  "endLocation": {
    "latitude": 48.2345678,
    "longitude": 17.6789012
  },
  "statistics": {
    "distanceMeters": 5432.5,
    "locationCount": 54
  },
  "triggers": {
    "end": "STATIONARY"
  }
}
```

**Response (200 OK):** Updated `Trip` object

---

#### List Trips
```http
GET /trips?deviceId={deviceId}&status={status}&from={timestamp}&to={timestamp}
```

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `deviceId` | string | Yes | - | Device identifier |
| `status` | enum | No | all | Filter by status |
| `from` | ISO 8601 | No | 30 days ago | Start timestamp |
| `to` | ISO 8601 | No | now | End timestamp |
| `limit` | integer | No | 20 | Max results (1-100) |

**Response (200 OK):**
```json
{
  "trips": [ /* Trip[] */ ],
  "total": 45
}
```

---

#### Get Trip
```http
GET /trips/{tripId}
```

**Response (200 OK):** `Trip` object with full details

---

#### Get Trip Locations
```http
GET /trips/{tripId}/locations
```

**Response (200 OK):**
```json
{
  "tripId": "trip-uuid",
  "locations": [
    {
      "timestamp": "2025-11-30T10:00:00.000Z",
      "latitude": 48.1234567,
      "longitude": 17.5678901,
      "accuracy": 10.5,
      "speed": 12.3,
      "transportationMode": "IN_VEHICLE"
    }
  ],
  "count": 54
}
```

---

#### Get Trip Path (Corrected)
```http
GET /trips/{tripId}/path
```

**Response (200 OK):**
```json
{
  "tripId": "trip-uuid",
  "path": [
    [48.1234567, 17.5678901],
    [48.1240000, 17.5685000],
    [48.1250000, 17.5695000]
  ],
  "corrected": true,
  "algorithm": "map_match",
  "totalPoints": 54,
  "correctedPoints": 48
}
```

---

#### Delete Trip
```http
DELETE /trips/{tripId}
```

**Response (204 No Content)**

---

### 4.3 Locations API (Enhanced)

#### Upload Location Batch (Enhanced Response)
```http
POST /locations/batch
```

**Request Body:**
```json
{
  "deviceId": "device-uuid",
  "locations": [
    { /* LocationPayload with new fields */ }
  ]
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "processedCount": 50,
  "corrections": [
    {
      "originalTimestamp": 1701340200000,
      "correctedLatitude": 48.1235000,
      "correctedLongitude": 17.5679000,
      "correctionSource": "ROAD_SNAP",
      "correctionConfidence": 0.95
    }
  ]
}
```

**Notes:**
- Corrections are returned for previously uploaded locations that have been processed
- App should store corrections locally for display

---

#### Get Pending Corrections
```http
GET /locations/corrections?deviceId={deviceId}&since={timestamp}
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `deviceId` | string | Yes | Device identifier |
| `since` | long | No | Unix timestamp (ms) - get corrections since |

**Response (200 OK):**
```json
{
  "corrections": [ /* LocationCorrection[] */ ],
  "count": 25,
  "lastProcessedAt": "2025-11-30T10:30:00.000Z"
}
```

---

## 5. Backend Processing Requirements

### 5.1 Path Correction (Map-Snapping)

**Purpose:** Convert raw GPS coordinates to road-aligned coordinates for accurate route display.

**Input:**
- Raw GPS coordinates with timestamps
- Transportation mode at each point

**Output:**
- Coordinates snapped to road/path network

**Algorithm Requirements:**

1. **Transportation Mode Awareness:**
   - `IN_VEHICLE`: Snap to roads (primary, secondary, tertiary)
   - `WALKING`: Snap to sidewalks, footpaths, pedestrian areas
   - `CYCLING`: Snap to bike paths, roads with bike lanes
   - `RUNNING`: Same as walking

2. **Temporal Constraints:**
   - Points must be reachable given time difference and reasonable speed
   - Maximum speed thresholds per mode:
     - WALKING: 7 km/h
     - RUNNING: 20 km/h
     - CYCLING: 40 km/h
     - IN_VEHICLE: 150 km/h

3. **Gap Handling:**
   - Interpolate missing segments when GPS gaps exist
   - Mark interpolated points with lower confidence

**Suggested Implementation:**
- OSRM Map Matching API (open source)
- Google Roads API (paid)
- Valhalla Map Matching (open source)

**Processing Trigger:**
- Process in batches when trip completes
- Or process periodically for active trips

---

### 5.2 Intelligent Path Detection (Future)

**Purpose:** Detect common routes and predict destinations.

**Data Requirements from App:**
- All location points with timestamps
- Transportation mode at each point
- Trip boundaries (start/end)
- Movement events (mode changes)

**Backend Processing:**

1. **Frequent Location Clustering:**
   - Cluster trip start/end points
   - Identify: Home, Work, Gym, etc.
   - Use DBSCAN or similar clustering algorithm

2. **Route Pattern Detection:**
   - Identify common routes between frequent locations
   - Store as representative polylines
   - Track frequency and time-of-day patterns

3. **Deviation Detection:**
   - Compare current trip to known routes
   - Alert on significant deviations
   - Threshold: >500m from expected route

4. **ETA Prediction:**
   - Use historical trip data
   - Factor in time of day, day of week
   - Consider traffic patterns (if available)

---

## 6. Database Schema

### 6.1 PostgreSQL with PostGIS

```sql
-- Enable PostGIS extension
CREATE EXTENSION IF NOT EXISTS postgis;

-- Movement Events Table
CREATE TABLE movement_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id VARCHAR(255) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    previous_mode VARCHAR(50) NOT NULL,
    new_mode VARCHAR(50) NOT NULL,
    detection_source JSONB NOT NULL,
    confidence DECIMAL(3,2),
    detection_latency_ms INTEGER,
    location GEOGRAPHY(POINT, 4326),
    device_state JSONB,
    telemetry JSONB,
    trip_id UUID REFERENCES trips(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_confidence CHECK (confidence >= 0 AND confidence <= 1)
);

-- Trips Table
CREATE TABLE trips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    local_trip_id UUID NOT NULL,
    device_id VARCHAR(255) NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    start_location GEOGRAPHY(POINT, 4326),
    end_location GEOGRAPHY(POINT, 4326),
    distance_meters DECIMAL(10,2),
    duration_seconds INTEGER,
    location_count INTEGER DEFAULT 0,
    movement_event_count INTEGER DEFAULT 0,
    dominant_mode VARCHAR(50),
    mode_breakdown JSONB,
    start_trigger VARCHAR(50),
    end_trigger VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_device_local_trip UNIQUE(device_id, local_trip_id),
    CONSTRAINT valid_status CHECK (status IN ('ACTIVE', 'COMPLETED', 'CANCELLED'))
);

-- Locations Table (Enhanced)
-- Add columns to existing locations table
ALTER TABLE locations
    ADD COLUMN IF NOT EXISTS transportation_mode VARCHAR(50),
    ADD COLUMN IF NOT EXISTS detection_source VARCHAR(50),
    ADD COLUMN IF NOT EXISTS mode_confidence DECIMAL(3,2),
    ADD COLUMN IF NOT EXISTS trip_id UUID REFERENCES trips(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS corrected_latitude DECIMAL(10,7),
    ADD COLUMN IF NOT EXISTS corrected_longitude DECIMAL(10,7),
    ADD COLUMN IF NOT EXISTS correction_source VARCHAR(50),
    ADD COLUMN IF NOT EXISTS corrected_at TIMESTAMPTZ;

-- Frequent Locations Table (for intelligent path detection)
CREATE TABLE frequent_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    label VARCHAR(50), -- HOME, WORK, GYM, etc.
    center GEOGRAPHY(POINT, 4326) NOT NULL,
    radius_meters INTEGER NOT NULL DEFAULT 100,
    visit_count INTEGER DEFAULT 0,
    last_visit TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Common Routes Table (for intelligent path detection)
CREATE TABLE common_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id VARCHAR(255) NOT NULL,
    start_location_id UUID REFERENCES frequent_locations(id),
    end_location_id UUID REFERENCES frequent_locations(id),
    representative_path GEOGRAPHY(LINESTRING, 4326),
    trip_count INTEGER DEFAULT 0,
    average_duration_seconds INTEGER,
    typical_departure_times JSONB, -- Array of hour ranges
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_movement_events_device_time
    ON movement_events(device_id, timestamp DESC);
CREATE INDEX idx_movement_events_trip
    ON movement_events(trip_id) WHERE trip_id IS NOT NULL;

CREATE INDEX idx_trips_device_status
    ON trips(device_id, status);
CREATE INDEX idx_trips_device_time
    ON trips(device_id, start_time DESC);

CREATE INDEX idx_locations_trip
    ON locations(trip_id) WHERE trip_id IS NOT NULL;
CREATE INDEX idx_locations_device_time
    ON locations(device_id, captured_at DESC);
CREATE INDEX idx_locations_needs_correction
    ON locations(device_id, captured_at)
    WHERE corrected_latitude IS NULL AND trip_id IS NOT NULL;

CREATE INDEX idx_frequent_locations_device
    ON frequent_locations(device_id);
CREATE INDEX idx_frequent_locations_geo
    ON frequent_locations USING GIST(center);

CREATE INDEX idx_common_routes_device
    ON common_routes(device_id);
CREATE INDEX idx_common_routes_endpoints
    ON common_routes(start_location_id, end_location_id);
```

---

## 7. Error Handling

### 7.1 Error Response Format

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error message",
  "details": {
    "field": "Additional context"
  },
  "timestamp": "2025-11-30T10:30:00.000Z",
  "requestId": "req-uuid"
}
```

### 7.2 Error Codes

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | `INVALID_REQUEST` | Malformed request body |
| 400 | `INVALID_TIMESTAMP` | Timestamp in future or too old |
| 400 | `INVALID_COORDINATES` | Coordinates out of valid range |
| 400 | `BATCH_TOO_LARGE` | Batch exceeds maximum size |
| 401 | `UNAUTHORIZED` | Invalid or missing API key |
| 403 | `FORBIDDEN` | Access denied to resource |
| 404 | `NOT_FOUND` | Resource not found |
| 409 | `CONFLICT` | Duplicate resource (idempotency) |
| 422 | `VALIDATION_ERROR` | Business rule validation failed |
| 429 | `RATE_LIMITED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Server error |
| 503 | `SERVICE_UNAVAILABLE` | Service temporarily unavailable |

---

## 8. Rate Limiting

### 8.1 Limits by Endpoint

| Endpoint | Rate Limit | Window |
|----------|------------|--------|
| `POST /movement-events` | 60/min | Per device |
| `POST /movement-events/batch` | 10/min | Per device |
| `POST /trips` | 30/min | Per device |
| `PATCH /trips/{id}` | 60/min | Per device |
| `POST /locations/batch` | 20/min | Per device |
| `GET *` | 100/min | Per device |

### 8.2 Rate Limit Headers

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1701340260
```

### 8.3 Rate Limited Response (429)

```json
{
  "error": "RATE_LIMITED",
  "message": "Too many requests. Please retry after 60 seconds.",
  "retryAfter": 60
}
```

---

## Appendix A: Webhook Events (Future)

For real-time notifications, the backend may support webhooks:

| Event | Description |
|-------|-------------|
| `trip.started` | New trip detected |
| `trip.completed` | Trip finished |
| `trip.deviation` | Route deviation detected |
| `location.corrected` | Batch of locations corrected |
| `geofence.entered` | Device entered geofence |
| `geofence.exited` | Device exited geofence |

---

## Appendix B: Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-30 | Initial specification |
