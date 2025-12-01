# B2B Enterprise Specification

## Phone Manager Backend - User Management & Device Control

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-01

---

## 1. Overview

This document defines the B2B enterprise features including organization management, device provisioning, policy engine, fleet management, and audit logging.

---

## 2. Organization Management

### 2.1 Organization Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/admin/v1/organizations` | Create organization | Yes (Platform Admin) |
| GET | `/api/admin/v1/organizations/{orgId}` | Get organization | Yes (Org Admin) |
| PUT | `/api/admin/v1/organizations/{orgId}` | Update organization | Yes (Org Admin) |
| DELETE | `/api/admin/v1/organizations/{orgId}` | Delete organization | Yes (Org Owner) |
| GET | `/api/admin/v1/organizations/{orgId}/usage` | Get usage stats | Yes (Org Admin) |

### 2.2 POST /api/admin/v1/organizations

Create a new organization.

#### Request

```json
{
  "name": "Acme Corporation",
  "slug": "acme-corp",
  "billing_email": "billing@acme.com",
  "plan_type": "business",
  "settings": {
    "default_tracking_interval": 5,
    "require_device_approval": true
  }
}
```

#### Response (201 Created)

```json
{
  "id": "org_01HXYZABC123",
  "name": "Acme Corporation",
  "slug": "acme-corp",
  "billing_email": "billing@acme.com",
  "plan_type": "business",
  "max_users": 50,
  "max_devices": 200,
  "max_groups": 20,
  "settings": {
    "default_tracking_interval": 5,
    "require_device_approval": true
  },
  "is_active": true,
  "created_at": "2025-12-01T10:30:00Z"
}
```

---

### 2.3 GET /api/admin/v1/organizations/{orgId}/usage

Get organization usage statistics.

#### Response (200 OK)

```json
{
  "organization_id": "org_01HXYZABC123",
  "users": {
    "current": 25,
    "max": 50,
    "percentage": 50
  },
  "devices": {
    "current": 45,
    "max": 200,
    "percentage": 22.5,
    "by_status": {
      "enrolled": 40,
      "pending": 3,
      "suspended": 1,
      "retired": 1
    }
  },
  "groups": {
    "current": 5,
    "max": 20,
    "percentage": 25
  },
  "locations_this_month": 125000,
  "api_calls_this_month": 450000,
  "storage_used_mb": 2500,
  "period": "2025-12"
}
```

---

## 3. Organization User Management

### 3.1 Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/admin/v1/organizations/{orgId}/users` | Add org user | Yes |
| GET | `/api/admin/v1/organizations/{orgId}/users` | List org users | Yes |
| PUT | `/api/admin/v1/organizations/{orgId}/users/{userId}` | Update org user | Yes |
| DELETE | `/api/admin/v1/organizations/{orgId}/users/{userId}` | Remove org user | Yes |

### 3.2 POST /api/admin/v1/organizations/{orgId}/users

Add user as organization administrator.

#### Request

```json
{
  "email": "admin@acme.com",
  "role": "admin",
  "permissions": ["device:manage", "user:manage", "policy:manage"]
}
```

#### Response (201 Created)

```json
{
  "id": "orguser_01HXYZABC123",
  "organization_id": "org_01HXYZABC123",
  "user": {
    "id": "user_01ADMIN456",
    "email": "admin@acme.com",
    "display_name": "Admin User"
  },
  "role": "admin",
  "permissions": ["device:manage", "user:manage", "policy:manage"],
  "granted_at": "2025-12-01T10:30:00Z"
}
```

---

## 4. Device Provisioning

### 4.1 Enrollment Token Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/admin/v1/organizations/{orgId}/enrollment-tokens` | Create token | Yes |
| GET | `/api/admin/v1/organizations/{orgId}/enrollment-tokens` | List tokens | Yes |
| DELETE | `/api/admin/v1/organizations/{orgId}/enrollment-tokens/{tokenId}` | Revoke token | Yes |
| GET | `/api/admin/v1/organizations/{orgId}/enrollment-tokens/{tokenId}/qr` | Get QR code | Yes |

### 4.2 POST /api/admin/v1/organizations/{orgId}/enrollment-tokens

