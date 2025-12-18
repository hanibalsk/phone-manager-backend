# Story UGM-5.4: Migration Data Preservation Verification

**Status**: Ready for Development

## Story

**As a** user migrating from a registration group,
**I want** all my device data and location history to be preserved,
**So that** I don't lose any tracking data during the migration process.

**Epic**: UGM-5: NFR Compliance
**Prerequisites**: Story UGM-2.2 (Migration Endpoint)
**FRs Covered**: FR7 (Preserve device data and location history)

## Acceptance Criteria

1. [ ] Given a registration group with devices having location history, when migration completes, then all location history remains queryable via the device endpoints
2. [ ] Given a device with 1000 location records before migration, when migration completes, then all 1000 records are accessible via `GET /api/v1/devices/:deviceId/locations`
3. [ ] Given a device in a registration group, when migration completes, then device metadata (display_name, platform, os_version, app_version) is unchanged
4. [ ] Given a device with FCM token registered, when migration completes, then FCM token remains valid and associated with the device
5. [ ] Given a device with geofences configured, when migration completes, then all geofences remain active and associated with the device
6. [ ] Given a device with webhooks configured, when migration completes, then all webhooks remain active
7. [ ] Given a migration of a group with 5 devices and 10,000 total location records, when migration completes, then a data integrity check confirms record counts match pre-migration
8. [ ] Given migration audit log, then it includes `location_records_preserved` count for verification

## Technical Notes

- Location data stored in `locations` table is keyed by `device_id` (UUID), not group
- Migration only creates entries in `device_group_memberships` - doesn't modify `locations` table
- Device metadata in `devices` table should remain unchanged
- Geofences and webhooks are device-scoped, not group-scoped
- Verification can be done via count comparison before/after migration

**Verification Query Example:**
```sql
-- Pre-migration count
SELECT COUNT(*) FROM locations WHERE device_id = ANY($1);

-- Post-migration count (should match)
SELECT COUNT(*) FROM locations l
JOIN device_group_memberships dgm ON l.device_id = dgm.device_id
WHERE dgm.group_id = $2;
```

## Tasks/Subtasks

- [ ] 1. Add pre-migration data count collection in migration service
- [ ] 2. Add post-migration data verification step
- [ ] 3. Include `location_records_preserved` in migration audit log
- [ ] 4. Add integration test: migrate group, verify location history accessible
- [ ] 5. Add integration test: migrate group, verify device metadata unchanged
- [ ] 6. Add integration test: migrate group, verify geofences remain active
- [ ] 7. Add data integrity assertion in migration response

## File List

### Files to Modify

- `crates/api/src/routes/groups.rs` - Add data verification to migration
- `crates/persistence/src/repositories/migration_audit.rs` - Add preserved counts
- `crates/domain/src/models/migration.rs` - Add preservation stats to response

### Files to Create

- `crates/api/tests/migration_data_preservation_test.rs` - Integration tests

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Integration tests verify data preservation
- [ ] Migration audit log includes preservation counts
- [ ] Code compiles without warnings
- [ ] Code passes clippy

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created from gap analysis | Dev Agent |
