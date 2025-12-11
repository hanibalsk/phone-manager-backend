# Admin Portal Backend API - Epic Definitions

**Project:** phone-manager-backend
**PRD Reference:** PRD-admin-portal.md
**Date:** 2025-12-10
**Author:** Martin Janci

---

## Epic Overview

| Epic ID | Name | Endpoints | Priority | Status |
|---------|------|-----------|----------|--------|
| AP-1 | RBAC & Access Control | 3 | High | 33% |
| AP-2 | Organization Management | 7 | High | 71% |
| AP-3 | User Administration | 13 | High | 31% |
| AP-4 | Device Fleet Administration | 15 | High | 53% |
| AP-5 | Groups Administration | 10 | Medium | 50% |
| AP-6 | Location & Geofence Administration | 13 | Low | 0% |
| AP-7 | Webhook Administration | 9 | High | 56% |
| AP-8 | App Usage & Unlock Requests | 13 | Low | 0% |
| AP-9 | System Configuration | 14 | Medium | 36% |
| AP-10 | Dashboard & Analytics | 10 | Medium | 30% |
| AP-11 | Audit & Compliance | 8 | High | 0% |

**Total:** 115 endpoints across 11 epics
**Current Implementation:** 36 endpoints (31%)
**Remaining:** 79 endpoints (69%)

---

## AP-1: RBAC & Access Control

**Priority:** High
**Status:** 33% (1/3 endpoints)
**Dependencies:** None (foundation epic)

### Description
Implement comprehensive role-based access control allowing organizations to define custom roles with granular permissions. This epic provides the security foundation for all other admin features.

### User Stories

#### AP-1.1: List Permissions
**As an** organization admin
**I want to** view all available system permissions
**So that** I can understand what capabilities can be assigned to roles

**Acceptance Criteria:**
- [ ] GET `/api/admin/v1/organizations/:org_id/permissions` returns all permissions
- [ ] Permissions grouped by category (users, devices, groups, etc.)
- [ ] Each permission includes name, description, and category
- [ ] Response supports filtering by category
- [ ] Requires admin authentication

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/permissions`
**Status:** Not Implemented

---

#### AP-1.2: Create Custom Role
**As an** organization admin
**I want to** create custom roles with specific permissions
**So that** I can define access levels appropriate for my organization

**Acceptance Criteria:**
- [ ] POST `/api/admin/v1/organizations/:org_id/roles` creates a new role
- [ ] Role name must be unique within organization
- [ ] Permissions array validated against available permissions
- [ ] System roles (admin, viewer) cannot be duplicated
- [ ] Returns created role with ID and permissions
- [ ] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/roles`
**Status:** Implemented (existing user role system)

---

#### AP-1.3: Delete Custom Role
**As an** organization admin
**I want to** delete custom roles no longer needed
**So that** I can maintain a clean role structure

**Acceptance Criteria:**
- [ ] DELETE `/api/admin/v1/organizations/:org_id/roles/:role_id` removes role
- [ ] System roles cannot be deleted (return 403)
- [ ] Roles with assigned users cannot be deleted (return 409)
- [ ] Audit log entry created
- [ ] Returns 204 No Content on success

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/roles/:role_id`
**Status:** Not Implemented

---

### Technical Notes
- Permission system should be extensible for future features
- Consider caching permission list (changes infrequently)
- Role assignments stored in `organization_users` table

---

## AP-2: Organization Management

**Priority:** High
**Status:** 71% (5/7 endpoints)
**Dependencies:** AP-1 (RBAC)

### Description
Complete organization lifecycle management including creation, configuration, usage tracking, and suspension capabilities.

### User Stories

#### AP-2.1: Create Organization ✅
**Status:** Implemented

**Endpoint:** `POST /api/admin/v1/organizations`

---

#### AP-2.2: List Organizations ✅
**Status:** Implemented

**Endpoint:** `GET /api/admin/v1/organizations`

---

#### AP-2.3: Get Organization Details ✅
**Status:** Implemented

**Endpoint:** `GET /api/admin/v1/organizations/:org_id`

---

#### AP-2.4: Update Organization ✅
**Status:** Implemented

**Endpoint:** `PUT /api/admin/v1/organizations/:org_id`

---

#### AP-2.5: Delete Organization ✅
**Status:** Implemented

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id`

---

