# Android App Specification - Movement Tracking & Intelligent Path Detection

**Version:** 1.0.0
**Date:** 2025-11-30
**Status:** Draft
**Platform:** Android (Kotlin)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Database Schema](#2-database-schema)
3. [Trip Auto-Detection System](#3-trip-auto-detection-system)
4. [Movement Event Logging](#4-movement-event-logging)
5. [Location Enhancement](#5-location-enhancement)
6. [UI Screens](#6-ui-screens)
7. [ViewModels](#7-viewmodels)
8. [File Structure](#8-file-structure)
9. [Implementation Priority](#9-implementation-priority)
10. [Testing Requirements](#10-testing-requirements)

---

## 1. Overview

### 1.1 Features

The Android app must implement:
- Movement state logging with full sensor telemetry
- Automatic trip detection and lifecycle management
- Enhanced location capture with transportation mode
- Local storage for offline operation
- Backend sync preparation (correction ingestion ready)

### 1.2 Architecture

- **UI Layer:** Jetpack Compose
- **State Management:** ViewModel + StateFlow
- **Dependency Injection:** Hilt
- **Database:** Room (SQLite)
- **Networking:** Retrofit + Kotlin Serialization
- **Background:** Foreground Service

### 1.3 Permissions Required

```xml
<!-- Existing -->
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE_LOCATION" />

<!-- Movement Detection -->
<uses-permission android:name="android.permission.ACTIVITY_RECOGNITION" />
<uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />

<!-- Sensor Telemetry -->
<uses-permission android:name="android.permission.HIGH_SAMPLING_RATE_SENSORS" />
```

---

## 2. Database Schema

### 2.1 Database Version

Current: **7** → New: **8**

### 2.2 TripEntity

**File:** `data/model/TripEntity.kt`

```kotlin
@Entity(
    tableName = "trips",
    indices = [
        Index("startTime"),
        Index("state"),
        Index("isSynced")
    ]
)
data class TripEntity(
    @PrimaryKey
    val id: String,  // UUID generated client-side

    // State
    val state: String,  // TripState: IDLE, ACTIVE, PENDING_END, COMPLETED

    // Timing
    val startTime: Long,  // Unix timestamp (ms)
    val endTime: Long?,   // Null if active

    // Start Location
    val startLatitude: Double,
    val startLongitude: Double,

    // End Location
    val endLatitude: Double?,
    val endLongitude: Double?,

    // Statistics
    val totalDistanceMeters: Double = 0.0,
    val locationCount: Int = 0,

    // Transportation Modes
    val dominantMode: String,  // TransportationMode.name
    val modesUsedJson: String,  // JSON array: ["WALKING", "IN_VEHICLE"]
    val modeBreakdownJson: String,  // JSON object: {"WALKING": 300000, "IN_VEHICLE": 2400000} (ms)

    // Triggers
    val startTrigger: String,  // TripTrigger: MODE_CHANGE, TIME, DISTANCE, MANUAL
    val endTrigger: String?,

    // Sync Status
    val isSynced: Boolean = false,
    val syncedAt: Long? = null,
    val serverId: String? = null,  // Server-assigned ID

    // Timestamps
    val createdAt: Long = System.currentTimeMillis(),
    val updatedAt: Long = System.currentTimeMillis(),
)
```

### 2.3 MovementEventEntity

**File:** `data/model/MovementEventEntity.kt`

```kotlin
@Entity(
    tableName = "movement_events",
    indices = [
        Index("timestamp"),
        Index("tripId"),
        Index("isSynced")
    ],
    foreignKeys = [
        ForeignKey(
            entity = TripEntity::class,
            parentColumns = ["id"],
            childColumns = ["tripId"],
            onDelete = ForeignKey.SET_NULL
        )
    ]
)
data class MovementEventEntity(
    @PrimaryKey(autoGenerate = true)
    val id: Long = 0,

    // Event Identification
    val timestamp: Long,  // Unix timestamp (ms)
    val tripId: String?,  // Associated trip ID

    // Mode Transition
    val previousMode: String,  // TransportationMode.name
    val newMode: String,       // TransportationMode.name
    val detectionSource: String,  // DetectionSource.name
    val confidence: Float,  // 0.0 - 1.0
    val detectionLatencyMs: Long,  // Time from sensor to detection

    // Location Snapshot
    val latitude: Double? = null,
    val longitude: Double? = null,
    val accuracy: Float? = null,
    val speed: Float? = null,

    // Device State
    val batteryLevel: Int? = null,  // 0-100
    val batteryCharging: Boolean? = null,
    val networkType: String? = null,  // WIFI, MOBILE, NONE
    val networkStrength: Int? = null,  // dBm

    // Sensor Telemetry
    val accelerometerMagnitude: Float? = null,
    val accelerometerVariance: Float? = null,
    val accelerometerPeakFrequency: Float? = null,
    val gyroscopeMagnitude: Float? = null,
    val stepCount: Int? = null,
    val significantMotion: Boolean? = null,
    val activityType: String? = null,  // Raw activity type
    val activityConfidence: Int? = null,  // 0-100

    // Distance Tracking
    val distanceFromLastLocation: Float? = null,
    val timeSinceLastLocation: Long? = null,

    // Sync Status
    val isSynced: Boolean = false,
    val syncedAt: Long? = null,
)
```

### 2.4 LocationEntity Enhancements

**File:** `data/model/LocationEntity.kt`

Add the following fields to the existing `LocationEntity`:

```kotlin
@Entity(
    tableName = "locations",
    indices = [
        // ... existing indexes ...
        Index("tripId"),
        Index("transportationMode")
    ]
)
data class LocationEntity(
    // ... existing fields ...

    // NEW: Transportation Mode Context
    val transportationMode: String? = null,  // TransportationMode.name
    val detectionSource: String? = null,     // DetectionSource.name
    val modeConfidence: Float? = null,       // 0.0 - 1.0

    // NEW: Trip Association
    val tripId: String? = null,  // Associated trip ID

    // NEW: Backend Corrections
    val correctedLatitude: Double? = null,
    val correctedLongitude: Double? = null,
    val correctionSource: String? = null,  // ROAD_SNAP, INTERPOLATION, etc.
    val correctedAt: Long? = null,
)
```

### 2.5 Migration SQL

**File:** `data/database/AppDatabase.kt`

```kotlin
val MIGRATION_7_8 = object : Migration(7, 8) {
    override fun migrate(db: SupportSQLiteDatabase) {
        // 1. Create trips table
        db.execSQL("""
            CREATE TABLE IF NOT EXISTS trips (
                id TEXT PRIMARY KEY NOT NULL,
                state TEXT NOT NULL,
                startTime INTEGER NOT NULL,
                endTime INTEGER,
                startLatitude REAL NOT NULL,
                startLongitude REAL NOT NULL,
                endLatitude REAL,
                endLongitude REAL,
                totalDistanceMeters REAL NOT NULL DEFAULT 0,
                locationCount INTEGER NOT NULL DEFAULT 0,
                dominantMode TEXT NOT NULL,
                modesUsedJson TEXT NOT NULL,
                modeBreakdownJson TEXT NOT NULL,
                startTrigger TEXT NOT NULL,
                endTrigger TEXT,
                isSynced INTEGER NOT NULL DEFAULT 0,
                syncedAt INTEGER,
                serverId TEXT,
                createdAt INTEGER NOT NULL,
                updatedAt INTEGER NOT NULL
            )
        """)

        // 2. Create trips indexes
        db.execSQL("CREATE INDEX IF NOT EXISTS index_trips_startTime ON trips(startTime)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_trips_state ON trips(state)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_trips_isSynced ON trips(isSynced)")

        // 3. Create movement_events table
        db.execSQL("""
            CREATE TABLE IF NOT EXISTS movement_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                timestamp INTEGER NOT NULL,
                tripId TEXT,
                previousMode TEXT NOT NULL,
                newMode TEXT NOT NULL,
                detectionSource TEXT NOT NULL,
                confidence REAL NOT NULL,
                detectionLatencyMs INTEGER NOT NULL,
                latitude REAL,
                longitude REAL,
                accuracy REAL,
                speed REAL,
                batteryLevel INTEGER,
                batteryCharging INTEGER,
                networkType TEXT,
                networkStrength INTEGER,
                accelerometerMagnitude REAL,
                accelerometerVariance REAL,
                accelerometerPeakFrequency REAL,
                gyroscopeMagnitude REAL,
                stepCount INTEGER,
                significantMotion INTEGER,
                activityType TEXT,
                activityConfidence INTEGER,
                distanceFromLastLocation REAL,
                timeSinceLastLocation INTEGER,
                isSynced INTEGER NOT NULL DEFAULT 0,
                syncedAt INTEGER,
                FOREIGN KEY (tripId) REFERENCES trips(id) ON DELETE SET NULL
            )
        """)

        // 4. Create movement_events indexes
        db.execSQL("CREATE INDEX IF NOT EXISTS index_movement_events_timestamp ON movement_events(timestamp)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_movement_events_tripId ON movement_events(tripId)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_movement_events_isSynced ON movement_events(isSynced)")

        // 5. Add new columns to locations table
        db.execSQL("ALTER TABLE locations ADD COLUMN transportationMode TEXT")
        db.execSQL("ALTER TABLE locations ADD COLUMN detectionSource TEXT")
        db.execSQL("ALTER TABLE locations ADD COLUMN modeConfidence REAL")
        db.execSQL("ALTER TABLE locations ADD COLUMN tripId TEXT")
        db.execSQL("ALTER TABLE locations ADD COLUMN correctedLatitude REAL")
        db.execSQL("ALTER TABLE locations ADD COLUMN correctedLongitude REAL")
        db.execSQL("ALTER TABLE locations ADD COLUMN correctionSource TEXT")
        db.execSQL("ALTER TABLE locations ADD COLUMN correctedAt INTEGER")

        // 6. Create locations indexes for new columns
        db.execSQL("CREATE INDEX IF NOT EXISTS index_locations_tripId ON locations(tripId)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_locations_transportationMode ON locations(transportationMode)")
    }
}
```

### 2.6 TripDao

**File:** `data/database/TripDao.kt`

```kotlin
@Dao
interface TripDao {
    // Insert/Update
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(trip: TripEntity): Long

    @Update
    suspend fun update(trip: TripEntity)

    // Queries
    @Query("SELECT * FROM trips WHERE id = :tripId")
    suspend fun getTripById(tripId: String): TripEntity?

    @Query("SELECT * FROM trips WHERE id = :tripId")
    fun observeTripById(tripId: String): Flow<TripEntity?>

    @Query("SELECT * FROM trips WHERE state = 'ACTIVE' LIMIT 1")
    suspend fun getActiveTrip(): TripEntity?

    @Query("SELECT * FROM trips WHERE state = 'ACTIVE' LIMIT 1")
    fun observeActiveTrip(): Flow<TripEntity?>

    @Query("SELECT * FROM trips ORDER BY startTime DESC LIMIT :limit")
    fun observeRecentTrips(limit: Int = 20): Flow<List<TripEntity>>

    @Query("SELECT * FROM trips WHERE startTime BETWEEN :startTime AND :endTime ORDER BY startTime DESC")
    suspend fun getTripsBetween(startTime: Long, endTime: Long): List<TripEntity>

    @Query("SELECT * FROM trips WHERE dominantMode = :mode ORDER BY startTime DESC")
    fun observeTripsByMode(mode: String): Flow<List<TripEntity>>

    // Statistics
    @Query("UPDATE trips SET locationCount = locationCount + 1, totalDistanceMeters = totalDistanceMeters + :distance, updatedAt = :timestamp WHERE id = :tripId")
    suspend fun incrementLocationCount(tripId: String, distance: Double, timestamp: Long)

    @Query("SELECT SUM(totalDistanceMeters) FROM trips WHERE startTime >= :since")
    fun observeTotalDistanceSince(since: Long): Flow<Double?>

    @Query("SELECT COUNT(*) FROM trips WHERE startTime >= :since")
    fun observeTripCountSince(since: Long): Flow<Int>

    // Sync
    @Query("SELECT * FROM trips WHERE isSynced = 0 ORDER BY startTime ASC LIMIT :limit")
    suspend fun getUnsyncedTrips(limit: Int = 50): List<TripEntity>

    @Query("UPDATE trips SET isSynced = 1, syncedAt = :syncedAt, serverId = :serverId WHERE id = :tripId")
    suspend fun markAsSynced(tripId: String, syncedAt: Long, serverId: String?)

    // Cleanup
    @Query("DELETE FROM trips WHERE startTime < :beforeTime AND state = 'COMPLETED'")
    suspend fun deleteOldTrips(beforeTime: Long): Int
}
```

### 2.7 MovementEventDao

**File:** `data/database/MovementEventDao.kt`

```kotlin
@Dao
interface MovementEventDao {
    // Insert
    @Insert
    suspend fun insert(event: MovementEventEntity): Long

    @Insert
    suspend fun insertAll(events: List<MovementEventEntity>)

    // Queries
    @Query("SELECT * FROM movement_events WHERE id = :eventId")
    suspend fun getEventById(eventId: Long): MovementEventEntity?

    @Query("SELECT * FROM movement_events ORDER BY timestamp DESC LIMIT :limit")
    fun observeRecentEvents(limit: Int = 50): Flow<List<MovementEventEntity>>

    @Query("SELECT * FROM movement_events WHERE tripId = :tripId ORDER BY timestamp ASC")
    fun observeEventsByTrip(tripId: String): Flow<List<MovementEventEntity>>

    @Query("SELECT * FROM movement_events ORDER BY timestamp DESC LIMIT 1")
    fun observeLatestEvent(): Flow<MovementEventEntity?>

    @Query("SELECT * FROM movement_events WHERE timestamp BETWEEN :startTime AND :endTime ORDER BY timestamp ASC")
    suspend fun getEventsBetween(startTime: Long, endTime: Long): List<MovementEventEntity>

    // Statistics
    @Query("SELECT COUNT(*) FROM movement_events WHERE timestamp >= :since")
    fun observeEventCountSince(since: Long): Flow<Int>

    @Query("SELECT COUNT(*) FROM movement_events WHERE tripId = :tripId")
    suspend fun getEventCountForTrip(tripId: String): Int

    // Sync
    @Query("SELECT * FROM movement_events WHERE isSynced = 0 ORDER BY timestamp ASC LIMIT :limit")
    suspend fun getUnsyncedEvents(limit: Int = 100): List<MovementEventEntity>

    @Query("UPDATE movement_events SET isSynced = 1, syncedAt = :syncedAt WHERE id IN (:ids)")
    suspend fun markAsSynced(ids: List<Long>, syncedAt: Long)

    @Query("SELECT COUNT(*) FROM movement_events WHERE isSynced = 0")
    fun observeUnsyncedCount(): Flow<Int>

    // Cleanup
    @Query("DELETE FROM movement_events WHERE timestamp < :beforeTime")
    suspend fun deleteOldEvents(beforeTime: Long): Int
}
```

---

## 3. Trip Auto-Detection System

### 3.1 State Machine

```
        ┌─────────────────────────────────────────────────────┐
        │                                                     │
        v                                                     │
    ┌──────┐     movement      ┌────────┐                    │
    │ IDLE │ ───────────────> │ ACTIVE │                     │
    └──────┘                   └───┬────┘                     │
        ^                          │                          │
        │                          │ stationary               │
        │                          v                          │
        │                   ┌─────────────┐    movement       │
        │                   │ PENDING_END │ ──────────────────┘
        │                   └──────┬──────┘
        │                          │
        │    threshold exceeded    │
        │                          v
        │                   ┌───────────┐
        └───────────────── │ COMPLETED │
                           └───────────┘
```

### 3.2 TripState Enum

**File:** `trip/TripState.kt`

```kotlin
enum class TripState {
    /** No active trip - waiting for movement to start */
    IDLE,

    /** Trip in progress - user is moving */
    ACTIVE,

    /** User became stationary - trip may end soon */
    PENDING_END,

    /** Trip has been finalized */
    COMPLETED
}
```

### 3.3 TripTrigger Enum

**File:** `trip/TripTrigger.kt`

```kotlin
enum class TripTrigger {
    /** Movement mode changed from stationary */
    MODE_CHANGE,

    /** Significant time elapsed with movement */
    TIME,

    /** Significant distance traveled */
    DISTANCE,

    /** Stationary threshold exceeded */
    STATIONARY,

    /** User manually started/ended trip */
    MANUAL
}
```

### 3.4 TripManager Interface

**File:** `trip/TripManager.kt`

```kotlin
interface TripManager {
    // Observable State
    val currentTripState: StateFlow<TripState>
    val activeTrip: StateFlow<Trip?>

    // Lifecycle
    suspend fun startMonitoring()
    fun stopMonitoring()

    // Manual Controls
    suspend fun forceStartTrip(): Trip
    suspend fun forceEndTrip(): Trip?

    // Queries
    suspend fun getTripById(id: String): Trip?
    suspend fun getTripsInRange(startTime: Long, endTime: Long): List<Trip>
    suspend fun getRecentTrips(limit: Int = 10): List<Trip>
    suspend fun getTripWithLocations(tripId: String): Trip?

    // Backfill
    suspend fun backfillTripsFromHistory(startTime: Long, endTime: Long): List<Trip>
}
```

### 3.5 Detection Algorithm Configuration

**File:** `data/preferences/PreferencesRepository.kt` (additions)

```kotlin
interface PreferencesRepository {
    // ... existing ...

    // Trip Detection Settings
    val isTripDetectionEnabled: Flow<Boolean>
    suspend fun setTripDetectionEnabled(enabled: Boolean)

    val tripStationaryThresholdMinutes: Flow<Int>
    suspend fun setTripStationaryThresholdMinutes(minutes: Int)

    val tripMinimumDurationMinutes: Flow<Int>
    suspend fun setTripMinimumDurationMinutes(minutes: Int)

    val tripMinimumDistanceMeters: Flow<Int>
    suspend fun setTripMinimumDistanceMeters(meters: Int)

    val isTripAutoMergeEnabled: Flow<Boolean>
    suspend fun setTripAutoMergeEnabled(enabled: Boolean)

    val tripVehicleGraceSeconds: Flow<Int>
    suspend fun setTripVehicleGraceSeconds(seconds: Int)

    val tripWalkingGraceSeconds: Flow<Int>
    suspend fun setTripWalkingGraceSeconds(seconds: Int)
}
```

#### Default Values

| Parameter | Key | Default | Range | Description |
|-----------|-----|---------|-------|-------------|
| Trip Detection Enabled | `trip_detection_enabled` | `true` | - | Master toggle |
| Stationary Threshold | `trip_stationary_threshold_minutes` | `5` | 1-30 | Minutes stationary to end trip |
| Minimum Duration | `trip_minimum_duration_minutes` | `2` | 1-10 | Discard shorter trips |
| Minimum Distance | `trip_minimum_distance_meters` | `100` | 50-500 | Discard shorter distances |
| Auto-Merge Enabled | `trip_auto_merge_enabled` | `true` | - | Merge brief stops |
| Vehicle Grace Period | `trip_vehicle_grace_seconds` | `90` | 30-180 | Traffic light tolerance |
| Walking Grace Period | `trip_walking_grace_seconds` | `60` | 30-120 | Brief stop tolerance |

### 3.6 Trip Start Conditions

Trigger new trip when ANY of:

1. `TransportationMode` changes from `STATIONARY`/`UNKNOWN` to movement mode
   - `WALKING`, `RUNNING`, `CYCLING`, `IN_VEHICLE`

2. Location displacement > 50m from last known position AND mode is not `STATIONARY`

**Anti-false-positive measures:**
- Require 2+ consecutive location updates showing movement
- Ignore mode transitions lasting < 30 seconds
- Validate with location displacement > 10m

### 3.7 Trip End Conditions

1. User's mode becomes `STATIONARY`
2. Timer starts (grace period based on previous mode)
3. If movement resumes within grace period → continue trip
4. If threshold exceeded → finalize trip

**Important:** Set `endTime = stationary_start_time` (not current time)

### 3.8 Mode Segment Tracking

**File:** `trip/TripModeSegment.kt`

```kotlin
data class TripModeSegment(
    val mode: TransportationMode,
    val startTime: Long,
    val endTime: Long
) {
    val durationMs: Long
        get() = endTime - startTime
}
```

Calculate dominant mode = mode with highest cumulative duration.

---

## 4. Movement Event Logging

### 4.1 When to Log

Log `MovementEventEntity` on **every** `TransportationState` change emitted by `TransportationModeManager`.

### 4.2 SensorTelemetryCollector

**File:** `trip/SensorTelemetryCollector.kt`

```kotlin
@Singleton
class SensorTelemetryCollector @Inject constructor(
    @ApplicationContext private val context: Context,
) {
    data class TelemetrySnapshot(
        // Accelerometer (5-second window)
        val accelerometerMagnitude: Float?,
        val accelerometerVariance: Float?,
        val accelerometerPeakFrequency: Float?,

        // Gyroscope
        val gyroscopeMagnitude: Float?,

        // Step Counter
        val stepCount: Int?,
        val significantMotion: Boolean?,

        // Device State
        val batteryLevel: Int?,
        val batteryCharging: Boolean?,
        val networkType: String?,
        val networkStrength: Int?,
    )

    /**
     * Collect current sensor telemetry.
     * Should be called at the moment of mode change detection.
     */
    suspend fun collect(): TelemetrySnapshot
}
```

**Data Sources:**

| Data | Source |
|------|--------|
| Accelerometer magnitude/variance | `SensorManager.TYPE_ACCELEROMETER` |
| Accelerometer peak frequency | FFT analysis of 5s window |
| Gyroscope magnitude | `SensorManager.TYPE_GYROSCOPE` |
| Step count | `SensorManager.TYPE_STEP_COUNTER` |
| Significant motion | `SensorManager.TYPE_SIGNIFICANT_MOTION` |
| Battery level | `BatteryManager.BATTERY_PROPERTY_CAPACITY` |
| Battery charging | `BatteryManager.isCharging()` |
| Network type | `ConnectivityManager.activeNetwork` |
| Network strength | `TelephonyManager` / `WifiManager` |

### 4.3 Integration in TransportationModeManager

**File:** `movement/TransportationModeManager.kt` (modifications)

```kotlin
@Singleton
class TransportationModeManager @Inject constructor(
    // ... existing dependencies ...
    private val movementEventRepository: MovementEventRepository,
    private val sensorTelemetryCollector: SensorTelemetryCollector,
    private val tripManager: TripManager,
) {
    private var lastState: TransportationState? = null
    private var lastStateTimestamp: Long = 0L

    init {
        // Subscribe to state changes for event recording
        transportationState
            .distinctUntilChanged { old, new -> old.mode == new.mode }
            .onEach { newState ->
                val previousState = lastState
                val previousTimestamp = lastStateTimestamp

                lastState = newState
                lastStateTimestamp = System.currentTimeMillis()

                if (previousState != null && previousState.mode != newState.mode) {
                    recordMovementEvent(previousState, newState, previousTimestamp)
                }
            }
            .launchIn(managerScope)
    }

    private suspend fun recordMovementEvent(
        previousState: TransportationState,
        newState: TransportationState,
        previousTimestamp: Long,
    ) {
        val telemetry = sensorTelemetryCollector.collect()
        val location = locationManager.getLastKnownLocation()
        val detectionLatency = System.currentTimeMillis() - previousTimestamp

        movementEventRepository.recordEvent(
            previousMode = previousState.mode,
            newMode = newState.mode,
            source = newState.source,
            confidence = newState.confidence,
            detectionLatencyMs = detectionLatency,
            location = location,
            telemetry = telemetry,
            tripId = tripManager.activeTrip.value?.id,
        )
    }
}
```

---

## 5. Location Enhancement

### 5.1 Capture Transportation Mode with Location

**File:** `service/LocationTrackingService.kt` (modifications)

```kotlin
private suspend fun captureLocation() {
    val location = locationManager.getCurrentLocation()
        .getOrNull() ?: return

    val transportState = transportationModeManager.transportationState.value
    val activeTrip = tripManager.activeTrip.value
    val lastLocation = locationRepository.getLastLocation()

    // Calculate distance from last location
    val distance = lastLocation?.let {
        calculateDistance(it.latitude, it.longitude, location.latitude, location.longitude)
    } ?: 0f

    val entity = LocationEntity(
        latitude = location.latitude,
        longitude = location.longitude,
        accuracy = location.accuracy,
        timestamp = location.timestamp,
        altitude = location.altitude,
        bearing = location.bearing,
        speed = location.speed,
        provider = location.provider,
        // NEW: Transportation mode context
        transportationMode = transportState.mode.name,
        detectionSource = transportState.source.name,
        modeConfidence = transportState.confidence,
        tripId = activeTrip?.id,
    )

    val locationId = locationRepository.insertLocation(entity)

    // Update trip statistics if active
    activeTrip?.let { trip ->
        tripRepository.incrementLocationCount(
            tripId = trip.id,
            distance = distance.toDouble(),
            timestamp = System.currentTimeMillis()
        )
    }

    // Queue for sync
    queueManager.enqueueLocation(locationId)
}
```

---

## 6. UI Screens

### 6.1 Trip History Screen

**File:** `ui/triphistory/TripHistoryScreen.kt`

**Navigation:** Settings → Trip History (or HomeScreen card tap)

```
┌─────────────────────────────────────────┐
│ ← Trip History                    🔍 📅 │  ← Search / Filter
├─────────────────────────────────────────┤
│ [Today]                                 │
│ ┌─────────────────────────────────────┐ │
│ │ 🚗 Home → Work           45 min    │ │
│ │    8:15 AM - 9:00 AM    12.5 km    │ │
│ │    ═══════════════════════════     │ │  ← Mini polyline
│ └─────────────────────────────────────┘ │
│ ┌─────────────────────────────────────┐ │
│ │ 🚶 Lunch Walk             15 min   │ │
│ │    12:30 PM - 12:45 PM   0.8 km    │ │
│ └─────────────────────────────────────┘ │
│                                         │
│ [Yesterday]                             │
│ ┌─────────────────────────────────────┐ │
│ │ 🚗 Work → Home           52 min    │ │
│ │    5:30 PM - 6:22 PM    13.1 km    │ │
│ └─────────────────────────────────────┘ │
│                                         │
│           ▼ Load More                   │
└─────────────────────────────────────────┘
```

**Features:**
- Grouped by day
- Filter by date range (calendar picker)
- Filter by transportation mode
- Swipe to delete
- Pull to refresh

**Trip Card Components:**
- Mode icon (🚗 🚶 🏃 🚲)
- Trip name (auto: "Start → End" or user-named)
- Duration and distance
- Start/end times
- Mini route polyline preview

**Tap Action:** Navigate to Trip Detail Screen

---

### 6.2 Trip Detail Screen

**File:** `ui/tripdetail/TripDetailScreen.kt`

```
┌─────────────────────────────────────────┐
│ ← Trip Details                     ✏️   │  ← Edit name
├─────────────────────────────────────────┤
│ ┌─────────────────────────────────────┐ │
│ │                                     │ │
│ │           [MAP VIEW]                │ │
│ │     Route polyline on map          │ │
│ │     Start (🟢) and End (🔴)         │ │
│ │     [Raw] [Corrected] toggle       │ │
│ │                                     │ │
│ └─────────────────────────────────────┘ │
├─────────────────────────────────────────┤
│ 🚗 Home → Work                          │
│ November 30, 2025                       │
├─────────────────────────────────────────┤
│ ⏱️ Duration      │ 📏 Distance          │
│    45 min        │    12.5 km           │
├─────────────────────────────────────────┤
│ 🕐 Start         │ 🕐 End               │
│    8:15 AM       │    9:00 AM           │
│    📍 Home       │    📍 Work           │
├─────────────────────────────────────────┤
│ 📊 Mode Breakdown                       │
│ ┌─────────────────────────────────────┐ │
│ │ 🚗 Driving ████████████████░ 92%   │ │
│ │ 🚶 Walking ██░░░░░░░░░░░░░░░ 8%    │ │
│ └─────────────────────────────────────┘ │
├─────────────────────────────────────────┤
│ 📈 Statistics                           │
│ • Average speed: 25 km/h                │
│ • Location points: 54                   │
│ • Movement events: 3                    │
│ • Path corrected: ✅ Yes                │
├─────────────────────────────────────────┤
│ [View Raw Data]  [Export GPX]  [Delete] │
└─────────────────────────────────────────┘
```

**Map Features:**
- Show route polyline (raw or corrected)
- Toggle between raw (dotted) and corrected (solid) paths
- Start marker (green)
- End marker (red)
- Tap points to see timestamp/mode
- Fit bounds to route

**Actions:**
- Edit trip name
- Export to GPX file
- Delete trip
- Share trip (future)

---

### 6.3 Movement Events Screen

**File:** `ui/movementevents/MovementEventsScreen.kt`

**Navigation:** Settings → Movement Events (Developer/Debug)

```
┌─────────────────────────────────────────┐
│ ← Movement Events                  📤   │  ← Export
├─────────────────────────────────────────┤
│ Live Mode: 🟢 Active               [OFF]│  ← Toggle
├─────────────────────────────────────────┤
│ [10:32:15] STATIONARY → IN_VEHICLE     │
│   Source: BLUETOOTH_CAR (95%)           │
│   📍 48.1234, 17.5678 (±10m)           │
│   🔋 75% │ 📶 WiFi (-65 dBm)           │
│   📊 Accel: 9.81 m/s² (var: 0.15)      │
│   🦶 Steps: 1234                        │
│   ⏱️ Latency: 250ms                     │
├─────────────────────────────────────────┤
│ [10:30:00] WALKING → STATIONARY        │
│   Source: ACTIVITY_RECOGNITION (88%)    │
│   📍 48.1230, 17.5670 (±15m)           │
│   🔋 76% │ 📶 Mobile (-85 dBm)         │
│   📊 Accel: 9.78 m/s² (var: 0.45)      │
├─────────────────────────────────────────┤
│ [10:15:22] STATIONARY → WALKING        │
│   Source: ACTIVITY_RECOGNITION (92%)    │
│   ...                                   │
├─────────────────────────────────────────┤
│           ▼ Load More                   │
└─────────────────────────────────────────┘
```

**Features:**
- Live mode: auto-refresh on new events
- Expandable event details
- Export all events to JSON/CSV
- Clear old events (>7 days)
- Filter by mode transition

**Purpose:** Developer/debug tool to verify movement detection accuracy.

---

### 6.4 Settings Screen Enhancements

**File:** `ui/settings/SettingsScreen.kt` (additions)

Add new section after existing Movement Detection settings:

```
┌─────────────────────────────────────────┐
│ Trip Detection                          │
├─────────────────────────────────────────┤
│ Enable Trip Detection              [ON] │
│ Automatically detect and log trips      │
├─────────────────────────────────────────┤
│ End Trip After Stationary              │
│ ○ 1 min │ ● 5 min │ ○ 10 min │ ○ 30 min│
├─────────────────────────────────────────┤
│ Minimum Trip Duration                   │
│ [2] minutes                        [-][+]│
├─────────────────────────────────────────┤
│ Minimum Trip Distance                   │
│ [100] meters                       [-][+]│
├─────────────────────────────────────────┤
│ Auto-Merge Brief Stops            [ON]  │
│ Combine trips separated by short stops  │
├─────────────────────────────────────────┤
│ [View Trip History →]                   │
│ [View Movement Events →]                │
└─────────────────────────────────────────┘
```

---

### 6.5 HomeScreen Enhancement

**File:** `ui/home/HomeScreen.kt` (additions)

Add trip status card when tracking is active:

**Active Trip:**
```
┌─────────────────────────────────────────┐
│ 🚗 Active Trip                          │
│ Started 8:15 AM • 23 min                │
│ 8.2 km • 15 locations                   │
│ ════════════════════════════════        │  ← Progress indicator
│            [End Trip]                   │
└─────────────────────────────────────────┘
```

**No Active Trip:**
```
┌─────────────────────────────────────────┐
│ 📊 Today's Activity                     │
│ 3 trips • 2.5 hrs moving                │
│ 45.2 km total distance                  │
│            [View History →]             │
└─────────────────────────────────────────┘
```

---

### 6.6 Notification Enhancement

**File:** `service/LocationTrackingService.kt` (notification)

When trip is active, show in tracking notification:

```
📍 Tracking Active
🚗 Trip in progress • 23 min • 8.2 km
Last update: 2 min ago
```

---

## 7. ViewModels

### 7.1 TripHistoryViewModel

**File:** `ui/triphistory/TripHistoryViewModel.kt`

```kotlin
@HiltViewModel
class TripHistoryViewModel @Inject constructor(
    private val tripRepository: TripRepository,
) : ViewModel() {

    // State
    val trips: StateFlow<List<Trip>>
    val isLoading: StateFlow<Boolean>
    val error: StateFlow<String?>

    // Filters
    val selectedDateRange: StateFlow<DateRange?>
    val selectedModeFilter: StateFlow<TransportationMode?>

    // Pagination
    private var currentPage = 0
    private val pageSize = 20

    // Actions
    fun loadTrips(
        dateRange: DateRange? = null,
        mode: TransportationMode? = null
    )

    fun loadMoreTrips()

    fun refreshTrips()

    fun deleteTrip(tripId: String)

    fun setDateRangeFilter(range: DateRange?)

    fun setModeFilter(mode: TransportationMode?)

    fun clearFilters()
}
```

### 7.2 TripDetailViewModel

**File:** `ui/tripdetail/TripDetailViewModel.kt`

```kotlin
@HiltViewModel
class TripDetailViewModel @Inject constructor(
    private val tripRepository: TripRepository,
    private val locationRepository: LocationRepository,
    private val movementEventRepository: MovementEventRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val tripId: String = savedStateHandle["tripId"]!!

    // State
    val trip: StateFlow<Trip?>
    val locations: StateFlow<List<LocationEntity>>
    val movementEvents: StateFlow<List<MovementEvent>>
    val isLoading: StateFlow<Boolean>

    // Map State
    val showCorrectedPath: StateFlow<Boolean>
    val selectedLocationIndex: StateFlow<Int?>

    // Actions
    fun togglePathView()

    fun selectLocation(index: Int?)

    fun updateTripName(name: String)

    fun exportToGpx(): Flow<Result<File>>

    fun deleteTrip(): Flow<Result<Unit>>
}
```

### 7.3 MovementEventsViewModel

**File:** `ui/movementevents/MovementEventsViewModel.kt`

```kotlin
@HiltViewModel
class MovementEventsViewModel @Inject constructor(
    private val movementEventRepository: MovementEventRepository,
) : ViewModel() {

    // State
    val events: StateFlow<List<MovementEvent>>
    val isLoading: StateFlow<Boolean>
    val isLiveMode: StateFlow<Boolean>

    // Statistics
    val totalEventCount: StateFlow<Int>
    val unsyncedCount: StateFlow<Int>

    // Actions
    fun toggleLiveMode()

    fun loadMoreEvents()

    fun refreshEvents()

    fun exportEvents(format: ExportFormat): Flow<Result<File>>

    fun clearOldEvents(beforeDays: Int)

    enum class ExportFormat { JSON, CSV }
}
```

### 7.4 SettingsViewModel Additions

**File:** `ui/settings/SettingsViewModel.kt` (additions)

```kotlin
@HiltViewModel
class SettingsViewModel @Inject constructor(
    // ... existing ...
    private val preferencesRepository: PreferencesRepository,
) : ViewModel() {

    // ... existing ...

    // Trip Detection Settings
    val isTripDetectionEnabled: StateFlow<Boolean>
    val tripStationaryThreshold: StateFlow<Int>
    val tripMinimumDuration: StateFlow<Int>
    val tripMinimumDistance: StateFlow<Int>
    val isTripAutoMergeEnabled: StateFlow<Boolean>

    // Actions
    fun setTripDetectionEnabled(enabled: Boolean)
    fun setTripStationaryThreshold(minutes: Int)
    fun setTripMinimumDuration(minutes: Int)
    fun setTripMinimumDistance(meters: Int)
    fun setTripAutoMergeEnabled(enabled: Boolean)
}
```

---

## 8. File Structure

### 8.1 New Files to Create

```
app/src/main/java/three/two/bit/phonemanager/
├── data/
│   ├── model/
│   │   ├── TripEntity.kt
│   │   └── MovementEventEntity.kt
│   ├── database/
│   │   ├── TripDao.kt
│   │   └── MovementEventDao.kt
│   └── repository/
│       ├── TripRepository.kt
│       ├── TripRepositoryImpl.kt
│       ├── MovementEventRepository.kt
│       └── MovementEventRepositoryImpl.kt
├── domain/
│   └── model/
│       ├── Trip.kt
│       └── MovementEvent.kt
├── trip/
│   ├── TripManager.kt
│   ├── TripManagerImpl.kt
│   ├── TripDetectionAlgorithm.kt
│   ├── TripState.kt
│   ├── TripTrigger.kt
│   ├── TripModeSegment.kt
│   └── SensorTelemetryCollector.kt
├── ui/
│   ├── triphistory/
│   │   ├── TripHistoryScreen.kt
│   │   ├── TripHistoryViewModel.kt
│   │   └── components/
│   │       ├── TripCard.kt
│   │       ├── TripFilterBar.kt
│   │       └── DateRangePicker.kt
│   ├── tripdetail/
│   │   ├── TripDetailScreen.kt
│   │   ├── TripDetailViewModel.kt
│   │   └── components/
│   │       ├── TripMap.kt
│   │       ├── ModeBreakdownChart.kt
│   │       └── TripStatistics.kt
│   └── movementevents/
│       ├── MovementEventsScreen.kt
│       ├── MovementEventsViewModel.kt
│       └── components/
│           └── MovementEventCard.kt
├── network/
│   └── models/
│       ├── MovementEventPayload.kt
│       ├── TripPayload.kt
│       └── LocationCorrectionDto.kt
└── di/
    └── TripModule.kt
```

### 8.2 Files to Modify

| File | Changes |
|------|---------|
| `data/model/LocationEntity.kt` | Add mode, trip, correction fields |
| `data/database/AppDatabase.kt` | Version 8, new entities, migration |
| `data/database/LocationDao.kt` | New queries for trip/mode filtering |
| `data/preferences/PreferencesRepository.kt` | Trip detection settings |
| `data/preferences/PreferencesRepositoryImpl.kt` | Implement new settings |
| `movement/TransportationModeManager.kt` | Event recording hook |
| `service/LocationTrackingService.kt` | TripManager integration, notification |
| `ui/home/HomeScreen.kt` | Trip status card |
| `ui/home/HomeViewModel.kt` | Trip state observation |
| `ui/settings/SettingsScreen.kt` | Trip detection section |
| `ui/settings/SettingsViewModel.kt` | Trip settings |
| `navigation/AppNavigation.kt` | New routes |
| `di/DatabaseModule.kt` | New DAO providers, migration |
| `di/RepositoryModule.kt` | New repository bindings |
| `network/models/LocationPayload.kt` | Add mode/trip fields |

---

## 9. Implementation Priority

### Phase 1: Data Foundation (Days 1-2)
1. Create `TripEntity.kt`
2. Create `MovementEventEntity.kt`
3. Modify `LocationEntity.kt` with new fields
4. Add `MIGRATION_7_8` to `AppDatabase.kt`
5. Create `TripDao.kt`
6. Create `MovementEventDao.kt`
7. Update `LocationDao.kt` with new queries
8. Update `DatabaseModule.kt`

### Phase 2: Domain & Repository (Days 2-3)
1. Create domain models (`Trip.kt`, `MovementEvent.kt`)
2. Create `TripRepository` interface + `TripRepositoryImpl`
3. Create `MovementEventRepository` interface + impl
4. Update `RepositoryModule.kt` with bindings

### Phase 3: Core Logic (Days 3-4)
1. Create `TripState.kt`, `TripTrigger.kt`
2. Implement `TripDetectionAlgorithm.kt`
3. Implement `TripManager.kt` + `TripManagerImpl.kt`
4. Create `SensorTelemetryCollector.kt`
5. Add trip preferences to `PreferencesRepository`
6. Create `TripModule.kt` (Hilt)

### Phase 4: Service Integration (Days 4-5)
1. Integrate event recording in `TransportationModeManager`
2. Integrate `TripManager` in `LocationTrackingService`
3. Update location capture with transportation mode
4. Update notification with trip status

### Phase 5: UI (Days 5-7)
1. Create Trip History Screen + ViewModel
2. Create Trip Detail Screen + ViewModel
3. Create Movement Events Screen + ViewModel
4. Update Settings Screen with trip section
5. Update HomeScreen with trip card
6. Add navigation routes

### Phase 6: Backend Prep (Day 7)
1. Create API payload models (`MovementEventPayload`, `TripPayload`)
2. Create `LocationCorrectionDto`
3. Modify `LocationPayload` with new fields

---

## 10. Testing Requirements

### 10.1 Unit Tests

| Component | Test Class | Key Tests |
|-----------|------------|-----------|
| `TripDetectionAlgorithm` | `TripDetectionAlgorithmTest` | State transitions, edge cases, mode changes |
| `TripManager` | `TripManagerTest` | Lifecycle, start/end conditions, statistics |
| `SensorTelemetryCollector` | `SensorTelemetryCollectorTest` | Data collection, null handling |
| `TripRepository` | `TripRepositoryTest` | CRUD operations, queries |
| `MovementEventRepository` | `MovementEventRepositoryTest` | Event recording, sync |

### 10.2 Integration Tests

| Test | Description |
|------|-------------|
| Database Migration | Verify v7→v8 migration preserves data |
| Trip-Location Relations | Verify FK constraints and cascades |
| Event Recording Flow | End-to-end from detection to storage |
| Trip Lifecycle | Full trip from start to completion |

### 10.3 UI Tests

| Screen | Tests |
|--------|-------|
| Trip History | Navigation, filtering, pagination, delete |
| Trip Detail | Map rendering, path toggle, export |
| Movement Events | Live mode, export, clear |
| Settings | Toggle states, value changes |

### 10.4 Manual Testing Scenarios

1. **Trip Detection:** Walk around, verify trip auto-starts
2. **Trip End:** Stop moving for 5+ minutes, verify trip ends
3. **Brief Stop:** Stop at traffic light <90s, verify trip continues
4. **Mode Change:** Walk to car, drive, verify mode segments
5. **Background:** Lock phone, verify trip continues
6. **Kill App:** Force stop, verify trip resumes on restart
7. **Path Display:** Compare raw vs corrected path on map

---

## Appendix A: Domain Models

### Trip Domain Model

**File:** `domain/model/Trip.kt`

```kotlin
data class Trip(
    val id: String,
    val state: TripState,
    val startTime: Instant,
    val endTime: Instant?,
    val startLocation: LatLng,
    val endLocation: LatLng?,
    val totalDistanceMeters: Double,
    val durationSeconds: Long?,
    val locationCount: Int,
    val dominantMode: TransportationMode,
    val modesUsed: Set<TransportationMode>,
    val modeBreakdown: Map<TransportationMode, Long>,  // Mode -> milliseconds
    val startTrigger: TripTrigger,
    val endTrigger: TripTrigger?,
    val isSynced: Boolean,
    val createdAt: Instant,
    val updatedAt: Instant,
) {
    val isActive: Boolean
        get() = state == TripState.ACTIVE || state == TripState.PENDING_END

    val averageSpeedKmh: Double?
        get() = durationSeconds?.let { duration ->
            if (duration > 0) (totalDistanceMeters / 1000.0) / (duration / 3600.0)
            else null
        }
}
```

### MovementEvent Domain Model

**File:** `domain/model/MovementEvent.kt`

```kotlin
data class MovementEvent(
    val id: Long,
    val timestamp: Instant,
    val tripId: String?,
    val previousMode: TransportationMode,
    val newMode: TransportationMode,
    val detectionSource: DetectionSource,
    val confidence: Float,
    val detectionLatencyMs: Long,
    val location: EventLocation?,
    val deviceState: DeviceState?,
    val sensorTelemetry: SensorTelemetry?,
    val isSynced: Boolean,
)

data class EventLocation(
    val latitude: Double,
    val longitude: Double,
    val accuracy: Float?,
    val speed: Float?,
)

data class DeviceState(
    val batteryLevel: Int?,
    val batteryCharging: Boolean?,
    val networkType: String?,
    val networkStrength: Int?,
)

data class SensorTelemetry(
    val accelerometerMagnitude: Float?,
    val accelerometerVariance: Float?,
    val accelerometerPeakFrequency: Float?,
    val gyroscopeMagnitude: Float?,
    val stepCount: Int?,
    val significantMotion: Boolean?,
    val activityType: String?,
    val activityConfidence: Int?,
)
```

---

## Appendix B: Navigation Routes

**File:** `navigation/AppNavigation.kt` (additions)

```kotlin
sealed class Screen(val route: String) {
    // ... existing ...

    object TripHistory : Screen("trip_history")
    object TripDetail : Screen("trip_detail/{tripId}") {
        fun createRoute(tripId: String) = "trip_detail/$tripId"
    }
    object MovementEvents : Screen("movement_events")
}
```

---

## Appendix C: Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-30 | Initial specification |
