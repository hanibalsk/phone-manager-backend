# Story 2.4: Device Soft Delete/Deactivation

**Status**: Ready for Review

## Story

**As a** mobile app
**I want** to deactivate a device without deleting its data
**So that** users can remove devices from active tracking while preserving history

**Prerequisites**: Story 2.1 ✅

## Acceptance Criteria

1. [x] `DELETE /api/v1/devices/:deviceId` sets `active=false` instead of deleting row
2. [x] Deactivated devices excluded from group device listings (active filter)
3. [x] Location records for deactivated devices remain in database
4. [x] Deactivated devices can be reactivated via re-registration
5. [x] Returns 204 No Content on successful deactivation
6. [x] Returns 404 if device doesn't exist

## Technical Notes

- Soft delete via `UPDATE devices SET active=false WHERE device_id=?`
- Location cleanup job respects retention policy regardless of device status

## Tasks/Subtasks

- [x] 1. Add deactivate method to repository
  - [x] 1.1 Implement `deactivate_device` method
  - [x] 1.2 Return affected row count for 404 detection
- [x] 2. Add delete endpoint to routes
  - [x] 2.1 Implement DELETE /api/v1/devices/:deviceId handler
  - [x] 2.2 Return 204 on success, 404 on not found
- [x] 3. Update upsert to handle reactivation
  - [x] 3.1 Set active=true when re-registering deactivated device
- [x] 4. Write tests
  - [x] 4.1 Test soft delete sets active=false
  - [x] 4.2 Test 404 for non-existent device
  - [x] 4.3 Test reactivation via re-registration
- [x] 5. Run linting and formatting checks

## Dev Notes

- Soft delete preserves all historical data
- Reactivation happens automatically through re-registration

## Dev Agent Record

### Debug Log

- Implemented `deactivate_device` with UPDATE ... WHERE active=true
- Returns rows_affected for 404 detection
- Upsert sets active=true on re-registration (via EXCLUDED.active)

### Completion Notes

Soft delete fully implemented. Deactivated devices are excluded from listings, can be reactivated via re-registration.

## File List

### Modified Files

- `crates/persistence/src/repositories/device.rs` - deactivate_device method
- `crates/api/src/routes/devices.rs` - delete_device handler
- `crates/api/src/app.rs` - DELETE route wiring

### New Files

(None)

### Deleted Files

(None)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |
| 2025-11-26 | Implementation complete | Dev Agent |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes
