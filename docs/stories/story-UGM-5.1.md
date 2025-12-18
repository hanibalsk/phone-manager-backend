# Story UGM-5.1: Invite Code Expiration Handling

**Status**: Ready for Development

## Story

**As a** user receiving a group invitation,
**I want** expired invite codes to return clear error messages,
**So that** I understand why joining failed and can request a new invite.

**Epic**: UGM-5: NFR Compliance
**Prerequisites**: None
**NFRs Covered**: NFR19, NFR20

## Acceptance Criteria

1. [ ] Given a group invite, when created without explicit expiry, then the default expiration is 48 hours from creation
2. [ ] Given an expired invite code, when a user attempts to join with it, then the response is 410 Gone with error code `invite/expired`
3. [ ] Given an expired invite code error response, then it includes `expired_at` timestamp showing when the invite expired
4. [ ] Given an expired invite code error response, then it includes `group_name` so user knows which group to request new invite from
5. [ ] Given a valid (non-expired) invite code, when a user joins, then the join succeeds as normal
6. [ ] Given an invite created with custom expiry (e.g., 24 hours), then the invite expires at the specified time
7. [ ] Given an invite approaching expiration (<1 hour remaining), when listing invites as admin, then the invite shows `expiring_soon: true`

## Technical Notes

- Default expiry: 48 hours (configurable via `PM__INVITES__DEFAULT_EXPIRY_HOURS`)
- Error response format for expired invites:
  ```json
  {
    "error": {
      "code": "invite/expired",
      "message": "This invitation has expired",
      "details": {
        "expired_at": "2025-12-18T10:30:00Z",
        "group_name": "Chen Family"
      }
    }
  }
  ```
- Existing `group_invites` table already has `expires_at` column

## Tasks/Subtasks

- [ ] 1. Verify default expiry is set to 48 hours when creating invites
- [ ] 2. Update join endpoint to return 410 Gone for expired invites
- [ ] 3. Include `expired_at` and `group_name` in error response
- [ ] 4. Add `expiring_soon` flag to invite list response
- [ ] 5. Add integration tests for expiry scenarios
- [ ] 6. Update OpenAPI spec with 410 error response

## File List

### Files to Modify

- `crates/api/src/routes/invites.rs` - Update join handler for expiry error response
- `crates/api/src/routes/groups.rs` - Update invite creation default expiry
- `crates/domain/src/models/invite.rs` - Add expiry-related response fields
- `docs/api/openapi.yaml` - Document 410 error response

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Integration tests for expiry scenarios pass
- [ ] OpenAPI spec updated
- [ ] Code compiles without warnings
- [ ] Code passes clippy

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created from gap analysis | Dev Agent |
