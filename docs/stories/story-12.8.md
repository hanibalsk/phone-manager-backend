# Story 12.8: Push Notification Integration for Settings

**Epic**: Epic 12 - Settings Control
**Status**: In Progress
**Created**: 2025-12-01

---

## User Story

**As a** device user
**I want** to receive notifications when my settings change
**So that** I am aware of configuration changes made by admins

## Prerequisites

- Story 12.7 complete (Settings sync)
- FCM token stored for devices
- Push notification infrastructure (basic)

## Acceptance Criteria

1. Settings changed by admin triggers push notification to device
2. Lock/unlock operations can trigger notification
3. Unlock request response triggers notification to requester
4. Notification payload includes setting key and change type
5. `notify_user` parameter controls whether to send notification
6. Notification includes enough info for device to sync
7. Uses existing FCM token from device registration
8. Gracefully handles missing FCM token (no error, just skip)

## Technical Notes

- Integrate with existing FCM infrastructure if available
- Create notification service/module
- Fire-and-forget notifications (don't block API response)
- Notification types: settings_changed, unlock_request_response

## Implementation Tasks

- [ ] Create NotificationService trait and implementation
- [ ] Create notification payload DTOs
- [ ] Add FCM HTTP client (or mock for initial implementation)
- [ ] Integrate notification calls in settings handlers
- [ ] Integrate notification calls in unlock request handlers
- [ ] Add unit tests with mock notification service
- [ ] Document notification payload format

## Notification Payload: Settings Changed

```json
{
  "type": "settings_changed",
  "deviceId": "dev_01...",
  "changes": [
    {
      "key": "tracking_interval_minutes",
      "action": "locked",
      "newValue": 5
    }
  ],
  "changedBy": "Admin User",
  "timestamp": "2025-12-01T10:35:00Z"
}
```

## Notification Payload: Unlock Request Response

```json
{
  "type": "unlock_request_response",
  "requestId": "req_01...",
  "settingKey": "tracking_interval_minutes",
  "status": "approved",
  "note": "Approved for battery saving",
  "decidedBy": "Admin User",
  "timestamp": "2025-12-01T10:40:00Z"
}
```

---

## Dev Notes

- Initial implementation can be a mock/stub if FCM credentials not available
- Notifications are best-effort, failures logged but don't affect API response
- Device should trigger sync when receiving notification

---

## Dev Agent Record

### Debug Log


### Completion Notes


---

## File List

- crates/domain/src/services/notification.rs (new)
- crates/domain/src/services/mod.rs
- crates/api/src/routes/device_settings.rs (integrate)
- crates/shared/src/notification.rs (optional FCM client)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

