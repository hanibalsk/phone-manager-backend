# Phone Manager Backend - Claude Code Context

## Project Overview

Rust backend API for the Phone Manager mobile application. Handles device registration, location tracking, and group management for family/friends location sharing.

## Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 1.83+ (Edition 2024) |
| Web Framework | Axum | 0.8 |
| Async Runtime | Tokio | 1.42 |
| Database | PostgreSQL 16 + SQLx | 0.8 |
| Validation | Validator | 0.19 |
| Serialization | Serde + JSON | 1.0 |
| Logging | Tracing | 0.1 |
| Metrics | Prometheus | 0.24 |
| Geospatial | geo | 0.28 |

## Project Structure

```
phone-manager-backend/
├── crates/
│   ├── api/           # HTTP handlers, middleware, extractors (binary)
│   ├── domain/        # Business logic, domain models, services
│   ├── persistence/   # Database layer, repositories, migrations
│   └── shared/        # Common utilities, crypto, validation
├── config/            # TOML configuration files
├── tests/             # Integration tests
└── docs/              # Specifications and design documents
```

## Architecture Pattern

**Layered Architecture** with strict separation:
- **Routes** → HTTP handlers, request/response serialization
- **Middleware** → Auth, logging, rate limiting, tracing
- **Services** → Business logic, orchestration
- **Repositories** → Data access, SQLx queries
- **Entities** → Database row mappings

## Key Design Decisions

### API Design
- REST JSON API with snake_case field names
- API key authentication via `X-API-Key` header
- Request tracing via `X-Request-ID` header
- CORS support for cross-origin requests

### Database
- SQLx for compile-time checked queries
- Migrations in `crates/persistence/src/migrations/`
- UUID for device identifiers
- Timestamps as `TIMESTAMPTZ`

### Error Handling
- `thiserror` for typed domain errors
- `anyhow` for infrastructure errors
- Structured error responses with validation details

## Configuration

Environment prefix: `PM__` (double underscore separator)

```bash
# Required
PM__DATABASE__URL=postgres://user:pass@host:5432/db
PM__JWT__PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----"
PM__JWT__PUBLIC_KEY="-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----"

# Required for production (have example.com defaults)
PM__SERVER__APP_BASE_URL=https://app.yourcompany.com  # Used for invite URLs, deep links
PM__EMAIL__SENDER_EMAIL=noreply@yourcompany.com       # Email sender address
PM__EMAIL__BASE_URL=https://app.yourcompany.com       # Links in emails

# Optional with defaults
PM__SERVER__PORT=8080
PM__LOGGING__LEVEL=info
PM__SECURITY__RATE_LIMIT_PER_MINUTE=100
PM__LIMITS__MAX_DEVICES_PER_GROUP=20
PM__LIMITS__MAX_BATCH_SIZE=50
PM__LIMITS__MAX_WEBHOOKS_PER_DEVICE=10

# FCM Push Notifications (optional)
PM__FCM__ENABLED=true
PM__FCM__PROJECT_ID=your-firebase-project
PM__FCM__CREDENTIALS=/path/to/service-account.json

# OAuth Social Login (optional)
PM__OAUTH__GOOGLE_CLIENT_ID=your-google-oauth-client-id
PM__OAUTH__APPLE_CLIENT_ID=your-apple-service-id
PM__OAUTH__APPLE_TEAM_ID=your-apple-team-id

# Auth Rate Limiting (per IP, optional)
PM__SECURITY__FORGOT_PASSWORD_RATE_LIMIT_PER_HOUR=5
PM__SECURITY__REQUEST_VERIFICATION_RATE_LIMIT_PER_HOUR=3

# Admin Frontend Static File Serving (optional)
PM__FRONTEND__ENABLED=true
PM__FRONTEND__BASE_DIR=/app/frontend
PM__FRONTEND__STAGING_HOSTNAME=admin-staging.example.com
PM__FRONTEND__PRODUCTION_HOSTNAME=admin.example.com
PM__FRONTEND__DEFAULT_ENVIRONMENT=production

# Auth Toggles (optional - all default to permissive)
PM__AUTH_TOGGLES__REGISTRATION_ENABLED=true   # Set false to disable registration
PM__AUTH_TOGGLES__INVITE_ONLY=false           # Set true to require invite tokens
PM__AUTH_TOGGLES__OAUTH_ONLY=false            # Set true to disable password auth

# Feature Toggles (optional - all default to enabled)
PM__FEATURES__GEOFENCES_ENABLED=true          # Geofence endpoints
PM__FEATURES__PROXIMITY_ALERTS_ENABLED=true   # Proximity alert endpoints
PM__FEATURES__WEBHOOKS_ENABLED=true           # Webhook endpoints
PM__FEATURES__MOVEMENT_TRACKING_ENABLED=true  # Trips and movement events
PM__FEATURES__B2B_ENABLED=true                # Organizations, policies, enrollment
PM__FEATURES__GEOFENCE_EVENTS_ENABLED=true    # Geofence event endpoints

# Admin Bootstrap (initial setup only - REMOVE after first startup!)
PM__ADMIN__BOOTSTRAP_EMAIL=admin@company.com
PM__ADMIN__BOOTSTRAP_PASSWORD=initial-secure-password

# Report Generation (optional)
PM__REPORTS__REPORTS_DIR=./reports            # Directory for generated reports
PM__REPORTS__BATCH_SIZE=5                     # Reports to process per batch
PM__REPORTS__EXPIRATION_DAYS=7                # Days before reports expire
```

