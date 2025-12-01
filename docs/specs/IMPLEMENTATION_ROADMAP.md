# Implementation Roadmap

## Phone Manager - User Management & Device Control

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-01

---

## 1. Overview

This document outlines the phased implementation plan for the User Management and Device Control feature, including dependencies, effort estimates, and risk assessment.

---

## 2. Implementation Phases

### Phase 1: Authentication Foundation

**Duration:** 3-4 weeks
**Priority:** Critical
**Dependencies:** None

#### Backend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| Database migrations | 3d | Create users, oauth_accounts, user_sessions tables |
| Password hashing | 1d | Implement Argon2id hashing utilities |
| JWT infrastructure | 2d | RS256 key management, token generation/validation |
| Register endpoint | 2d | Email/password registration with validation |
| Login endpoint | 2d | Email/password login, token issuance |
| Logout endpoint | 1d | Token invalidation |
| Refresh token endpoint | 2d | Token rotation, reuse detection |
| Google OAuth | 3d | ID token verification, account creation/linking |
| Apple OAuth | 3d | ID token verification, account creation/linking |
| Password reset | 2d | Token generation, email integration |
| Auth middleware update | 2d | Support both Bearer tokens and X-API-Key |

#### Frontend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| SecureTokenStorage | 2d | Extend for JWT tokens |
| AuthInterceptor | 2d | Token refresh logic |
| Login screen | 3d | UI with email/password |
| Register screen | 2d | UI with validation |
| OAuth integration | 3d | Google/Apple Sign-In SDKs |
| AuthViewModel | 2d | State management |
| Session management | 2d | Auto-logout, session persistence |

#### Deliverables
- Users can register with email/password
- Users can log in with email/password
- Users can log in with Google/Apple
- Tokens are securely stored and refreshed
- Existing devices continue to work (backward compatibility)

---

### Phase 2: User-Device Binding

**Duration:** 2 weeks
**Priority:** High
**Dependencies:** Phase 1

#### Backend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| Device table migration | 1d | Add owner_user_id, is_managed columns |
| Link device endpoint | 2d | POST /users/{id}/devices/{id}/link |
| Unlink device endpoint | 1d | DELETE /users/{id}/devices/{id}/unlink |
| User devices endpoint | 1d | GET /users/{id}/devices |
| Device registration update | 2d | Support authenticated registration |
| Transfer device endpoint | 2d | Transfer ownership between users |

#### Frontend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| Device models update | 1d | Add ownership fields |
| DeviceLinkScreen | 2d | UI for linking device to account |
| DeviceListScreen | 2d | Show user's devices |
| DeviceManagementViewModel | 2d | State management |
| Registration flow update | 1d | Integrate device linking |

#### Deliverables
- Users can link devices to their account
- Users can see all their linked devices
- Users can unlink/transfer devices
- Device registration works with authentication

---

### Phase 3: Group Management

**Duration:** 3 weeks
**Priority:** High
**Dependencies:** Phase 2

#### Backend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| Groups table migration | 1d | Create groups table |
| Memberships table migration | 1d | Create group_memberships table |
| Invites table migration | 1d | Create group_invites table |
| Group CRUD endpoints | 3d | Create, read, update, delete groups |
| Membership endpoints | 2d | Add, remove, list members |
| Role management endpoints | 2d | Update member roles |
| Invite CRUD endpoints | 3d | Create, list, revoke invites |
| Join group endpoint | 2d | Join with invite code |
| Permission middleware | 2d | Role-based access checks |

#### Frontend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| Group models | 1d | Group, Membership, Invite models |
| GroupListScreen | 2d | List user's groups |
| GroupCreateScreen | 2d | Create new group |
| GroupDetailScreen | 3d | View members, settings |
| InviteMemberScreen | 2d | Generate and share invites |
| JoinGroupScreen | 2d | Enter invite code |
| MemberManagementScreen | 2d | Manage roles, remove members |
| GroupRepository | 2d | API integration |
| GroupViewModel | 2d | State management |

#### Deliverables
- Users can create groups
- Users can invite others via codes
- Users can join groups with invite codes
- Admins can manage member roles
- Role-based permissions enforced

