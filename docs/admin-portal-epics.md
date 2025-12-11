# Admin Portal Backend API - Epic Definitions

**Project:** phone-manager-backend
**PRD Reference:** PRD-admin-portal.md
**Date:** 2025-12-11
**Author:** Martin Janci

---

## Epic Overview

| Epic ID | Name | Endpoints | Priority | Status |
|---------|------|-----------|----------|--------|
| AP-1 | RBAC & Access Control | 3 | High | 100% ✅ |
| AP-2 | Organization Management | 7 | High | 100% ✅ |
| AP-3 | User Administration | 13 | High | 100% ✅ |
| AP-4 | Device Fleet Administration | 15 | High | 100% ✅ |
| AP-5 | Groups Administration | 10 | Medium | 100% ✅ |
| AP-6 | Location & Geofence Administration | 13 | Low | 100% ✅ |
| AP-7 | Webhook Administration | 9 | High | 100% ✅ |
| AP-8 | App Usage & Unlock Requests | 13 | Low | 100% ✅ |
| AP-9 | System Configuration | 14 | Medium | 100% ✅ |
| AP-10 | Dashboard & Analytics | 10 | Medium | 100% ✅ |
| AP-11 | Audit & Compliance | 8 | High | 100% ✅ |

**Total:** 115 endpoints across 11 epics
**Current Implementation:** 115 endpoints (100%)
**Remaining:** 0 endpoints (0%)

### ⚠️ Known Discrepancies
- **AP-3**: Route path is `/admin-users` not `/users` - documented correctly to match implementation

---

## AP-1: RBAC & Access Control

**Priority:** High
**Status:** 100% ✅ (3/3 endpoints)
**Dependencies:** None (foundation epic)

### Description
Implement comprehensive role-based access control allowing organizations to define custom roles with granular permissions. This epic provides the security foundation for all other admin features.

### User Stories

#### AP-1.1: List Permissions ✅
**As an** organization admin
**I want to** view all available system permissions
**So that** I can understand what capabilities can be assigned to roles

**Acceptance Criteria:**
- [x] GET `/api/admin/v1/organizations/:org_id/permissions` returns all permissions
- [x] Permissions grouped by category (users, devices, groups, etc.)
- [x] Each permission includes name, description, and category
- [x] Response supports filtering by category
- [x] Requires admin authentication

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/permissions`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/permissions.rs`

---

#### AP-1.2: Create Custom Role ✅
**As an** organization admin
**I want to** create custom roles with specific permissions
**So that** I can define access levels appropriate for my organization

**Acceptance Criteria:**
- [x] POST `/api/admin/v1/organizations/:org_id/roles` creates a new role
- [x] Role name must be unique within organization
- [x] Permissions array validated against available permissions
- [x] System roles (admin, viewer) cannot be duplicated
- [x] Returns created role with ID and permissions
- [x] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/roles`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/roles.rs`

---

#### AP-1.3: Delete Custom Role ✅
**As an** organization admin
**I want to** delete custom roles no longer needed
**So that** I can maintain a clean role structure

**Acceptance Criteria:**
- [x] DELETE `/api/admin/v1/organizations/:org_id/roles/:role_id` removes role
- [x] System roles cannot be deleted (return 403)
- [x] Roles with assigned users cannot be deleted (return 409)
- [x] Audit log entry created
- [x] Returns 204 No Content on success

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/roles/:role_id`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/roles.rs`

---

### Technical Notes
- Permission system should be extensible for future features
- Consider caching permission list (changes infrequently)
- Role assignments stored in `organization_users` table

---

## AP-2: Organization Management

**Priority:** High
**Status:** 100% ✅ (7/7 endpoints)
**Dependencies:** AP-1 (RBAC)

### Description
Complete organization lifecycle management including creation, configuration, usage tracking, and suspension capabilities.

### User Stories

#### AP-2.1: Create Organization ✅
**Status:** ✅ Implemented
**Endpoint:** `POST /api/admin/v1/organizations`
**File:** `crates/api/src/routes/organizations.rs`

