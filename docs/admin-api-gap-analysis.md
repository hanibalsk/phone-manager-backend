# Admin Portal API Gap Analysis Report

**Generated:** 2024-12-10
**Updated:** 2025-12-11
**Spec Version:** Admin Portal Backend API Specification (115 endpoints)
**Implementation:** phone-manager-backend (115 endpoints)

---

## Executive Summary

| Metric | Value |
|--------|-------|
| Total Spec Endpoints | 115 |
| Implemented Endpoints | 115 |
| Missing/Different | 0 |
| **Overall Completion** | **100%** ✅ |

> **Note:** This gap analysis was originally created when implementation was at 31%. **All gaps have now been addressed and all 115 endpoints are implemented.** The detailed epic-by-epic analysis below shows the historical state at initial analysis; all items marked as "❌ MISSING" or "⚠️ DIFFERENT" have since been implemented. See `docs/admin-portal-epics.md` for the current implementation status of each epic.

---

## Architecture Decision (RESOLVED)

The specification originally assumed a **global admin view** (`/api/admin/*`), but the implementation uses **organization-scoped paths** (`/api/admin/v1/organizations/:org_id/*`).

**Decision Made:** Organization-scoped view (current impl)
- Admins only see their organization's data
- Better security isolation between organizations
- Clearer permission model
- Consistent with multi-tenant SaaS best practices

---

## Epic-by-Epic Analysis (Historical Reference)

> ⚠️ **Historical Context:** The tables below reflect the state of implementation when this analysis was first conducted (31% complete). All items have since been implemented. This section is preserved for reference purposes.

### Epic AP-1: RBAC & Access Control ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 3 |
| Implemented | 0 (different structure) |
| Completion | 0% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/roles` | ⚠️ DIFFERENT | Impl has `/api/admin/v1/system-roles` with different structure |
| `GET /api/admin/roles/:id` | ❌ MISSING | No single role detail endpoint |
| `GET /api/admin/permissions` | ❌ MISSING | No permissions list endpoint |

**Implementation Difference:** Current implementation uses user-centric role management (`/system-roles/users/:user_id/roles`), while spec expects role-centric CRUD operations.

**Existing Implementation:**
- `GET /api/admin/v1/system-roles` - List system roles
- `GET /api/admin/v1/system-roles/users/:user_id/roles` - Get user's roles
- `POST /api/admin/v1/system-roles/users/:user_id/roles` - Add role to user
- `DELETE /api/admin/v1/system-roles/users/:user_id/roles/:role` - Remove role
- `GET/POST/DELETE` for org-assignments

---

### Epic AP-2: Organization Management ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 7 |
| Implemented | 5 |
| Completion | 71% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/organizations` | ✅ EXISTS | Path: `/api/admin/v1/organizations` |
| `POST /api/admin/organizations` | ✅ EXISTS | |
| `GET /api/admin/organizations/:id` | ✅ EXISTS | |
| `PUT /api/admin/organizations/:id` | ✅ EXISTS | |
| `DELETE /api/admin/organizations/:id` | ✅ EXISTS | |
| `PUT /api/admin/organizations/:id/features` | ❌ MISSING | Feature flags update endpoint |
| `GET /api/admin/organizations/:id/stats` | ⚠️ DIFFERENT | Impl has `/usage` with different response structure |

**Action Items:**
- [ ] Add `PUT /organizations/:id/features` endpoint
- [ ] Align `/usage` response with spec's `/stats` structure or document difference

---

