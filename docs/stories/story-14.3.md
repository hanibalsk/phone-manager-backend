# Story 14.3: User Management Endpoints

**Epic**: Epic 14 - Admin Portal Backend
**Status**: Complete
**Completed**: 2025-12-01
**Created**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to manage users within my organization through the admin API
**So that** I can add, update roles, and remove users while viewing their details and activity

## Prerequisites

- Story 14.2 complete (Device fleet list)
- Organization user infrastructure from Epic 13

## Acceptance Criteria

1. GET `/api/admin/v1/organizations/{orgId}/users` returns paginated user list
2. Each user includes: id, email, displayName, avatarUrl, role, permissions
3. Each user includes device count and group count statistics
4. Filtering supported by: role, search text, has_device
5. Sorting supported by: display_name, email, granted_at
6. Pagination with page/per_page params (default 50, max 100)
7. GET `/api/admin/v1/organizations/{orgId}/users/{userId}` returns user details
8. User detail includes: profile info, assigned devices, group memberships, activity summary
9. PUT `/api/admin/v1/organizations/{orgId}/users/{userId}` updates user role/permissions
10. DELETE `/api/admin/v1/organizations/{orgId}/users/{userId}` removes user from org
11. Only org admins and owners can access these endpoints
12. Owners cannot be removed or demoted by admins

## Technical Notes

- Create new admin-specific user management routes in `crates/api/src/routes/admin_users.rs`
- Extend OrgUserRepository with additional query methods
- Use LEFT JOINs to count devices and groups per user
- Include activity summary from audit logs (last 30 days)

## API Specification

### GET /api/admin/v1/organizations/{orgId}/users

Query Parameters:
- `page` (optional): Page number (default: 1)
- `perPage` (optional): Items per page (default: 50, max: 100)
- `role` (optional): Filter by role (owner, admin, member)
- `hasDevice` (optional): Filter by device assignment (true/false)
- `search` (optional): Search in email and display name
- `sort` (optional): Sort field (display_name, email, granted_at)
- `order` (optional): Sort order (asc, desc)

Response (200):
```json
{
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "john@acme.com",
      "display_name": "John Smith",
      "avatarUrl": "https://example.com/avatar.png",
      "role": "admin",
      "permissions": ["device:read", "device:manage", "user:read"],
      "deviceCount": 2,
      "groupCount": 3,
      "grantedAt": "2025-11-01T09:00:00Z",
      "lastLoginAt": "2025-12-01T08:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "perPage": 50,
    "total": 25,
    "totalPages": 1
  },
  "summary": {
    "owners": 1,
    "admins": 3,
    "members": 21,
    "withDevices": 18,
    "withoutDevices": 7
  }
}
```

### GET /api/admin/v1/organizations/{orgId}/users/{userId}

Response (200):
```json
{
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "john@acme.com",
    "display_name": "John Smith",
    "avatarUrl": "https://example.com/avatar.png",
    "emailVerified": true,
    "role": "admin",
    "permissions": ["device:read", "device:manage", "user:read"],
    "grantedAt": "2025-11-01T09:00:00Z",
    "grantedBy": "admin@acme.com",
    "lastLoginAt": "2025-12-01T08:00:00Z",
    "created_at": "2025-10-15T12:00:00Z"
  },
  "devices": [
    {
      "id": 42,
      "deviceUuid": "550e8400-e29b-41d4-a716-446655440000",
      "display_name": "Field Tablet #42",
      "platform": "android",
      "lastSeenAt": "2025-12-01T10:25:00Z"
    }
  ],
  "groups": [
    {
      "id": "field-workers",
      "name": "Field Workers",
      "role": "member"
    }
  ],
  "activitySummary": {
    "totalActions": 45,
    "lastActionAt": "2025-12-01T10:00:00Z",
    "recentActions": [
      {
        "action": "device.assign",
        "resourceType": "device",
        "timestamp": "2025-12-01T10:00:00Z"
      }
    ]
  }
}
```

### PUT /api/admin/v1/organizations/{orgId}/users/{userId}

Request Body:
```json
{
  "role": "member",
  "permissions": ["device:read", "user:read"]
}
```

