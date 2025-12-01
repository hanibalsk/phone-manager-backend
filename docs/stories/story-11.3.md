# Story 11.3: Role Management Endpoint

**Epic**: Epic 11 - Group Management API
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** group admin or owner
**I want** to change member roles within my group
**So that** I can grant appropriate permissions to group members

## Prerequisites

- Story 11.2 complete (Membership Management)
- GroupRole enum with owner, admin, member, viewer
- GroupRepository with update_member_role method

## Acceptance Criteria

1. PUT /api/v1/groups/:group_id/members/:user_id/role updates member role
2. Only admins and owners can change roles
3. Cannot change the owner's role (must use transfer endpoint)
4. Cannot promote anyone to owner (must use transfer endpoint)
5. Admins cannot promote others to admin (only owner can)
6. Role changes are validated against enum values
7. Returns updated membership info

## Technical Notes

- Reuse update_member_role repository method from Story 11.1
- Request body: { "role": "admin" | "member" | "viewer" }
- Response: MembershipInfo with updated role
- Validation: role must be valid GroupRole value

## Implementation Tasks

- [x] Add UpdateRoleRequest DTO
- [x] Create update_member_role handler with permission checks
- [x] Add route to app.rs
- [x] Add unit tests for permission logic

---

## Dev Notes

- Owner role is special - cannot be assigned via this endpoint
- Owner must use /transfer endpoint to change ownership
- Admins can demote other admins but cannot promote

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Added UpdateRoleRequest and UpdateRoleResponse DTOs
- Created update_member_role handler with permission checks:
  - Only admins/owners can change roles
  - Cannot promote to owner (use transfer endpoint)
  - Cannot change owner's role (use transfer endpoint)
  - Admins cannot promote to admin (only owner can)
  - Admins cannot change other admins' roles
- Added PUT /api/v1/groups/:group_id/members/:user_id/role route
- All tests pass


---

## File List

- crates/domain/src/models/group.rs
- crates/api/src/routes/groups.rs
- crates/api/src/app.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