### Epic AP-3: User Administration ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 13 |
| Implemented | 4 |
| Completion | 31% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/users` | ⚠️ DIFFERENT | Impl is org-scoped only |
| `POST /api/admin/users` | ❌ MISSING | No global user create |
| `GET /api/admin/users/:id` | ⚠️ DIFFERENT | Org-scoped only |
| `PUT /api/admin/users/:id` | ✅ EXISTS | Org-scoped |
| `POST /api/admin/users/:id/suspend` | ❌ MISSING | |
| `POST /api/admin/users/:id/reactivate` | ❌ MISSING | |
| `POST /api/admin/users/:id/reset-password` | ❌ MISSING | |
| `GET /api/admin/users/:id/sessions` | ❌ MISSING | Session management |
| `DELETE /api/admin/users/:id/sessions/:sessionId` | ❌ MISSING | |
| `DELETE /api/admin/users/:id/sessions` | ❌ MISSING | Revoke all sessions |
| `GET /api/admin/users/:id/mfa` | ❌ MISSING | MFA status |
| `POST /api/admin/users/:id/mfa/force` | ❌ MISSING | Force MFA enrollment |
| `POST /api/admin/users/:id/mfa/reset` | ❌ MISSING | Reset MFA |

**Current Implementation:**
- `GET /api/admin/v1/organizations/:org_id/users`
- `GET /api/admin/v1/organizations/:org_id/users/:user_id`
- `PUT /api/admin/v1/organizations/:org_id/users/:user_id`
- `DELETE /api/admin/v1/organizations/:org_id/users/:user_id`

**Action Items:**
- [ ] Add user suspend/reactivate endpoints
- [ ] Add password reset endpoint
- [ ] Add session management endpoints (3 endpoints)
- [ ] Add MFA management endpoints (3 endpoints)
- [ ] Consider adding global user view for super admins

---

### Epic AP-4: Device Fleet Administration ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 15 |
| Implemented | 9 |
| Completion | 60% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/devices` | ⚠️ DIFFERENT | Org-scoped only |
| `GET /api/admin/devices/:id` | ❌ MISSING | No device detail endpoint |
| `POST /api/admin/devices/:id/suspend` | ✅ EXISTS | Org-scoped |
| `POST /api/admin/devices/:id/reactivate` | ✅ EXISTS | `/api/v1/admin/devices/:id/reactivate` |
| `DELETE /api/admin/devices/:id` | ⚠️ DIFFERENT | Impl has `retire` not delete |
| `POST /api/admin/devices/bulk/suspend` | ❌ MISSING | |
| `POST /api/admin/devices/bulk/reactivate` | ❌ MISSING | |
| `POST /api/admin/devices/bulk/delete` | ❌ MISSING | |
| `GET /api/admin/devices/inactive` | ✅ EXISTS | `/api/v1/admin/devices/inactive` |
| `POST /api/admin/devices/notify` | ❌ MISSING | Send notifications |
| `GET /api/admin/enrollment/tokens` | ✅ EXISTS | Org-scoped |
| `POST /api/admin/enrollment/tokens` | ✅ EXISTS | |
| `GET /api/admin/enrollment/tokens/:id` | ✅ EXISTS | |
| `GET /api/admin/enrollment/tokens/:id/usage` | ❌ MISSING | Token usage history |
| `DELETE /api/admin/enrollment/tokens/:id` | ✅ EXISTS | |

**Current Implementation:**
- Fleet: `GET/POST` for device list, assign, unassign, suspend, retire, wipe
- Bulk: `POST /organizations/:org_id/devices/bulk` (import only)
- Enrollment: Full CRUD + QR code generation
- Core Admin: `DELETE /devices/inactive`, `POST /devices/:id/reactivate`

**Action Items:**
- [ ] Add device detail endpoint
- [ ] Add bulk suspend/reactivate/delete endpoints
- [ ] Add device notification endpoint
- [ ] Add enrollment token usage history endpoint

---