Response (200):
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "email": "john@acme.com",
  "display_name": "John Smith",
  "role": "member",
  "permissions": ["device:read", "user:read"],
  "updated_at": "2025-12-01T11:00:00Z"
}
```

### DELETE /api/admin/v1/organizations/{orgId}/users/{userId}

Response (200):
```json
{
  "removed": true,
  "userId": "123e4567-e89b-12d3-a456-426614174000",
  "removedAt": "2025-12-01T11:00:00Z"
}
```

---

## Implementation Tasks

- [x] Create domain models for admin user management in domain crate
- [x] Add list_admin_users method to OrgUserRepository with joins
- [x] Add get_user_detail method with devices, groups, activity
- [x] Add count methods for user summary statistics
- [x] Create admin_users.rs route handler file
- [x] Implement list users endpoint with filtering and sorting
- [x] Implement get user detail endpoint
- [x] Implement update user role/permissions endpoint
- [x] Implement remove user from org endpoint
- [x] Add role-based access control checks
- [x] Write unit tests for query building logic

---

## Dev Notes

- Existing OrgUserRepository has basic CRUD operations
- Need to extend with aggregate counts and joined data
- Activity summary should query audit_logs table
- Must prevent owners from being demoted by admins
- Must prevent removing the last owner from an organization

---

## Dev Agent Record

### Debug Log

- Created admin_user domain models with filtering, sorting, and pagination support
- Created AdminUserRepository with summary statistics, list, and detail queries
- Used LEFT JOIN for devices and groups counts via correlated subqueries
- Activity summary queries audit_logs table for last 30 days
- Route endpoints use JWT auth via UserAuth extractor

### Completion Notes

- Admin user management provides advanced filtering by role, has_device, and search
- User detail includes assigned devices, group memberships, and activity summary
- Role-based access control prevents admins from modifying owners
- Last owner protection prevents removing the last owner from organization
- Uses `/api/admin/v1/organizations/{orgId}/admin-users` path to avoid conflict with existing `/users` routes

---

## File List

- `crates/domain/src/models/admin_user.rs` - Domain models for admin user management
- `crates/domain/src/models/mod.rs` - Export admin user models
- `crates/persistence/src/entities/admin_user.rs` - Database entity mappings
- `crates/persistence/src/entities/mod.rs` - Export admin user entities
- `crates/persistence/src/repositories/admin_user.rs` - Admin user repository
- `crates/persistence/src/repositories/mod.rs` - Export AdminUserRepository
- `crates/api/src/routes/admin_users.rs` - Admin user route handlers
- `crates/api/src/routes/mod.rs` - Export admin_users module
- `crates/api/src/app.rs` - Added admin-users route nest

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete |
| 2025-12-01 | Senior developer review: APPROVED |

---

## Senior Developer Review

**Reviewer**: Martin Janci
**Date**: 2025-12-01
**Outcome**: ✅ APPROVED

### Summary
User management endpoints implementation meets all acceptance criteria with proper RBAC controls and comprehensive filtering.

### Findings
- **Positive**: Owner protection logic (cannot be removed/demoted by admins)
- **Positive**: Last owner protection prevents orphaned organizations
- **Positive**: Activity summary from audit logs (last 30 days)
- **Positive**: Correlated subqueries for device/group counts
- **Note**: Uses `/admin-users` path to avoid route conflicts

### Acceptance Criteria Verification
| AC | Status |
|----|--------|
| Paginated user list endpoint | ✅ |
| User details (email, displayName, role, permissions) | ✅ |
| Device and group count statistics | ✅ |
| Filtering (role, search, has_device) | ✅ |
| Sorting (display_name, email, granted_at) | ✅ |
| Pagination (page/perPage) | ✅ |
| User detail endpoint | ✅ |
| User detail with devices, groups, activity | ✅ |
| Update user role/permissions | ✅ |
| Remove user from org | ✅ |
| Admin/Owner access only | ✅ |
| Owner protection | ✅ |

### Security
- JWT authentication enforced
- Organization isolation verified
- Role hierarchy enforced (admins cannot modify owners)
- Last owner protection implemented

### Action Items
None