### Production Configuration Validation

The application validates configuration on startup:
- **Development mode**: If placeholder values detected (`app.example.com`, `noreply@example.com`), logs a warning
- **Production mode**: Fails to start if critical configuration is missing or invalid

Critical production requirements:
- `PM__SERVER__APP_BASE_URL` must not be `https://app.example.com`
- `PM__EMAIL__SENDER_EMAIL` must not be `noreply@example.com` (when email enabled)
- `PM__FCM__PROJECT_ID` and `PM__FCM__CREDENTIALS` required when FCM enabled

## Core Domains

### Device Management
- Registration with UUID, display name, group ID
- Group membership (max 20 devices/group)
- FCM token storage for push notifications

### Location Tracking
- Single and batch location uploads (max 50/batch)
- Coordinate validation (-90/90 lat, -180/180 lon)
- 30-day retention with automated cleanup
- Last location aggregation per device
- Location history with cursor-based pagination
- Trajectory simplification via Ramer-Douglas-Peucker algorithm
  - `tolerance` parameter (0-10000 meters)
  - Reduces points while preserving trajectory shape
  - Response includes simplification metadata

### Geofences
- Per-device circular geofences (max 50/device)
- Event types: enter, exit, dwell
- Radius: 20-50000 meters
- Optional metadata for client customization

### Proximity Alerts
- Device-to-device proximity monitoring
- Same-group constraint enforced
- Radius: 50-100000 meters
- Max 20 alerts per source device
- Haversine distance calculation (PostgreSQL function)

### Webhooks
- Per-device webhook registration (max 10/device)
- HTTPS required for target URLs
- HMAC-SHA256 signature for payload verification
- Secret: 16-256 characters
- Async delivery with retry logic

### Geofence Events
- Track enter/exit/dwell transitions
- Automatic webhook delivery on event creation
- Webhook delivery status tracking per event

### Webhook Delivery (Circuit Breaker)
- Exponential backoff: 0s, 60s, 300s, 900s
- Max 4 retry attempts per delivery
- Circuit breaker: opens after 5 failures
- 5-minute cooldown when circuit is open
- Retry worker skips deliveries when circuit is open

### Settings Control (Epic 12)
- Per-device settings with key-value storage (JSONB)
- Setting lock mechanism for parental control
- Unlock request workflow with approval/denial
- Settings sync endpoint for device state reconciliation
- **Change history tracking**:
  - Audit log of all setting changes (value changes, locks, unlocks)
  - Captures old/new values for each change
  - Preserves history when users are deleted (`ON DELETE SET NULL`)
  - Offset-based pagination with configurable limit (1-100)

### Authentication

**Three authentication methods:**