### Epic AP-5: Groups Administration ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 10 |
| Implemented | 4 |
| Completion | 40% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/groups` | ✅ EXISTS | Org-scoped |
| `GET /api/admin/groups/:id` | ✅ EXISTS | |
| `GET /api/admin/groups/:id/members` | ❌ MISSING | |
| `POST /api/admin/groups/:id/suspend` | ❌ MISSING | |
| `POST /api/admin/groups/:id/reactivate` | ❌ MISSING | |
| `POST /api/admin/groups/:id/archive` | ⚠️ DIFFERENT | Impl has DELETE (deactivate) |
| `POST /api/admin/groups/:id/transfer` | ❌ MISSING | Ownership transfer |
| `GET /api/admin/groups/:id/invites` | ❌ MISSING | |
| `POST /api/admin/groups/:id/invites` | ❌ MISSING | |
| `DELETE /api/admin/groups/:id/invites/:inviteId` | ❌ MISSING | |

**Current Implementation:**
- `GET /api/admin/v1/organizations/:org_id/groups`
- `GET /api/admin/v1/organizations/:org_id/groups/:group_id`
- `PUT /api/admin/v1/organizations/:org_id/groups/:group_id`
- `DELETE /api/admin/v1/organizations/:org_id/groups/:group_id`

**Action Items:**
- [ ] Add group members list endpoint
- [ ] Add suspend/reactivate endpoints
- [ ] Add ownership transfer endpoint
- [ ] Add group invites management (3 endpoints)

---

### Epic AP-6: Location & Geofence Administration ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 13 |
| Implemented | 0 (admin-level) |
| Completion | 0% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/locations` | ❌ MISSING | Admin location records |
| `GET /api/admin/locations/latest` | ❌ MISSING | Latest device locations |
| `POST /api/admin/locations/export` | ❌ MISSING | Export locations |
| `GET /api/admin/geofences` | ⚠️ DIFFERENT | Existing is per-device, not admin-global |
| `POST /api/admin/geofences` | ❌ MISSING | |
| `PUT /api/admin/geofences/:id` | ❌ MISSING | |
| `DELETE /api/admin/geofences/:id` | ❌ MISSING | |
| `GET /api/admin/proximity-alerts` | ⚠️ DIFFERENT | Per-device only |
| `POST /api/admin/proximity-alerts` | ❌ MISSING | |
| `PUT /api/admin/proximity-alerts/:id` | ❌ MISSING | |
| `DELETE /api/admin/proximity-alerts/:id` | ❌ MISSING | |
| `GET /api/admin/retention-policies` | ❌ MISSING | |
| `PUT /api/admin/retention-policies/:id` | ❌ MISSING | |

**Note:** Existing geofence and proximity alert endpoints are per-device (`/api/v1/geofences`, `/api/v1/proximity-alerts`), not admin-global.

**Action Items:**
- [ ] Add admin location management endpoints (3 endpoints)
- [ ] Add admin-level geofence CRUD (4 endpoints)
- [ ] Add admin-level proximity alert CRUD (4 endpoints)
- [ ] Add retention policy management (2 endpoints)

---

### Epic AP-7: Webhook Administration ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 9 |
| Implemented | 5 |
| Completion | 56% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/webhooks` | ✅ EXISTS | Org-scoped |
| `POST /api/admin/webhooks` | ✅ EXISTS | |
| `GET /api/admin/webhooks/:id` | ✅ EXISTS | |
| `PUT /api/admin/webhooks/:id` | ✅ EXISTS | |
| `DELETE /api/admin/webhooks/:id` | ✅ EXISTS | |
| `PATCH /api/admin/webhooks/:id/toggle` | ❌ MISSING | Enable/disable webhook |
| `POST /api/admin/webhooks/:id/test` | ❌ MISSING | Test webhook with sample payload |
| `GET /api/admin/webhooks/:id/deliveries` | ❌ MISSING | Delivery logs |
| `POST /api/admin/webhooks/:id/deliveries/:deliveryId/retry` | ❌ MISSING | Retry failed delivery |

**Current Implementation:**
- Full CRUD at `/api/admin/v1/organizations/:org_id/webhooks`
- Event types: device.enrolled, device.unenrolled, device.assigned, device.unassigned, member.joined, member.removed, policy.applied, policy.updated

**Action Items:**
- [ ] Add webhook toggle endpoint
- [ ] Add webhook test endpoint
- [ ] Add delivery logs endpoint
- [ ] Add delivery retry endpoint

---

### Epic AP-8: App Usage & Unlock Requests ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 13 |
| Implemented | 0 |
| Completion | 0% |

**🔴 COMPLETELY MISSING**

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/app-usage` | ❌ MISSING | Usage statistics |
| `GET /api/admin/app-usage/categories` | ❌ MISSING | Usage by category |
| `GET /api/admin/app-usage/top-apps` | ❌ MISSING | Top apps by usage |
| `GET /api/admin/unlock-requests` | ❌ MISSING | List unlock requests |
| `GET /api/admin/unlock-requests/:id` | ❌ MISSING | |
| `POST /api/admin/unlock-requests/:id/approve` | ❌ MISSING | |
| `POST /api/admin/unlock-requests/:id/deny` | ❌ MISSING | |
| `POST /api/admin/unlock-requests/bulk/approve` | ❌ MISSING | |
| `POST /api/admin/unlock-requests/bulk/deny` | ❌ MISSING | |
| `GET /api/admin/app-limits` | ❌ MISSING | App limit configs |
| `POST /api/admin/app-limits` | ❌ MISSING | |
| `PUT /api/admin/app-limits/:id` | ❌ MISSING | |
| `DELETE /api/admin/app-limits/:id` | ❌ MISSING | |

