# User & Group Management API Specification

## Phone Manager Backend - User Management & Device Control

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-01

---

## 1. Overview

This document defines the API endpoints for user profile management, device linking, group management, membership management, and invitation handling.

---

## 2. User Endpoints

### 2.1 Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/v1/users/{userId}` | Get user by ID | Yes |
| GET | `/api/v1/users/{userId}/devices` | List user's devices | Yes |
| POST | `/api/v1/users/{userId}/devices/{deviceId}/link` | Link device to user | Yes |
| DELETE | `/api/v1/users/{userId}/devices/{deviceId}/unlink` | Unlink device from user | Yes |
| POST | `/api/v1/users/{userId}/devices/{deviceId}/transfer` | Transfer device to another user | Yes |

### 2.2 GET /api/v1/users/{userId}

Get user details.

**Authentication:** Bearer Token
**Authorization:** Self or group member

#### Response (200 OK)

```json
{
  "id": "user_01HXYZABC123",
  "email": "user@example.com",
  "display_name": "John Doe",
  "avatar_url": "https://...",
  "created_at": "2025-12-01T10:30:00Z"
}
```

**Note:** Email only visible to self or org admins.

---

### 2.3 GET /api/v1/users/{userId}/devices

List devices linked to user.

**Authentication:** Bearer Token
**Authorization:** Self only

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| page | integer | 1 | Page number |
| per_page | integer | 20 | Items per page (max 100) |
| include_inactive | boolean | false | Include inactive devices |

#### Response (200 OK)

```json
{
  "data": [
    {
      "id": "dev_01HXYZABC123",
      "device_uuid": "550e8400-e29b-41d4-a716-446655440000",
      "display_name": "John's Pixel 8",
      "platform": "android",
      "is_primary": true,
      "is_active": true,
      "last_seen_at": "2025-12-01T10:30:00Z",
      "last_location": {
        "latitude": 48.8566,
        "longitude": 2.3522,
        "accuracy": 10.5,
        "timestamp": "2025-12-01T10:30:00Z"
      },
      "created_at": "2025-11-01T08:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 3,
    "total_pages": 1
  }
}
```

---

### 2.4 POST /api/v1/users/{userId}/devices/{deviceId}/link

Link an existing device to user account.

**Authentication:** Bearer Token
**Authorization:** Self only

#### Request

