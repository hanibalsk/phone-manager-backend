# Story 13.3: Device Policies Table and CRUD Endpoints

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: To Do
**Created**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to create and manage device policies
**So that** I can define standard configurations and locks for device groups

## Prerequisites

- Story 13.1 complete (Organizations)
- Story 12.1 complete (Device settings schema)

## Acceptance Criteria

1. Migration creates `device_policies` table: id (UUID), organization_id (FK), name, description, is_default, settings (JSONB), locked_settings (TEXT[]), priority, device_count, created_at, updated_at
2. POST `/api/admin/v1/organizations/{orgId}/policies` creates new policy
3. GET `/api/admin/v1/organizations/{orgId}/policies` lists policies with device counts
4. GET `/api/admin/v1/organizations/{orgId}/policies/{policyId}` returns policy details
5. PUT `/api/admin/v1/organizations/{orgId}/policies/{policyId}` updates policy
6. DELETE `/api/admin/v1/organizations/{orgId}/policies/{policyId}` deletes policy (only if no devices assigned)
7. POST `/api/admin/v1/organizations/{orgId}/policies/{policyId}/apply` applies policy to devices/groups
8. Only one default policy per organization
9. Settings must reference valid setting_definitions keys
10. Higher priority policies take precedence in resolution

## Technical Notes

- Create migration 025_device_policies.sql
- policies.settings JSONB stores setting key-value pairs
- locked_settings is TEXT array of setting keys that cannot be modified
- device_count is a denormalized count, updated on policy assignment
- Priority: higher number = higher precedence

## API Specification

### POST /api/admin/v1/organizations/{orgId}/policies

Request:
```json
{
  "name": "Field Worker Standard",
  "description": "Standard policy for field workers",
  "is_default": false,
  "settings": {
    "tracking_enabled": true,
    "tracking_interval_minutes": 5,
    "secret_mode_enabled": false
  },
  "locked_settings": ["tracking_enabled", "secret_mode_enabled"],
  "priority": 10
}
```

Response (201):
```json
{
  "id": "uuid",
  "organization_id": "org_uuid",
  "name": "Field Worker Standard",
  "description": "Standard policy for field workers",
  "is_default": false,
  "settings": {...},
  "locked_settings": [...],
  "priority": 10,
  "device_count": 0,
  "created_at": "timestamp"
}
```

### POST /api/admin/v1/organizations/{orgId}/policies/{policyId}/apply

Request:
```json
{
  "targets": [
    { "type": "device", "id": "dev_uuid" },
    { "type": "group", "id": "grp_uuid" }
  ],
  "replace_existing": true
}
```

Response (200):
```json
{
  "policy_id": "uuid",
  "applied_to": {
    "devices": 25,
    "groups": 1
  },
  "total_devices_affected": 45
}
```

---

## Implementation Tasks

- [ ] Create migration 025_device_policies.sql with table and indexes
- [ ] Add policy_id column to devices table (FK, nullable)
- [ ] Create DevicePolicyEntity in persistence layer
- [ ] Create DevicePolicy domain model with validation
- [ ] Create DevicePolicyRepository with CRUD
- [ ] Create DevicePolicyService with apply logic
- [ ] Implement policy CRUD endpoints
- [ ] Implement policy apply endpoint
- [ ] Update device_count on policy changes
- [ ] Add audit logging for policy operations
- [ ] Write unit tests for validation
- [ ] Write integration tests for endpoints

---

## Dev Notes

- Policies define organization-wide device configurations
- locked_settings prevent users from changing those settings
- Policy application cascades: applying to group affects all group devices
- Consider triggering settings sync push after policy apply

---

## Dev Agent Record

### Debug Log


### Completion Notes


---

## File List


---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

