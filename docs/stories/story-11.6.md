# Story 11.6: Group Ownership Transfer

**Epic**: Epic 11 - Group Management API
**Status**: Done
**Priority**: High

## User Story

**As a** group owner
**I want** to transfer ownership to another member
**So that** I can step down from the owner role while ensuring the group continues

## Acceptance Criteria

1. `POST /api/v1/groups/:group_id/transfer` accepts JSON: `{"newOwnerId": "<user-uuid>"}`
2. Only the current owner can initiate transfer
3. Target user must be an existing member of the group
4. Transfer is atomic (old owner becomes admin, new owner becomes owner)
5. Returns 200 with transfer details
6. Returns 403 if not the owner
7. Returns 404 if group or target user not found
8. Returns 400 if target user is not a group member
9. Requires JWT authentication

## Technical Notes

- Route: `POST /api/v1/groups/:group_id/transfer`
- Requires `UserAuth` extractor for JWT validation
- Transaction for atomic role swap
- Old owner demoted to admin (not removed)

## Implementation Checklist

- [x] Add TransferOwnershipRequest and TransferOwnershipResponse DTOs
- [x] Add transfer_ownership method to GroupRepository
- [x] Create transfer_ownership handler in groups.rs
- [x] Add POST route for ownership transfer
- [x] Integration tests

## API Specification

### Request

```http
POST /api/v1/groups/:group_id/transfer
Authorization: Bearer <jwt>
Content-Type: application/json

{
  "newOwnerId": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Response (200 OK)

```json
{
  "group_id": "grp_01HXYZABC123",
  "previousOwnerId": "user_01HXYZABC123",
  "newOwnerId": "user_01NEWOWNER456",
  "transferredAt": "2025-12-01T10:30:00Z"
}
```

### Error Responses

- 400: Target user is not a group member
- 403: Only the owner can transfer ownership
- 404: Group not found or you are not a member
