# Device Settings API Specification

## Phone Manager Backend - User Management & Device Control

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-01

---

## 1. Overview

This document defines the API endpoints for device registration with authentication, device settings management, setting locks, and unlock requests.

---

## 2. Device Registration Endpoints

### 2.1 Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/v1/devices/register` | Register device | Optional |
| GET | `/api/v1/devices/{deviceId}` | Get device details | Yes |
| PUT | `/api/v1/devices/{deviceId}` | Update device | Yes |
| DELETE | `/api/v1/devices/{deviceId}` | Deactivate device | Yes |

### 2.2 POST /api/v1/devices/register

Register a new device or update existing registration.

**Authentication:** Optional (Bearer Token or X-API-Key for backward compatibility)

#### Request

```json
{
  "device_uuid": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "John's Pixel 8",
  "platform": "android",
  "device_info": {
    "manufacturer": "Google",
    "model": "Pixel 8",
    "os_version": "Android 14",
    "app_version": "2.0.0"
  },
  "fcm_token": "fMc123...",
  "group_id": "my-family"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| device_uuid | string (UUID) | Yes | Hardware device identifier |
| display_name | string | Yes | Human-readable name |
| platform | string | Yes | "android" or "ios" |
| device_info | object | No | Device metadata |
| fcm_token | string | No | Firebase Cloud Messaging token |
| group_id | string | No | Legacy group ID (backward compat) |

#### Response (200 OK)

```json
{
  "id": "dev_01HXYZABC123",
  "device_uuid": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "John's Pixel 8",
  "platform": "android",
  "owner_user_id": "user_01HXYZABC123",
  "organization_id": null,
  "is_managed": false,
  "enrollment_status": "enrolled",
  "is_active": true,
  "created_at": "2025-12-01T10:30:00Z",
  "updated_at": "2025-12-01T10:30:00Z"
}
```

**Behavior:**
- If authenticated: Device linked to user
- If not authenticated: Device created without owner (legacy mode)
- If device_uuid exists: Update existing registration

---

## 3. Device Settings Endpoints

### 3.1 Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/v1/devices/{deviceId}/settings` | Get all settings | Yes |
| PUT | `/api/v1/devices/{deviceId}/settings` | Update settings | Yes |
| GET | `/api/v1/devices/{deviceId}/settings/{key}` | Get single setting | Yes |
| PUT | `/api/v1/devices/{deviceId}/settings/{key}` | Update single setting | Yes |

### 3.2 GET /api/v1/devices/{deviceId}/settings

Get all device settings with lock status.

**Authentication:** Bearer Token or Device Token
**Authorization:** Device owner, group admin, or org admin

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| include_definitions | boolean | false | Include setting definitions |

#### Response (200 OK)

```json
{
  "device_id": "dev_01HXYZABC123",
  "settings": {
    "tracking_enabled": {
      "value": true,
      "is_locked": false,
      "updated_at": "2025-12-01T10:30:00Z",
      "updated_by": null
    },
    "tracking_interval_minutes": {
      "value": 5,
      "is_locked": true,
      "locked_by": "user_01ADMIN456",
      "locked_at": "2025-12-01T09:00:00Z",
      "lock_reason": "Company policy",
      "updated_at": "2025-12-01T09:00:00Z",
      "updated_by": "user_01ADMIN456"
    },
    "secret_mode_enabled": {
      "value": false,
      "is_locked": true,
      "locked_by": "user_01ADMIN456",
      "locked_at": "2025-12-01T09:00:00Z",
      "lock_reason": "Privacy policy"
    }
  },
  "policy_id": null,
  "managed_by_group_id": "grp_01ABC",
  "last_synced_at": "2025-12-01T10:30:00Z",
  "definitions": [
    {
      "key": "tracking_enabled",
      "display_name": "Location Tracking",
      "description": "Enable or disable location tracking",
      "data_type": "boolean",
      "default_value": true,
      "is_lockable": true,
      "category": "tracking"
    }
  ]
}
```

---

### 3.3 PUT /api/v1/devices/{deviceId}/settings

Update multiple device settings.

**Authentication:** Bearer Token or Device Token
**Authorization:** Device owner (unlocked settings), group admin (all settings)

#### Request

```json
{
  "settings": {
    "tracking_enabled": true,
    "tracking_interval_minutes": 10,
    "movement_detection_enabled": true
  }
}
```

#### Response (200 OK)

```json
{
  "updated": ["tracking_enabled", "movement_detection_enabled"],
  "locked": ["tracking_interval_minutes"],
  "settings": {
    "tracking_enabled": {
      "value": true,
      "is_locked": false,
      "updated_at": "2025-12-01T10:35:00Z"
    },
    "tracking_interval_minutes": {
      "value": 5,
      "is_locked": true,
      "error": "Setting is locked by admin"
    },
    "movement_detection_enabled": {
      "value": true,
      "is_locked": false,
      "updated_at": "2025-12-01T10:35:00Z"
    }
  }
}
```