#### AP-2.6: Get Organization Usage
**As an** organization admin
**I want to** view usage metrics
**So that** I can monitor resource consumption

**Acceptance Criteria:**
- [ ] GET `/api/admin/v1/organizations/:org_id/usage` returns metrics
- [ ] User count vs quota
- [ ] Device count vs quota
- [ ] API call counts
- [ ] Storage usage
- [ ] Supports date range filtering

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/usage`
**Status:** Not Implemented

---

#### AP-2.7: Suspend/Reactivate Organization
**As a** platform admin
**I want to** suspend organizations for policy violations
**So that** I can enforce platform terms

**Acceptance Criteria:**
- [ ] POST `/api/admin/v1/organizations/:org_id/suspend` suspends org
- [ ] POST `/api/admin/v1/organizations/:org_id/reactivate` reactivates
- [ ] Suspended orgs: users cannot login, API calls rejected
- [ ] Data preserved during suspension
- [ ] Audit log entries created

**Endpoints:**
- `POST /api/admin/v1/organizations/:org_id/suspend` - Not Implemented
- `POST /api/admin/v1/organizations/:org_id/reactivate` - Not Implemented

---

### Technical Notes
- Organization settings stored as JSONB for flexibility
- Usage metrics should be cached and updated periodically
- Suspension should be immediate and reversible

---

## AP-3: User Administration

**Priority:** High
**Status:** 31% (4/13 endpoints)
**Dependencies:** AP-1 (RBAC), AP-2 (Organizations)

### Description
Comprehensive user lifecycle management including creation, suspension, password management, MFA administration, and session management.

### User Stories

#### AP-3.1: List Users ✅
**Status:** Implemented

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/users`

---

#### AP-3.2: Get User Details ✅
**Status:** Implemented

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/users/:user_id`

---

#### AP-3.3: Create User ✅
**Status:** Implemented

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users`

---

#### AP-3.4: Update User ✅
**Status:** Implemented

**Endpoint:** `PUT /api/admin/v1/organizations/:org_id/users/:user_id`

---

#### AP-3.5: Suspend User
**As an** organization admin
**I want to** suspend user accounts
**So that** I can revoke access without deleting data

**Acceptance Criteria:**
- [ ] POST `/api/admin/v1/organizations/:org_id/users/:user_id/suspend` suspends user
- [ ] Immediately invalidates all sessions
- [ ] User cannot login while suspended
- [ ] Data and assignments preserved
- [ ] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users/:user_id/suspend`
**Status:** Not Implemented

---

#### AP-3.6: Reactivate User
**As an** organization admin
**I want to** reactivate suspended users
**So that** they can resume platform access

**Acceptance Criteria:**
- [ ] POST `/api/admin/v1/organizations/:org_id/users/:user_id/reactivate` reactivates
- [ ] User can login after reactivation
- [ ] Previous assignments restored
- [ ] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users/:user_id/reactivate`
**Status:** Not Implemented

---

#### AP-3.7: Trigger Password Reset
**As an** organization admin
**I want to** trigger password resets for users
**So that** I can help users who are locked out

**Acceptance Criteria:**
- [ ] POST `/api/admin/v1/organizations/:org_id/users/:user_id/reset-password` sends email
- [ ] Generates secure reset token
- [ ] Token expires in 24 hours
- [ ] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users/:user_id/reset-password`
**Status:** Not Implemented

---

#### AP-3.8: Get MFA Status
**As an** organization admin
**I want to** view user MFA status
**So that** I can ensure security compliance

**Acceptance Criteria:**
- [ ] GET `/api/admin/v1/organizations/:org_id/users/:user_id/mfa` returns status
- [ ] Shows if MFA is enabled
- [ ] Shows MFA method (TOTP, etc.)
- [ ] Shows enrollment date

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/users/:user_id/mfa`
**Status:** Not Implemented

---

#### AP-3.9: Force MFA Enrollment
**As an** organization admin
**I want to** require MFA for specific users
**So that** I can enforce security policies

**Acceptance Criteria:**
- [ ] POST `/api/admin/v1/organizations/:org_id/users/:user_id/mfa/force` enables requirement
- [ ] User prompted to set up MFA on next login
- [ ] Audit log entry created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/users/:user_id/mfa/force`
**Status:** Not Implemented

---

#### AP-3.10: Reset User MFA
**As an** organization admin
**I want to** reset user MFA
**So that** users who lose their device can re-enroll

**Acceptance Criteria:**
- [ ] DELETE `/api/admin/v1/organizations/:org_id/users/:user_id/mfa` resets MFA
- [ ] Removes current MFA configuration
- [ ] User must re-enroll
- [ ] Audit log entry created

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/users/:user_id/mfa`
**Status:** Not Implemented

