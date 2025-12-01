# Story 11.4: Invite CRUD Endpoints

**Epic**: Epic 11 - Group Management API
**Status**: In Progress
**Created**: 2025-12-01

---

## User Story

**As a** group admin or owner
**I want** to create and manage invitation codes
**So that** I can invite new members to my group

## Prerequisites

- Story 11.3 complete (Role Management)
- groups and group_memberships tables exist
- JWT authentication operational

## Acceptance Criteria

1. POST /api/v1/groups/:group_id/invites creates a new invite
2. GET /api/v1/groups/:group_id/invites lists active invites
3. DELETE /api/v1/groups/:group_id/invites/:invite_id revokes an invite
4. GET /api/v1/invites/:code returns invite info (public, no auth)
5. Only admins and owners can manage invites
6. Invite codes are unique 9-character codes (ABC-123-XYZ format)
7. Invites have preset_role, max_uses, and expiration
8. Invites track current_uses

## Technical Notes

- Create migration 020_group_invites.sql
- Code format: XXX-XXX-XXX (uppercase letters and digits)
- Default: 1 use, 24 hour expiration, member role
- Max uses: 1-100
- Expiration: 1-168 hours (1 week max)

## Implementation Tasks

- [ ] Create migration 020_group_invites.sql
- [ ] Add invite entity and DTOs
- [ ] Add InviteRepository with CRUD methods
- [ ] Create invite handlers
- [ ] Add routes to app.rs
- [ ] Add unit tests

---

## Dev Notes

- Invite codes should be easy to type and share
- Public info endpoint shows group name but not sensitive data
- Expired and fully-used invites should be excluded from listings

---

## Dev Agent Record

### Debug Log


### Completion Notes


---

## File List

- crates/persistence/src/migrations/020_group_invites.sql
- crates/persistence/src/entities/invite.rs
- crates/persistence/src/entities/mod.rs
- crates/persistence/src/repositories/invite.rs
- crates/persistence/src/repositories/mod.rs
- crates/domain/src/models/invite.rs
- crates/domain/src/models/mod.rs
- crates/api/src/routes/invites.rs
- crates/api/src/routes/mod.rs
- crates/api/src/app.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