| Method | Header | Routes | Middleware/Extractor |
|--------|--------|--------|---------------------|
| API Key | `X-API-Key` | Devices, locations, geofences, webhooks, etc. | `require_auth` middleware |
| JWT | `Authorization: Bearer` | User profile, group management | `UserAuth` extractor |
| Admin API Key | `X-API-Key` (admin flag) | Admin routes | `require_admin` middleware |

**API Key Details:**
- SHA-256 hashed storage
- Key prefix (8 chars after `pm_`) for identification via `shared::crypto::extract_key_prefix()`

**JWT Details:**
- RS256 algorithm (RSA-SHA256 with 2048-bit keys)
- OAuth social login (Google, Apple) with proper token validation
- Tokens include user ID, email, and role claims

**Rate Limiting:**
- Per API key for protected endpoints
- Per IP for auth endpoints
- Forgot password: 5/hour, verification: 3/hour

## API Endpoints

### Devices
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/devices/register` | Register/update device |
| GET | `/api/v1/devices?groupId=` | List group devices with last location |
| DELETE | `/api/v1/devices/:device_id` | Deactivate device (soft delete) |

### Locations
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/locations` | Upload single location |
| POST | `/api/v1/locations/batch` | Upload batch locations (max 50) |
| GET | `/api/v1/devices/:device_id/locations` | Get location history (cursor pagination, optional simplification) |

#### Location History Query Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `cursor` | string | Pagination cursor from previous response |
| `limit` | int | Results per page (1-100, default 50) |
| `from` | int64 | Start timestamp (milliseconds) |
| `to` | int64 | End timestamp (milliseconds) |
| `order` | string | Sort order: `asc` or `desc` (default) |
| `tolerance` | float | Simplification tolerance in meters (0-10000). When > 0, applies RDP simplification and disables pagination |

### Geofences
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/geofences` | Create geofence |
| GET | `/api/v1/geofences?deviceId=` | List device geofences |
| GET | `/api/v1/geofences/:geofence_id` | Get geofence |
| PATCH | `/api/v1/geofences/:geofence_id` | Update geofence |
| DELETE | `/api/v1/geofences/:geofence_id` | Delete geofence |

### Proximity Alerts
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/proximity-alerts` | Create proximity alert |
| GET | `/api/v1/proximity-alerts?sourceDeviceId=` | List alerts |
| GET | `/api/v1/proximity-alerts/:alert_id` | Get alert |
| PATCH | `/api/v1/proximity-alerts/:alert_id` | Update alert |
| DELETE | `/api/v1/proximity-alerts/:alert_id` | Delete alert |

### Webhooks
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/webhooks` | Create webhook |
| GET | `/api/v1/webhooks?ownerDeviceId=` | List device webhooks |
| GET | `/api/v1/webhooks/:webhook_id` | Get webhook |
| PUT | `/api/v1/webhooks/:webhook_id` | Update webhook |
| DELETE | `/api/v1/webhooks/:webhook_id` | Delete webhook |

### Geofence Events
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/geofence-events` | Create geofence event (triggers webhook delivery) |
| GET | `/api/v1/geofence-events?deviceId=` | List device events |
| GET | `/api/v1/geofence-events/:event_id` | Get event |

### Trips
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/trips` | Create trip with idempotency |
| PATCH | `/api/v1/trips/:trip_id` | Update trip state (complete/cancel) |
| GET | `/api/v1/devices/:device_id/trips` | List device trips (paginated) |
| GET | `/api/v1/trips/:trip_id/movement-events` | Get trip movement events |
| GET | `/api/v1/trips/:trip_id/path` | Get trip path correction data |
| POST | `/api/v1/trips/:trip_id/correct-path` | Trigger path correction |

### Movement Events
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/movement-events` | Create single movement event |
| POST | `/api/v1/movement-events/batch` | Create batch movement events (max 100) |
| GET | `/api/v1/devices/:device_id/movement-events` | Get device movement events (paginated) |
| GET | `/api/v1/trips/:trip_id/movement-events` | Get trip movement events |