---

#### AP-2.2: List Organizations ✅
**Status:** ✅ Implemented
**Endpoint:** `GET /api/admin/v1/organizations`
**File:** `crates/api/src/routes/organizations.rs`

---

#### AP-2.3: Get Organization Details ✅
**Status:** ✅ Implemented
**Endpoint:** `GET /api/admin/v1/organizations/:org_id`
**File:** `crates/api/src/routes/organizations.rs`

---

#### AP-2.4: Update Organization ✅
**Status:** ✅ Implemented
**Endpoint:** `PUT /api/admin/v1/organizations/:org_id`
**File:** `crates/api/src/routes/organizations.rs`

---

#### AP-2.5: Delete Organization ✅
**Status:** ✅ Implemented
**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id`
**File:** `crates/api/src/routes/organizations.rs`

---

#### AP-2.6: Get Organization Usage ✅
**As an** organization admin
**I want to** view usage metrics
**So that** I can monitor resource consumption

**Acceptance Criteria:**
- [x] GET `/api/admin/v1/organizations/:org_id/usage` returns metrics
- [x] User count vs quota
- [x] Device count vs quota
- [x] API call counts
- [x] Storage usage
- [x] Supports date range filtering

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/usage`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/organizations.rs`

---

#### AP-2.7: Suspend/Reactivate Organization ✅
**As a** platform admin
**I want to** suspend organizations for policy violations
**So that** I can enforce platform terms

**Acceptance Criteria:**
- [x] POST `/api/admin/v1/organizations/:org_id/suspend` suspends org
- [x] POST `/api/admin/v1/organizations/:org_id/reactivate` reactivates
- [x] Suspended orgs: users cannot login, API calls rejected
- [x] Data preserved during suspension
- [x] Audit log entries created

**Endpoints:**
- `POST /api/admin/v1/organizations/:org_id/suspend` - ✅ Implemented
- `POST /api/admin/v1/organizations/:org_id/reactivate` - ✅ Implemented
**File:** `crates/api/src/routes/organizations.rs`

---

### Technical Notes
- Organization settings stored as JSONB for flexibility
- Usage metrics should be cached and updated periodically
- Suspension should be immediate and reversible

---

## AP-3: User Administration

**Priority:** High
**Status:** 100% ✅ (13/13 endpoints)
**Dependencies:** AP-1 (RBAC), AP-2 (Organizations)

### Description
Comprehensive user lifecycle management including creation, suspension, password management, MFA administration, and session management.

### ⚠️ Route Path Discrepancy
**Documented Path:** `/api/admin/v1/organizations/:org_id/users`
**Actual Path:** `/api/admin/v1/organizations/:org_id/admin-users`

The actual routes are mounted at `/admin-users`, not `/users`. This needs alignment for API contract consistency.

### User Stories

#### AP-3.1: List Users ✅
**Status:** ✅ Implemented
**Documented Endpoint:** `GET /api/admin/v1/organizations/:org_id/users`
**Actual Endpoint:** `GET /api/admin/v1/organizations/:org_id/admin-users`
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.2: Get User Details ✅
**Status:** ✅ Implemented
**Documented Endpoint:** `GET /api/admin/v1/organizations/:org_id/users/:user_id`
**Actual Endpoint:** `GET /api/admin/v1/organizations/:org_id/admin-users/:user_id`
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.3: Create User ✅
**Status:** ✅ Implemented
**Documented Endpoint:** `POST /api/admin/v1/organizations/:org_id/users`
**Actual Endpoints:**
- `POST /api/admin/v1/organizations/:org_id/users` (API key auth, in organizations.rs)
- `POST /api/admin/v1/organizations/:org_id/admin-users` (JWT auth, in admin_users.rs)
**File:** `crates/api/src/routes/admin_users.rs`, `crates/api/src/routes/organizations.rs`

---

#### AP-3.4: Update User ✅
**Status:** ✅ Implemented
**Documented Endpoint:** `PUT /api/admin/v1/organizations/:org_id/users/:user_id`
**Actual Endpoint:** `PUT /api/admin/v1/organizations/:org_id/admin-users/:user_id`
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.5: Suspend User ✅
**As an** organization admin
**I want to** suspend user accounts
**So that** I can revoke access without deleting data

**Acceptance Criteria:**
- [x] POST `/api/admin/v1/organizations/:org_id/users/:user_id/suspend` suspends user
- [x] Immediately invalidates all sessions
- [x] User cannot login while suspended
- [x] Data and assignments preserved
- [x] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users/:user_id/suspend`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.6: Reactivate User ✅
**As an** organization admin
**I want to** reactivate suspended users
**So that** they can resume platform access

