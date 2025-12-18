# Story UGM-2.2: Group Migration Endpoint

**Status**: Complete ✅

## Story

**As a** user with an anonymous registration group,
**I want** to migrate my registration group to an authenticated group,
**So that** I can access full group management features while keeping all my devices.

**Epic**: UGM-2: Group Migration
**Prerequisites**: Story UGM-2.1: Create Migration Audit Log Infrastructure

## Acceptance Criteria

1. [x] Given an authenticated user with a device in a registration group "camping-2025", when the user calls `POST /api/v1/groups/migrate` with `registration_group_id: "camping-2025"`, then a new authenticated group is created and all devices are migrated
2. [x] Given a user provides a custom `group_name: "Chen Family"`, when the migration is executed, then the new authenticated group is named "Chen Family"
3. [x] Given a user does not provide a custom group name, when the migration is executed, then the new authenticated group uses the registration_group_id as its name
4. [x] Given a registration group that has already been migrated, when a user attempts to migrate it again, then the response is 409 Conflict
5. [x] Given a registration group with no devices, when a user attempts to migrate it, then the response is 400 Bad Request
6. [x] Given a group name that already exists, when a user attempts to use that name, then the response is 409 Conflict
7. [x] Given the migration operation, when any step fails, then the entire operation is rolled back (atomic transaction)
8. [x] The response includes `authenticated_group_id`, `name`, `devices_migrated`, `migration_id`
9. [x] The user becomes the owner of the new authenticated group

## Technical Notes

- Endpoint: `POST /api/v1/groups/migrate`
- Request body: `{ registration_group_id: string, group_name?: string }`
- Response: `{ migration_id, authenticated_group_id, name, devices_migrated, device_ids }`
- Uses atomic PostgreSQL transaction
- Logs migration to `migration_audit_logs` table
- User must own at least one device in the registration group

## Tasks/Subtasks

- [x] 1. Create MigrateGroupRequest and MigrateGroupResponse structs
- [x] 2. Add find_devices_by_registration_group to DeviceRepository
- [x] 3. Create RegistrationGroupDevice struct for migration
- [x] 4. Implement migrate_registration_group handler
- [x] 5. Register route at POST /api/v1/groups/migrate
- [x] 6. Export RegistrationGroupDevice from repositories

## File List

### Files Created

- `docs/stories/story-UGM-2.2.md` - This story file

### Files Modified

- `crates/api/src/routes/groups.rs` - Added MigrateGroupRequest, MigrateGroupResponse, and migrate_registration_group handler
- `crates/api/src/app.rs` - Registered migration route
- `crates/persistence/src/repositories/device.rs` - Added find_devices_by_registration_group and RegistrationGroupDevice
- `crates/persistence/src/repositories/mod.rs` - Exported RegistrationGroupDevice

## Implementation Details

### Request Validation
- `registration_group_id`: 1-255 characters, required
- `group_name`: 1-100 characters, optional

### Authorization
- JWT authentication required (UserAuth extractor)
- User must own at least one device in the registration group

### Migration Flow
1. Check if registration group already migrated (409 if so)
2. Get all devices in registration group
3. Verify at least one device exists (400 if empty)
4. Verify user owns at least one device (403 if not)
5. Check group name doesn't exist (409 if conflict)
6. Begin transaction
7. Create new authenticated group
8. Add user as owner in group_memberships
9. Update all devices' group_id to new slug
10. Create migration audit log
11. Commit transaction
12. Return migration response

### Error Handling
- 400: No devices in registration group, validation errors
- 403: User doesn't own any device in registration group
- 409: Already migrated, group name exists
- 500: Transaction/database errors

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass (unit tests in workspace)
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Code passes clippy
- [x] Story file updated with completion notes

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created and implemented | Dev Agent |