### Privacy (GDPR)
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/devices/:device_id/data-export` | Export device data |
| DELETE | `/api/v1/devices/:device_id/data` | Delete all device data |

### Device Settings (Epic 12)
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/devices/:device_id/settings` | Get all device settings |
| PUT | `/api/v1/devices/:device_id/settings` | Bulk update device settings |
| PUT | `/api/v1/devices/:device_id/settings/:key` | Update single setting |
| GET | `/api/v1/devices/:device_id/settings/locks` | Get all setting locks |
| PUT | `/api/v1/devices/:device_id/settings/locks` | Bulk update setting locks |
| POST | `/api/v1/devices/:device_id/settings/:key/lock` | Lock a setting |
| DELETE | `/api/v1/devices/:device_id/settings/:key/lock` | Unlock a setting |
| POST | `/api/v1/devices/:device_id/settings/sync` | Sync settings from device |
| GET | `/api/v1/devices/:device_id/settings/history` | Get settings change history |
| POST | `/api/v1/devices/:device_id/settings/:key/unlock-request` | Request setting unlock |

#### Settings History Query Parameters
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | int | 50 | Results per page (1-100) |
| `offset` | int | 0 | Number of records to skip |

#### Settings History Response
```json
{
  "changes": [
    {
      "id": "uuid",
      "setting_key": "daily_screen_time_limit",
      "old_value": 3600,
      "new_value": 7200,
      "changed_by": "user-uuid",
      "changed_by_name": "John Doe",
      "changed_at": "2025-12-15T10:30:00Z",
      "change_type": "VALUE_CHANGED"
    }
  ],
  "total_count": 42,
  "has_more": true
}
```

**Change Types**: `VALUE_CHANGED`, `LOCKED`, `UNLOCKED`, `RESET`

**Authorization**: Device owner, group admin, or organization admin

### Admin
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/admin/stats` | Get system statistics |
| DELETE | `/api/v1/admin/devices/inactive` | Delete inactive devices |
| POST | `/api/v1/admin/devices/:device_id/reactivate` | Reactivate device |

#### Admin Managed Users (Epic 9)

Routes for managing users. Org admins manage users in their organizations; non-org admins manage users not in any organization.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/users` | List managed users (paginated, searchable) |
| DELETE | `/api/admin/v1/users/:user_id` | Remove/deactivate managed user |
| GET | `/api/admin/v1/users/:user_id/location` | Get user's last known location |
| PUT | `/api/admin/v1/users/:user_id/tracking` | Update user tracking status |
| GET | `/api/admin/v1/users/:user_id/geofences` | List user's geofences |
| POST | `/api/admin/v1/users/:user_id/geofences` | Create geofence for user |
| PUT | `/api/admin/v1/users/:user_id/geofences/:geofence_id` | Update user geofence |
| DELETE | `/api/admin/v1/users/:user_id/geofences/:geofence_id` | Delete user geofence |

**Query Parameters for List Managed Users:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | int | 1 | Page number |
| `per_page` | int | 20 | Results per page (1-100) |
| `search` | string | - | Search by email or display name |
| `tracking_enabled` | bool | - | Filter by tracking status |

### B2B Organization Admin (requires `require_b2b` middleware)

#### Organizations
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/admin/v1/organizations` | Create organization |
| GET | `/api/admin/v1/organizations` | List organizations |
| GET | `/api/admin/v1/organizations/:org_id` | Get organization |
| PUT | `/api/admin/v1/organizations/:org_id` | Update organization |
| DELETE | `/api/admin/v1/organizations/:org_id` | Delete organization |
| GET | `/api/admin/v1/organizations/:org_id/usage` | Get organization usage metrics |

#### Organization Users
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/admin/v1/organizations/:org_id/users` | Add user to organization |
| GET | `/api/admin/v1/organizations/:org_id/users` | List organization users |
| PUT | `/api/admin/v1/organizations/:org_id/users/:user_id` | Update user role/permissions |
| DELETE | `/api/admin/v1/organizations/:org_id/users/:user_id` | Remove user from organization |

