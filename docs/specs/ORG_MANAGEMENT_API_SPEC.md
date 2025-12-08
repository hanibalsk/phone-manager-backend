# Organization Management API Specification

## Phone Manager Backend - API Keys, Invitations & Webhooks

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-08

---

## 1. Overview

This document defines the organization management APIs for the admin portal:
- **API Keys Management**: Create, list, update, and revoke organization-scoped API keys
- **Member Invitations**: Email-based invitations for new organization members
- **Organization Webhooks**: Organization-level webhook configuration for event notifications

### 1.1 Authentication

All endpoints require B2B admin authentication via `X-API-Key` header with admin privileges.

| Header | Description |
|--------|-------------|
| `X-API-Key` | Admin API key with `is_admin: true` |

### 1.2 Base URL

```
/api/admin/v1/organizations/{orgId}
```

### 1.3 Common Error Responses

| Status | Description |
|--------|-------------|
| 400 | Validation error - invalid request body |
| 401 | Authentication failed - missing or invalid API key |
| 403 | Forbidden - insufficient permissions |
| 404 | Organization or resource not found |
| 409 | Conflict - duplicate resource |
| 429 | Rate limit exceeded |
| 500 | Internal server error |

---

## 2. API Keys Management

Manage organization-scoped API keys for programmatic access.