**Acceptance Criteria:**
- [x] POST `/api/admin/v1/organizations/:org_id/users/:user_id/reactivate` reactivates
- [x] User can login after reactivation
- [x] Previous assignments restored
- [x] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users/:user_id/reactivate`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.7: Trigger Password Reset ✅
**As an** organization admin
**I want to** trigger password resets for users
**So that** I can help users who are locked out

**Acceptance Criteria:**
- [x] POST `/api/admin/v1/organizations/:org_id/users/:user_id/reset-password` sends email
- [x] Generates secure reset token
- [x] Token expires in 24 hours
- [x] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users/:user_id/reset-password`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.8: Get MFA Status ✅
**As an** organization admin
**I want to** view user MFA status
**So that** I can ensure security compliance

**Acceptance Criteria:**
- [x] GET `/api/admin/v1/organizations/:org_id/users/:user_id/mfa` returns status
- [x] Shows if MFA is enabled
- [x] Shows MFA method (TOTP, etc.)
- [x] Shows enrollment date

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/users/:user_id/mfa`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.9: Force MFA Enrollment ✅
**As an** organization admin
**I want to** require MFA for specific users
**So that** I can enforce security policies

**Acceptance Criteria:**
- [x] POST `/api/admin/v1/organizations/:org_id/users/:user_id/mfa/force` enables requirement
- [x] User prompted to set up MFA on next login
- [x] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users/:user_id/mfa/force`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.10: Reset User MFA ✅
**As an** organization admin
**I want to** reset user MFA
**So that** users who lose their device can re-enroll

**Acceptance Criteria:**
- [x] DELETE `/api/admin/v1/organizations/:org_id/users/:user_id/mfa` resets MFA
- [x] Removes current MFA configuration
- [x] User must re-enroll
- [x] Audit log entry created

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/users/:user_id/mfa`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.11: List User Sessions ✅
**As an** organization admin
**I want to** view user active sessions
**So that** I can monitor for suspicious activity

**Acceptance Criteria:**
- [x] GET `/api/admin/v1/organizations/:org_id/users/:user_id/sessions` returns sessions
- [x] Shows device/browser info
- [x] Shows IP address and location
- [x] Shows session start time
- [x] Shows last activity time

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/users/:user_id/sessions`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.12: Revoke Session ✅
**As an** organization admin
**I want to** revoke specific user sessions
**So that** I can terminate suspicious sessions