---

#### AP-3.11: List User Sessions
**As an** organization admin
**I want to** view user active sessions
**So that** I can monitor for suspicious activity

**Acceptance Criteria:**
- [ ] GET `/api/admin/v1/organizations/:org_id/users/:user_id/sessions` returns sessions
- [ ] Shows device/browser info
- [ ] Shows IP address and location
- [ ] Shows session start time
- [ ] Shows last activity time

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/users/:user_id/sessions`
**Status:** Not Implemented

---

#### AP-3.12: Revoke Session
**As an** organization admin
**I want to** revoke specific user sessions
**So that** I can terminate suspicious sessions

**Acceptance Criteria:**
- [ ] DELETE `/api/admin/v1/organizations/:org_id/users/:user_id/sessions/:session_id` revokes
- [ ] Session immediately invalidated
- [ ] User must re-authenticate on that device
- [ ] Audit log entry created

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/users/:user_id/sessions/:session_id`
**Status:** Not Implemented

---

#### AP-3.13: Revoke All Sessions
**As an** organization admin
**I want to** revoke all user sessions
**So that** I can force complete re-authentication

**Acceptance Criteria:**
- [ ] DELETE `/api/admin/v1/organizations/:org_id/users/:user_id/sessions` revokes all
- [ ] All sessions immediately invalidated
- [ ] User must re-authenticate everywhere
- [ ] Audit log entry created

**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/users/:user_id/sessions`
**Status:** Not Implemented

---

### Technical Notes
- Session management requires new sessions table
- MFA requires TOTP library integration
- Consider session table with Redis for performance
- Password reset uses existing email infrastructure

---

## AP-4: Device Fleet Administration

**Priority:** High
**Status:** 53% (8/15 endpoints)
**Dependencies:** AP-2 (Organizations), AP-3 (Users)

### Description
Comprehensive device fleet management including listing, updating, bulk operations, and remote commands for enterprise device management.

### Implemented Endpoints ✅

| Endpoint | Status |
|----------|--------|
| `GET /api/admin/v1/organizations/:org_id/devices` | ✅ Implemented |
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id` | ✅ Implemented |
| `PATCH /api/admin/v1/organizations/:org_id/devices/:device_id` | ✅ Implemented |
| `DELETE /api/admin/v1/organizations/:org_id/devices/:device_id` | ✅ Implemented |
| `POST /api/admin/v1/organizations/:org_id/devices/:device_id/reactivate` | ✅ Implemented |
| `POST /api/admin/v1/organizations/:org_id/devices/:device_id/assign` | ✅ Implemented |
| `DELETE /api/admin/v1/organizations/:org_id/devices/:device_id/assign` | ✅ Implemented |
| `GET /api/admin/v1/organizations/:org_id/devices/summary` | ✅ Implemented |

### Missing Endpoints

#### AP-4.4: Bulk Update Devices
**As an** organization admin
**I want to** update multiple devices at once
**So that** I can efficiently manage large fleets

**Acceptance Criteria:**
- [ ] POST `/api/admin/v1/organizations/:org_id/devices/bulk-update` updates multiple
- [ ] Accepts array of device IDs and updates
- [ ] Maximum 100 devices per request
- [ ] Returns success/failure per device
- [ ] Audit log entries created

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/devices/bulk-update`
**Status:** Not Implemented

---

#### AP-4.9: Get Device Command History
**As an** organization admin
**I want to** view device command history
**So that** I can track administrative actions

**Acceptance Criteria:**
- [ ] GET `/api/admin/v1/organizations/:org_id/devices/:device_id/commands` returns history
- [ ] Shows command type, status, timestamp
- [ ] Shows who issued the command
- [ ] Cursor-based pagination

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/devices/:device_id/commands`
**Status:** Not Implemented

---

