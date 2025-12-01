# Story 13.7: Fleet Management Endpoints

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: To Do
**Created**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to manage all devices in my organization fleet
**So that** I can assign, suspend, retire, and monitor devices centrally

## Prerequisites

- Story 13.5 complete (Device enrollment)
- Story 13.2 complete (Organization users)

## Acceptance Criteria

1. GET `/api/admin/v1/organizations/{orgId}/devices` lists all org devices with filtering
2. POST `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/assign` assigns user to device
3. POST `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/unassign` removes user assignment
4. POST `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/suspend` suspends device (blocks API access)
5. POST `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/retire` retires device (permanent)
6. POST `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/wipe` triggers remote wipe command
7. List endpoint supports: pagination, filtering by status/group/policy/assigned, search by name/UUID, sorting
8. Response includes summary counts by status
9. All fleet operations create audit log entries

## Technical Notes

- Add assigned_user_id column to devices table (FK to users)
- Remote wipe stores pending command in device_commands table (new)
- Suspended devices receive 403 on API calls
- Retired devices cannot be re-activated

## API Specification

### GET /api/admin/v1/organizations/{orgId}/devices

Query Parameters:
- page, per_page: pagination
- status: enrolled, pending, suspended, retired
- group_id: filter by group
- policy_id: filter by policy
- assigned: true/false for assigned/unassigned
- search: name or UUID
- sort: last_seen_at, display_name, created_at
- order: asc, desc

Response (200):
```json
{
  "data": [
    {
      "id": "uuid",
      "device_uuid": "550e8400-...",
      "display_name": "Field Tablet #42",
      "platform": "android",
      "enrollment_status": "enrolled",
      "is_managed": true,
      "assigned_user": {
        "id": "user_uuid",
        "email": "john@acme.com",
        "display_name": "John Smith"
      },
      "group": { "id": "...", "name": "..." },
      "policy": { "id": "...", "name": "..." },
      "last_seen_at": "timestamp",
      "last_location": { "latitude": 0.0, "longitude": 0.0 }
    }
  ],
  "pagination": { "page": 1, "per_page": 50, "total": 45, "total_pages": 1 },
  "summary": {
    "enrolled": 40,
    "pending": 3,
    "suspended": 1,
    "retired": 1,
    "assigned": 38,
    "unassigned": 7
  }
}
```

### POST /api/admin/v1/organizations/{orgId}/devices/{deviceId}/assign

Request:
```json
{
  "user_id": "user_uuid",
  "notify_user": true
}
```

Response (200):
```json
{
  "device_id": "uuid",
  "assigned_user": { "id": "...", "email": "...", "display_name": "..." },
  "assigned_at": "timestamp",
  "notification_sent": true
}
```

---

## Implementation Tasks

- [ ] Add assigned_user_id column to devices table
- [ ] Create device_commands table for pending commands
- [ ] Create FleetManagementService in domain layer
- [ ] Implement device list endpoint with filtering
- [ ] Implement assign/unassign endpoints
- [ ] Implement suspend/retire endpoints with status validation
- [ ] Implement wipe endpoint with command queue
- [ ] Add middleware to block suspended devices
- [ ] Add audit logging for all fleet operations
- [ ] Write unit tests for fleet operations
- [ ] Write integration tests for endpoints

---

## Dev Notes

- Fleet view is organization-scoped only
- Wipe command stored until device polls for commands
- Consider push notification to trigger immediate check-in
- Summary stats help dashboard display fleet health

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

