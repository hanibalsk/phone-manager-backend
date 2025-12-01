# Phone Manager Backend - User Management & Device Control PRD

**Author:** Martin
**Date:** 2025-12-01
**Project Level:** 3 (Full Product)
**Project Type:** Backend API Extension
**Target Scale:** ~53 stories across 6 epics
**Source:** Specification files (docs/specs/)

---

## Description, Context and Goals

### Description

Extend the Phone Manager Rust/Axum backend with comprehensive user management, authentication, group-based access control, and remote device settings management APIs. This feature transforms the backend from a device-centric to a user-centric platform while maintaining full backward compatibility with existing device-only API consumers.

### Deployment Intent

- **Backend**: Rust/Axum API extension
- **Database**: PostgreSQL schema extensions
- **Authentication**: JWT RS256 with OAuth2 (Google, Apple)
- **Target Audience**: B2C families, B2B enterprises

### Context

The Phone Manager backend currently operates in a device-centric model where each device has a unique API key and belongs to a group identified by a simple group ID. There is no user authentication, ownership model, or administrative control over device settings.

**Current State:**
- Device registration via X-API-Key header
- Simple group membership by groupId string
- All settings controlled locally on device
- No user accounts or authentication
- No administrative oversight APIs

**Desired State:**
- User accounts with email/password and OAuth
- Devices owned by users with ownership transfer
- Groups with role-based membership (owner, admin, member, viewer)
- Remote settings control APIs for administrators
- Setting locks that prevent user modification
- B2B support for enterprise device fleets
- Admin API endpoints for portal integration

### Goals

1. **Enable User Identity** - API endpoints for user registration and authentication
2. **Establish Device Ownership** - Link devices to user accounts via API
3. **Implement Group Hierarchy** - Groups with owners, admins, and role-based permissions
4. **Remote Settings Control** - API endpoints for viewing and modifying member device settings
5. **Setting Locks** - Lock mechanism to prevent user changes to specific settings
6. **B2B Enterprise Support** - Organization, policy, and enrollment APIs
7. **Maintain Backward Compatibility** - Existing API consumers continue working without changes

---

## Requirements

### Functional Requirements

#### FR-9: Authentication API
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-9.1 | Register endpoint with email/password | Critical |
| FR-9.2 | Login endpoint with email/password | Critical |
| FR-9.3 | Google OAuth endpoint | High |
| FR-9.4 | Apple Sign-In endpoint | High |
| FR-9.5 | JWT RS256 token infrastructure | Critical |
| FR-9.6 | Refresh token rotation endpoint | Critical |
| FR-9.7 | Logout and token invalidation | High |
| FR-9.8 | Password reset flow endpoints | High |
| FR-9.9 | Email verification endpoint | Medium |
| FR-9.10 | Current user profile endpoints (GET/PUT) | High |

#### FR-10: User-Device Binding API
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-10.1 | Link device to user endpoint | Critical |
| FR-10.2 | List user's devices endpoint | High |
| FR-10.3 | Unlink device endpoint | High |
| FR-10.4 | Transfer device ownership endpoint | Medium |
| FR-10.5 | Device registration with optional auth | Critical |

#### FR-11: Group Management API
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-11.1 | Group CRUD endpoints | Critical |
| FR-11.2 | Membership management endpoints | High |
| FR-11.3 | Role management (owner, admin, member, viewer) | High |
| FR-11.4 | Invite CRUD endpoints | Critical |
| FR-11.5 | Join group with code endpoint | Critical |
| FR-11.6 | Group ownership transfer | Medium |
| FR-11.7 | RBAC middleware for all endpoints | High |

#### FR-12: Settings Control API
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-12.1 | Get device settings endpoint | Critical |
| FR-12.2 | Update device settings endpoint | Critical |
| FR-12.3 | Lock/unlock settings endpoints | Critical |
| FR-12.4 | Unlock request workflow endpoints | High |
| FR-12.5 | Settings sync endpoint | Critical |
| FR-12.6 | Setting definitions endpoint | High |
| FR-12.7 | Push notification triggers for setting changes | High |

#### FR-13: B2B Enterprise API
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-13.1 | Organization CRUD endpoints | High |
| FR-13.2 | Organization user management | High |
| FR-13.3 | Device policy CRUD endpoints | High |
| FR-13.4 | Enrollment token management | High |
| FR-13.5 | Device enrollment endpoint | High |
| FR-13.6 | Policy resolution algorithm | High |
| FR-13.7 | Fleet management endpoints | High |
| FR-13.8 | Bulk device import endpoint | Medium |
| FR-13.9 | Audit logging system | High |
| FR-13.10 | Audit query and export endpoints | High |