---

### Phase 4: Settings Control

**Duration:** 2-3 weeks
**Priority:** High
**Dependencies:** Phase 3

#### Backend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| Device settings table | 2d | Create device_settings table with locks |
| Setting definitions table | 1d | Create lockable settings catalog |
| Get settings endpoint | 2d | GET /devices/{id}/settings |
| Update settings endpoint | 2d | PUT /devices/{id}/settings (respects locks) |
| Lock settings endpoint | 2d | PUT /devices/{id}/settings/locks |
| Unlock request endpoints | 2d | Request, list, approve/deny |
| Settings sync push | 2d | FCM notifications for settings changes |

#### Frontend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| DeviceSettings models | 1d | Settings, locks, unlock requests |
| SettingsScreen update | 3d | Show lock indicators, managed status |
| AdminSettingsScreen | 3d | Manage member device settings |
| LockManagementUI | 2d | Toggle locks per setting |
| UnlockRequestScreen | 2d | Request unlocks, view status |
| Settings sync logic | 2d | Sync on app start, handle push |
| PreferencesRepository update | 2d | Server-synced settings |

#### Deliverables
- Admins can view member device settings
- Admins can lock/unlock settings
- Users see locked settings with indicators
- Users can request setting unlocks
- Settings sync between server and device

---

### Phase 5: B2B Enterprise Features

**Duration:** 4 weeks
**Priority:** Medium
**Dependencies:** Phase 4

#### Backend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| Organizations table | 2d | Create organizations table |
| Org users table | 1d | Create org_users table |
| Device policies table | 2d | Create device_policies table |
| Enrollment tokens table | 1d | Create enrollment_tokens table |
| Audit logs table | 2d | Create audit_logs table |
| Organization CRUD | 3d | Organization management endpoints |
| Policy CRUD | 3d | Policy management endpoints |
| Device enrollment | 3d | Token-based enrollment flow |
| Bulk device import | 2d | CSV/JSON bulk import |
| Audit logging | 2d | Log all admin actions |
| Audit query endpoint | 2d | Search and export logs |
| Policy engine | 3d | Policy resolution algorithm |

#### Frontend Tasks

| Task | Effort | Description |
|------|--------|-------------|
| B2B models | 2d | Organization, Policy models |
| Enrollment flow | 3d | QR code / token enrollment |
| Policy display | 2d | Show applied policy info |
| Managed device indicators | 1d | Show "managed by" info |

#### Deliverables
- Organizations can be created
- Device policies can be defined
- Devices can be enrolled via tokens
- Policies automatically applied to devices
- All admin actions audited

---

### Phase 6: Admin Web Portal

**Duration:** 4-5 weeks
**Priority:** Medium
**Dependencies:** Phase 5

#### Tasks

| Task | Effort | Description |
|------|--------|-------------|
| Portal project setup | 2d | React/Next.js project setup |
| Authentication | 3d | Login, session management |
| Dashboard | 3d | Overview, stats, alerts |
| Device management | 5d | List, search, filter, bulk actions |
| User management | 3d | List, invite, manage users |
| Group management | 3d | Create, manage groups |
| Policy management | 4d | Create, edit, assign policies |
| Audit logs | 3d | View, search, export logs |
| Reports | 3d | Device inventory, compliance |

#### Deliverables
- Web-based admin dashboard
- Device fleet management
- User and group management
- Policy creation and assignment
- Audit log viewing and export

---

## 3. Dependency Graph

```
Phase 1: Authentication Foundation
    │
    ├──► Phase 2: User-Device Binding
    │        │
    │        └──► Phase 3: Group Management
    │                 │
    │                 └──► Phase 4: Settings Control
    │                          │
    │                          └──► Phase 5: B2B Enterprise
    │                                   │
    │                                   └──► Phase 6: Admin Portal
    │
    └──► (Backward Compatibility: Existing devices work throughout)
```

---

## 4. Effort Summary

