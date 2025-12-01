# Story 14.7: Organization Usage Statistics Endpoint

**Epic**: Epic 14 - Admin Portal Backend
**Status**: Complete (Already Implemented)
**Completed**: 2025-12-01 (via Epic 13)
**Created**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to view usage statistics for my organization
**So that** I can monitor resource consumption and plan capacity

## Prerequisites

- Story 13.1 complete (Organizations)
- Story 13.2 complete (Organization users)

## Acceptance Criteria

1. GET `/api/admin/v1/organizations/{orgId}/usage` returns usage statistics
2. Response includes device counts, user counts, storage usage
3. Response includes usage metrics by device status
4. Only org admins and owners can access

## Technical Notes

- Already implemented as part of Epic 13
- Uses aggregation queries across devices, org_users, and related tables
- Returns comprehensive usage breakdown

## API Specification

### GET /api/admin/v1/organizations/{orgId}/usage

Response (200):
```json
{
  "organizationId": "uuid",
  "plan": "enterprise",
  "limits": {
    "maxDevices": 1000,
    "maxUsers": 100,
    "maxPolicies": 50,
    "maxGroups": 100
  },
  "currentUsage": {
    "devices": 150,
    "users": 25,
    "policies": 5,
    "groups": 10
  },
  "devicesByStatus": {
    "active": 120,
    "inactive": 20,
    "pending": 10
  },
  "usersByRole": {
    "owner": 1,
    "admin": 5,
    "member": 19
  },
  "lastUpdated": "2025-12-01T00:00:00Z"
}
```

---

## Implementation Tasks

- [x] Organization usage repository method (completed in Epic 13)
- [x] Usage statistics aggregation queries
- [x] Route handler with RBAC
- [x] Route registered in app.rs

---

## Dev Notes

- This story was already fully implemented as part of Epic 13
- The `get_organization_usage` endpoint returns `OrganizationUsageResponse`
- Includes device, user, policy, and group metrics

---

## Dev Agent Record

### Completion Notes

- No additional implementation needed
- Epic 13 already includes the organization usage endpoint
- Endpoint: GET `/api/admin/v1/organizations/:org_id/usage`

---

## File List

- `crates/api/src/routes/organizations.rs` - Contains `get_organization_usage` handler
- `crates/domain/src/models/organization.rs` - Contains `OrganizationUsageResponse` model
- `crates/persistence/src/repositories/organization.rs` - Contains usage aggregation methods

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Marked complete - already implemented in Epic 13 |