Create device enrollment token.

#### Request

```json
{
  "group_id": "grp_01FIELD789",
  "policy_id": "pol_01STANDARD123",
  "max_uses": 50,
  "expires_in_days": 30,
  "auto_assign_user_by_email": true
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| group_id | string | No | Auto-assign to group |
| policy_id | string | No | Auto-apply policy |
| max_uses | integer | No | Max devices (default: 1) |
| expires_in_days | integer | No | Days until expiry (default: 7) |
| auto_assign_user_by_email | boolean | No | Match user by device email |

#### Response (201 Created)

```json
{
  "id": "enroll_01HXYZABC123",
  "token": "enroll_abc123xyz789...",
  "token_prefix": "enroll_a",
  "organization_id": "org_01HXYZABC123",
  "group_id": "grp_01FIELD789",
  "policy_id": "pol_01STANDARD123",
  "max_uses": 50,
  "current_uses": 0,
  "expires_at": "2025-12-31T10:30:00Z",
  "created_at": "2025-12-01T10:30:00Z",
  "qr_code_url": "/api/admin/v1/organizations/org_01.../enrollment-tokens/enroll_01.../qr"
}
```

---

### 4.3 Device Enrollment Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DEVICE ENROLLMENT FLOW                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Admin creates enrollment token                                              │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────┐                                                        │
│  │  QR Code / Token │  ──────►  Shared with IT / Employee                   │
│  └────────┬────────┘                                                        │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │  Device scans QR or enters token                                 │       │
│  │  POST /api/v1/devices/enroll                                     │       │
│  │  {                                                               │       │
│  │    "enrollment_token": "enroll_abc123...",                       │       │
│  │    "device_uuid": "550e8400-...",                                │       │
│  │    "device_info": { ... }                                        │       │
│  │  }                                                               │       │
│  └────────┬────────────────────────────────────────────────────────┘       │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │  Server validates token and creates device                       │       │
│  │  - Assigns to organization                                       │       │
│  │  - Applies policy                                                │       │
│  │  - Adds to group (if specified)                                  │       │
│  │  - Generates device token                                        │       │
│  └────────┬────────────────────────────────────────────────────────┘       │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │  Response:                                                       │       │
│  │  {                                                               │       │
│  │    "device_id": "dev_01...",                                     │       │
│  │    "device_token": "dt_abc123...",                               │       │
│  │    "policy": { settings, locks },                                │       │
│  │    "organization": { name, ... }                                 │       │
│  │  }                                                               │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4.4 POST /api/v1/devices/enroll

Enroll device using enrollment token.

#### Request

```json
{
  "enrollment_token": "enroll_abc123xyz789...",
  "device_uuid": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "Field Tablet #42",
  "device_info": {
    "manufacturer": "Samsung",
    "model": "Galaxy Tab A8",
    "os_version": "Android 14"
  }
}
```

#### Response (201 Created)

```json
{
  "device": {
    "id": "dev_01HXYZABC123",
    "device_uuid": "550e8400-e29b-41d4-a716-446655440000",
    "display_name": "Field Tablet #42",
    "organization_id": "org_01HXYZABC123",
    "is_managed": true,
    "enrollment_status": "enrolled"
  },
  "device_token": "dt_eyJhbGciOiJSUzI1NiIs...",
  "device_token_expires_at": "2026-03-01T10:30:00Z",
  "policy": {
    "id": "pol_01STANDARD123",
    "name": "Field Worker Standard",
    "settings": {
      "tracking_enabled": true,
      "tracking_interval_minutes": 5
    },
    "locked_settings": ["tracking_enabled", "secret_mode_enabled"]
  },
  "group": {
    "id": "grp_01FIELD789",
    "name": "Field Workers"
  }
}
```

---

## 5. Policy Engine

### 5.1 Policy Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/admin/v1/organizations/{orgId}/policies` | Create policy | Yes |
| GET | `/api/admin/v1/organizations/{orgId}/policies` | List policies | Yes |
| GET | `/api/admin/v1/organizations/{orgId}/policies/{policyId}` | Get policy | Yes |
| PUT | `/api/admin/v1/organizations/{orgId}/policies/{policyId}` | Update policy | Yes |
| DELETE | `/api/admin/v1/organizations/{orgId}/policies/{policyId}` | Delete policy | Yes |
| POST | `/api/admin/v1/organizations/{orgId}/policies/{policyId}/apply` | Apply policy | Yes |