| Phase | Backend | Frontend | Admin Portal | Total |
|-------|---------|----------|--------------|-------|
| Phase 1 | 21d | 16d | - | 37d |
| Phase 2 | 9d | 8d | - | 17d |
| Phase 3 | 17d | 18d | - | 35d |
| Phase 4 | 13d | 15d | - | 28d |
| Phase 5 | 23d | 8d | - | 31d |
| Phase 6 | - | - | 29d | 29d |
| **Total** | **83d** | **65d** | **29d** | **177d** |

**Estimated Duration:** 18-22 weeks (with parallel work)

---

## 5. Risk Assessment

### High Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Authentication security vulnerabilities | Critical | Security review, penetration testing |
| Data migration issues | High | Extensive testing, rollback plan |
| Performance degradation with RBAC | Medium | Query optimization, caching |
| OAuth provider API changes | Medium | Abstract provider layer, monitoring |

### Medium Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Token refresh race conditions | Medium | Mutex/locking, retry logic |
| Settings sync conflicts | Medium | Server-authoritative for locks |
| B2B policy complexity | Medium | Incremental rollout, testing |
| Mobile app store review delays | Medium | Early submission, feature flags |

### Low Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Invite code collisions | Low | CSPRNG, uniqueness check |
| Audit log storage growth | Low | Retention policies, archival |

---

## 6. Testing Strategy

### Phase 1 Testing
- Unit tests for password hashing
- Unit tests for JWT operations
- Integration tests for auth endpoints
- E2E tests for login/register flows
- Security testing (auth bypass, token manipulation)

### Phase 2-4 Testing
- Unit tests for permission logic
- Integration tests for all endpoints
- E2E tests for user flows
- Load testing for settings sync

### Phase 5-6 Testing
- Integration tests for B2B flows
- E2E tests for admin portal
- Performance testing for bulk operations
- Security audit

---

## 7. Rollout Strategy

### Feature Flags

```kotlin
object FeatureFlags {
    val USER_ACCOUNTS_ENABLED = false      // Phase 1
    val DEVICE_LINKING_ENABLED = false     // Phase 2
    val GROUPS_ENABLED = false             // Phase 3
    val SETTINGS_CONTROL_ENABLED = false   // Phase 4
    val B2B_ENABLED = false                // Phase 5
}
```

### Rollout Phases

1. **Internal Testing** - Enable for development team
2. **Beta Testing** - Enable for beta testers
3. **Staged Rollout** - 10% → 25% → 50% → 100%
4. **Full Release** - Enable for all users

### Backward Compatibility

- Existing devices continue working with X-API-Key
- No forced migration
- Features progressively unlock with account creation

---

## 8. Success Metrics

### Phase 1
- [ ] User registration success rate > 95%
- [ ] Login success rate > 99%
- [ ] Token refresh success rate > 99.9%
- [ ] No authentication security incidents

### Phase 2-4
- [ ] Device linking success rate > 95%
- [ ] Group creation success rate > 99%
- [ ] Settings sync latency < 5s
- [ ] Unlock request response time < 24h average

### Phase 5-6
- [ ] Bulk import success rate > 98%
- [ ] Policy application success rate > 99%
- [ ] Admin portal page load time < 3s
- [ ] Audit log query time < 1s

---

## 9. Critical Files to Modify

### Android (phone-manager)

| File | Changes |
|------|---------|
| `security/SecureStorage.kt` | JWT token storage methods |
| `network/NetworkManager.kt` | Auth interceptor integration |
| `network/DeviceApiService.kt` | Bearer token auth |
| `data/repository/DeviceRepository.kt` | User-device binding |
| `data/preferences/PreferencesRepository.kt` | Server-synced settings |
| `ui/navigation/NavGraph.kt` | New screens |
| `di/NetworkModule.kt` | Auth interceptor DI |

### Backend (phone-manager-backend)

| File | Changes |
|------|---------|
| `crates/persistence/src/migrations/` | New table migrations |
| `crates/api/src/middleware/auth.rs` | JWT + API key support |
| `crates/domain/src/models/` | New domain models |
| `crates/persistence/src/repositories/` | New repositories |
| `crates/api/src/routes/` | New route handlers |
| `crates/api/src/config.rs` | JWT configuration |

---

## 10. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-01 | Initial roadmap |
