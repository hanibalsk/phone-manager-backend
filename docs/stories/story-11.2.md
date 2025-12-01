# Story 11.2: Membership Management Endpoints

**Epic**: Epic 11 - Group Management API
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** group member
**I want** to view and manage group members
**So that** I can see who is in my group and control membership

## Prerequisites

- Story 11.1 complete (Group CRUD endpoints)
- group_memberships table with role enum exists
- JWT authentication operational

## Acceptance Criteria

1. GET /api/v1/groups/:group_id/members lists all group members
2. GET /api/v1/groups/:group_id/members/:user_id gets member details
3. DELETE /api/v1/groups/:group_id/members/:user_id removes member from group
4. Members can view other members in same group
5. Only admins/owners can remove other members
6. Members can remove themselves (leave group)
7. Owners cannot leave without transferring ownership first
8. Pagination supported on list endpoint
9. Optional role filter on list endpoint
10. Optional include_devices parameter on list endpoint

## Technical Notes

- Reuse GroupRepository methods from Story 11.1
- Add list_members repository method with pagination
- Add get_member_details repository method
- Use existing remove_member repository method
- Enforce business rules in handler layer
- UserPublic DTO for member user info (no email for non-self)

## Implementation Tasks

- [x] Add MemberResponse, MemberListResponse DTOs
- [x] Add list_members, get_member_details repository methods
- [x] Create membership handlers in groups.rs
- [x] Add member routes to app.rs
- [x] Add unit tests

---

## Dev Notes

- Owner cannot be removed via this endpoint (must transfer first)
- Self-removal (leave) allowed for non-owners
- Pagination uses page/per_page pattern

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Added ListMembersQuery, ListMembersResponse, MemberResponse, Pagination, UserPublic DTOs
- Added MemberWithUserEntity for DB queries joining memberships with users
- Added list_members, count_members, get_member_with_user repository methods
- Created list_members handler with pagination (page/per_page, max 100)
- Created get_member handler to view individual member details
- Created remove_member handler with permission logic:
  - Owner cannot leave (must transfer first)
  - Non-owners can leave (self-removal)
  - Only admins/owners can remove others
  - Cannot remove the owner
  - Admins cannot remove other admins
- All 319+ tests pass


---

## File List

- crates/domain/src/models/group.rs
- crates/persistence/src/repositories/group.rs
- crates/api/src/routes/groups.rs
- crates/api/src/app.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

