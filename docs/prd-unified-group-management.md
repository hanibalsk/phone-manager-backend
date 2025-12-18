---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
lastStep: 11
status: complete
inputDocuments:
  - /Users/martinjanci/.claude/plans/async-wondering-turtle.md
  - /Users/martinjanci/projects/github.com/hanibalsk/phone-manager-backend/docs/PRD.md
  - /Users/martinjanci/projects/github.com/hanibalsk/phone-manager-backend/docs/specs/USER_GROUP_API_SPEC.md
  - /Users/martinjanci/projects/github.com/hanibalsk/phone-manager-backend/CLAUDE.md
documentCounts:
  briefs: 0
  research: 0
  brainstorming: 0
  projectDocs: 4
workflowType: 'prd'
lastStep: 0
project_name: 'Phone Manager Backend - Unified Group Management'
user_name: 'Martin'
date: '2025-12-18'
---

# Product Requirements Document - Phone Manager Backend - Unified Group Management

**Author:** Martin
**Date:** 2025-12-18

## Executive Summary

Phone Manager Backend requires a critical architectural enhancement to unify its two conflicting group systems. Currently, devices register to simple string-based "registration groups" for anonymous discovery, while authenticated users manage separate "authenticated groups" with roles and invite codes. This creates a gap where devices get stuck between the two systems when users log in.

This PRD defines the backend API changes needed to bridge this gap, enabling:
- **Seamless device-to-user linking** on authentication
- **Automatic group migration** from registration groups to authenticated groups
- **Multi-group device support** for complex family structures
- **Backwards compatibility** for unauthenticated users

The solution preserves zero-friction onboarding (anonymous registration groups still work) while providing a smooth upgrade path when users decide to authenticate. Auto-migration ensures users don't lose their existing group members or devices.

### What Makes This Special

The key insight driving this feature is the **friction-filled upgrade journey** users currently experience:

1. Install app → anonymous registration group (works immediately)
2. Later decide to log in → device is "orphaned" from the authenticated group system
3. No smooth path to upgrade their experience

This solution delivers:
- **Zero-friction onboarding preserved** - Anonymous registration groups continue to work
- **Seamless upgrade path** - When users authenticate, their devices and groups migrate automatically
- **Auto-migration** - Users don't lose existing group members or devices
- **Multi-group support** - Devices can belong to multiple authenticated groups (e.g., "Parents Group" + "Extended Family Group")

## Project Classification

**Technical Type:** api_backend (REST API with Axum)
**Domain:** general (family location sharing)
**Complexity:** low (extends existing patterns)
**Project Context:** Brownfield - extending existing system

**Existing Tech Stack:**
- Rust 1.83+ (Edition 2024), Axum 0.8, Tokio
- PostgreSQL 16 + SQLx 0.8
- JWT + API Key authentication
- Layered architecture (Routes → Services → Repositories → Entities)

**Companion PRD:** Phone Manager Android App (docs/prd.md) - provides the mobile client implementation for this backend feature.

## Success Criteria

### User Success

- **Instant Migration:** User logs in and migration completes instantly (< 2 seconds perceived latency)
- **Zero Manual Steps:** Device linking and group migration happen automatically on authentication
- **No Data Loss:** All devices from the registration group appear in the new authenticated group
- **One-Tap Group Join:** Adding device to additional groups requires one tap with confirmation dialog
- **Seamless Experience:** User doesn't need to understand the underlying group architecture change

### Business Success

- **90% Migration Success Rate:** Migrations complete without errors for 90%+ of users
- **Reduced Support Tickets:** Measurable decrease in "lost device", "can't see my group", and "device not showing" support tickets
- **Increased Authentication Adoption:** More anonymous users convert to authenticated users due to smooth upgrade path

### Technical Success

- **API Response Time:** Migration endpoint responds in < 200ms at p95
- **Atomicity:** Migration is transactional - all devices migrate or none do (no partial state)
- **Backwards Compatibility:** Registration group endpoints (`POST /api/v1/devices/register`, `GET /api/v1/devices?groupId=`) continue working for unauthenticated users
- **No Breaking Changes:** Existing authenticated group APIs remain unchanged
- **Audit Logging:** All migration events logged with: user ID, registration group ID, new authenticated group ID, devices migrated, timestamp, success/failure status

