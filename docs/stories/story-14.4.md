# Story 14.4: Group Management Admin Endpoints

**Epic**: Epic 14 - Admin Portal Backend
**Status**: Complete
**Completed**: 2025-12-01
**Created**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to view and manage groups within my organization through the admin API
**So that** I can oversee group activity, membership, and organization-wide group settings

## Prerequisites

- Story 14.3 complete (User management endpoints)
- Group management infrastructure from Epic 11

## Acceptance Criteria

1. GET `/api/admin/v1/organizations/{orgId}/groups` returns paginated group list
2. Each group includes: id, name, slug, description, member_count, device_count
3. Each group includes owner info and creation date
4. Filtering supported by: search text, active status, has_devices
5. Sorting supported by: name, created_at, member_count, device_count
6. Pagination with page/per_page params (default 50, max 100)
7. GET `/api/admin/v1/organizations/{orgId}/groups/{groupId}` returns group details
8. Group detail includes: full group info, members list with roles, devices list
9. PUT `/api/admin/v1/organizations/{orgId}/groups/{groupId}` updates group settings
10. DELETE `/api/admin/v1/organizations/{orgId}/groups/{groupId}` deactivates group
11. Only org admins and owners can access these endpoints

## Technical Notes

- Create admin group management routes in `crates/api/src/routes/admin_groups.rs`
- Query groups that belong to the organization (via device->organization relationship)
- Use LEFT JOINs for member and device counts
- Group detail should include recent activity from audit logs

## API Specification

### GET /api/admin/v1/organizations/{orgId}/groups

Query Parameters:
- `page` (optional): Page number (default: 1)
- `perPage` (optional): Items per page (default: 50, max: 100)
- `search` (optional): Search in group name
- `active` (optional): Filter by active status (true/false)
- `hasDevices` (optional): Filter by having devices (true/false)
- `sort` (optional): Sort field (name, created_at, member_count, device_count)
- `order` (optional): Sort order (asc, desc)

Response (200):
```json
{
  "data": [
    {
      "id": "field-workers",
      "name": "Field Workers",
      "slug": "field-workers",
      "description": "Field team devices",
      "iconEmoji": "👷",
      "memberCount": 8,
      "deviceCount": 15,
      "isActive": true,
      "owner": {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "email": "manager@acme.com",
        "displayName": "Team Manager"
      },
      "createdAt": "2025-11-01T09:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "perPage": 50,
    "total": 5,
    "totalPages": 1
  },
  "summary": {
    "totalGroups": 5,
    "activeGroups": 4,
    "totalMembers": 25,
    "totalDevices": 45
  }
}
```

### GET /api/admin/v1/organizations/{orgId}/groups/{groupId}

Response (200):
```json
{
  "group": {
    "id": "field-workers",
    "name": "Field Workers",
    "slug": "field-workers",
    "description": "Field team devices",
    "iconEmoji": "👷",
    "maxDevices": 50,
    "memberCount": 8,
    "deviceCount": 15,
    "isActive": true,
    "createdBy": "123e4567-e89b-12d3-a456-426614174000",
    "createdAt": "2025-11-01T09:00:00Z"
  },
  "members": [
    {
      "userId": "123e4567-e89b-12d3-a456-426614174000",
      "email": "manager@acme.com",
      "displayName": "Team Manager",
      "role": "owner",
      "joinedAt": "2025-11-01T09:00:00Z"
    }
  ],
  "devices": [
    {
      "id": 42,
      "deviceUuid": "550e8400-e29b-41d4-a716-446655440000",
      "displayName": "Field Tablet #42",
      "lastSeenAt": "2025-12-01T10:25:00Z"
    }
  ]
}
```

### PUT /api/admin/v1/organizations/{orgId}/groups/{groupId}

Request Body:
```json
{
  "name": "Field Workers Team",
  "description": "Updated description",
  "maxDevices": 100,
  "isActive": true
}
```

Response (200):
```json
{
  "id": "field-workers",
  "name": "Field Workers Team",
  "description": "Updated description",
  "maxDevices": 100,
  "isActive": true,
  "updatedAt": "2025-12-01T11:00:00Z"
}
```

### DELETE /api/admin/v1/organizations/{orgId}/groups/{groupId}

Response (200):
```json
{
  "deactivated": true,
  "groupId": "field-workers",
  "deactivatedAt": "2025-12-01T11:00:00Z"
}
```

---

## Implementation Tasks

- [x] Create domain models for admin group management
- [x] Add admin group repository methods
- [x] Create admin_groups.rs route handler file
- [x] Implement list groups endpoint with filtering
- [x] Implement get group detail endpoint
- [x] Implement update group endpoint
- [x] Implement deactivate group endpoint
- [x] Add role-based access control checks
- [x] Write unit tests

---

## Dev Notes

- Groups are linked to organizations indirectly via devices
- Query: groups where any device in the group belongs to the org
- Need to verify the group-org relationship exists
- Existing GroupRepository can be extended with admin methods

---

## Dev Agent Record

### Debug Log

- Created admin group domain models with filtering, sorting, and pagination support
- Created AdminGroupRepository with summary statistics, list, and detail queries
- Groups are linked to organizations via group_memberships JOIN org_users
- Used CTE (WITH org_groups AS) to find groups belonging to org via user memberships
- Devices table uses group slug as group_id (VARCHAR), not UUID
- Route endpoints use JWT auth via UserAuth extractor

### Completion Notes

- Admin group management provides advanced filtering by active status, has_devices, and search
- Group detail includes members with roles and devices with last seen times
- Role-based access control: only org admins and owners can access endpoints
- Uses `/api/admin/v1/organizations/{orgId}/groups` path
- Deactivate endpoint soft-deletes by setting is_active=false

---

## File List

- `crates/domain/src/models/admin_group.rs` - Domain models for admin group management
- `crates/domain/src/models/mod.rs` - Export admin group models
- `crates/persistence/src/entities/admin_group.rs` - Database entity mappings
- `crates/persistence/src/entities/mod.rs` - Export admin group entities
- `crates/persistence/src/repositories/admin_group.rs` - Admin group repository
- `crates/persistence/src/repositories/mod.rs` - Export AdminGroupRepository
- `crates/api/src/routes/admin_groups.rs` - Admin group route handlers
- `crates/api/src/routes/mod.rs` - Export admin_groups module
- `crates/api/src/app.rs` - Added admin-groups route nest

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