#### Organization API Keys
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/admin/v1/organizations/:org_id/api-keys` | Create organization API key |
| GET | `/api/admin/v1/organizations/:org_id/api-keys` | List organization API keys |
| GET | `/api/admin/v1/organizations/:org_id/api-keys/:key_id` | Get API key details |
| PATCH | `/api/admin/v1/organizations/:org_id/api-keys/:key_id` | Update API key (name/description) |
| DELETE | `/api/admin/v1/organizations/:org_id/api-keys/:key_id` | Revoke API key |

#### Organization Member Invitations
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/admin/v1/organizations/:org_id/invitations` | Create member invitation |
| GET | `/api/admin/v1/organizations/:org_id/invitations` | List invitations |
| GET | `/api/admin/v1/organizations/:org_id/invitations/:invite_id` | Get invitation details |
| DELETE | `/api/admin/v1/organizations/:org_id/invitations/:invite_id` | Revoke invitation |
| POST | `/api/v1/invitations/:token/accept` | Accept invitation (public) |

#### Organization Webhooks
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/admin/v1/organizations/:org_id/webhooks` | Create organization webhook |
| GET | `/api/admin/v1/organizations/:org_id/webhooks` | List organization webhooks |
| GET | `/api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | Get webhook details |
| PUT | `/api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | Update webhook |
| DELETE | `/api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | Delete webhook |

**Supported Webhook Event Types:**
- `device.enrolled`, `device.unenrolled`, `device.assigned`, `device.unassigned`
- `member.joined`, `member.removed`
- `policy.applied`, `policy.updated`

#### Device Policies
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/admin/v1/organizations/:org_id/policies` | Create device policy |
| GET | `/api/admin/v1/organizations/:org_id/policies` | List policies |
| GET | `/api/admin/v1/organizations/:org_id/policies/:policy_id` | Get policy |
| PUT | `/api/admin/v1/organizations/:org_id/policies/:policy_id` | Update policy |
| DELETE | `/api/admin/v1/organizations/:org_id/policies/:policy_id` | Delete policy |
| POST | `/api/admin/v1/organizations/:org_id/policies/:policy_id/apply` | Apply policy to targets |
| POST | `/api/admin/v1/organizations/:org_id/policies/:policy_id/unapply` | Remove policy from targets |

#### Enrollment Tokens
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/admin/v1/organizations/:org_id/enrollment-tokens` | Create enrollment token |
| GET | `/api/admin/v1/organizations/:org_id/enrollment-tokens` | List enrollment tokens |
| GET | `/api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id` | Get token details |
| DELETE | `/api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id` | Revoke token |
| GET | `/api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id/qr` | Get QR code |

#### Fleet Management
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/devices` | List fleet devices |
| GET | `/api/admin/v1/organizations/:org_id/devices/summary` | Get fleet summary |
| POST | `/api/admin/v1/organizations/:org_id/devices/:device_id/assign` | Assign device to user |
| DELETE | `/api/admin/v1/organizations/:org_id/devices/:device_id/assign` | Unassign device |
| POST | `/api/admin/v1/organizations/:org_id/devices/:device_id/commands` | Issue device command |

#### Dashboard & Audit
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/dashboard` | Get dashboard metrics |
| GET | `/api/admin/v1/organizations/:org_id/audit-logs` | List audit logs |
| GET | `/api/admin/v1/organizations/:org_id/audit-logs/export` | Export audit logs (async) |

#### User Administration (JWT Auth)
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/admin-users` | List users with details |
| POST | `/api/admin/v1/organizations/:org_id/admin-users` | Create/add user to org |
| GET | `/api/admin/v1/organizations/:org_id/admin-users/:user_id` | Get user details |
| PUT | `/api/admin/v1/organizations/:org_id/admin-users/:user_id` | Update user |
| DELETE | `/api/admin/v1/organizations/:org_id/admin-users/:user_id` | Remove user |
| POST | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/suspend` | Suspend user |
| POST | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/reactivate` | Reactivate user |
| POST | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/reset-password` | Trigger password reset |
| GET | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/mfa` | Get MFA status |
| POST | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/mfa/force` | Force MFA enrollment |
| DELETE | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/mfa` | Reset MFA |
| GET | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/sessions` | List user sessions |
| DELETE | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/sessions/:session_id` | Revoke session |
| DELETE | `/api/admin/v1/organizations/:org_id/admin-users/:user_id/sessions` | Revoke all sessions |

#### Group Administration
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/admin-groups` | List groups |
| GET | `/api/admin/v1/organizations/:org_id/admin-groups/:group_id` | Get group details |
| PUT | `/api/admin/v1/organizations/:org_id/admin-groups/:group_id` | Update group |
| DELETE | `/api/admin/v1/organizations/:org_id/admin-groups/:group_id` | Deactivate group |
| GET | `/api/admin/v1/organizations/:org_id/admin-groups/:group_id/members` | List group members |
| POST | `/api/admin/v1/organizations/:org_id/admin-groups/:group_id/members` | Add member |
| DELETE | `/api/admin/v1/organizations/:org_id/admin-groups/:group_id/members/:user_id` | Remove member |
| GET | `/api/admin/v1/organizations/:org_id/admin-groups/:group_id/invitations` | List invitations |
| POST | `/api/admin/v1/organizations/:org_id/admin-groups/:group_id/invitations` | Create invitation |

