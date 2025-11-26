# Story 2.2: Device Update via Re-Registration

**Status**: Ready for Review

## Story

**As a** mobile app
**I want** to update device information by re-registering
**So that** I can change display name or FCM token without losing history

**Prerequisites**: Story 2.1 ✅

## Acceptance Criteria

1. [x] Re-registration with same `deviceId` updates existing record
2. [x] Updates: `display_name`, `fcm_token`, `updated_at`, `last_seen_at`
3. [x] Preserves: `id`, `device_id`, `created_at`, all associated location records
4. [x] Allows updating `group_id` if new group has capacity (implements FR-23)
5. [x] If changing groups, validates new group size limit
6. [x] Returns 200 with updated device information
7. [x] Returns 409 if moving to full group (20 devices)

## Technical Notes

- Use `INSERT ... ON CONFLICT (device_id) DO UPDATE` for upsert
- Transaction ensures atomic group change validation

## Tasks/Subtasks

- [x] 1. Enhance upsert logic in repository
  - [x] 1.1 Update `upsert_device` to handle group changes
  - [x] 1.2 Add transaction wrapper for atomic operations
- [x] 2. Add group change validation
  - [x] 2.1 Check new group capacity before allowing group change
  - [x] 2.2 Return appropriate error if target group full
- [x] 3. Write tests
  - [x] 3.1 Test device update preserves id and created_at
  - [x] 3.2 Test group change with capacity check
  - [x] 3.3 Test rejection when target group full
- [x] 4. Run linting and formatting checks

## Dev Notes

- Builds on Story 2.1 repository implementation
- Group size limit enforced at registration/update time

## Dev Agent Record

### Debug Log

- Upsert uses INSERT ... ON CONFLICT DO UPDATE with RETURNING clause
- Handler detects group change by comparing existing device's group_id
- Group capacity checked before any group change operation

### Completion Notes

Implemented via the same upsert_device repository method and register_device handler. Group change validation checks capacity before allowing device to move groups.

## File List

### Modified Files

- `crates/persistence/src/repositories/device.rs` - upsert_device with ON CONFLICT DO UPDATE
- `crates/api/src/routes/devices.rs` - register_device with group change detection

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