**Acceptance Criteria:**
- [x] DELETE `/api/admin/v1/organizations/:org_id/users/:user_id/sessions/:session_id` revokes
- [x] Session immediately invalidated
- [x] User must re-authenticate on that device
- [x] Audit log entry created

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/users/:user_id/sessions/:session_id`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

#### AP-3.13: Revoke All Sessions ✅
**As an** organization admin
**I want to** revoke all user sessions
**So that** I can force complete re-authentication

**Acceptance Criteria:**
- [x] DELETE `/api/admin/v1/organizations/:org_id/users/:user_id/sessions` revokes all
- [x] All sessions immediately invalidated
- [x] User must re-authenticate everywhere
- [x] Audit log entry created

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/users/:user_id/sessions`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_users.rs`

---

### Technical Notes
- Session management requires new sessions table
- MFA requires TOTP library integration
- Consider session table with Redis for performance
- Password reset uses existing email infrastructure

---

## AP-4: Device Fleet Administration

**Priority:** High
**Status:** 100% ✅ (15/15 endpoints)
**Dependencies:** AP-2 (Organizations), AP-3 (Users)

### Description
Comprehensive device fleet management including listing, updating, bulk operations, and remote commands for enterprise device management.

### Implemented Endpoints ✅

| Endpoint | Status | File |
|----------|--------|------|
| `GET /api/admin/v1/organizations/:org_id/devices` | ✅ | `fleet.rs` |
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id` | ✅ | `fleet.rs` |
| `PATCH /api/admin/v1/organizations/:org_id/devices/:device_id` | ✅ | `fleet.rs` |
| `DELETE /api/admin/v1/organizations/:org_id/devices/:device_id` | ✅ | `fleet.rs` |
| `POST /api/admin/v1/organizations/:org_id/devices/:device_id/reactivate` | ✅ | `fleet.rs` |
| `POST /api/admin/v1/organizations/:org_id/devices/:device_id/assign` | ✅ | `fleet.rs` |
| `DELETE /api/admin/v1/organizations/:org_id/devices/:device_id/assign` | ✅ | `fleet.rs` |
| `GET /api/admin/v1/organizations/:org_id/devices/summary` | ✅ | `fleet.rs` |

### Additional Implemented Endpoints

#### AP-4.4: Bulk Update Devices ✅
**As an** organization admin
**I want to** update multiple devices at once
**So that** I can efficiently manage large fleets

**Acceptance Criteria:**
- [x] POST `/api/admin/v1/organizations/:org_id/devices/bulk-update` updates multiple
- [x] Accepts array of device IDs and updates
- [x] Maximum 100 devices per request
- [x] Returns success/failure per device
- [x] Audit log entries created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/devices/bulk-update`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/fleet.rs`

---

#### AP-4.9: Get Device Command History ✅
**As an** organization admin
**I want to** view device command history
**So that** I can track administrative actions

**Acceptance Criteria:**
- [x] GET `/api/admin/v1/organizations/:org_id/devices/:device_id/commands` returns history
- [x] Shows command type, status, timestamp
- [x] Shows who issued the command
- [x] Cursor-based pagination

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/devices/:device_id/commands`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/fleet.rs`

---