### 2.1 Endpoints Overview

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/admin/v1/organizations/{orgId}/api-keys` | Create API key |
| GET | `/api/admin/v1/organizations/{orgId}/api-keys` | List API keys |
| GET | `/api/admin/v1/organizations/{orgId}/api-keys/{keyId}` | Get API key details |
| PATCH | `/api/admin/v1/organizations/{orgId}/api-keys/{keyId}` | Update API key |
| DELETE | `/api/admin/v1/organizations/{orgId}/api-keys/{keyId}` | Revoke API key |

---

### 2.2 POST /api/admin/v1/organizations/{orgId}/api-keys

Create a new organization API key.

#### Request

```json
{
  "name": "Production Mobile App",
  "description": "API key for mobile application",
  "expires_in_days": 365
}
```

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| name | string | Yes | 1-100 chars | Human-readable key name |
| description | string | No | max 255 chars | Detailed description |
| expires_in_days | integer | No | 1-365, null=never | Days until expiration |

#### Response (201 Created)

```json
{
  "id": 42,
  "key": "pm_live_abc123def456ghi789jkl012mno345pqr678stu901vwx234yz",
  "key_prefix": "pm_live_ab",
  "name": "Production Mobile App",
  "description": "API key for mobile application",
  "expires_at": "2026-12-08T00:00:00Z",
  "created_at": "2025-12-08T10:30:00Z"
}
```

> **Security Note**: The `key` field is shown only once. Store it securely immediately.

#### Error Responses

| Status | Code | Description |
|--------|------|-------------|
| 400 | VALIDATION_ERROR | Invalid request body |
| 404 | ORG_NOT_FOUND | Organization not found |
| 409 | KEY_LIMIT_REACHED | Maximum API keys limit (50) reached |

---

### 2.3 GET /api/admin/v1/organizations/{orgId}/api-keys

List all API keys for the organization.

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| include_inactive | boolean | false | Include revoked keys |
| page | integer | 1 | Page number |
| per_page | integer | 50 | Items per page (max 100) |

#### Response (200 OK)

```json
{
  "api_keys": [
    {
      "id": 42,
      "key_prefix": "pm_live_ab",
      "name": "Production Mobile App",
      "description": "API key for mobile application",
      "is_active": true,
      "last_used_at": "2025-12-07T15:45:00Z",
      "created_at": "2025-01-15T08:00:00Z",
      "expires_at": "2026-01-15T08:00:00Z"
    },
    {
      "id": 41,
      "key_prefix": "pm_live_cd",
      "name": "Staging Environment",
      "description": null,
      "is_active": true,
      "last_used_at": null,
      "created_at": "2025-01-10T12:00:00Z",
      "expires_at": null
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 2,
    "total_pages": 1
  }
}
```

---

### 2.4 GET /api/admin/v1/organizations/{orgId}/api-keys/{keyId}

Get details for a specific API key.

#### Response (200 OK)

```json
{
  "id": 42,
  "key_prefix": "pm_live_ab",
  "name": "Production Mobile App",
  "description": "API key for mobile application",
  "is_active": true,
  "last_used_at": "2025-12-07T15:45:00Z",
  "created_at": "2025-01-15T08:00:00Z",
  "expires_at": "2026-01-15T08:00:00Z"
}
```

---

### 2.5 PATCH /api/admin/v1/organizations/{orgId}/api-keys/{keyId}

Update an API key's metadata.

#### Request

```json
{
  "name": "Production Mobile App (Updated)",
  "description": "Updated description for the API key"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | No | New key name (1-100 chars) |
| description | string | No | New description (max 255 chars) |

#### Response (200 OK)

```json
{
  "id": 42,
  "key_prefix": "pm_live_ab",
  "name": "Production Mobile App (Updated)",
  "description": "Updated description for the API key",
  "is_active": true,
  "last_used_at": "2025-12-07T15:45:00Z",
  "created_at": "2025-01-15T08:00:00Z",
  "expires_at": "2026-01-15T08:00:00Z"
}
```

---

### 2.6 DELETE /api/admin/v1/organizations/{orgId}/api-keys/{keyId}

Revoke an API key (soft delete).

#### Response (204 No Content)

No body returned.

#### Notes
- Revoked keys cannot be reactivated
- Key remains in database with `is_active: false` for audit purposes
- Any requests using the revoked key will receive 401 Unauthorized

---

## 3. Organization Member Invitations

Email-based invitations for new organization members.

### 3.1 Endpoints Overview

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/admin/v1/organizations/{orgId}/invitations` | Create invitation |
| GET | `/api/admin/v1/organizations/{orgId}/invitations` | List invitations |
| GET | `/api/admin/v1/organizations/{orgId}/invitations/{inviteId}` | Get invitation |
| DELETE | `/api/admin/v1/organizations/{orgId}/invitations/{inviteId}` | Revoke invitation |
| POST | `/api/v1/invitations/{token}/accept` | Accept invitation (public) |

---

### 3.2 POST /api/admin/v1/organizations/{orgId}/invitations

Create a new member invitation.

#### Request

```json
{
  "email": "newmember@company.com",
  "role": "member",
  "note": "Welcome to the team!",
  "expires_in_days": 7
}
```

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| email | string | Yes | Valid email | Invitee's email address |
| role | string | No | "admin" \| "member" | Role to assign (default: member) |
| note | string | No | max 255 chars | Personal message for invitee |
| expires_in_days | integer | No | 1-30 (default: 7) | Days until expiration |

#### Response (201 Created)

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "newmember@company.com",
  "role": "member",
  "token": "abc123XYZ789...",
  "invite_url": "https://app.example.com/invite/abc123XYZ789...",
  "expires_at": "2025-12-15T10:30:00Z",
  "created_at": "2025-12-08T10:30:00Z",
  "note": "Welcome to the team!"
}
```

#### Error Responses

| Status | Code | Description |
|--------|------|-------------|
| 400 | VALIDATION_ERROR | Invalid email or role |
| 409 | DUPLICATE_INVITATION | Pending invitation already exists for email |
| 409 | ALREADY_MEMBER | User is already a member of this organization |

---

### 3.3 GET /api/admin/v1/organizations/{orgId}/invitations

List all invitations for the organization.

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| status | string | "pending" | Filter: "pending", "accepted", "expired", "all" |
| page | integer | 1 | Page number |
| per_page | integer | 50 | Items per page (max 100) |

#### Response (200 OK)

```json
{
  "invitations": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "newmember@company.com",
      "role": "member",
      "status": "pending",
      "expires_at": "2025-12-15T10:30:00Z",
      "created_at": "2025-12-08T10:30:00Z",
      "note": "Welcome to the team!",
      "invited_by": {
        "id": "user_01ADMIN",
        "email": "admin@company.com"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 1,
    "total_pages": 1
  },
  "summary": {
    "pending": 3,
    "accepted": 15,
    "expired": 2
  }
}
```

---

### 3.4 DELETE /api/admin/v1/organizations/{orgId}/invitations/{inviteId}

Revoke a pending invitation.

#### Response (204 No Content)

No body returned.

#### Notes
- Only pending invitations can be revoked
- Accepted invitations cannot be revoked (remove member instead)

---

### 3.5 POST /api/v1/invitations/{token}/accept

Accept an invitation and create user account (public endpoint - no auth required).

#### Request

```json
{
  "password": "SecurePassword123!",
  "display_name": "John Smith"
}
```

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| password | string | Yes | 8-128 chars, complexity rules | Account password |
| display_name | string | Yes | 1-100 chars | User's display name |

#### Response (201 Created)

```json
{
  "user": {
    "id": "user_01NEWMEMBER",
    "email": "newmember@company.com",
    "display_name": "John Smith"
  },
  "organization": {
    "id": "org_01ACME",
    "name": "Acme Corporation"
  },
  "role": "member",
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_expires_at": "2025-12-09T10:30:00Z"
}
```

#### Error Responses

| Status | Code | Description |
|--------|------|-------------|
| 400 | VALIDATION_ERROR | Invalid password or display name |
| 404 | INVITATION_NOT_FOUND | Invalid or expired token |
| 409 | INVITATION_USED | Invitation already accepted |
| 409 | EMAIL_EXISTS | User with email already exists |

---

### 3.6 Invitation Flow Diagram

```
+-----------------------------------------------------------------------------+
|                        MEMBER INVITATION FLOW                                |
+-----------------------------------------------------------------------------+
|                                                                              |
|  Admin creates invitation                                                    |
|  POST /api/admin/v1/organizations/{orgId}/invitations                        |
|           |                                                                  |
|           v                                                                  |
|  +-----------------+                                                         |
|  |  Invitation     |                                                         |
|  |  - Token        |  ------->  Email sent to invitee (optional)             |
|  |  - Invite URL   |                                                         |
|  +--------+--------+                                                         |
|           |                                                                  |
|           v                                                                  |
|  +-------------------------------------------------------------+            |
|  |  Invitee clicks link or enters token                        |            |
|  |  Frontend shows registration form                           |            |
|  +--------+----------------------------------------------------+            |
|           |                                                                  |
|           v                                                                  |
|  +-------------------------------------------------------------+            |
|  |  POST /api/v1/invitations/{token}/accept                    |            |
|  |  {                                                          |            |
|  |    "password": "...",                                       |            |
|  |    "display_name": "..."                                    |            |
|  |  }                                                          |            |
|  +--------+----------------------------------------------------+            |
|           |                                                                  |
|           v                                                                  |
|  +-------------------------------------------------------------+            |
|  |  Server:                                                    |            |
|  |  1. Validates invitation token                              |            |
|  |  2. Creates user account                                    |            |
|  |  3. Adds user to organization with role                     |            |
|  |  4. Marks invitation as accepted                            |            |
|  |  5. Returns JWT access token                                |            |
|  +-------------------------------------------------------------+            |
|                                                                              |
+------------------------------------------------------------------------------+
```

---

## 4. Organization Webhooks

Organization-level webhooks for event notifications.

### 4.1 Endpoints Overview

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/admin/v1/organizations/{orgId}/webhooks` | Create webhook |
| GET | `/api/admin/v1/organizations/{orgId}/webhooks` | List webhooks |
| GET | `/api/admin/v1/organizations/{orgId}/webhooks/{webhookId}` | Get webhook |
| PUT | `/api/admin/v1/organizations/{orgId}/webhooks/{webhookId}` | Update webhook |
| DELETE | `/api/admin/v1/organizations/{orgId}/webhooks/{webhookId}` | Delete webhook |

---

### 4.2 POST /api/admin/v1/organizations/{orgId}/webhooks

Create a new organization webhook.

#### Request

```json
{
  "name": "Production Events",
  "target_url": "https://api.example.com/webhooks/phone-manager",
  "secret": "whsec_abc123def456ghi789",
  "event_types": [
    "device.enrolled",
    "device.unenrolled",
    "member.joined",
    "member.removed"
  ]
}
```

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| name | string | Yes | 1-100 chars | Webhook name |
| target_url | string | Yes | Valid HTTPS URL | Endpoint to receive events |
| secret | string | Yes | 16-256 chars | HMAC-SHA256 signing secret |
| event_types | string[] | Yes | Valid event types | Events to subscribe to |

#### Supported Event Types

| Event Type | Description |
|------------|-------------|
| `device.enrolled` | Device enrolled in organization |
| `device.unenrolled` | Device removed from organization |
| `device.assigned` | Device assigned to user |
| `device.unassigned` | Device unassigned from user |
| `member.joined` | New member joined organization |
| `member.removed` | Member removed from organization |
| `policy.applied` | Policy applied to device/group |
| `policy.updated` | Policy settings changed |

#### Response (201 Created)

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Production Events",
  "target_url": "https://api.example.com/webhooks/phone-manager",
  "enabled": true,
  "event_types": [
    "device.enrolled",
    "device.unenrolled",
    "member.joined",
    "member.removed"
  ],
  "consecutive_failures": 0,
  "circuit_open_until": null,
  "created_at": "2025-12-08T10:30:00Z",
  "updated_at": "2025-12-08T10:30:00Z"
}
```

> **Note**: `secret` is never returned in responses for security.

---

### 4.3 GET /api/admin/v1/organizations/{orgId}/webhooks

List all webhooks for the organization.

#### Response (200 OK)

```json
{
  "webhooks": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Production Events",
      "target_url": "https://api.example.com/webhooks/phone-manager",
      "enabled": true,
      "event_types": ["device.enrolled", "device.unenrolled"],
      "consecutive_failures": 0,
      "circuit_open_until": null,
      "created_at": "2025-12-08T10:30:00Z",
      "updated_at": "2025-12-08T10:30:00Z"
    },
    {
      "id": "661f9500-f39c-52e5-b827-557766551111",
      "name": "Analytics Integration",
      "target_url": "https://analytics.example.com/ingest",
      "enabled": false,
      "event_types": ["device.enrolled"],
      "consecutive_failures": 5,
      "circuit_open_until": "2025-12-08T11:00:00Z",
      "created_at": "2025-12-01T08:00:00Z",
      "updated_at": "2025-12-08T10:30:00Z"
    }
  ]
}
```

---

### 4.4 PUT /api/admin/v1/organizations/{orgId}/webhooks/{webhookId}

Update a webhook.

#### Request

```json
{
  "name": "Production Events (Updated)",
  "target_url": "https://api.example.com/v2/webhooks",
  "secret": "whsec_new_secret_xyz789",
  "enabled": true,
  "event_types": [
    "device.enrolled",
    "device.unenrolled",
    "member.joined",
    "member.removed",
    "policy.updated"
  ]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | No | New webhook name |
| target_url | string | No | New target URL (HTTPS required) |
| secret | string | No | New signing secret |
| enabled | boolean | No | Enable/disable webhook |
| event_types | string[] | No | New event subscriptions |

#### Response (200 OK)

Same as GET response with updated values.

---

### 4.5 DELETE /api/admin/v1/organizations/{orgId}/webhooks/{webhookId}

Delete a webhook.

#### Response (204 No Content)

No body returned.

---

### 4.6 Webhook Payload Format

All webhook deliveries include:

#### Headers

```
Content-Type: application/json
X-Webhook-Id: 550e8400-e29b-41d4-a716-446655440000
X-Webhook-Event: device.enrolled
X-Webhook-Timestamp: 1733657400
X-Webhook-Signature: sha256=abc123...
```

#### Body

```json
{
  "id": "evt_01HXYZABC123",
  "type": "device.enrolled",
  "organization_id": "org_01ACME",
  "timestamp": "2025-12-08T10:30:00Z",
  "data": {
    "device": {
      "id": "dev_01ABC",
      "device_uuid": "550e8400-...",
      "display_name": "Field Tablet #42"
    },
    "enrollment": {
      "token_id": "enroll_01XYZ",
      "group_id": "grp_01FIELD",
      "policy_id": "pol_01STD"
    }
  }
}
```

#### Signature Verification

```python
import hmac
import hashlib

def verify_signature(payload: bytes, signature: str, secret: str) -> bool:
    expected = hmac.new(
        secret.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(f"sha256={expected}", signature)
```

---

### 4.7 Circuit Breaker

Webhooks implement a circuit breaker pattern:

| State | Condition | Behavior |
|-------|-----------|----------|
| Closed | 0-4 consecutive failures | Normal delivery |
| Open | 5+ consecutive failures | Skip delivery for 5 minutes |
| Half-Open | After cooldown | Attempt single delivery |

**Retry Schedule**: 0s, 60s, 300s, 900s (max 4 attempts)

---

## 5. Database Schema

### 5.1 api_keys (Modified)

```sql
ALTER TABLE api_keys ADD COLUMN organization_id UUID
    REFERENCES organizations(id) ON DELETE CASCADE;

CREATE INDEX idx_api_keys_organization_id
    ON api_keys(organization_id) WHERE organization_id IS NOT NULL;
```

### 5.2 org_member_invites (New)

```sql
CREATE TABLE org_member_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    token VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    invited_by UUID REFERENCES users(id),
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    accepted_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    note VARCHAR(255)
);

CREATE INDEX idx_org_member_invites_token ON org_member_invites(token);
CREATE INDEX idx_org_member_invites_org_id ON org_member_invites(organization_id);
CREATE INDEX idx_org_member_invites_email ON org_member_invites(email);
```

### 5.3 org_webhooks (New)

```sql
CREATE TABLE org_webhooks (
    id BIGSERIAL PRIMARY KEY,
    webhook_id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    target_url VARCHAR(2048) NOT NULL,
    secret VARCHAR(256) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    event_types VARCHAR(255)[] NOT NULL DEFAULT '{}',
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    circuit_open_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_org_webhooks_org_id ON org_webhooks(organization_id);
CREATE INDEX idx_org_webhooks_enabled ON org_webhooks(organization_id, enabled)
    WHERE enabled = true;
```

---

## 6. Security Considerations

### 6.1 API Keys

- Keys are hashed with SHA-256 before storage
- Full key is returned only once on creation
- Key prefix (first 8 chars after `pm_`) stored for identification
- Revoked keys remain in database for audit trail

### 6.2 Invitations

- Tokens are cryptographically secure random strings (32 chars)
- Tokens can only be used once
- Invitations expire after configured period (default: 7 days)
- Email validation prevents invitation to existing members

### 6.3 Webhooks

- Requires HTTPS for target URLs
- HMAC-SHA256 signature for payload verification
- Secret is never exposed in API responses
- Circuit breaker prevents DoS from failing endpoints

---

## 7. Rate Limits

| Endpoint Category | Rate Limit |
|-------------------|------------|
| API Key Management | 100 requests/minute |
| Invitation Management | 50 requests/minute |
| Webhook Management | 50 requests/minute |
| Accept Invitation (public) | 10 requests/minute per IP |

---

## 8. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-08 | Initial specification |