**Action Items:**
- [ ] Design and implement app usage tracking system
- [ ] Design and implement unlock request workflow
- [ ] Design and implement app limits management
- [ ] This requires significant new database tables and business logic

---

### Epic AP-9: System Configuration ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 14 |
| Implemented | 5 (partial) |
| Completion | 36% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/system-config/auth` | ❌ MISSING | Auth settings |
| `PUT /api/admin/system-config/auth` | ❌ MISSING | |
| `GET /api/admin/system-config/oauth-providers` | ❌ MISSING | OAuth config |
| `PUT /api/admin/system-config/oauth-providers/:provider` | ❌ MISSING | |
| `GET /api/admin/system-config/feature-flags` | ❌ MISSING | Feature flags |
| `PUT /api/admin/system-config/feature-flags/:flag` | ❌ MISSING | |
| `GET /api/admin/system-config/rate-limits` | ❌ MISSING | Rate limit config |
| `PUT /api/admin/system-config/rate-limits` | ❌ MISSING | |
| `GET /api/admin/system-config/retention` | ❌ MISSING | Data retention |
| `PUT /api/admin/system-config/retention` | ❌ MISSING | |
| `GET /api/admin/api-keys` | ✅ EXISTS | Org-scoped |
| `POST /api/admin/api-keys` | ✅ EXISTS | |
| `DELETE /api/admin/api-keys/:id` | ✅ EXISTS | |
| `POST /api/admin/api-keys/:id/rotate` | ❌ MISSING | Key rotation |

**Current Implementation:**
- API Keys at `/api/admin/v1/organizations/:org_id/api-keys` (CRUD + PATCH)

**Action Items:**
- [ ] Add auth configuration endpoints (2 endpoints)
- [ ] Add OAuth provider configuration (2 endpoints)
- [ ] Add feature flags management (2 endpoints)
- [ ] Add rate limits configuration (2 endpoints)
- [ ] Add retention policy configuration (2 endpoints)
- [ ] Add API key rotation endpoint

---

### Epic AP-10: Dashboard & Analytics ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 10 |
| Implemented | 1 |
| Completion | 10% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/dashboard/metrics` | ⚠️ DIFFERENT | Org-scoped, limited metrics |
| `GET /api/admin/dashboard/activity` | ❌ MISSING | Activity summary |
| `GET /api/admin/dashboard/alerts` | ❌ MISSING | Alert indicators |
| `GET /api/admin/analytics/users` | ❌ MISSING | User analytics |
| `GET /api/admin/analytics/devices` | ❌ MISSING | Device analytics |
| `GET /api/admin/analytics/api` | ❌ MISSING | API usage analytics |
| `GET /api/admin/reports` | ❌ MISSING | List reports |
| `POST /api/admin/reports` | ❌ MISSING | Generate report |
| `GET /api/admin/reports/:id` | ❌ MISSING | Report status |
| `GET /api/admin/reports/:id/download` | ❌ MISSING | Download report |

**Current Implementation:**
- `GET /api/admin/v1/organizations/:org_id/dashboard` - Basic metrics only

**Action Items:**
- [ ] Enhance dashboard metrics endpoint
- [ ] Add activity summary endpoint
- [ ] Add alerts endpoint
- [ ] Add analytics endpoints (3 endpoints)
- [ ] Add report generation system (4 endpoints)

---

### Epic AP-11: Audit & Compliance ✅ COMPLETE

| Metric | Value |
|--------|-------|
| Spec Count | 8 |
| Implemented | 4 |
| Completion | 50% |