#### FR-14: Admin Portal Backend
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-14.1 | Dashboard metrics endpoint | High |
| FR-14.2 | Device fleet list with filtering | High |
| FR-14.3 | User management endpoints | High |
| FR-14.4 | Policy management endpoints | High |
| FR-14.5 | Enrollment token QR generation | High |
| FR-14.6 | Audit log viewer endpoints | High |
| FR-14.7 | Usage statistics endpoint | Medium |
| FR-14.8 | Reports and export endpoints | Medium |

### Non-Functional Requirements

| Category | Requirement | Target |
|----------|-------------|--------|
| **Security** | Password hashing | Argon2id |
| **Security** | Token algorithm | RS256 JWT |
| **Security** | Access token lifetime | 1 hour |
| **Security** | Refresh token lifetime | 30 days |
| **Security** | Rate limiting | Per-endpoint limits |
| **Performance** | Auth endpoint latency | < 500ms |
| **Performance** | Settings sync latency | < 200ms |
| **Performance** | List endpoint latency | < 300ms |
| **Availability** | Backward compatibility | 100% for existing endpoints |
| **Scalability** | Concurrent users | 10,000+ |
| **Compliance** | Data protection | GDPR-ready |

---

## API Design Summary

### Authentication Endpoints (/api/v1/auth/*)
- POST `/register` - Create account
- POST `/login` - Email/password login
- POST `/logout` - Invalidate tokens
- POST `/refresh` - Refresh access token
- POST `/oauth/google` - Google OAuth
- POST `/oauth/apple` - Apple Sign-In
- POST `/forgot-password` - Request reset
- POST `/reset-password` - Reset with token
- POST `/verify-email` - Verify email
- GET/PUT `/me` - Current user profile
- PUT `/me/password` - Change password

### User Endpoints (/api/v1/users/*)
- GET `/{userId}` - Get user
- GET `/{userId}/devices` - List user's devices
- POST `/{userId}/devices/{deviceId}/link` - Link device
- DELETE `/{userId}/devices/{deviceId}/unlink` - Unlink device
- POST `/{userId}/devices/{deviceId}/transfer` - Transfer ownership

### Group Endpoints (/api/v1/groups/*)
- POST `/` - Create group
- GET `/` - List user's groups
- GET `/{groupId}` - Get group
- PUT `/{groupId}` - Update group
- DELETE `/{groupId}` - Delete group
- GET `/{groupId}/members` - List members
- PUT `/{groupId}/members/{userId}/role` - Update role
- DELETE `/{groupId}/members/{userId}` - Remove member
- POST `/{groupId}/invites` - Create invite
- GET `/{groupId}/invites` - List invites
- DELETE `/{groupId}/invites/{inviteId}` - Revoke invite
- POST `/join` - Join with code
- POST `/{groupId}/transfer` - Transfer ownership

### Device Settings Endpoints (/api/v1/devices/*)
- POST `/register` - Register device (optional auth)
- POST `/enroll` - Enroll with token (B2B)
- GET `/{deviceId}/settings` - Get all settings
- PUT `/{deviceId}/settings` - Update settings
- GET `/{deviceId}/settings/{key}` - Get single setting
- PUT `/{deviceId}/settings/{key}` - Update single setting
- GET `/{deviceId}/settings/locks` - Get all locks
- PUT `/{deviceId}/settings/locks` - Update locks
- POST `/{deviceId}/settings/{key}/lock` - Lock setting
- DELETE `/{deviceId}/settings/{key}/lock` - Unlock setting
- POST `/{deviceId}/settings/{key}/unlock-request` - Request unlock
- POST `/{deviceId}/settings/sync` - Force sync

### Admin Endpoints (/api/admin/v1/*)
- Organizations: CRUD, usage stats
- Organization users: CRUD
- Enrollment tokens: CRUD, QR generation
- Policies: CRUD, apply
- Fleet: list, bulk import, assign, suspend, retire, wipe
- Audit logs: query, export

---

## Database Schema

### New Tables (17 total)
1. `organizations` - B2B tenant organizations
2. `users` - User accounts
3. `oauth_accounts` - OAuth provider links
4. `user_sessions` - Refresh token tracking
5. `groups` - User groups
6. `group_memberships` - Group membership with roles
7. `group_invites` - Invitation codes
8. `device_policies` - B2B policy templates
9. `device_settings` - Per-device settings with locks
10. `setting_definitions` - Setting catalog
11. `unlock_requests` - Unlock request workflow
12. `enrollment_tokens` - B2B provisioning tokens
13. `device_tokens` - Long-lived device tokens
14. `audit_logs` - Admin action audit trail
15. `org_users` - Organization admin users