#### AP-4.10: Issue Device Command ✅
**Status:** Implemented

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/devices/:device_id/commands`

---

#### AP-4.11-15: Enrollment Token Management ✅
**Status:** All Implemented

**Endpoints:**
- `POST /api/admin/v1/organizations/:org_id/enrollment-tokens` ✅
- `GET /api/admin/v1/organizations/:org_id/enrollment-tokens` ✅
- `GET /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id` ✅
- `DELETE /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id` ✅

---

### Technical Notes
- Bulk operations should use database transactions
- Device commands use FCM for delivery
- Consider rate limiting on command issuance

---

## AP-5: Groups Administration

**Priority:** Medium
**Status:** 50% (5/10 endpoints)
**Dependencies:** AP-3 (Users), AP-4 (Devices)

### Description
Group management for organizing users and devices, including membership management and group-level settings.

### Implemented Endpoints ✅

| Endpoint | Status |
|----------|--------|
| `GET /api/admin/v1/organizations/:org_id/groups` | ✅ Implemented |
| `GET /api/admin/v1/organizations/:org_id/groups/:group_id` | ✅ Implemented |
| `POST /api/admin/v1/organizations/:org_id/groups` | ✅ Implemented |
| `PUT /api/admin/v1/organizations/:org_id/groups/:group_id` | ✅ Implemented |
| `DELETE /api/admin/v1/organizations/:org_id/groups/:group_id` | ✅ Implemented |

### Missing Endpoints

#### AP-5.6: List Group Members
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/groups/:group_id/members`
**Status:** Not Implemented

---

#### AP-5.7: Add Group Member
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/groups/:group_id/members`
**Status:** Not Implemented

---

#### AP-5.8: Remove Group Member
**Endpoint:** `DELETE /api/admin/v1/organizations/:org_id/groups/:group_id/members/:member_id`
**Status:** Not Implemented

---

#### AP-5.9-10: Group Invitations
**Endpoints:**
- `POST /api/admin/v1/organizations/:org_id/groups/:group_id/invitations` - Not Implemented
- `GET /api/admin/v1/organizations/:org_id/groups/:group_id/invitations` - Not Implemented

---

## AP-6: Location & Geofence Administration

**Priority:** Low
**Status:** 0% (0/13 endpoints)
**Dependencies:** AP-4 (Devices)

### Description
Administrative management of location data and organization-wide geofences. **This is a completely new domain requiring database schema additions.**

### Required Database Changes
- New `admin_geofences` table for organization-wide geofences
- Separate from user-created geofences

### Endpoints (All Not Implemented)

| Endpoint | Description |
|----------|-------------|
| `GET /api/admin/v1/organizations/:org_id/geofences` | List admin geofences |
| `POST /api/admin/v1/organizations/:org_id/geofences` | Create admin geofence |
| `GET /api/admin/v1/organizations/:org_id/geofences/:geofence_id` | Get geofence |
| `PUT /api/admin/v1/organizations/:org_id/geofences/:geofence_id` | Update geofence |
| `DELETE /api/admin/v1/organizations/:org_id/geofences/:geofence_id` | Delete geofence |
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id/location` | Get device location |
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id/location-history` | Get location history |
| `GET /api/admin/v1/organizations/:org_id/locations/current` | All device locations |
| `GET /api/admin/v1/organizations/:org_id/locations/history` | Org location history |
| `GET /api/admin/v1/organizations/:org_id/geofence-events` | Geofence events |
| `GET /api/admin/v1/organizations/:org_id/location-analytics` | Location analytics |
| + 2 additional reporting endpoints | |

---

## AP-7: Webhook Administration

**Priority:** High
**Status:** 56% (5/9 endpoints)
**Dependencies:** AP-2 (Organizations)

### Description
Complete webhook lifecycle management including testing, delivery logs, and retry capabilities.

### Implemented Endpoints ✅

| Endpoint | Status |
|----------|--------|
| `GET /api/admin/v1/organizations/:org_id/webhooks` | ✅ Implemented |
| `POST /api/admin/v1/organizations/:org_id/webhooks` | ✅ Implemented |
| `GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | ✅ Implemented |
| `PUT /api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | ✅ Implemented |
| `DELETE /api/admin/v1/organizations/:org_id/webhooks/:webhook_id` | ✅ Implemented |

### Missing Endpoints

#### AP-7.5: Test Webhook
**As an** organization admin
**I want to** test webhook endpoints
**So that** I can verify configuration before enabling

**Acceptance Criteria:**
- [ ] POST `/api/admin/v1/organizations/:org_id/webhooks/:webhook_id/test` sends test
- [ ] Sends test payload to webhook URL
- [ ] Returns response status and timing
- [ ] Does not affect delivery statistics

**Endpoint:** `POST /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/test`
**Status:** Not Implemented

---

#### AP-7.6: Get Delivery Logs
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/deliveries`
**Status:** Not Implemented