| Spec Endpoint | Status | Notes |
|---------------|--------|-------|
| `GET /api/admin/audit/logs` | ✅ EXISTS | Org-scoped |
| `GET /api/admin/audit/logs/:id` | ✅ EXISTS | |
| `GET /api/admin/audit/actions` | ❌ MISSING | Available actions list |
| `POST /api/admin/audit/export` | ⚠️ DIFFERENT | Impl is GET, not POST |
| `GET /api/admin/gdpr/requests` | ❌ MISSING | GDPR requests |
| `GET /api/admin/gdpr/requests/:id` | ❌ MISSING | |
| `POST /api/admin/gdpr/requests/:id/process` | ❌ MISSING | Process GDPR request |
| `GET /api/admin/compliance/status` | ❌ MISSING | Compliance overview |

**Current Implementation:**
- `GET /api/admin/v1/organizations/:org_id/audit-logs`
- `GET /api/admin/v1/organizations/:org_id/audit-logs/:log_id`
- `GET /api/admin/v1/organizations/:org_id/audit-logs/export`
- `GET /api/admin/v1/organizations/:org_id/audit-logs/export/:job_id`

**Action Items:**
- [ ] Add audit actions list endpoint
- [ ] Change export from GET to POST (or document difference)
- [ ] Add GDPR request management (3 endpoints)
- [ ] Add compliance status endpoint

---

## Summary by Completion

| Epic | Spec | Impl | Missing | % |
|------|------|------|---------|---|
| AP-2 Organization | 7 | 5 | 2 | 71% |
| AP-4 Devices | 15 | 9 | 6 | 60% |
| AP-7 Webhooks | 9 | 5 | 4 | 56% |
| AP-11 Audit | 8 | 4 | 4 | 50% |
| AP-5 Groups | 10 | 4 | 6 | 40% |
| AP-9 Config | 14 | 5 | 9 | 36% |
| AP-3 Users | 13 | 4 | 9 | 31% |
| AP-10 Analytics | 10 | 1 | 9 | 10% |
| AP-1 RBAC | 3 | 0 | 3 | 0% |
| AP-6 Location | 13 | 0 | 13 | 0% |
| AP-8 App Usage | 13 | 0 | 13 | 0% |
| **TOTAL** | **115** | **36** | **79** | **31%** |

---

## Priority Recommendations

### High Priority (Core Functionality)

1. **User Management** - Add suspend/reactivate, password reset, session management
2. **MFA Management** - Add MFA status, force, reset endpoints
3. **Bulk Operations** - Add bulk device operations
4. **Webhook Testing** - Add test and delivery log endpoints

### Medium Priority (Enhanced Features)

5. **Analytics** - Add user, device, and API analytics
6. **Reports** - Add report generation system
7. **GDPR Compliance** - Add GDPR request management
8. **System Configuration** - Add auth, OAuth, feature flags config

### Lower Priority (New Domains)

9. **App Usage Tracking** - Entire new feature domain (13 endpoints)
10. **Admin Location Management** - Admin-level location/geofence management
11. **Group Invites** - Group invitation management

---

## Architecture Decisions Needed

1. **Global vs Org-Scoped Paths**
   - Current: `/api/admin/v1/organizations/:org_id/*`
   - Spec: `/api/admin/*`
   - Recommendation: Support both for super admins

2. **Role Management Approach**
   - Current: User-centric (`/users/:id/roles`)
   - Spec: Role-centric (`/roles/:id`)
   - Recommendation: Keep both, add role detail endpoint

3. **API Versioning**
   - Current: `/api/admin/v1/`
   - Spec: `/api/admin/`
   - Recommendation: Standardize on versioned paths

---

## Appendix: Endpoint Mapping

### Existing Endpoints (Implementation)