### Modified Tables
- `devices` - Add owner_user_id, organization_id, policy_id, is_managed, enrollment_status
- `api_keys` - Add user_id, organization_id, scopes

---

## Epics

### Epic 9: Authentication Foundation

**Priority:** Critical
**Estimated Effort:** 3-4 weeks
**Dependencies:** None
**Stories:** 11

Implement user authentication infrastructure with email/password and OAuth.

| Story | Title | Effort | Priority |
|-------|-------|--------|----------|
| E9.1 | Database schema for users and sessions | M | Critical |
| E9.2 | Password hashing with Argon2id | S | Critical |
| E9.3 | JWT RS256 infrastructure (key management, signing, verification) | M | Critical |
| E9.4 | Register endpoint with validation | M | Critical |
| E9.5 | Login endpoint with rate limiting | M | Critical |
| E9.6 | Logout and token invalidation | S | High |
| E9.7 | Refresh token rotation with family tracking | M | Critical |
| E9.8 | Google OAuth endpoint | M | High |
| E9.9 | Apple Sign-In endpoint | M | High |
| E9.10 | Password reset flow (forgot/reset endpoints) | M | High |
| E9.11 | Email verification endpoint | S | Medium |

---

### Epic 10: User-Device Binding

**Priority:** High
**Estimated Effort:** 2 weeks
**Dependencies:** Epic 9
**Stories:** 6

Link devices to user accounts with ownership management.

| Story | Title | Effort | Priority |
|-------|-------|--------|----------|
| E10.1 | Device table migration (owner_user_id, organization_id) | S | Critical |
| E10.2 | Link device to user endpoint | M | Critical |
| E10.3 | List user's devices endpoint | S | High |
| E10.4 | Unlink device endpoint | S | High |
| E10.5 | Transfer device ownership endpoint | M | Medium |
| E10.6 | Update device registration for optional auth | M | Critical |

---

### Epic 11: Group Management

**Priority:** High
**Estimated Effort:** 3 weeks
**Dependencies:** Epic 10
**Stories:** 9

Full group lifecycle with role-based permissions.

| Story | Title | Effort | Priority |
|-------|-------|--------|----------|
| E11.1 | Database schema for groups and memberships | M | Critical |
| E11.2 | Group CRUD endpoints | M | Critical |
| E11.3 | Membership list and detail endpoints | M | High |
| E11.4 | Role management endpoint (update member role) | M | High |
| E11.5 | Remove member endpoint | S | High |
| E11.6 | Invite CRUD endpoints | M | Critical |
| E11.7 | Join group with code endpoint | M | Critical |
| E11.8 | Group ownership transfer endpoint | M | Medium |
| E11.9 | RBAC middleware for authorization | L | High |

---

### Epic 12: Settings Control

**Priority:** High
**Estimated Effort:** 2-3 weeks
**Dependencies:** Epic 11
**Stories:** 8

Remote device settings management with locking mechanism.

| Story | Title | Effort | Priority |
|-------|-------|--------|----------|
| E12.1 | Device settings and setting_definitions tables | M | Critical |
| E12.2 | Get device settings endpoint | M | Critical |
| E12.3 | Update device settings endpoint (respects locks) | M | Critical |
| E12.4 | Lock/unlock settings endpoints | M | Critical |
| E12.5 | Bulk lock update endpoint | M | High |
| E12.6 | Unlock request workflow (create, list, approve/deny) | L | High |
| E12.7 | Settings sync endpoint | M | Critical |
| E12.8 | Push notification integration for setting changes | M | High |

---

### Epic 13: B2B Enterprise Features

**Priority:** Medium
**Estimated Effort:** 4 weeks
**Dependencies:** Epic 12
**Stories:** 10

Organization management, policies, enrollment, and fleet control.

| Story | Title | Effort | Priority |
|-------|-------|--------|----------|
| E13.1 | Organizations table and CRUD endpoints | M | High |
| E13.2 | Organization users management endpoints | M | High |
| E13.3 | Device policies table and CRUD endpoints | M | High |
| E13.4 | Enrollment tokens management endpoints | M | High |
| E13.5 | Device enrollment endpoint | L | High |
| E13.6 | Policy resolution algorithm implementation | L | High |
| E13.7 | Fleet management endpoints (list, assign, suspend, retire) | L | High |
| E13.8 | Bulk device import endpoint | M | Medium |
| E13.9 | Audit logging system | M | High |
| E13.10 | Audit query and export endpoints | M | Medium |

---