```json
{
  "display_name": "My Tablet",
  "is_primary": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| display_name | string | No | Override device name |
| is_primary | boolean | No | Set as primary device |

#### Response (200 OK)

```json
{
  "device": {
    "id": "dev_01HXYZABC123",
    "device_uuid": "550e8400-e29b-41d4-a716-446655440000",
    "display_name": "My Tablet",
    "owner_user_id": "user_01HXYZABC123",
    "is_primary": false,
    "linked_at": "2025-12-01T10:30:00Z"
  },
  "linked": true
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 404 | resource/not-found | Device not found |
| 409 | resource/already-exists | Device already linked to another user |

---

### 2.5 DELETE /api/v1/users/{userId}/devices/{deviceId}/unlink

Unlink device from user account.

**Authentication:** Bearer Token
**Authorization:** Self only

#### Response (200 OK)

```json
{
  "device_id": "dev_01HXYZABC123",
  "unlinked": true
}
```

---

### 2.6 POST /api/v1/users/{userId}/devices/{deviceId}/transfer

Transfer device ownership to another user.

**Authentication:** Bearer Token
**Authorization:** Device owner

#### Request

```json
{
  "new_owner_email": "newowner@example.com"
}
```

#### Response (200 OK)

```json
{
  "device_id": "dev_01HXYZABC123",
  "previous_owner_id": "user_01HXYZABC123",
  "new_owner_id": "user_01NEWOWNER456",
  "transferred_at": "2025-12-01T10:30:00Z"
}
```

---

## 3. Group Endpoints

### 3.1 Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/v1/groups` | Create new group | Yes |
| GET | `/api/v1/groups` | List user's groups | Yes |
| GET | `/api/v1/groups/{groupId}` | Get group details | Yes |
| PUT | `/api/v1/groups/{groupId}` | Update group | Yes |
| DELETE | `/api/v1/groups/{groupId}` | Delete group | Yes |

### 3.2 POST /api/v1/groups

Create a new group.

**Authentication:** Bearer Token

#### Request

```json
{
  "name": "Smith Family",
  "description": "Our family location sharing group",
  "icon_emoji": "👨‍👩‍👧‍👦",
  "max_devices": 20
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| name | string | Yes | 1-100 chars |
| description | string | No | Max 500 chars |
| icon_emoji | string | No | Single emoji |
| max_devices | integer | No | 1-100, default 20 |

#### Response (201 Created)

```json
{
  "id": "grp_01HXYZABC123",
  "organization_id": null,
  "name": "Smith Family",
  "slug": "smith-family",
  "description": "Our family location sharing group",
  "icon_emoji": "👨‍👩‍👧‍👦",
  "max_devices": 20,
  "member_count": 1,
  "device_count": 0,
  "is_active": true,
  "created_by": "user_01HXYZABC123",
  "created_at": "2025-12-01T10:30:00Z",
  "your_role": "owner"
}
```

---

### 3.3 GET /api/v1/groups

List groups the current user belongs to.

**Authentication:** Bearer Token

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| page | integer | 1 | Page number |
| per_page | integer | 20 | Items per page (max 100) |
| role | string | - | Filter by role (owner, admin, member, viewer) |

#### Response (200 OK)

```json
{
  "data": [
    {
      "id": "grp_01HXYZABC123",
      "name": "Smith Family",
      "slug": "smith-family",
      "icon_emoji": "👨‍👩‍👧‍👦",
      "member_count": 4,
      "device_count": 6,
      "your_role": "owner",
      "joined_at": "2025-12-01T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 2,
    "total_pages": 1
  }
}
```

---

### 3.4 GET /api/v1/groups/{groupId}

Get group details.

**Authentication:** Bearer Token
**Authorization:** Group member

#### Response (200 OK)

```json
{
  "id": "grp_01HXYZABC123",
  "organization_id": null,
  "name": "Smith Family",
  "slug": "smith-family",
  "description": "Our family location sharing group",
  "icon_emoji": "👨‍👩‍👧‍👦",
  "max_devices": 20,
  "member_count": 4,
  "device_count": 6,
  "is_active": true,
  "settings": {},
  "created_by": "user_01HXYZABC123",
  "created_at": "2025-12-01T10:30:00Z",
  "updated_at": "2025-12-01T10:30:00Z",
  "your_role": "admin",
  "your_membership": {
    "id": "mem_01ABC",
    "role": "admin",
    "joined_at": "2025-12-01T10:30:00Z"
  }
}
```

---

### 3.5 PUT /api/v1/groups/{groupId}

Update group settings.

**Authentication:** Bearer Token
**Authorization:** Group admin or owner

#### Request

```json
{
  "name": "Smith-Johnson Family",
  "description": "Our combined family group",
  "icon_emoji": "🏠",
  "max_devices": 30
}
```

#### Response (200 OK)

Updated group object.

---

### 3.6 DELETE /api/v1/groups/{groupId}

Delete a group.

**Authentication:** Bearer Token
**Authorization:** Group owner only

#### Response (204 No Content)

No response body.

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 403 | authz/not-group-owner | Only owner can delete group |

---

## 4. Membership Endpoints

### 4.1 Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/v1/groups/{groupId}/members` | List group members | Yes |
| GET | `/api/v1/groups/{groupId}/members/{userId}` | Get member details | Yes |
| PUT | `/api/v1/groups/{groupId}/members/{userId}/role` | Update member role | Yes |
| DELETE | `/api/v1/groups/{groupId}/members/{userId}` | Remove member | Yes |

### 4.2 GET /api/v1/groups/{groupId}/members

List group members.

**Authentication:** Bearer Token
**Authorization:** Group member

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| page | integer | 1 | Page number |
| per_page | integer | 20 | Items per page (max 100) |
| role | string | - | Filter by role |
| include_devices | boolean | false | Include member devices |

#### Response (200 OK)

```json
{
  "data": [
    {
      "id": "mem_01HXYZABC123",
      "user": {
        "id": "user_01HXYZABC123",
        "display_name": "John Doe",
        "avatar_url": "https://..."
      },
      "role": "owner",
      "joined_at": "2025-12-01T10:30:00Z",
      "invited_by": null,
      "devices": [
        {
          "id": "dev_01ABC",
          "display_name": "John's Phone",
          "last_seen_at": "2025-12-01T10:30:00Z",
          "last_location": {
            "latitude": 48.8566,
            "longitude": 2.3522
          }
        }
      ]
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 4,
    "total_pages": 1
  }
}
```

---

### 4.3 PUT /api/v1/groups/{groupId}/members/{userId}/role

Update member role.

**Authentication:** Bearer Token
**Authorization:** Group admin or owner (cannot change owner role)

#### Request

```json
{
  "role": "admin"
}
```

| Field | Type | Required | Values |
|-------|------|----------|--------|
| role | string | Yes | owner, admin, member, viewer |

#### Response (200 OK)

```json
{
  "id": "mem_01HXYZABC123",
  "user_id": "user_01HXYZABC123",
  "group_id": "grp_01HXYZABC123",
  "role": "admin",
  "updated_at": "2025-12-01T10:30:00Z"
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 403 | authz/cannot-change-owner | Cannot change owner's role |
| 403 | authz/cannot-promote-to-owner | Cannot promote to owner (use transfer) |

---

### 4.4 DELETE /api/v1/groups/{groupId}/members/{userId}

Remove member from group.

**Authentication:** Bearer Token
**Authorization:** Admin/owner (others) or self (leave)

#### Response (204 No Content)

No response body.

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 403 | authz/owner-cannot-leave | Owner must transfer ownership first |

---

## 5. Invitation Endpoints

### 5.1 Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/v1/groups/{groupId}/invites` | Create invite | Yes |
| GET | `/api/v1/groups/{groupId}/invites` | List invites | Yes |
| DELETE | `/api/v1/groups/{groupId}/invites/{inviteId}` | Revoke invite | Yes |
| POST | `/api/v1/groups/join` | Join with invite code | Yes |
| GET | `/api/v1/invites/{code}` | Get invite info (public) | No |

### 5.2 POST /api/v1/groups/{groupId}/invites

Create invitation code.

**Authentication:** Bearer Token
**Authorization:** Group admin or owner

#### Request

```json
{
  "preset_role": "member",
  "max_uses": 1,
  "expires_in_hours": 24
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| preset_role | string | No | member | Role when joining |
| max_uses | integer | No | 1 | Max uses (1-100) |
| expires_in_hours | integer | No | 24 | Hours until expiry (1-168) |

#### Response (201 Created)

```json
{
  "id": "inv_01HXYZABC123",
  "group_id": "grp_01HXYZABC123",
  "code": "ABC-123-XYZ",
  "preset_role": "member",
  "max_uses": 1,
  "current_uses": 0,
  "expires_at": "2025-12-02T10:30:00Z",
  "created_by": "user_01HXYZABC123",
  "created_at": "2025-12-01T10:30:00Z",
  "invite_url": "https://phonemanager.com/join/ABC-123-XYZ"
}
```

---

### 5.3 GET /api/v1/groups/{groupId}/invites

List active invitations.

**Authentication:** Bearer Token
**Authorization:** Group admin or owner

#### Response (200 OK)

```json
{
  "data": [
    {
      "id": "inv_01HXYZABC123",
      "code": "ABC-123-XYZ",
      "preset_role": "member",
      "max_uses": 5,
      "current_uses": 2,
      "expires_at": "2025-12-02T10:30:00Z",
      "created_by": {
        "id": "user_01HXYZABC123",
        "display_name": "John Doe"
      },
      "created_at": "2025-12-01T10:30:00Z"
    }
  ]
}
```

---

### 5.4 DELETE /api/v1/groups/{groupId}/invites/{inviteId}

Revoke invitation.

**Authentication:** Bearer Token
**Authorization:** Group admin or owner

#### Response (204 No Content)

---

### 5.5 POST /api/v1/groups/join

Join group with invite code.

**Authentication:** Bearer Token

#### Request

```json
{
  "code": "ABC-123-XYZ"
}
```

#### Response (200 OK)

```json
{
  "group": {
    "id": "grp_01HXYZABC123",
    "name": "Smith Family",
    "member_count": 5
  },
  "membership": {
    "id": "mem_01NEWMEM456",
    "role": "member",
    "joined_at": "2025-12-01T10:30:00Z"
  }
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | validation/invalid-invite-code | Invalid or expired code |
| 409 | resource/already-exists | Already a member |
| 410 | resource/expired | Invite expired or fully used |

---

### 5.6 GET /api/v1/invites/{code}

Get invite info without joining.

**Authentication:** None (public)

#### Response (200 OK)

```json
{
  "group": {
    "name": "Smith Family",
    "icon_emoji": "👨‍👩‍👧‍👦",
    "member_count": 4
  },
  "preset_role": "member",
  "expires_at": "2025-12-02T10:30:00Z",
  "is_valid": true
}
```

---

## 6. Transfer Ownership Endpoint

### POST /api/v1/groups/{groupId}/transfer

Transfer group ownership to another member.

**Authentication:** Bearer Token
**Authorization:** Group owner only

#### Request

```json
{
  "new_owner_id": "user_01NEWOWNER456"
}
```

#### Response (200 OK)

```json
{
  "group_id": "grp_01HXYZABC123",
  "previous_owner_id": "user_01HXYZABC123",
  "new_owner_id": "user_01NEWOWNER456",
  "transferred_at": "2025-12-01T10:30:00Z"
}
```

---

## 7. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-01 | Initial specification |