---

#### AP-7.7: Retry Delivery
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/deliveries/:delivery_id/retry`
**Status:** Not Implemented

---

#### AP-7.8: Get Delivery Statistics
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/stats`
**Status:** Not Implemented

---

## AP-8: App Usage & Unlock Requests

**Priority:** Low
**Status:** 0% (0/13 endpoints)
**Dependencies:** AP-4 (Devices)

### Description
App usage tracking and unlock request management. **This is a completely new domain requiring new database tables and business logic.**

### Required Database Changes
- New `app_usage` table for tracking application usage
- New `unlock_requests` table for managing unlock requests

### Endpoints (All Not Implemented)

| Endpoint | Description |
|----------|-------------|
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id/app-usage` | Usage summary |
| `GET /api/admin/v1/organizations/:org_id/devices/:device_id/app-usage/history` | Usage history |
| `GET /api/admin/v1/organizations/:org_id/app-usage/analytics` | Usage analytics |
| `GET /api/admin/v1/organizations/:org_id/unlock-requests` | List requests |
| `GET /api/admin/v1/organizations/:org_id/unlock-requests/:request_id` | Get request |
| `POST /api/admin/v1/organizations/:org_id/unlock-requests/:request_id/approve` | Approve |
| `POST /api/admin/v1/organizations/:org_id/unlock-requests/:request_id/deny` | Deny |
| `POST /api/admin/v1/organizations/:org_id/unlock-requests/bulk-process` | Bulk process |
| + 5 additional endpoints | |

---

## AP-9: System Configuration

**Priority:** Medium
**Status:** 36% (5/14 endpoints)
**Dependencies:** AP-2 (Organizations)

### Description
System-wide configuration management including settings, templates, rate limits, and feature flags.

### Implemented Endpoints ✅
- Health check endpoints (via /api/health)
- Some configuration endpoints

### Missing Endpoints

| Endpoint | Description | Status |
|----------|-------------|--------|
| `GET /api/admin/v1/system/settings` | Get system settings | Not Implemented |
| `PUT /api/admin/v1/system/settings` | Update settings | Not Implemented |
| `GET /api/admin/v1/system/templates` | List templates | Not Implemented |
| `PUT /api/admin/v1/system/templates/:template_id` | Update template | Not Implemented |
| `GET /api/admin/v1/system/feature-flags` | List feature flags | Not Implemented |
| `PUT /api/admin/v1/system/feature-flags/:flag_id` | Update flag | Not Implemented |
| `GET /api/admin/v1/system/rate-limits` | Get rate limits | Not Implemented |
| `PUT /api/admin/v1/system/rate-limits` | Update rate limits | Not Implemented |
| `POST /api/admin/v1/system/maintenance` | Toggle maintenance | Not Implemented |

---

## AP-10: Dashboard & Analytics

**Priority:** Medium
**Status:** 30% (3/10 endpoints)
**Dependencies:** AP-2, AP-3, AP-4

### Description
Dashboard metrics and analytics reporting for organizational insights.

### Implemented Endpoints ✅

| Endpoint | Status |
|----------|--------|
| `GET /api/admin/v1/organizations/:org_id/dashboard` | ✅ Implemented |
| Partial analytics endpoints | ✅ Implemented |

### Missing Endpoints

| Endpoint | Description | Status |
|----------|-------------|--------|
| `GET /api/admin/v1/organizations/:org_id/analytics/users` | User analytics | Not Implemented |
| `GET /api/admin/v1/organizations/:org_id/analytics/devices` | Device analytics | Not Implemented |
| `GET /api/admin/v1/organizations/:org_id/analytics/api` | API analytics | Not Implemented |
| `POST /api/admin/v1/organizations/:org_id/reports/users` | Generate user report | Not Implemented |
| `POST /api/admin/v1/organizations/:org_id/reports/devices` | Generate device report | Not Implemented |
| `POST /api/admin/v1/organizations/:org_id/reports/compliance` | Generate compliance report | Not Implemented |
| `GET /api/admin/v1/organizations/:org_id/reports/:report_id/status` | Check report status | Not Implemented |
| `GET /api/admin/v1/organizations/:org_id/reports/:report_id/download` | Download report | Not Implemented |

---

## AP-11: Audit & Compliance

**Priority:** High
**Status:** 0% (0/8 endpoints)
**Dependencies:** All other epics

### Description
Comprehensive audit logging and GDPR compliance features. **Foundation for enterprise compliance requirements.**

### Required Database Changes
- New `audit_logs` table for immutable audit records
- New `data_subject_requests` table for GDPR DSR tracking

### Endpoints (All Not Implemented)

#### AP-11.1: List Audit Logs
**As an** organization admin
**I want to** view audit logs
**So that** I can track administrative actions

**Acceptance Criteria:**
- [ ] GET `/api/admin/v1/organizations/:org_id/audit-logs` returns logs
- [ ] Filter by action type
- [ ] Filter by user
- [ ] Filter by date range
- [ ] Cursor-based pagination

**Endpoint:** `GET /api/admin/v1/organizations/:org_id/audit-logs`

---

#### AP-11.2: Get Audit Log Entry
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/audit-logs/:log_id`