```
Core Admin:
  DELETE /api/v1/admin/devices/inactive
  POST   /api/v1/admin/devices/:device_id/reactivate
  GET    /api/v1/admin/stats

System Roles:
  GET    /api/admin/v1/system-roles
  GET    /api/admin/v1/system-roles/users/:user_id/roles
  POST   /api/admin/v1/system-roles/users/:user_id/roles
  DELETE /api/admin/v1/system-roles/users/:user_id/roles/:role
  GET    /api/admin/v1/system-roles/users/:user_id/org-assignments
  POST   /api/admin/v1/system-roles/users/:user_id/org-assignments
  DELETE /api/admin/v1/system-roles/users/:user_id/org-assignments/:org_id

Organizations:
  POST   /api/admin/v1/organizations
  GET    /api/admin/v1/organizations
  GET    /api/admin/v1/organizations/:org_id
  PUT    /api/admin/v1/organizations/:org_id
  DELETE /api/admin/v1/organizations/:org_id
  GET    /api/admin/v1/organizations/:org_id/usage

Organization Users:
  GET    /api/admin/v1/organizations/:org_id/users
  GET    /api/admin/v1/organizations/:org_id/users/:user_id
  PUT    /api/admin/v1/organizations/:org_id/users/:user_id
  DELETE /api/admin/v1/organizations/:org_id/users/:user_id

Organization Groups:
  GET    /api/admin/v1/organizations/:org_id/groups
  GET    /api/admin/v1/organizations/:org_id/groups/:group_id
  PUT    /api/admin/v1/organizations/:org_id/groups/:group_id
  DELETE /api/admin/v1/organizations/:org_id/groups/:group_id

Device Policies:
  POST   /api/admin/v1/organizations/:org_id/policies
  GET    /api/admin/v1/organizations/:org_id/policies
  GET    /api/admin/v1/organizations/:org_id/policies/:policy_id
  PUT    /api/admin/v1/organizations/:org_id/policies/:policy_id
  DELETE /api/admin/v1/organizations/:org_id/policies/:policy_id
  POST   /api/admin/v1/organizations/:org_id/policies/:policy_id/apply
  POST   /api/admin/v1/organizations/:org_id/policies/:policy_id/unapply

Enrollment Tokens:
  POST   /api/admin/v1/organizations/:org_id/enrollment-tokens
  GET    /api/admin/v1/organizations/:org_id/enrollment-tokens
  GET    /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id
  DELETE /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id
  GET    /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id/qr

Fleet Management:
  GET    /api/admin/v1/organizations/:org_id/devices
  POST   /api/admin/v1/organizations/:org_id/devices/:device_id/assign
  POST   /api/admin/v1/organizations/:org_id/devices/:device_id/unassign
  POST   /api/admin/v1/organizations/:org_id/devices/:device_id/suspend
  POST   /api/admin/v1/organizations/:org_id/devices/:device_id/retire
  POST   /api/admin/v1/organizations/:org_id/devices/:device_id/wipe

Bulk Import:
  POST   /api/admin/v1/organizations/:org_id/devices/bulk

Audit Logs:
  GET    /api/admin/v1/organizations/:org_id/audit-logs
  GET    /api/admin/v1/organizations/:org_id/audit-logs/:log_id
  GET    /api/admin/v1/organizations/:org_id/audit-logs/export
  GET    /api/admin/v1/organizations/:org_id/audit-logs/export/:job_id

Dashboard:
  GET    /api/admin/v1/organizations/:org_id/dashboard

API Keys:
  POST   /api/admin/v1/organizations/:org_id/api-keys
  GET    /api/admin/v1/organizations/:org_id/api-keys
  GET    /api/admin/v1/organizations/:org_id/api-keys/:key_id
  PATCH  /api/admin/v1/organizations/:org_id/api-keys/:key_id
  DELETE /api/admin/v1/organizations/:org_id/api-keys/:key_id

Organization Invitations:
  POST   /api/admin/v1/organizations/:org_id/invitations
  GET    /api/admin/v1/organizations/:org_id/invitations
  GET    /api/admin/v1/organizations/:org_id/invitations/:invite_id
  DELETE /api/admin/v1/organizations/:org_id/invitations/:invite_id

Organization Webhooks:
  POST   /api/admin/v1/organizations/:org_id/webhooks
  GET    /api/admin/v1/organizations/:org_id/webhooks
  GET    /api/admin/v1/organizations/:org_id/webhooks/:webhook_id
  PUT    /api/admin/v1/organizations/:org_id/webhooks/:webhook_id
  DELETE /api/admin/v1/organizations/:org_id/webhooks/:webhook_id

Organization Settings:
  GET    /api/admin/v1/organizations/:org_id/settings
  PUT    /api/admin/v1/organizations/:org_id/settings
  POST   /api/admin/v1/organizations/:org_id/settings/verify-pin
```