#### AP-4.10: Issue Device Command ✅
**Status:** ✅ Implemented
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/devices/:device_id/commands`
**File:** `crates/api/src/routes/fleet.rs`

---

#### AP-4.11-15: Enrollment Token Management ✅
**Status:** All Implemented
**File:** `crates/api/src/routes/enrollment_tokens.rs`

**Endpoints:**
- `POST /api/admin/v1/organizations/:org_id/enrollment-tokens` ✅
- `GET /api/admin/v1/organizations/:org_id/enrollment-tokens` ✅
- `GET /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id` ✅
- `DELETE /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id` ✅
- `GET /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id/qr` ✅

---

### Technical Notes
- Bulk operations should use database transactions
- Device commands use FCM for delivery
- Consider rate limiting on command issuance

---

## AP-5: Groups Administration

**Priority:** Medium
**Status:** 100% ✅ (10/10 endpoints)
**Dependencies:** AP-3 (Users), AP-4 (Devices)

### Description
Group management for organizing users and devices, including membership management and group-level settings.

### Implemented Endpoints ✅

| Endpoint | Status | File |
|----------|--------|------|
| `GET /api/admin/v1/organizations/:org_id/groups` | ✅ | `admin_groups.rs` |
| `GET /api/admin/v1/organizations/:org_id/groups/:group_id` | ✅ | `admin_groups.rs` |
| `POST /api/admin/v1/organizations/:org_id/groups` | ✅ | `admin_groups.rs` |
| `PUT /api/admin/v1/organizations/:org_id/groups/:group_id` | ✅ | `admin_groups.rs` |
| `DELETE /api/admin/v1/organizations/:org_id/groups/:group_id` | ✅ | `admin_groups.rs` |

### Member Management Endpoints ✅

#### AP-5.6: List Group Members ✅
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/groups/:group_id/members`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_groups.rs`

---

#### AP-5.7: Add Group Member ✅
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/groups/:group_id/members`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_groups.rs`

---

#### AP-5.8: Remove Group Member ✅
**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/groups/:group_id/members/:member_id`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/admin_groups.rs`

---

#### AP-5.9-10: Group Invitations ✅
**Endpoints:**
- `POST /api/admin/v1/organizations/:org_id/groups/:group_id/invitations` - ✅ Implemented
- `GET /api/admin/v1/organizations/:org_id/groups/:group_id/invitations` - ✅ Implemented
**File:** `crates/api/src/routes/admin_groups.rs`

---

## AP-6: Location & Geofence Administration

**Priority:** Low
**Status:** 100% ✅ (13/13 endpoints)
**Dependencies:** AP-4 (Devices)

### Description
Administrative management of location data and organization-wide geofences.

### Implemented Endpoints ✅

| Endpoint | Description | File |
|----------|-------------|------|
| `GET /api/admin/v1/organizations/:org_id/geofences` | List admin geofences | `admin_geofences.rs` |
| `POST /api/admin/v1/organizations/:org_id/geofences` | Create admin geofence | `admin_geofences.rs` |
| `GET /api/admin/v1/organizations/:org_id/geofences/:geofence_id` | Get geofence | `admin_geofences.rs` |
| `PUT /api/admin/v1/organizations/:org_id/geofences/:geofence_id` | Update geofence | `admin_geofences.rs` |
| `DELETE /api/admin/v1/organizations/:org_id/geofences/:geofence_id` | Delete geofence | `admin_geofences.rs` |
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id/location` | Get device location | `admin_locations.rs` |
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id/location-history` | Get location history | `admin_locations.rs` |
| `GET /api/admin/v1/organizations/:org_id/locations/current` | All device locations | `admin_locations.rs` |
| `GET /api/admin/v1/organizations/:org_id/locations/history` | Org location history | `admin_locations.rs` |
| `GET /api/admin/v1/organizations/:org_id/geofence-events` | Geofence events | `admin_geofences.rs` |
| `GET /api/admin/v1/organizations/:org_id/location-analytics` | Location analytics | `admin_locations.rs` |

### Features
- Trajectory simplification via Ramer-Douglas-Peucker algorithm
- Date range filtering
- Cursor-based pagination for history
- Visit counts and dwell time analytics

---

## AP-7: Webhook Administration

**Priority:** High
**Status:** 100% ✅ (9/9 endpoints)
**Dependencies:** AP-2 (Organizations)

### Description
Complete webhook lifecycle management including testing, delivery logs, and retry capabilities.

### Implemented Endpoints ✅

| Endpoint | Status | File |
|----------|--------|------|
| `GET /api/admin/v1/organizations/:org_id/webhooks` | ✅ | `org_webhooks.rs` |
| `POST /api/admin/v1/organizations/:org_id/webhooks` | ✅ | `org_webhooks.rs` |
| `GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | ✅ | `org_webhooks.rs` |
| `PUT /api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | ✅ | `org_webhooks.rs` |
| `DELETE /api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | ✅ | `org_webhooks.rs` |

### Additional Endpoints ✅

#### AP-7.5: Test Webhook ✅
**As an** organization admin
**I want to** test webhook endpoints
**So that** I can verify configuration before enabling

**Acceptance Criteria:**
- [x] POST `/api/admin/v1/organizations/:org_id/webhooks/:webhook_id/test` sends test
- [x] Sends test payload to webhook URL
- [x] Returns response status and timing
- [x] Does not affect delivery statistics

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/test`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/org_webhooks.rs`

---

#### AP-7.6: Get Delivery Logs ✅
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/deliveries`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/org_webhooks.rs`

---

#### AP-7.7: Retry Delivery ✅
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/deliveries/:delivery_id/retry`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/org_webhooks.rs`

---

#### AP-7.8: Get Delivery Statistics ✅
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/stats`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/org_webhooks.rs`