#### Geofence Administration
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/admin-geofences` | List geofences |
| POST | `/api/admin/v1/organizations/:org_id/admin-geofences` | Create geofence |
| GET | `/api/admin/v1/organizations/:org_id/admin-geofences/:geofence_id` | Get geofence |
| PUT | `/api/admin/v1/organizations/:org_id/admin-geofences/:geofence_id` | Update geofence |
| DELETE | `/api/admin/v1/organizations/:org_id/admin-geofences/:geofence_id` | Delete geofence |
| GET | `/api/admin/v1/organizations/:org_id/admin-geofences/events` | List geofence events |
| GET | `/api/admin/v1/organizations/:org_id/devices/:device_id/locations` | Get device locations |
| GET | `/api/admin/v1/organizations/:org_id/devices/locations` | Get all device locations |
| GET | `/api/admin/v1/organizations/:org_id/locations/analytics` | Location analytics |

#### App Usage & Unlock Requests
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/app-usage/summary` | App usage summary |
| GET | `/api/admin/v1/organizations/:org_id/devices/:device_id/app-usage` | Device app usage |
| GET | `/api/admin/v1/organizations/:org_id/app-usage/analytics` | App usage analytics |
| GET | `/api/admin/v1/organizations/:org_id/unlock-requests` | List unlock requests |
| GET | `/api/admin/v1/organizations/:org_id/unlock-requests/:request_id` | Get unlock request |
| POST | `/api/admin/v1/organizations/:org_id/unlock-requests/:request_id/approve` | Approve request |
| POST | `/api/admin/v1/organizations/:org_id/unlock-requests/:request_id/deny` | Deny request |
| POST | `/api/admin/v1/organizations/:org_id/unlock-requests/bulk` | Bulk process requests |

#### Analytics & Reports
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/analytics/users` | User analytics |
| GET | `/api/admin/v1/organizations/:org_id/analytics/devices` | Device analytics |
| GET | `/api/admin/v1/organizations/:org_id/analytics/api` | API usage analytics |
| POST | `/api/admin/v1/organizations/:org_id/reports/users` | Generate user report |
| POST | `/api/admin/v1/organizations/:org_id/reports/devices` | Generate device report |
| GET | `/api/admin/v1/organizations/:org_id/reports/:report_id/status` | Get report status |
| GET | `/api/admin/v1/organizations/:org_id/reports/:report_id/download` | Download report |

#### System Configuration (Super Admin)
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/system/settings` | Get system settings |
| PUT | `/api/admin/v1/system/settings` | Update system settings |
| GET | `/api/admin/v1/system/feature-flags` | List feature flags |
| GET | `/api/admin/v1/system/feature-flags/:flag_id` | Get feature flag |
| PUT | `/api/admin/v1/system/feature-flags/:flag_id` | Update feature flag |
| GET | `/api/admin/v1/system/rate-limits` | Get rate limits |
| PUT | `/api/admin/v1/system/rate-limits` | Update rate limits |
| GET | `/api/admin/v1/system/maintenance` | Get maintenance status |
| POST | `/api/admin/v1/system/maintenance` | Toggle maintenance mode |
| GET | `/api/admin/v1/system/email-templates` | List email templates |
| PUT | `/api/admin/v1/system/email-templates/:template_id` | Update email template |
| GET | `/api/admin/v1/system/notification-templates` | List notification templates |
| PUT | `/api/admin/v1/system/notification-templates/:template_id` | Update notification template |