### Measurable Outcomes

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Migration success rate | ≥ 90% | Backend logs / Prometheus metrics |
| Migration latency (p95) | < 200ms | API metrics |
| Support tickets (group issues) | -50% within 3 months | Support system tracking |
| Authentication conversion | +20% within 3 months | User analytics |

## Product Scope

### MVP - Minimum Viable Product

1. **Auto-link device on login** - Set `owner_user_id` when user authenticates
2. **Group migration endpoint** - `POST /api/v1/groups/migrate` converts registration group to authenticated group
3. **Auto-migrate all devices** - All devices in registration group move to new authenticated group
4. **Device-to-group assignment** - `POST /api/v1/groups/:groupId/devices` adds device to authenticated group
5. **Audit logging** - Migration events logged for troubleshooting and compliance

### Growth Features (Post-MVP)

- **Migration rollback** - Ability to undo migration if issues detected
- **Partial migration** - Select specific devices to migrate (user choice)
- **Migration analytics dashboard** - Admin visibility into migration patterns
- **Bulk device management** - Add/remove multiple devices from groups at once

### Vision (Future)

- **Cross-platform sync** - iOS devices participate in same groups
- **Group templates** - Pre-configured group types (Family, Friends, Work)
- **Smart group suggestions** - AI-powered recommendations for group organization
- **Federation** - Groups spanning multiple Phone Manager instances

## User Journeys

### Journey 1: David Chen - From Quick Setup to Family Hub (Converting User)

David is a 35-year-old father of two who discovered Phone Manager before a weekend camping trip in the mountains. With spotty cell coverage expected, he wanted a simple way to keep track of his kids (ages 10 and 14) during hikes. He downloaded the app on all three phones, typed "camping-2025" as the group name, and within minutes they could see each other's locations. No account needed, no fuss.

The camping trip was a success. When his 14-year-old wandered too far exploring a creek, David could see exactly where she was without panicking. On the drive home, he realizes this isn't just useful for trips - it's exactly what they need for everyday life: school pickups, after-school activities, knowing when the kids arrive home.

David decides to create an account. He taps "Sign Up", enters his email and password, and logs in. Instantly, a message appears: "Welcome back, David! We found your group 'camping-2025' with 3 devices. Converting to your Family group..." Within a second, it's done. His new "Chen Family" group shows all three devices - his phone and both kids' phones - already there with their location history intact.

The next week, David's wife Sarah wants to join. David taps "Invite Member", gets a code (ABC-123-XYZ), and texts it to Sarah. She installs the app, logs in, and joins with one tap. When prompted "Add your device to Chen Family?", she confirms. Now all four family members can see each other. David smiles - what started as a quick camping hack is now their family's coordination hub.

### Journey 2: David Chen - The Family Tech Lead (Group Admin)

Three months after setting up the Chen Family group, David has become the household's "tech lead" for location sharing. His group now has 6 members: himself, his wife Sarah, both kids (Emma 14, Jake 10), and his parents who joined after seeing how useful it was during Sunday dinners.

One Tuesday afternoon, David gets a call from his mother. "The app isn't showing Jake's location anymore - it says 'last seen 3 hours ago'." David opens Phone Manager and switches to the admin view. He can see all 6 members listed with their devices. Jake's phone shows a yellow warning icon - battery at 2%, last location was at school.

David taps on Jake's entry to see details. He notices Jake's phone has tracking disabled - Jake must have turned it off to save battery. David toggles "Request Tracking Enable" which sends a gentle push notification to Jake's phone. Twenty minutes later, Jake plugs in his phone at the after-school program, sees the notification, and re-enables tracking. David's mother calls back, relieved: "I can see him now, he's at the community center!"

The following week, Emma asks to join her friend Sofia's family group for an upcoming beach trip. David goes to Emma's device in his admin view and taps "Manage Groups." He can see Emma's device is in "Chen Family." He explains that Emma can join Sofia's family group herself using their invite code - her device can be in both groups simultaneously. Emma joins the "Martinez Beach Trip" group, and now both families can coordinate during the vacation while the Chens still see Emma in their own group.