### Supported Event Types
- `device.enrolled`, `device.unenrolled`, `device.assigned`, `device.unassigned`
- `member.joined`, `member.removed`
- `policy.applied`, `policy.updated`

---

## AP-8: App Usage & Unlock Requests

**Priority:** Low
**Status:** 100% ✅ (11/11 endpoints)
**Dependencies:** AP-4 (Devices)

### Description
App usage tracking and unlock request management for device settings.

### App Usage Endpoints ✅

| Endpoint | Description | File |
|----------|-------------|------|
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id/app-usage` | App usage summary | `app_usage.rs` |
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id/app-usage/history` | Usage history | `app_usage.rs` |
| `GET /api/admin/v1/organizations/:org_id/app-usage/analytics` | Org-wide analytics | `app_usage.rs` |

### Unlock Request Endpoints ✅

| Endpoint | Description | File |
|----------|-------------|------|
| `GET /api/admin/v1/organizations/:org_id/unlock-requests` | List requests | `admin_unlock_requests.rs` |
| `GET /api/admin/v1/organizations/:org_id/unlock-requests/:request_id` | Get request | `admin_unlock_requests.rs` |
| `POST /api/admin/v1/organizations/:org_id/unlock-requests/:request_id/approve` | Approve | `admin_unlock_requests.rs` |
| `POST /api/admin/v1/organizations/:org_id/unlock-requests/:request_id/deny` | Deny | `admin_unlock_requests.rs` |
| `POST /api/admin/v1/organizations/:org_id/unlock-requests/bulk-process` | Bulk process | `admin_unlock_requests.rs` |

### User-Level Device Settings ✅

| Endpoint | Description | File |
|----------|-------------|------|
| `GET /api/v1/devices/:device_id/settings` | Get device settings | `device_settings.rs` |
| `PUT /api/v1/devices/:device_id/settings` | Update device settings | `device_settings.rs` |
| `POST /api/v1/devices/:device_id/settings/:key/unlock-request` | Create unlock request | `device_settings.rs` |

### Features (Implemented)
- App usage tracking per device with date filtering
- Top apps by foreground time
- Usage history with pagination
- Organization-wide usage analytics with trends
- Setting lock/unlock workflow
- Admin approval for unlock requests
- Bulk processing capabilities

### Database Migration
- Migration `051_app_usage.sql` adds `app_usage` and `app_usage_daily_aggregates` tables

---

## AP-9: System Configuration

**Priority:** Medium
**Status:** 100% ✅ (14/14 endpoints)
**Dependencies:** AP-2 (Organizations)

### Description
System-wide configuration management including settings, templates, rate limits, and feature flags.

### System Configuration Endpoints ✅

| Endpoint | Description | File |
|----------|-------------|------|
| `GET /api/admin/v1/system/settings` | Get system settings | `system_config.rs` |
| `GET /api/admin/v1/system/feature-flags` | List feature flags | `system_config.rs` |
| `PUT /api/admin/v1/system/feature-flags/:flag_id` | Update feature flag (FR-9.5) | `system_config.rs` |
| `GET /api/admin/v1/system/rate-limits` | Get rate limits | `system_config.rs` |
| `PUT /api/admin/v1/system/rate-limits` | Configure rate limits (FR-9.4) | `system_config.rs` |
| `GET /api/admin/v1/system/maintenance` | Get maintenance status | `system_config.rs` |
| `POST /api/admin/v1/system/maintenance` | Toggle maintenance | `system_config.rs` |
| `GET /api/admin/v1/system/templates` | List notification templates (FR-9.3) | `system_config.rs` |
| `PUT /api/admin/v1/system/templates/:template_id` | Update notification template (FR-9.3) | `system_config.rs` |
| `GET /api/admin/v1/system/email-templates` | List email templates (FR-9.6) | `system_config.rs` |
| `PUT /api/admin/v1/system/email-templates/:template_id` | Update email template (FR-9.6) | `system_config.rs` |

### Organization Settings ✅