### 5.2 POST /api/admin/v1/organizations/{orgId}/policies

Create device policy.

#### Request

```json
{
  "name": "Field Worker Standard",
  "description": "Standard policy for field workers",
  "is_default": false,
  "settings": {
    "tracking_enabled": true,
    "tracking_interval_minutes": 5,
    "high_accuracy_mode": true,
    "secret_mode_enabled": false,
    "movement_detection_enabled": true,
    "trip_detection_enabled": true,
    "geofence_enabled": true
  },
  "locked_settings": [
    "tracking_enabled",
    "secret_mode_enabled"
  ],
  "priority": 10
}
```

#### Response (201 Created)

```json
{
  "id": "pol_01HXYZABC123",
  "organization_id": "org_01HXYZABC123",
  "name": "Field Worker Standard",
  "description": "Standard policy for field workers",
  "is_default": false,
  "settings": {
    "tracking_enabled": true,
    "tracking_interval_minutes": 5
  },
  "locked_settings": ["tracking_enabled", "secret_mode_enabled"],
  "priority": 10,
  "device_count": 0,
  "created_at": "2025-12-01T10:30:00Z"
}
```

---

### 5.3 POST /api/admin/v1/organizations/{orgId}/policies/{policyId}/apply

Apply policy to devices or groups.

#### Request

```json
{
  "targets": [
    { "type": "device", "id": "dev_01ABC" },
    { "type": "device", "id": "dev_02DEF" },
    { "type": "group", "id": "grp_01FIELD" }
  ],
  "replace_existing": true
}
```

#### Response (200 OK)

```json
{
  "policy_id": "pol_01HXYZABC123",
  "applied_to": {
    "devices": 25,
    "groups": 1
  },
  "total_devices_affected": 45
}
```

---

### 5.4 Policy Resolution Algorithm

```
function resolveEffectiveSettings(device):
    # Start with organization defaults
    settings = copy(device.organization.settings)
    locked_keys = Set()

    # Apply group policy (if device in group)
    if device.group and device.group.policy:
        merge(settings, device.group.policy.settings)
        locked_keys.add_all(device.group.policy.locked_settings)

    # Apply device-specific policy (highest priority)
    if device.policy:
        merge(settings, device.policy.settings)
        locked_keys.add_all(device.policy.locked_settings)

    # Apply individual device settings (only non-locked)
    for key, value in device.custom_settings:
        if key not in locked_keys:
            settings[key] = value

    return { settings, locked_keys }
```

---

## 6. Fleet Management

### 6.1 Fleet Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/admin/v1/organizations/{orgId}/devices` | List all devices | Yes |
| POST | `/api/admin/v1/organizations/{orgId}/devices/bulk` | Bulk import | Yes |
| POST | `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/assign` | Assign user | Yes |
| POST | `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/unassign` | Unassign user | Yes |
| POST | `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/suspend` | Suspend device | Yes |
| POST | `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/retire` | Retire device | Yes |
| POST | `/api/admin/v1/organizations/{orgId}/devices/{deviceId}/wipe` | Remote wipe | Yes |

### 6.2 GET /api/admin/v1/organizations/{orgId}/devices

List all organization devices with filtering.

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| page | integer | 1 | Page number |
| per_page | integer | 50 | Items per page (max 200) |
| status | string | - | Filter: enrolled, pending, suspended, retired |
| group_id | string | - | Filter by group |
| policy_id | string | - | Filter by policy |
| assigned | boolean | - | Filter assigned/unassigned |
| search | string | - | Search name, UUID |
| sort | string | last_seen_at | Sort field |
| order | string | desc | Sort order |

#### Response (200 OK)

