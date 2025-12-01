# Story 11.1: Group Database Schema & CRUD Endpoints

**Epic**: Epic 11 - Group Management API
**Status**: In Progress
**Created**: 2025-12-01

---

## User Story

**As a** user
**I want** to create and manage location sharing groups
**So that** I can organize location sharing with family and friends

## Prerequisites

- Story 10.6 complete (device registration with optional auth)
- User authentication system operational
- PostgreSQL database ready

## Acceptance Criteria

1. Database migration creates `groups` and `group_memberships` tables
2. POST /api/v1/groups creates a new group (creator becomes owner)
3. GET /api/v1/groups lists user's groups with role
4. GET /api/v1/groups/:group_id gets group details
5. PUT /api/v1/groups/:group_id updates group (admin/owner only)
6. DELETE /api/v1/groups/:group_id deletes group (owner only)
7. All endpoints require JWT authentication
8. Group names are 1-100 characters
9. Max devices per group defaults to 20

## Technical Notes

- Use UUID for group IDs (not string group_id)
- Create slug from name for URL-friendly identifiers
- Group settings stored as JSONB for flexibility
- Membership includes role enum: owner, admin, member, viewer
- Foreign keys to users table for created_by and memberships

## Implementation Tasks

- [x] Create migration 019_groups.sql with tables
- [x] Create Group domain model in domain/models/group.rs
- [x] Create GroupRepository in persistence/repositories
- [x] Create group handlers in api/routes/groups.rs
- [x] Add routes to app.rs
- [x] Add unit tests

---

## Dev Notes

- Legacy string group_id in devices table will coexist initially
- Migration to UUID-based groups can be done incrementally
- Creator is automatically added as owner member

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Created migration 019_groups.sql with groups and group_memberships tables
- Created group_role enum type for roles (owner, admin, member, viewer)
- Added database trigger to prevent removing last owner from group
- Created GroupRole domain enum with permission helper methods
- Created Group, GroupMembership domain models
- Created CreateGroupRequest, UpdateGroupRequest DTOs with validation
- Created GroupSummary, GroupDetail, CreateGroupResponse response DTOs
- Created GroupRepository with CRUD operations
- Implemented slug generation from group names with uniqueness handling
- Created group handlers: create_group, list_groups, get_group, update_group, delete_group
- All endpoints require JWT authentication via UserAuth extractor
- Permission checks: admins/owners can update, only owners can delete
- All 319 tests pass

---

## File List

- crates/persistence/src/migrations/019_groups.sql
- crates/domain/src/models/group.rs
- crates/domain/src/models/mod.rs
- crates/persistence/src/entities/group.rs
- crates/persistence/src/entities/mod.rs
- crates/persistence/src/repositories/group.rs
- crates/persistence/src/repositories/mod.rs
- crates/api/src/routes/groups.rs
- crates/api/src/routes/mod.rs
- crates/api/src/app.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

