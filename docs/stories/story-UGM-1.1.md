# Story UGM-1.1: Auto-link Device on Authentication

**Status**: Complete ✅

## Story

**As a** user who previously used the app anonymously
**I want** my device to be automatically linked to my account when I log in
**So that** I don't have to manually re-register my device after creating an account

**Epic**: UGM-1: Registration Group Status & Device Linking
**Prerequisites**: None

## Acceptance Criteria

1. [x] Given a user has a device registered with a registration group (anonymous), when the user logs in with valid credentials on that device, then the device's `owner_user_id` is set to the authenticated user's ID and the response includes confirmation of device linking
2. [x] Given a user logs in on a device that is already linked to their account, when authentication succeeds, then no changes are made to device ownership and the login succeeds normally
3. [x] Given a user logs in on a device linked to a different user, when authentication succeeds, then the device ownership is NOT changed (device stays with original owner) and the login succeeds for the new user without device linking

## Technical Notes

- Login endpoint (`POST /api/v1/auth/login`) already accepts optional `device_id` and `device_name` fields
- OAuth login endpoint (`POST /api/v1/auth/oauth`) also accepts these fields
- `DeviceRepository::link_device_to_user` method already exists
- `DeviceRepository::find_by_device_id` method already exists
- Need to add device linking logic to auth routes after successful authentication
- Login response should include `device_linked: bool` field

## Tasks/Subtasks

- [x] 1. Update login response to include `device_linked` field
- [x] 2. Implement device linking logic in `login` handler
  - [x] 2.1 Parse device_id from request if provided
  - [x] 2.2 Check if device exists and is not owned by another user
  - [x] 2.3 Link device to user if conditions are met
- [x] 3. Implement device linking logic in `oauth_login` handler
- [ ] 4. Add integration tests for device auto-linking (future)
- [ ] 5. Update OpenAPI documentation (future)

## File List

### Files Modified

- `crates/api/src/routes/auth.rs` - Added `try_link_device_to_user` helper function and device linking logic to login/oauth_login handlers

### Files Created

- `docs/stories/story-UGM-1.1.md` - This story file

## Implementation Details

Added helper function `try_link_device_to_user()` that:
1. Validates device_id is provided and is a valid UUID
2. Checks if device exists in the database
3. Verifies device is not already linked to another user
4. Links the device to the authenticated user (first device becomes primary)
5. Returns `true` if device was linked, `false` otherwise

Updated `LoginResponse` to include `device_linked: bool` field.

Updated both `login` and `oauth_login` handlers to call `try_link_device_to_user()` after successful authentication.

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created and implemented | Dev Agent |
