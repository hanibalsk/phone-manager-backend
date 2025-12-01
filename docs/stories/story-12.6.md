# Story 12.6: Unlock Request Workflow

**Epic**: Epic 12 - Settings Control
**Status**: In Progress
**Created**: 2025-12-01

---

## User Story

**As a** device user
**I want** to request unlocking of a locked setting
**So that** I can ask admin permission to modify restricted settings

## Prerequisites

- Story 12.4 complete (Lock/unlock settings)
- Group membership and RBAC operational

## Acceptance Criteria

1. `POST /api/v1/devices/:device_id/settings/:key/unlock-request` creates request
2. `GET /api/v1/groups/:group_id/unlock-requests` lists pending requests for group
3. `PUT /api/v1/unlock-requests/:request_id` approves or denies request
4. Request includes reason from user
5. Only device owner can create request
6. Only group admin/owner can approve/deny
7. Approved request automatically unlocks the setting
8. Status enum: pending, approved, denied, expired
9. Requests expire after 7 days
10. Returns 409 if pending request already exists for same setting

## Technical Notes

- Create `unlock_requests` table in migration
- Routes:
  - `POST /api/v1/devices/:device_id/settings/:key/unlock-request`
  - `GET /api/v1/groups/:group_id/unlock-requests?status=pending`
  - `PUT /api/v1/unlock-requests/:request_id`
- Approval triggers automatic unlock

## Implementation Tasks

- [ ] Create migration 022_unlock_requests.sql
- [ ] Create UnlockRequest entity
- [ ] Create UnlockRequestRepository
- [ ] Create request DTOs (create, list, update)
- [ ] Add create_unlock_request handler
- [ ] Add list_unlock_requests handler
- [ ] Add respond_to_unlock_request handler
- [ ] Add routes to app.rs
- [ ] Add unit tests

## API Request Example (Create)

```json
{
  "reason": "I need to change the tracking interval for battery saving"
}
```

## API Response Example (List)

```json
{
  "data": [
    {
      "id": "req_01...",
      "device": {
        "id": "dev_01...",
        "displayName": "John's Phone"
      },
      "settingKey": "tracking_interval_minutes",
      "settingDisplayName": "Tracking Interval",
      "status": "pending",
      "requestedBy": {
        "id": "user_01...",
        "displayName": "John Doe"
      },
      "reason": "Battery saving",
      "createdAt": "2025-12-01T10:35:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "perPage": 20,
    "total": 1
  }
}
```

## API Request Example (Respond)

```json
{
  "status": "approved",
  "note": "Approved for battery saving purposes"
}
```

---

## Dev Notes

- Only one pending request per device+setting allowed
- Approval side-effect: unlock the setting
- Push notification support deferred to Story 12.8

---

## Dev Agent Record

### Debug Log


### Completion Notes


---

## File List

- crates/persistence/src/migrations/022_unlock_requests.sql
- crates/persistence/src/entities/unlock_request.rs
- crates/persistence/src/entities/mod.rs
- crates/persistence/src/repositories/unlock_request.rs
- crates/persistence/src/repositories/mod.rs
- crates/domain/src/models/unlock_request.rs
- crates/domain/src/models/mod.rs
- crates/api/src/routes/device_settings.rs
- crates/api/src/app.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