```json
{
  "data": [
    {
      "id": "dev_01HXYZABC123",
      "device_uuid": "550e8400-...",
      "display_name": "Field Tablet #42",
      "platform": "android",
      "enrollment_status": "enrolled",
      "is_managed": true,
      "assigned_user": {
        "id": "user_01EMP456",
        "email": "john@acme.com",
        "display_name": "John Smith"
      },
      "group": {
        "id": "grp_01FIELD",
        "name": "Field Workers"
      },
      "policy": {
        "id": "pol_01STD",
        "name": "Standard"
      },
      "last_seen_at": "2025-12-01T10:30:00Z",
      "last_location": {
        "latitude": 48.8566,
        "longitude": 2.3522
      }
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 45,
    "total_pages": 1
  },
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

---

### 6.3 POST /api/admin/v1/organizations/{orgId}/devices/bulk

Bulk import devices.

#### Request

```json
{
  "devices": [
    {
      "external_id": "ASSET-001",
      "display_name": "Field Tablet 1",
      "group_id": "grp_01FIELD",
      "policy_id": "pol_01STD",
      "assigned_user_email": "john@acme.com",
      "metadata": {
        "asset_tag": "ASSET-001",
        "purchase_date": "2025-01-15"
      }
    }
  ],
  "options": {
    "update_existing": true,
    "create_missing_users": false,
    "send_welcome_email": true
  }
}
```

#### Response (200 OK)

```json
{
  "processed": 100,
  "created": 85,
  "updated": 10,
  "skipped": 3,
  "errors": [
    {
      "row": 52,
      "external_id": "ASSET-052",
      "error": "User not found: invalid@acme.com"
    }
  ]
}
```

---

### 6.4 POST /api/admin/v1/organizations/{orgId}/devices/{deviceId}/assign

Assign user to device.

#### Request

```json
{
  "user_id": "user_01EMP456",
  "notify_user": true
}
```

**Alternative:** Assign by email

```json
{
  "user_email": "john@acme.com",
  "create_user_if_missing": true,
  "notify_user": true
}
```

#### Response (200 OK)

```json
{
  "device_id": "dev_01HXYZABC123",
  "assigned_user": {
    "id": "user_01EMP456",
    "email": "john@acme.com",
    "display_name": "John Smith"
  },
  "assigned_at": "2025-12-01T10:30:00Z",
  "notification_sent": true
}
```

---

## 7. Audit Logging

### 7.1 Audit Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/admin/v1/organizations/{orgId}/audit-logs` | List audit logs | Yes |
| GET | `/api/admin/v1/organizations/{orgId}/audit-logs/export` | Export logs | Yes |

### 7.2 GET /api/admin/v1/organizations/{orgId}/audit-logs

Query audit logs.

#### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| page | integer | Page number |
| per_page | integer | Items per page (max 100) |
| actor_id | string | Filter by actor |
| action | string | Filter by action (e.g., "device.assign") |
| resource_type | string | Filter by resource (device, user, policy) |
| resource_id | string | Filter by specific resource |
| from | datetime | Start date (ISO 8601) |
| to | datetime | End date (ISO 8601) |

#### Response (200 OK)

```json
{
  "data": [
    {
      "id": "log_01HXYZABC123",
      "timestamp": "2025-12-01T10:30:00Z",
      "actor": {
        "id": "user_01ADMIN",
        "type": "user",
        "email": "admin@acme.com"
      },
      "action": "device.assign",
      "resource": {
        "type": "device",
        "id": "dev_01ABC",
        "name": "Field Tablet #42"
      },
      "changes": {
        "assigned_user_id": {
          "old": null,
          "new": "user_01EMP"
        }
      },
      "metadata": {
        "ip_address": "192.168.1.100",
        "user_agent": "Mozilla/5.0..."
      }
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 1250
  }
}
```

### 7.3 Audited Actions

| Category | Actions |
|----------|---------|
| Organization | create, update, delete, settings_change |
| User | create, update, delete, role_change, invite, login |
| Device | register, enroll, assign, unassign, suspend, retire, wipe, settings_change |
| Policy | create, update, delete, apply, unapply |
| Group | create, update, delete, member_add, member_remove |
| Settings | lock, unlock, override, bulk_update |

---

## 8. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-01 | Initial specification |
