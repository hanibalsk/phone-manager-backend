# Story UGM-3.6: Enhanced Member List with Device Count

**Status**: Complete ✅

## Story

**As a** group admin,
**I want** to see all members in my group with their device counts,
**So that** I can understand how many devices each family member has.

**Epic**: UGM-3: Device-Group Management
**Prerequisites**: Story UGM-3.1: Device-Group Membership Table

## Acceptance Criteria

1. [x] Given an authenticated user who is a member of a group, when calling `GET /api/v1/groups/:groupId/members`, then the response includes all members with their details
2. [x] Each member includes a `device_count` field showing how many devices they have in this group
3. [x] Given a member with 2 devices in the group, when viewing the member list, then that member's `device_count` is 2
4. [x] Given a member with no devices in the group, when viewing the member list, then that member's `device_count` is 0

## Technical Notes

- Endpoint: `GET /api/v1/groups/:groupId/members` (enhanced)
- Endpoint: `GET /api/v1/groups/:groupId/members/:userId` (enhanced)
- Requires JWT authentication (UserAuth extractor)
- Uses `device_group_memberships` table to count devices per user
- Added `device_count` field to `MemberResponse` struct

## Tasks/Subtasks

- [x] 1. Add `device_count` field to `MemberResponse` struct in domain model
- [x] 2. Update `list_members` handler to include device counts
- [x] 3. Update `get_member` handler to include device count

## File List

### Files Created

- `docs/stories/story-UGM-3.6.md` - This story file

### Files Modified

- `crates/domain/src/models/group.rs` - Add `device_count` field to `MemberResponse`
- `crates/api/src/routes/groups.rs` - Update `list_members` and `get_member` handlers

## Implementation Details

### Response Format

The `MemberResponse` now includes a `device_count` field:

```json
{
  "data": [
    {
      "id": "uuid",
      "user": {
        "id": "uuid",
        "display_name": "John",
        "avatar_url": null
      },
      "role": "member",
      "joined_at": "2025-12-18T10:30:00Z",
      "invited_by": null,
      "devices": [...],
      "device_count": 2
    }
  ],
  "pagination": {...}
}
```

### Query

The device count is retrieved using `count_devices_per_user_in_group` which:
1. Queries `device_group_memberships` joined with `devices`
2. Groups by `owner_user_id`
3. Returns count per user

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