| Endpoint | Description | File |
|----------|-------------|------|
| `GET /api/admin/v1/organizations/:org_id/settings` | Get org settings | `organization_settings.rs` |
| `PUT /api/admin/v1/organizations/:org_id/settings` | Update org settings | `organization_settings.rs` |
| `POST /api/admin/v1/organizations/:org_id/settings/verify-pin` | Verify unlock PIN | `organization_settings.rs` |

### Features (Implemented)
- Super admin role enforcement
- In-memory maintenance mode state
- Database-backed feature flag management with enable/disable
- Database-backed rate limit configuration
- Notification template management (push, in-app, SMS)
- Email template management (welcome, password reset, verification, invitation)
- Unlock PIN with Argon2 hashing
- Full audit trail via updated_by tracking

### Database Tables
- `feature_flags` - Feature flag states with category grouping
- `rate_limit_configs` - Rate limit configurations per endpoint type
- `notification_templates` - Push/in-app notification templates
- `email_templates` - HTML and text email templates
- `system_settings` - Key-value store for runtime settings

---

## AP-10: Dashboard & Analytics

**Priority:** Medium
**Status:** 100% ✅ (10/10 endpoints)
**Dependencies:** AP-2, AP-3, AP-4

### Description
Dashboard metrics and analytics reporting for organizational insights.

### Implemented Endpoints ✅

| Endpoint | Description | File |
|----------|-------------|------|
| `GET /api/admin/v1/organizations/:org_id/dashboard` | Dashboard metrics | `dashboard.rs` |
| `GET /api/admin/v1/organizations/:org_id/compliance` | Compliance dashboard | `compliance.rs` |
| `GET /api/admin/v1/organizations/:org_id/compliance/report` | Compliance report | `compliance.rs` |
| `GET /api/admin/v1/organizations/:org_id/analytics/users` | User analytics | `analytics.rs` |
| `GET /api/admin/v1/organizations/:org_id/analytics/devices` | Device analytics | `analytics.rs` |
| `GET /api/admin/v1/organizations/:org_id/analytics/api` | API usage analytics | `analytics.rs` |
| `POST /api/admin/v1/organizations/:org_id/reports/users` | Generate user report | `analytics.rs` |
| `POST /api/admin/v1/organizations/:org_id/reports/devices` | Generate device report | `analytics.rs` |
| `GET /api/admin/v1/organizations/:org_id/reports/:report_id/status` | Check report status | `analytics.rs` |
| `GET /api/admin/v1/organizations/:org_id/reports/:report_id/download` | Download report | `analytics.rs` |

### Dashboard Metrics Include
- Device counts (total, enrolled, pending, suspended, retired)
- User counts (total, active, suspended)
- Group counts
- Policy counts
- Enrollment trends
- Activity summaries

### Compliance Features
- DSR statistics (pending, in_progress, completed, rejected, overdue)
- Audit log statistics (24h, 7d, 30d counts)
- Data retention status
- Compliance scoring (0-100)
- Compliance status (Compliant, NeedsAttention, NonCompliant)
- Findings with severity levels
- Automated recommendations

### User Analytics (FR-10.1)
- Total users, active users, new users in period
- Average sessions per user, session duration
- Role breakdown (owners, admins, members)
- Activity trends by day/week/month

### Device Analytics (FR-10.2)
- Total devices, active devices
- Enrollments and unenrollments in period
- Location reports, geofence events, commands issued
- Device status breakdown (registered, enrolled, suspended, retired)
- Activity trends by day/week/month

### API Usage Analytics (FR-10.3)
- Total requests, success/error counts, success rate
- Average and P95 response times
- Total data transferred
- Top endpoints by usage
- Trends by day/week/month

### Report Generation (FR-10.4 - FR-10.7)
- Async report job creation (user, device analytics)
- Report status polling (pending, processing, completed, failed)
- Report download with presigned URLs
- Configurable date ranges and formats (CSV, JSON, XLSX, PDF)

---

## AP-11: Audit & Compliance