When the beach trip ends, Emma removes herself from the Martinez group with one tap. David checks his admin view and confirms Emma's device is back to just "Chen Family." Everything worked exactly as it should - flexible enough for real family life.

### Journey Requirements Summary

| Journey | Capability Required |
|---------|---------------------|
| Converting User | Device auto-linking on authentication |
| Converting User | Registration group detection |
| Converting User | Automatic migration of all devices |
| Converting User | Group renaming during migration |
| Converting User | Invite code generation |
| Converting User | One-tap device addition with confirmation |
| Converting User | Location history preservation |
| Admin | Admin view with all members/devices |
| Admin | Device status indicators (battery, last seen, tracking) |
| Admin | Per-device detail view |
| Admin | Remote tracking enable/disable request |
| Admin | Multi-group membership visibility |
| Admin | Device group management |

## Innovation & Novel Patterns

### Detected Innovation Areas

**The "Why Not Both?" Paradigm**

Phone Manager challenges the traditional "either/or" paradigm in location sharing apps:

| Traditional Approach | Phone Manager Innovation |
|---------------------|--------------------------|
| Simple anonymous sharing OR complex account-based management | Both - start anonymous, upgrade seamlessly |
| One device belongs to one group | Multi-group device membership |
| Easy setup OR powerful features | Zero-friction onboarding WITH full management capabilities |

This approach delivers instant value with zero friction while enabling seamless upgrade to authenticated features without losing data or group members.

### Market Context & Competitive Landscape

**Pain Points in Existing Solutions:**
- Life360, Find My, and similar apps require account creation upfront
- Anonymous→authenticated transitions typically lose data or require re-setup
- Competitors don't offer multi-group device membership (device locked to single family group)

**Phone Manager Differentiation:**
- Only solution offering true zero-friction start with seamless upgrade path
- Multi-group flexibility enables complex real-world scenarios (e.g., "Parents Group" + "Extended Family" + "Beach Trip")
- No forced choice between simplicity and power

### Validation Approach

| Innovation Aspect | Validation Metric | Target |
|-------------------|-------------------|--------|
| Seamless migration | Migration success rate | ≥ 90% |
| Instant experience | Migration latency | < 2 seconds |
| Multi-group adoption | Users with 2+ groups | Track adoption rate |
| No data loss | Devices migrated vs. original count | 100% preservation |

### Risk Mitigation

**Migration Failure Handling:**
- Target 90% success rate provides acceptable baseline
- Failed migrations logged with detailed audit trail for debugging
- Future enhancement: Manual migration fallback option (post-MVP)

**Multi-Group Complexity:**
- No anticipated performance or UX complexity issues
- Backend designed for efficient multi-group queries
- UI clearly shows which groups a device belongs to

## API Backend Specific Requirements

### Project-Type Overview

This feature extends the existing Phone Manager REST API with new endpoints for unified group management. All endpoints follow existing patterns: JSON with `snake_case` fields, `/api/v1/` versioning, JWT authentication for user operations, and standard error response format.

### New API Endpoints

#### POST /api/v1/groups/migrate

Migrate a registration group to an authenticated group.

| Attribute | Value |
|-----------|-------|
| **Auth** | JWT (Bearer token) |
| **Rate Limit** | Standard (100/min) |
| **Authorization** | Authenticated user with device in the registration group |

