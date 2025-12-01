# Story 12.1: Device Settings Database Schema

**Epic**: Epic 12 - Settings Control
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** backend developer
**I want** database tables for device settings and setting definitions
**So that** I can store and manage device settings with their lock states

## Prerequisites

- Story 11.7 complete (RBAC Middleware)
- PostgreSQL database operational
- Device user binding implemented (Epic 10)

## Acceptance Criteria

1. Migration creates `setting_definitions` table with setting metadata
2. Migration creates `device_settings` table with setting values and locks
3. `setting_definitions` has: key (unique), display_name, description, data_type, default_value, is_lockable, category
4. `device_settings` has: device_id, setting_key, value (JSONB), is_locked, locked_by, locked_at, lock_reason
5. Foreign key from device_settings.device_id to devices.id
6. Foreign key from device_settings.locked_by to users.id (nullable)
7. Composite unique constraint on (device_id, setting_key)
8. Indexes on device_id and setting_key for query performance

## Technical Notes

- Create migration 022_device_settings.sql (021 was already taken)
- `value` field is JSONB to support any data type (boolean, integer, string, etc.)
- `data_type` enum: boolean, integer, string, float, json
- `category` for grouping settings in UI: tracking, privacy, notifications, battery, general
- Seed initial setting definitions for core settings

## Implementation Tasks

- [x] Create migration 022_device_settings.sql
- [x] Create SettingDefinition entity in persistence layer
- [x] Create DeviceSetting entity in persistence layer
- [x] Add entities to mod.rs
- [x] Add domain models for setting types
- [x] Seed initial setting definitions

## Setting Definitions to Seed

| Key | Display Name | Data Type | Default | Lockable | Category |
|-----|--------------|-----------|---------|----------|----------|
| tracking_enabled | Location Tracking | boolean | true | true | tracking |
| tracking_interval_minutes | Tracking Interval | integer | 5 | true | tracking |
| movement_detection_enabled | Movement Detection | boolean | true | true | tracking |
| secret_mode_enabled | Secret Mode | boolean | false | true | privacy |
| battery_optimization_enabled | Battery Optimization | boolean | true | false | battery |
| notification_sounds_enabled | Notification Sounds | boolean | true | false | notifications |
| geofence_notifications_enabled | Geofence Alerts | boolean | true | true | notifications |
| sos_enabled | SOS Feature | boolean | true | true | privacy |

---

## Dev Notes

- JSONB value allows flexibility for different setting types
- Locks can only be set by admins/owners
- Default values from definitions used when device has no explicit setting

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Created migration 022_device_settings.sql with setting_definitions and device_settings tables
- Added setting_data_type enum (boolean, integer, string, float, json)
- Added setting_category enum (tracking, privacy, notifications, battery, general)
- Seeded 8 initial setting definitions for core device settings
- Created SettingDefinitionEntity, DeviceSettingEntity with DB enums
- Created comprehensive domain models including DTOs for all settings endpoints
- All tests pass (661+ tests across workspace)

---

## File List

- crates/persistence/src/migrations/022_device_settings.sql
- crates/persistence/src/entities/setting.rs
- crates/persistence/src/entities/mod.rs
- crates/domain/src/models/setting.rs
- crates/domain/src/models/mod.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