#### Organization Settings
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/settings` | Get org settings |
| PUT | `/api/admin/v1/organizations/:org_id/settings` | Update org settings |
| POST | `/api/admin/v1/organizations/:org_id/settings/verify-pin` | Verify security PIN |

#### Compliance
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/admin/v1/organizations/:org_id/compliance/dashboard` | Compliance dashboard |
| GET | `/api/admin/v1/organizations/:org_id/compliance/report` | Compliance report |
| GET | `/api/admin/v1/organizations/:org_id/data-subject-requests` | List DSR requests |
| POST | `/api/admin/v1/organizations/:org_id/data-subject-requests` | Create DSR |
| GET | `/api/admin/v1/organizations/:org_id/data-subject-requests/:request_id` | Get DSR |
| POST | `/api/admin/v1/organizations/:org_id/data-subject-requests/:request_id/process` | Process DSR |

### Health
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/health/live` | Liveness probe |
| GET | `/api/health/ready` | Readiness probe |
| GET | `/metrics` | Prometheus metrics |

### Documentation
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/docs/` | Swagger UI |
| GET | `/api/docs/openapi.yaml` | OpenAPI spec |

## Development Commands

```bash
# Build
cargo build --workspace
cargo build --release --bin phone-manager

# Test
cargo test --workspace
cargo test --workspace -- --nocapture

# Lint & Format
cargo fmt --all
cargo clippy --workspace -- -D warnings

# Database
sqlx database create
sqlx migrate run --source crates/persistence/src/migrations
cargo sqlx prepare --workspace  # Generate offline query data

# Run
cargo run --bin phone-manager
```

## Performance Requirements

| Metric | Target |
|--------|--------|
| API Response Time | < 200ms (p95) |
| Uptime | 99.9% |
| Concurrent Connections | 10,000+ |
| Max Batch Size | 50 locations |

## Implementation Phases

### Phase 1: Foundation ✓
- Project structure, config, database
- API key authentication
- Device registration
- Health checks

### Phase 2: Location Tracking
- Single/batch location upload
- Group device listing
- Last location aggregation

### Phase 3: Production Readiness
- Prometheus metrics
- Rate limiting
- Security hardening
- Background jobs

## Code Conventions

### Rust Style
- Use `#[derive(Debug, Clone)]` on all public types
- Prefer `thiserror` for error types
- Use `tracing` macros for logging
- Validate inputs at API boundary with `validator`

### Naming
- Snake case for Rust identifiers
- snake_case for JSON serialization (`#[serde(rename_all = "snake_case")]`)
- Prefix environment vars with `PM__`

### Testing
- Unit tests in module `#[cfg(test)]` blocks
- Integration tests in `tests/` directory
- Use `fake` crate for test data generation
- Test database isolation per test

## Key Files Reference

- **Entry Point**: `crates/api/src/main.rs`
- **App Builder**: `crates/api/src/app.rs`
- **Configuration**: `crates/api/src/config.rs`
- **Error Types**: `crates/api/src/error.rs`
- **Webhook Delivery**: `crates/api/src/services/webhook_delivery.rs`
- **Device Settings**: `crates/api/src/routes/device_settings.rs`
- **Setting Change Models**: `crates/domain/src/models/setting_change.rs`
- **Setting Change Repository**: `crates/persistence/src/repositories/setting_change.rs`
- **Migrations**: `crates/persistence/src/migrations/`
- **Spec Document**: `docs/rust-backend-spec.md`

## Dependencies Policy

- Prefer workspace dependencies in root `Cargo.toml`
- Use stable, well-maintained crates only
- Pin major versions for stability
- Document rationale for non-obvious dependencies

## Security Considerations

- Never log API keys or tokens
- Validate all user inputs
- Use parameterized queries (SQLx enforces this)
- Rate limit API endpoints
- Hash API keys with SHA-256 before storage