**Request:**
```json
{
  "registration_group_id": "camping-2025",
  "group_name": "Chen Family"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| registration_group_id | string | Yes | The registration group ID to migrate |
| group_name | string | No | Name for the new authenticated group (defaults to registration_group_id) |

**Response (201 Created):**
```json
{
  "authenticated_group_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Chen Family",
  "devices_migrated": 3,
  "migration_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

**Error Responses:**
| Status | Code | Description |
|--------|------|-------------|
| 400 | validation/invalid-group | Registration group not found |
| 400 | validation/no-devices | No devices in registration group |
| 409 | resource/already-migrated | Registration group already migrated |
| 409 | resource/group-name-exists | Authenticated group name already taken |

---

#### POST /api/v1/groups/:groupId/devices

Add a device to an authenticated group.

| Attribute | Value |
|-----------|-------|
| **Auth** | JWT (Bearer token) |
| **Rate Limit** | Standard (100/min) |
| **Authorization** | Device owner who is a group member |

**Request:**
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response (200 OK):**
```json
{
  "group_id": "550e8400-e29b-41d4-a716-446655440000",
  "device_id": "660e8400-e29b-41d4-a716-446655440001",
  "added_at": "2025-12-18T10:30:00Z"
}
```

**Error Responses:**
| Status | Code | Description |
|--------|------|-------------|
| 403 | authz/not-device-owner | User doesn't own this device |
| 403 | authz/not-group-member | User is not a member of this group |
| 404 | resource/not-found | Device or group not found |
| 409 | resource/already-exists | Device already in this group |

---

#### DELETE /api/v1/groups/:groupId/devices/:deviceId

Remove a device from an authenticated group.

| Attribute | Value |
|-----------|-------|
| **Auth** | JWT (Bearer token) |
| **Rate Limit** | Standard (100/min) |
| **Authorization** | Device owner OR group admin/owner |

**Response (204 No Content)**

**Error Responses:**
| Status | Code | Description |
|--------|------|-------------|
| 403 | authz/forbidden | User cannot remove this device |
| 404 | resource/not-found | Device not in this group |

---

#### GET /api/v1/devices/me/registration-group

Check if the current user's device is in a registration group (for migration prompt).

| Attribute | Value |
|-----------|-------|
| **Auth** | JWT (Bearer token) |
| **Rate Limit** | Standard (100/min) |
| **Authorization** | Authenticated user |

**Response (200 OK) - Device in registration group:**
```json
{
  "has_registration_group": true,
  "registration_group_id": "camping-2025",
  "device_count": 3,
  "already_migrated": false
}
```

**Response (200 OK) - No registration group:**
```json
{
  "has_registration_group": false,
  "registration_group_id": null,
  "device_count": 0,
  "already_migrated": false
}
```

**Response (200 OK) - Already migrated:**
```json
{
  "has_registration_group": false,
  "registration_group_id": "camping-2025",
  "device_count": 0,
  "already_migrated": true,
  "migrated_to_group_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Note:** Frontend calls this endpoint after login to determine if migration prompt should be shown.

---

#### GET /api/v1/groups/:groupId/members (Enhanced)

List all members in an authenticated group with device counts.

| Attribute | Value |
|-----------|-------|
| **Auth** | JWT (Bearer token) |
| **Rate Limit** | Standard (100/min) |
| **Authorization** | Group member |

**Response (200 OK):**
```json
{
  "data": [
    {
      "user_id": "user_01HXYZABC123",
      "display_name": "David Chen",
      "email": "david@example.com",
      "role": "owner",
      "joined_at": "2025-12-18T10:30:00Z",
      "device_count": 2
    },
    {
      "user_id": "user_01HXYZABC456",
      "display_name": "Sarah Chen",
      "email": "sarah@example.com",
      "role": "admin",
      "joined_at": "2025-12-18T11:00:00Z",
      "device_count": 1
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 6,
    "total_pages": 1
  }
}
```

**Note:** The `device_count` field is added to support frontend FR21 (device count badge per member).

---

#### GET /api/v1/groups/:groupId/devices

List all devices in an authenticated group.

| Attribute | Value |
|-----------|-------|
| **Auth** | JWT (Bearer token) |
| **Rate Limit** | Standard (100/min) |
| **Authorization** | Group member |

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| page | integer | 1 | Page number |
| per_page | integer | 20 | Items per page (max 100) |
| include_location | boolean | false | Include last known location |

**Response (200 OK):**
```json
{
  "data": [
    {
      "device_id": "550e8400-e29b-41d4-a716-446655440000",
      "display_name": "David's Pixel 8",
      "owner_user_id": "user_01HXYZABC123",
      "owner_display_name": "David Chen",
      "added_at": "2025-12-18T10:30:00Z",
      "last_seen_at": "2025-12-18T14:22:00Z",
      "last_location": {
        "latitude": 37.7749,
        "longitude": -122.4194,
        "accuracy": 10.5,
        "timestamp": "2025-12-18T14:22:00Z"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 6,
    "total_pages": 1
  }
}
```

### Data Schemas

#### Migration Audit Log Schema

All migrations are logged for audit and debugging:

| Field | Type | Description |
|-------|------|-------------|
| migration_id | UUID | Unique migration identifier |
| user_id | UUID | User who initiated migration |
| registration_group_id | string | Original registration group |
| authenticated_group_id | UUID | New authenticated group |
| devices_migrated | integer | Count of devices migrated |
| device_ids | UUID[] | List of migrated device IDs |
| status | enum | success, failed, partial |
| error_message | string | Error details if failed |
| created_at | timestamp | Migration timestamp |

#### Device-Group Membership Schema

New junction table for multi-group device membership:

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Membership ID |
| device_id | UUID | FK to devices |
| group_id | UUID | FK to groups |
| added_by | UUID | User who added device |
| added_at | timestamp | When device was added |

### Technical Architecture Considerations

**Database Changes:**
- New `device_group_memberships` table for multi-group support
- New `migration_audit_logs` table for audit trail
- Index on `device_id` and `group_id` for efficient queries

**Transaction Boundaries:**
- Migration operation is fully atomic (all devices migrate or none)
- Uses PostgreSQL transaction with SERIALIZABLE isolation

**Backwards Compatibility:**
- Existing `devices.group_id` column preserved for registration groups
- Existing endpoints continue to work unchanged
- New endpoints operate on `device_group_memberships` table

### Implementation Considerations

**Layered Architecture:**
Following existing patterns:
- Routes: `crates/api/src/routes/group_migration.rs`
- Service: `crates/domain/src/services/group_migration_service.rs`
- Repository: `crates/persistence/src/repositories/device_group_membership.rs`
- Models: `crates/domain/src/models/migration_audit.rs`

**Error Handling:**
- Use existing `thiserror` patterns for domain errors
- Structured error responses per FR-21 from main PRD

**Metrics:**
- `migration_total` counter (success/failure)
- `migration_duration_seconds` histogram
- `device_group_memberships_total` gauge

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Problem-Solving MVP
**Rationale:** Fix the specific "device orphan" architectural gap that causes user friction when transitioning from anonymous to authenticated usage.

**Resource Requirements:**
- 1 backend developer
- Estimated effort: Small (6 endpoints, 2 database tables)
- Follows existing patterns, minimal new architecture

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**
- Converting User journey (David Chen - anonymous to authenticated)
- Admin journey (managing family group devices)

**Must-Have Capabilities:**

| Feature | Endpoint/Component | Priority |
|---------|-------------------|----------|
| Auto-link device on login | Auth flow enhancement | Critical |
| Registration group check | `GET /api/v1/devices/me/registration-group` | Critical |
| Group migration | `POST /api/v1/groups/migrate` | Critical |
| Add device to group | `POST /api/v1/groups/:groupId/devices` | Critical |
| Remove device from group | `DELETE /api/v1/groups/:groupId/devices/:deviceId` | Critical |
| List group devices | `GET /api/v1/groups/:groupId/devices` | Critical |
| List members with device count | `GET /api/v1/groups/:groupId/members` (enhanced) | Critical |
| Migration audit logging | `migration_audit_logs` table | Critical |
| Multi-group membership | `device_group_memberships` table | Critical |

### Post-MVP Features

**Phase 2 (Growth):**
- Migration rollback capability - Undo migration if issues detected
- Partial migration - Select specific devices to migrate instead of all
- Migration analytics dashboard - Admin visibility into migration patterns
- Bulk device management - Add/remove multiple devices in single operation

**Phase 3 (Expansion):**
- Cross-platform sync - iOS devices participate in same groups
- Group templates - Pre-configured group types (Family, Friends, Work)
- Smart group suggestions - AI-powered recommendations for group organization
- Federation - Groups spanning multiple Phone Manager instances

### Risk Mitigation Strategy

**Technical Risks:**
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Migration transaction failure | Low | High | PostgreSQL SERIALIZABLE isolation ensures atomicity |
| Multi-group query performance | Low | Medium | Proper indexing on device_id and group_id |
| Backwards compatibility break | Low | High | Preserve existing `devices.group_id` column |

**Market Risks:**
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Users don't adopt migration | Low | Medium | Solving known pain point with seamless UX |

**Resource Risks:**
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Developer unavailable | Low | Medium | Small scope allows any backend dev to pick up |
| Scope creep | Medium | Medium | Clear MVP boundaries defined above |

## Functional Requirements

### Device-User Linking

- FR1: System can automatically link a device to its owner when the owner authenticates
- FR2: System can detect if an authenticating user has a device in a registration group
- FR3: User can see which devices are linked to their account
- FR25: System can return registration group status for frontend migration prompt (via `GET /api/v1/devices/me/registration-group`)

### Group Migration

- FR4: Authenticated user can migrate a registration group to an authenticated group
- FR5: User can specify a custom name for the new authenticated group during migration
- FR6: System can automatically migrate all devices in a registration group to the new authenticated group
- FR7: System can preserve device data and location history during migration
- FR8: System can prevent duplicate migrations of the same registration group
- FR9: System can log all migration events with user, groups, devices, and status

### Device-Group Management

- FR10: Group member can add their device to an authenticated group
- FR11: Device owner can remove their device from an authenticated group
- FR12: Group admin/owner can remove any device from an authenticated group
- FR13: Group member can view all devices in an authenticated group
- FR14: Group member can view device details including owner and last seen timestamp
- FR15: Group member can view last known location of devices in the group
- FR26: Group member can view member list with device count per member (via enhanced `GET /api/v1/groups/:groupId/members`)

### Multi-Group Support

- FR16: Device can belong to multiple authenticated groups simultaneously
- FR17: User can view which groups a device belongs to
- FR18: System can track device membership across multiple groups independently

### Backwards Compatibility

- FR19: Unauthenticated devices can continue to use registration groups
- FR20: Existing registration group endpoints continue to function unchanged
- FR21: Existing authenticated group endpoints continue to function unchanged

### Audit & Observability

- FR22: System can record migration audit logs with full context
- FR23: System can track migration success/failure metrics
- FR24: Admin can query migration history for troubleshooting

## Non-Functional Requirements

### Performance

- NFR1: Migration endpoint (`POST /api/v1/groups/migrate`) responds in < 200ms at p95
- NFR2: Device list endpoint (`GET /api/v1/groups/:groupId/devices`) responds in < 200ms at p95
- NFR3: Add/remove device endpoints respond in < 100ms at p95
- NFR4: Migration operation completes in < 2 seconds perceived latency for users
- NFR5: System maintains performance under concurrent migration requests

### Security

- NFR6: All new endpoints require valid JWT authentication
- NFR7: Users can only migrate registration groups where they own a device
- NFR8: Users can only add/remove their own devices to/from groups
- NFR9: Group admins/owners can remove any device from their group
- NFR10: Only group members can view devices in a group
- NFR11: All migration events are logged with user identity for audit compliance
- NFR12: Location data in device list responses respects existing privacy controls
- NFR19: Invite codes expire after 48 hours by default (configurable per group)
- NFR20: Expired invite codes return appropriate error response with expiry information

### Reliability

- NFR13: Migration operation is atomic - all devices migrate or none do
- NFR14: System maintains 99.9% uptime for new endpoints
- NFR15: No data loss during migration (device data, location history preserved)
- NFR16: Failed migrations are logged with sufficient detail for troubleshooting
- NFR17: System gracefully handles concurrent migrations of the same registration group
- NFR18: Backwards compatibility maintained - existing endpoints unaffected

### API Behavior

- NFR21: All endpoints return within 30 seconds or return timeout error (504)
- NFR22: Timeout errors include `retry_after` header suggesting retry delay
- NFR23: Long-running operations (migration) provide progress indication if exceeding 2 seconds
- NFR24: API returns appropriate `Content-Type` headers for all responses