### Epic 14: Admin Portal Backend

**Priority:** Medium
**Estimated Effort:** 3-4 weeks
**Dependencies:** Epic 13
**Stories:** 9

Backend API support for web administration dashboard.

| Story | Title | Effort | Priority |
|-------|-------|--------|----------|
| E14.1 | Dashboard metrics endpoint | M | High |
| E14.2 | Device fleet list with advanced filtering | L | High |
| E14.3 | User management endpoints | M | High |
| E14.4 | Group management admin endpoints | M | High |
| E14.5 | Policy apply/unapply endpoints | M | High |
| E14.6 | Enrollment token QR code generation | S | High |
| E14.7 | Organization usage statistics endpoint | M | Medium |
| E14.8 | Reports generation endpoints | M | Medium |
| E14.9 | Export endpoints (CSV, JSON) | M | Medium |

---

## Epic Summary

| Epic | Name | Stories | Effort | Dependencies |
|------|------|---------|--------|--------------|
| E9 | Authentication Foundation | 11 | 3-4 weeks | None |
| E10 | User-Device Binding | 6 | 2 weeks | E9 |
| E11 | Group Management | 9 | 3 weeks | E10 |
| E12 | Settings Control | 8 | 2-3 weeks | E11 |
| E13 | B2B Enterprise | 10 | 4 weeks | E12 |
| E14 | Admin Portal Backend | 9 | 3-4 weeks | E13 |
| **Total** | | **53** | **17-21 weeks** | |

**Note:** Stories use new numbering (E9.x - E14.x) to avoid conflicts with existing stories (E1.x - E8.x).

---

## Security Considerations

### Authentication Security
- Argon2id for password hashing (memory: 64MB, iterations: 3, parallelism: 1)
- RS256 JWT with key rotation support
- Refresh token rotation with family invalidation
- Rate limiting on auth endpoints

### Token Specifications
- **Access Token:** 1 hour lifetime, RS256 signed
- **Refresh Token:** 30 days lifetime, rotated on use
- **Device Token:** 90 days lifetime for B2B managed devices

### Rate Limits
| Endpoint | Limit |
|----------|-------|
| Registration | 10/hour/IP |
| Login | 20/hour/IP, 5 failures = 15min lockout |
| Password reset | 5/hour/IP |
| Token refresh | 60/hour |

### RBAC Permissions
- **Owner:** Full control, can delete group, transfer ownership
- **Admin:** Manage members, lock settings, view all devices
- **Member:** View group members and locations
- **Viewer:** View only, no modifications

---

## Out of Scope

1. **Multi-tenancy database isolation** - Single database with org_id filtering
2. **SSO/SAML integration** - OAuth only for initial release
3. **Two-factor authentication** - Planned for future release
4. **Real-time WebSocket APIs** - Polling-based updates only
5. **GraphQL API** - REST only for initial release
6. **Admin portal frontend** - Separate project

---

## Migration Strategy

### Database Migration Order
```
1. organizations
2. users
3. oauth_accounts
4. user_sessions
5. groups
6. group_memberships
7. group_invites
8. device_policies
9. devices modifications
10. device_settings
11. setting_definitions
12. unlock_requests
13. enrollment_tokens
14. device_tokens
15. audit_logs
16. org_users
17. api_keys modifications
```

### Backward Compatibility
- Existing devices continue to work with X-API-Key authentication
- All existing endpoints remain functional
- New auth is optional for device registration
- Legacy group_id field supported during transition

---

## Related Specifications

All detailed specifications are in `/docs/specs/`:

| Document | Description |
|----------|-------------|
| `DATA_MODELS_SPEC.md` | Domain models, API contracts |
| `SECURITY_SPEC.md` | JWT design, password hashing, RBAC |
| `DATABASE_SCHEMA_SPEC.md` | PostgreSQL schema design |
| `AUTH_API_SPEC.md` | Authentication endpoints |
| `USER_GROUP_API_SPEC.md` | User/Group endpoints |
| `DEVICE_SETTINGS_API_SPEC.md` | Settings endpoints |
| `B2B_ENTERPRISE_SPEC.md` | Enterprise features |
| `ADMIN_PORTAL_SPEC.md` | Admin dashboard backend |

---

## Document Status

- [x] Goals and context validated
- [x] All functional requirements reviewed
- [x] API endpoints documented
- [x] Database schema designed
- [x] Epic structure approved
- [x] Security requirements specified
- [ ] Ready for implementation

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-12-01 | Martin | Initial PRD from specifications |

---

_This PRD covers backend API development with 53 stories across 6 epics, estimated at 17-21 weeks._