**Priority:** High
**Status:** 100% ✅ (8/8 endpoints)
**Dependencies:** All other epics

### Description
Comprehensive audit logging and GDPR compliance features. Foundation for enterprise compliance requirements.

### Implemented Endpoints ✅

#### AP-11.1: List Audit Logs ✅
**As an** organization admin
**I want to** view audit logs
**So that** I can track administrative actions

**Acceptance Criteria:**
- [x] GET `/api/admin/v1/organizations/:org_id/audit-logs` returns logs
- [x] Filter by action type
- [x] Filter by user
- [x] Filter by date range
- [x] Cursor-based pagination

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/audit-logs`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/audit_logs.rs`

---

#### AP-11.2: Get Audit Log Entry ✅
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/audit-logs/:log_id`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/audit_logs.rs`

---

#### AP-11.3: Export Audit Logs ✅
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/audit-logs/export`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/audit_logs.rs`
**Note:** Uses GET with query parameters for filtering (format, actor_id, action, from, to)

---

#### AP-11.4: List Data Subject Requests ✅
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/data-requests`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/data_subject_requests.rs`

---

#### AP-11.5: Create Data Subject Request ✅
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/data-requests`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/data_subject_requests.rs`

---

#### AP-11.6: Process Data Subject Request ✅
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/data-requests/:request_id/process`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/data_subject_requests.rs`

---

#### AP-11.7: Compliance Dashboard ✅
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/compliance`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/compliance.rs`

---

#### AP-11.8: Compliance Report ✅
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/compliance/report`
**Status:** ✅ Implemented
**File:** `crates/api/src/routes/compliance.rs`

---

### DSR Features
- Request types: Access, Deletion, Portability, Rectification, Restriction, Objection
- Status tracking: Pending, InProgress, Completed, Rejected, Cancelled
- State transition validation
- Overdue detection (30-day GDPR deadline)
- Processor tracking
- Result file/data storage

### Technical Notes
- Audit logs are immutable (append-only table design)
- DSR automation requires integration with all data stores
- Retention policies configurable via config
- 30-day processing deadline for GDPR DSRs

---

## Implementation Summary

All Admin Portal epics have been fully implemented. The backend API provides comprehensive B2B organization management capabilities.

### Key Implementation Files

| Domain | Key Files |
|--------|-----------|
| **Authentication** | `auth.rs`, `permissions.rs`, `roles.rs`, `system_roles.rs` |
| **Organizations** | `organizations.rs`, `organization_settings.rs` |
| **Users** | `admin_users.rs`, `org_invitations.rs` |
| **Devices** | `fleet.rs`, `device_policies.rs`, `enrollment_tokens.rs`, `bulk_import.rs` |
| **Groups** | `admin_groups.rs` |
| **Locations** | `admin_locations.rs`, `admin_geofences.rs` |
| **Webhooks** | `org_webhooks.rs` |
| **Settings** | `device_settings.rs`, `admin_unlock_requests.rs` |
| **System** | `system_config.rs` |
| **Dashboard** | `dashboard.rs`, `compliance.rs` |
| **Audit** | `audit_logs.rs`, `data_subject_requests.rs` |

---

## Endpoint Status Summary

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ Implemented | 115 | 100% |
| ❌ Not Implemented | 0 | 0% |
| **Total** | **115** | **100%** |

### By Priority

| Priority | Total | Implemented | Remaining |
|----------|-------|-------------|-----------|
| High | 50 | 50 (100%) | 0 |
| Medium | 39 | 39 (100%) | 0 |
| Low | 26 | 26 (100%) | 0 |

---

## Related Documents

| Document | Purpose |
|----------|---------|
| `docs/PRD-admin-portal.md` | Product requirements document |
| `docs/admin-api-gap-analysis.md` | Detailed gap analysis |
| `docs/project-workflow-analysis.md` | Project classification |
| `CLAUDE.md` | Technical context |

---

*Document Version: 2.0*
*Last Updated: 2025-12-11*
*Status: ✅ COMPLETE - All 115 endpoints implemented*