---

#### AP-11.3: Export Audit Logs
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/audit-logs/export`
**Note:** Basic version implemented

---

#### AP-11.4: List Data Subject Requests
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/data-requests`

---

#### AP-11.5: Create Data Subject Request
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/data-requests`

---

#### AP-11.6: Process Data Subject Request
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/data-requests/:request_id/process`

---

#### AP-11.7: Compliance Dashboard
**Endpoint:** `GET /api/admin/v1/organizations/:org_id/compliance`

---

#### AP-11.8: Compliance Report
**Endpoint:** `POST /api/admin/v1/organizations/:org_id/compliance/report`

---

### Technical Notes
- Audit logs must be immutable (append-only table design)
- DSR automation requires integration with all data stores
- Retention policies must be configurable
- 30-day processing deadline for GDPR DSRs

---

## Implementation Roadmap

### Phase 1: Core Functionality (Weeks 1-4)
**Focus:** High-priority gaps, compliance foundation

| Epic | Endpoints | Stories |
|------|-----------|---------|
| AP-11 | 8 | Audit logging, GDPR DSR |
| AP-3 | 9 | User suspend/reactivate, MFA, sessions |
| AP-1 | 2 | Permissions, role deletion |
| AP-7 | 4 | Webhook test, delivery logs, retry |

**Deliverable:** Enterprise compliance readiness

### Phase 2: Enhanced Features (Weeks 5-8)
**Focus:** Complete existing domains

| Epic | Endpoints | Stories |
|------|-----------|---------|
| AP-4 | 7 | Bulk operations, command history |
| AP-2 | 2 | Usage metrics, suspend/reactivate |
| AP-10 | 7 | Analytics, report generation |
| AP-9 | 9 | System configuration |
| AP-5 | 5 | Group members, invitations |

**Deliverable:** Full admin functionality for existing domains

### Phase 3: New Domains (Weeks 9-12)
**Focus:** Completely new feature areas

| Epic | Endpoints | Stories |
|------|-----------|---------|
| AP-6 | 13 | Admin location management |
| AP-8 | 13 | App usage, unlock requests |

**Deliverable:** Complete 115-endpoint coverage

### Phase 4: Polish & Production (Weeks 13-16)
**Focus:** Quality and performance

- Performance optimization
- Security hardening
- Documentation completion
- Load testing
- Security audit

---

## Endpoint Status Summary

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ Implemented | 36 | 31% |
| ❌ Not Implemented | 79 | 69% |
| **Total** | **115** | **100%** |

### By Priority

| Priority | Total | Implemented | Remaining |
|----------|-------|-------------|-----------|
| High | 50 | 19 (38%) | 31 |
| Medium | 39 | 13 (33%) | 26 |
| Low | 26 | 4 (15%) | 22 |

---

## Related Documents

| Document | Purpose |
|----------|---------|
| `docs/PRD-admin-portal.md` | Product requirements document |
| `docs/admin-api-gap-analysis.md` | Detailed gap analysis |
| `docs/project-workflow-analysis.md` | Project classification |
| `CLAUDE.md` | Technical context |

---

*Document Version: 1.0*
*Last Updated: 2025-12-10*