**Note:** Locked settings are silently skipped for regular users. Admin override required.

---

### 3.4 GET /api/v1/devices/{deviceId}/settings/{key}

Get single setting value.

**Authentication:** Bearer Token or Device Token
**Authorization:** Device owner or group member

#### Response (200 OK)

```json
{
  "key": "tracking_interval_minutes",
  "value": 5,
  "is_locked": true,
  "locked_by": "user_01ADMIN456",
  "locked_at": "2025-12-01T09:00:00Z",
  "lock_reason": "Company policy",
  "updated_at": "2025-12-01T09:00:00Z",
  "definition": {
    "display_name": "Tracking Interval",
    "description": "Minutes between location updates",
    "data_type": "integer",
    "default_value": 5,
    "is_lockable": true,
    "category": "tracking"
  }
}
```

---

### 3.5 PUT /api/v1/devices/{deviceId}/settings/{key}

Update single setting.

**Authentication:** Bearer Token
**Authorization:** Device owner (if unlocked), group admin (any)

#### Request

```json
{
  "value": 10
}
```

#### Response (200 OK)

```json
{
  "key": "tracking_interval_minutes",
  "value": 10,
  "is_locked": false,
  "updated_at": "2025-12-01T10:35:00Z",
  "updated_by": "user_01HXYZABC123"
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 403 | authz/setting-locked | Setting is locked |
| 400 | validation/invalid-value | Value doesn't match data type |

---

## 4. Setting Lock Endpoints

### 4.1 Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/v1/devices/{deviceId}/settings/locks` | Get all locks | Yes |
| PUT | `/api/v1/devices/{deviceId}/settings/locks` | Update locks | Yes |
| POST | `/api/v1/devices/{deviceId}/settings/{key}/lock` | Lock setting | Yes |
| DELETE | `/api/v1/devices/{deviceId}/settings/{key}/lock` | Unlock setting | Yes |

### 4.2 GET /api/v1/devices/{deviceId}/settings/locks

Get all locked settings.

**Authentication:** Bearer Token
**Authorization:** Group member (view), admin (manage)

#### Response (200 OK)

```json
{
  "device_id": "dev_01HXYZABC123",
  "locks": [
    {
      "key": "tracking_interval_minutes",
      "is_locked": true,
      "locked_by": {
        "id": "user_01ADMIN456",
        "display_name": "Admin User"
      },
      "locked_at": "2025-12-01T09:00:00Z",
      "reason": "Company policy"
    },
    {
      "key": "secret_mode_enabled",
      "is_locked": true,
      "locked_by": {
        "id": "user_01ADMIN456",
        "display_name": "Admin User"
      },
      "locked_at": "2025-12-01T09:00:00Z",
      "reason": "Privacy policy"
    }
  ],
  "locked_count": 2,
  "total_lockable": 8
}
```

---

### 4.3 PUT /api/v1/devices/{deviceId}/settings/locks

Update multiple setting locks.

**Authentication:** Bearer Token
**Authorization:** Group admin or org admin

#### Request

