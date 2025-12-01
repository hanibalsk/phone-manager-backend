# Story 11.5: Join Group with Invite Code

**Epic**: Epic 11 - Group Management API
**Status**: In Progress
**Priority**: High

## User Story

**As a** mobile app user
**I want** to join a group using an invite code
**So that** I can participate in location sharing with my family/friends

## Acceptance Criteria

1. `POST /api/v1/groups/join` accepts JSON: `{"code": "<invite-code>"}`
2. Validates invite code format (XXX-XXX-XXX)
3. Returns 400 if code format invalid
4. Returns 404 if invite not found
5. Returns 410 Gone if invite expired or fully used
6. Returns 409 Conflict if user already a member of the group
7. Creates membership with preset role from invite
8. Increments invite `current_uses` counter
9. Returns 200 with group info and membership details
10. Requires JWT authentication

## Technical Notes

- Route: `POST /api/v1/groups/join`
- Requires `UserAuth` extractor for JWT validation
- Transaction for atomicity (membership creation + invite increment)
- Response includes group summary and membership info

## Implementation Checklist

- [x] Migration 020_group_invites.sql (completed in Story 11.4)
- [x] Domain model invite.rs (completed in Story 11.4)
- [x] InviteRepository (completed in Story 11.4)
- [ ] Add JoinGroupRequest and JoinGroupResponse DTOs
- [ ] Add join_group handler in groups.rs
- [ ] Add POST /api/v1/groups/join route
- [ ] Update GroupRepository with membership creation method if needed
- [ ] Integration tests

## API Specification

### Request

```http
POST /api/v1/groups/join
Authorization: Bearer <jwt>
Content-Type: application/json

{
  "code": "ABC-123-XYZ"
}
```

### Response (200 OK)

```json
{
  "group": {
    "id": "grp_01HXYZABC123",
    "name": "Smith Family",
    "memberCount": 5
  },
  "membership": {
    "id": "mem_01NEWMEM456",
    "role": "member",
    "joinedAt": "2025-12-01T10:30:00Z"
  }
}
```

### Error Responses

- 400: Invalid code format
- 404: Invite not found
- 409: Already a member
- 410: Invite expired or fully used