```json
{
  "locks": {
    "tracking_enabled": true,
    "tracking_interval_minutes": true,
    "secret_mode_enabled": true,
    "movement_detection_enabled": false
  },
  "reason": "Updated security policy",
  "notify_user": true
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| locks | object | Yes | Setting key -> is_locked |
| reason | string | No | Lock/unlock reason |
| notify_user | boolean | No | Send push notification |

#### Response (200 OK)

```json
{
  "updated": [
    {
      "key": "tracking_enabled",
      "is_locked": true,
      "locked_at": "2025-12-01T10:35:00Z"
    },
    {
      "key": "movement_detection_enabled",
      "is_locked": false,
      "unlocked_at": "2025-12-01T10:35:00Z"
    }
  ],
  "notification_sent": true
}
```

---

### 4.4 POST /api/v1/devices/{deviceId}/settings/{key}/lock

Lock a specific setting.

**Authentication:** Bearer Token
**Authorization:** Group admin or org admin

#### Request

```json
{
  "reason": "Company policy requires this setting",
  "value": 5,
  "notify_user": true
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| reason | string | No | Lock reason |
| value | any | No | Force value when locking |
| notify_user | boolean | No | Send push notification |

#### Response (200 OK)

```json
{
  "key": "tracking_interval_minutes",
  "is_locked": true,
  "value": 5,
  "locked_by": "user_01ADMIN456",
  "locked_at": "2025-12-01T10:35:00Z",
  "reason": "Company policy requires this setting"
}
```

---

### 4.5 DELETE /api/v1/devices/{deviceId}/settings/{key}/lock

Unlock a specific setting.

**Authentication:** Bearer Token
**Authorization:** Group admin or org admin

#### Response (200 OK)

```json
{
  "key": "tracking_interval_minutes",
  "is_locked": false,
  "unlocked_by": "user_01ADMIN456",
  "unlocked_at": "2025-12-01T10:35:00Z"
}
```

---

## 5. Unlock Request Endpoints

### 5.1 Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/v1/devices/{deviceId}/settings/{key}/unlock-request` | Request unlock | Yes |
| GET | `/api/v1/groups/{groupId}/unlock-requests` | List pending requests | Yes |
| PUT | `/api/v1/unlock-requests/{requestId}` | Approve/deny request | Yes |

### 5.2 POST /api/v1/devices/{deviceId}/settings/{key}/unlock-request

Request to unlock a locked setting.

**Authentication:** Bearer Token
**Authorization:** Device owner

#### Request

```json
{
  "reason": "I need to change the tracking interval for battery saving"
}
```

#### Response (201 Created)

```json
{
  "id": "req_01HXYZABC123",
  "device_id": "dev_01HXYZABC123",
  "setting_key": "tracking_interval_minutes",
  "status": "pending",
  "requested_by": {
    "id": "user_01HXYZABC123",
    "display_name": "John Doe"
  },
  "reason": "I need to change the tracking interval for battery saving",
  "created_at": "2025-12-01T10:35:00Z"
}
```

---

### 5.3 GET /api/v1/groups/{groupId}/unlock-requests

List pending unlock requests for group.

**Authentication:** Bearer Token
**Authorization:** Group admin or owner

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| status | string | pending | Filter by status |
| page | integer | 1 | Page number |
| per_page | integer | 20 | Items per page |

#### Response (200 OK)

```json
{
  "data": [
    {
      "id": "req_01HXYZABC123",
      "device": {
        "id": "dev_01HXYZABC123",
        "display_name": "John's Phone"
      },
      "setting_key": "tracking_interval_minutes",
      "setting_display_name": "Tracking Interval",
      "status": "pending",
      "requested_by": {
        "id": "user_01HXYZABC123",
        "display_name": "John Doe"
      },
      "reason": "I need to change the tracking interval for battery saving",
      "created_at": "2025-12-01T10:35:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 1,
    "total_pages": 1
  }
}
```

---

### 5.4 PUT /api/v1/unlock-requests/{requestId}

Approve or deny unlock request.

**Authentication:** Bearer Token
**Authorization:** Group admin or owner

#### Request

```json
{
  "status": "approved",
  "note": "Approved for battery saving purposes"
}
```

| Field | Type | Required | Values |
|-------|------|----------|--------|
| status | string | Yes | approved, denied |
| note | string | No | Response note |

#### Response (200 OK)

```json
{
  "id": "req_01HXYZABC123",
  "status": "approved",
  "decided_by": {
    "id": "user_01ADMIN456",
    "display_name": "Admin User"
  },
  "decided_at": "2025-12-01T10:40:00Z",
  "decision_note": "Approved for battery saving purposes",
  "setting_unlocked": true
}
```

**Side Effects:**
- If approved: Setting is automatically unlocked
- Notification sent to requesting user

---

## 6. Settings Sync Endpoint

### POST /api/v1/devices/{deviceId}/settings/sync

Force sync settings from server.

**Authentication:** Bearer Token or Device Token
**Authorization:** Device owner

#### Response (200 OK)

```json
{
  "synced_at": "2025-12-01T10:40:00Z",
  "settings": {
    "tracking_enabled": {
      "value": true,
      "is_locked": false
    },
    "tracking_interval_minutes": {
      "value": 5,
      "is_locked": true
    }
  },
  "changes_applied": [
    {
      "key": "tracking_interval_minutes",
      "old_value": 10,
      "new_value": 5,
      "reason": "Admin changed value"
    }
  ]
}
```

---

## 7. Push Notification Events

### Settings Changed

When an admin changes settings or locks:

```json
{
  "type": "settings_changed",
  "device_id": "dev_01HXYZABC123",
  "changes": [
    {
      "key": "tracking_interval_minutes",
      "action": "locked",
      "new_value": 5
    }
  ],
  "changed_by": "Admin User",
  "timestamp": "2025-12-01T10:35:00Z"
}
```

### Unlock Request Response

When admin responds to unlock request:

```json
{
  "type": "unlock_request_response",
  "request_id": "req_01HXYZABC123",
  "setting_key": "tracking_interval_minutes",
  "status": "approved",
  "note": "Approved for battery saving",
  "decided_by": "Admin User",
  "timestamp": "2025-12-01T10:40:00Z"
}
```

---

## 8. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-01 | Initial specification |
