---
stepsCompleted: [1, 2, 3, 4]
status: complete
inputDocuments:
  - /Users/martinjanci/projects/github.com/hanibalsk/phone-manager-backend/docs/prd-unified-group-management.md
  - /Users/martinjanci/projects/github.com/hanibalsk/phone-manager-backend/docs/solution-architecture.md
project_name: 'Phone Manager Backend - Unified Group Management'
user_name: 'Martin'
date: '2025-12-18'
---

# Phone Manager Backend - Unified Group Management - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for Phone Manager Backend - Unified Group Management, decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

- FR1: System can automatically link a device to its owner when the owner authenticates
- FR2: System can detect if an authenticating user has a device in a registration group
- FR3: User can see which devices are linked to their account
- FR25: System can return registration group status for frontend migration prompt (via `GET /api/v1/devices/me/registration-group`)
- FR4: Authenticated user can migrate a registration group to an authenticated group
- FR5: User can specify a custom name for the new authenticated group during migration
- FR6: System can automatically migrate all devices in a registration group to the new authenticated group
- FR7: System can preserve device data and location history during migration
- FR8: System can prevent duplicate migrations of the same registration group
- FR9: System can log all migration events with user, groups, devices, and status
- FR10: Group member can add their device to an authenticated group
- FR11: Device owner can remove their device from an authenticated group
- FR12: Group admin/owner can remove any device from an authenticated group
- FR13: Group member can view all devices in an authenticated group
- FR14: Group member can view device details including owner and last seen timestamp
- FR15: Group member can view last known location of devices in the group
- FR26: Group member can view member list with device count per member (via enhanced `GET /api/v1/groups/:groupId/members`)
- FR16: Device can belong to multiple authenticated groups simultaneously
- FR17: User can view which groups a device belongs to
- FR18: System can track device membership across multiple groups independently
- FR19: Unauthenticated devices can continue to use registration groups
- FR20: Existing registration group endpoints continue to function unchanged
- FR21: Existing authenticated group endpoints continue to function unchanged
- FR22: System can record migration audit logs with full context
- FR23: System can track migration success/failure metrics
- FR24: Admin can query migration history for troubleshooting

### Non-Functional Requirements

- NFR1: Migration endpoint (`POST /api/v1/groups/migrate`) responds in < 200ms at p95
- NFR2: Device list endpoint (`GET /api/v1/groups/:groupId/devices`) responds in < 200ms at p95
- NFR3: Add/remove device endpoints respond in < 100ms at p95
- NFR4: Migration operation completes in < 2 seconds perceived latency for users
- NFR5: System maintains performance under concurrent migration requests
- NFR6: All new endpoints require valid JWT authentication
- NFR7: Users can only migrate registration groups where they own a device
- NFR8: Users can only add/remove their own devices to/from groups
- NFR9: Group admins/owners can remove any device from their group
- NFR10: Only group members can view devices in a group
- NFR11: All migration events are logged with user identity for audit compliance
- NFR12: Location data in device list responses respects existing privacy controls
- NFR19: Invite codes expire after 48 hours by default (configurable per group)
- NFR20: Expired invite codes return appropriate error response with expiry information
- NFR13: Migration operation is atomic - all devices migrate or none do
- NFR14: System maintains 99.9% uptime for new endpoints
- NFR15: No data loss during migration (device data, location history preserved)
- NFR16: Failed migrations are logged with sufficient detail for troubleshooting
- NFR17: System gracefully handles concurrent migrations of the same registration group
- NFR18: Backwards compatibility maintained - existing endpoints unaffected
- NFR21: All endpoints return within 30 seconds or return timeout error (504)
- NFR22: Timeout errors include `retry_after` header suggesting retry delay
- NFR23: Long-running operations (migration) provide progress indication if exceeding 2 seconds
- NFR24: API returns appropriate `Content-Type` headers for all responses

### Additional Requirements (from Architecture)

- **Brownfield Project**: This extends an existing Rust/Axum API with established patterns
- **Layered Architecture**: New code must follow Routes → Services → Repositories pattern
- **Database Tables**: Two new tables required (`device_group_memberships`, `migration_audit_logs`)
- **SQLx Migrations**: All database changes via SQLx migration files in `crates/persistence/src/migrations/`
- **Error Handling**: Use existing `thiserror` patterns for domain errors
- **Metrics**: Add Prometheus counters/histograms for migration operations
- **JWT Authentication**: New endpoints use `UserAuth` extractor for JWT validation
- **Transaction Handling**: Migration must use PostgreSQL SERIALIZABLE isolation for atomicity
- **Backwards Compatibility**: Preserve existing `devices.group_id` column for registration groups

### FR Coverage Map

| FR | Epic | Description |
|----|------|-------------|
| FR1 | Epic UGM-1 | Auto-link device on authentication |
| FR2 | Epic UGM-1 | Detect registration group on login |
| FR3 | Epic UGM-1 | View linked devices |
| FR25 | Epic UGM-1 | Registration group status endpoint |
| FR4 | Epic UGM-2 | Migrate registration group to authenticated group |
| FR5 | Epic UGM-2 | Custom name for new group |
| FR6 | Epic UGM-2 | Auto-migrate all devices |
| FR7 | Epic UGM-2 | Preserve device data and history |
| FR8 | Epic UGM-2 | Prevent duplicate migrations |
| FR9 | Epic UGM-2 | Log migration events |
| FR22 | Epic UGM-2 | Record migration audit logs |
| FR23 | Epic UGM-2 | Track migration metrics |
| FR24 | Epic UGM-2 | Query migration history |
| FR10 | Epic UGM-3 | Add device to group |
| FR11 | Epic UGM-3 | Device owner removes device |
| FR12 | Epic UGM-3 | Admin removes any device |
| FR13 | Epic UGM-3 | View all devices in group |
| FR14 | Epic UGM-3 | View device details |
| FR15 | Epic UGM-3 | View device location |
| FR16 | Epic UGM-3 | Multi-group device membership |
| FR17 | Epic UGM-3 | View device's groups |
| FR18 | Epic UGM-3 | Track membership independently |
| FR26 | Epic UGM-3 | Member list with device count |
| FR19 | Epic UGM-4 | Unauthenticated devices use registration groups |
| FR20 | Epic UGM-4 | Registration group endpoints unchanged |
| FR21 | Epic UGM-4 | Authenticated group endpoints unchanged |

## Epic List

### Epic UGM-1: Registration Group Status & Device Linking
When users log in, the system automatically links their device and detects if they have a registration group ready for migration.

**FRs covered:** FR1, FR2, FR3, FR25

**User Value:** Users who previously used anonymous registration groups can now log in and immediately see their migration options. Their device is automatically linked to their account.

---

### Epic UGM-2: Group Migration
Users can convert their anonymous registration group into a full authenticated group with all devices preserved.

**FRs covered:** FR4, FR5, FR6, FR7, FR8, FR9, FR22, FR23, FR24

**User Value:** The seamless upgrade path from anonymous to authenticated usage. Users don't lose their existing group members or location history. Full audit trail for troubleshooting.

---

### Epic UGM-3: Device-Group Management
Users can manage devices within their authenticated groups - adding, removing, and viewing devices with their locations.

**FRs covered:** FR10, FR11, FR12, FR13, FR14, FR15, FR16, FR17, FR18, FR26

**User Value:** Complete device management within groups. Devices can belong to multiple groups for flexible family structures. Group admins can see all members and their devices.

---

### Epic UGM-4: Backwards Compatibility & Stability
Existing users continue to use the app without disruption - all current functionality remains unchanged.

**FRs covered:** FR19, FR20, FR21

**User Value:** Zero disruption for existing users. Anonymous registration groups continue to work. No breaking changes for any existing integrations.

---

## Epic UGM-1: Registration Group Status & Device Linking

When users log in, the system automatically links their device and detects if they have a registration group ready for migration.

### Story UGM-1.1: Auto-link Device on Authentication

As a **user who previously used the app anonymously**,
I want **my device to be automatically linked to my account when I log in**,
So that **I don't have to manually re-register my device after creating an account**.

**Acceptance Criteria:**

**Given** a user has a device registered with a registration group (anonymous)
**When** the user logs in with valid credentials on that device
**Then** the device's `owner_user_id` is set to the authenticated user's ID
**And** the response includes confirmation of device linking

**Given** a user logs in on a device that is already linked to their account
**When** authentication succeeds
**Then** no changes are made to device ownership
**And** the login succeeds normally

**Given** a user logs in on a device linked to a different user
**When** authentication succeeds
**Then** the device ownership is NOT changed (device stays with original owner)
**And** the login succeeds for the new user without device linking

---

### Story UGM-1.2: Registration Group Status Endpoint

As a **newly authenticated user**,
I want **to check if my device has a registration group that can be migrated**,
So that **the app can prompt me to upgrade my anonymous group to an authenticated group**.

**Acceptance Criteria:**

**Given** an authenticated user with a device in a registration group
**When** the user calls `GET /api/v1/devices/me/registration-group`
**Then** the response includes `has_registration_group: true`
**And** the response includes `registration_group_id` with the group name
**And** the response includes `device_count` showing how many devices are in the group
**And** the response includes `already_migrated: false`

**Given** an authenticated user with no device in a registration group
**When** the user calls `GET /api/v1/devices/me/registration-group`
**Then** the response includes `has_registration_group: false`
**And** the response includes `registration_group_id: null`

**Given** an authenticated user whose registration group was already migrated
**When** the user calls `GET /api/v1/devices/me/registration-group`
**Then** the response includes `already_migrated: true`
**And** the response includes `migrated_to_group_id` with the authenticated group UUID

**Given** an unauthenticated request
**When** calling `GET /api/v1/devices/me/registration-group`
**Then** the response is 401 Unauthorized

---

### Story UGM-1.3: View User's Linked Devices

As a **user with multiple devices**,
I want **to see all devices linked to my account**,
So that **I know which devices I own and can manage them**.

**Acceptance Criteria:**

**Given** an authenticated user with multiple devices linked to their account
**When** the user calls `GET /api/v1/devices/me`
**Then** the response includes a list of all devices owned by the user
**And** each device includes `device_id`, `display_name`, `last_seen_at`
**And** each device includes `registration_group_id` if in a registration group

**Given** an authenticated user with no linked devices
**When** the user calls `GET /api/v1/devices/me`
**Then** the response includes an empty list

**Given** an unauthenticated request
**When** calling `GET /api/v1/devices/me`
**Then** the response is 401 Unauthorized

---

## Epic UGM-2: Group Migration

Users can convert their anonymous registration group into a full authenticated group with all devices preserved.

### Story UGM-2.1: Create Migration Audit Log Infrastructure

As a **system administrator**,
I want **all group migrations to be logged with full context**,
So that **I can troubleshoot issues and maintain compliance records**.

**Acceptance Criteria:**

**Given** the system needs to track migration events
**When** a database migration is run
**Then** a new `migration_audit_logs` table is created with columns:
- `migration_id` (UUID, primary key)
- `user_id` (UUID, FK to users)
- `registration_group_id` (string)
- `authenticated_group_id` (UUID, FK to groups)
- `devices_migrated` (integer)
- `device_ids` (UUID array)
- `status` (enum: success, failed, partial)
- `error_message` (text, nullable)
- `created_at` (timestamp with timezone)

**Given** the migration audit log table exists
**When** a migration event occurs (success or failure)
**Then** a record is inserted with all relevant context
**And** the record includes the user who initiated the migration

---

### Story UGM-2.2: Group Migration Endpoint

As a **user with an anonymous registration group**,
I want **to migrate my registration group to an authenticated group**,
So that **I can access full group management features while keeping all my devices**.

**Acceptance Criteria:**

**Given** an authenticated user with a device in a registration group "camping-2025"
**When** the user calls `POST /api/v1/groups/migrate` with `registration_group_id: "camping-2025"`
**Then** a new authenticated group is created
**And** all devices in the registration group are added to the new authenticated group
**And** the user becomes the owner of the new authenticated group
**And** the response includes `authenticated_group_id`, `name`, `devices_migrated`, `migration_id`
**And** the operation completes in < 2 seconds

**Given** a user provides a custom `group_name: "Chen Family"`
**When** the migration is executed
**Then** the new authenticated group is named "Chen Family"

**Given** a user does not provide a custom group name
**When** the migration is executed
**Then** the new authenticated group uses the registration_group_id as its name

**Given** a registration group that has already been migrated
**When** a user attempts to migrate it again
**Then** the response is 409 Conflict with code `resource/already-migrated`

**Given** an invalid registration group ID
**When** a user attempts to migrate it
**Then** the response is 400 Bad Request with code `validation/invalid-group`

**Given** a registration group with no devices
**When** a user attempts to migrate it
**Then** the response is 400 Bad Request with code `validation/no-devices`

**Given** a group name that already exists for an authenticated group
**When** a user attempts to use that name
**Then** the response is 409 Conflict with code `resource/group-name-exists`

**Given** the migration operation
**When** any step fails
**Then** the entire operation is rolled back (atomic transaction)
**And** no partial state is created
**And** the failure is logged in the migration audit log

---

### Story UGM-2.3: Migration Metrics

As a **system operator**,
I want **to monitor migration success and failure rates**,
So that **I can detect issues and track feature adoption**.

**Acceptance Criteria:**

**Given** the Prometheus metrics endpoint exists
**When** a migration succeeds
**Then** the `migration_total{status="success"}` counter is incremented

**Given** a migration fails
**When** the failure is recorded
**Then** the `migration_total{status="failure"}` counter is incremented

**Given** a migration operation runs
**When** it completes (success or failure)
**Then** the `migration_duration_seconds` histogram records the duration

**Given** the `/metrics` endpoint is called
**When** migration metrics are requested
**Then** all migration counters and histograms are included in the response

---

### Story UGM-2.4: Migration History Query (Admin)

As an **administrator**,
I want **to query migration history**,
So that **I can troubleshoot user issues and analyze migration patterns**.

**Acceptance Criteria:**

**Given** an admin user with appropriate permissions
**When** calling `GET /api/admin/v1/migrations`
**Then** the response includes a paginated list of migration records
**And** each record includes: migration_id, user_id, registration_group_id, authenticated_group_id, devices_migrated, status, created_at

**Given** a query parameter `status=failed`
**When** the admin queries migrations
**Then** only failed migrations are returned

**Given** a query parameter `user_id=<uuid>`
**When** the admin queries migrations
**Then** only migrations for that user are returned

**Given** a non-admin user
**When** attempting to access migration history
**Then** the response is 403 Forbidden

---

## Epic UGM-3: Device-Group Management

Users can manage devices within their authenticated groups - adding, removing, and viewing devices with their locations.

### Story UGM-3.1: Device-Group Membership Table

As a **system**,
I want **to track which devices belong to which authenticated groups**,
So that **devices can belong to multiple groups simultaneously**.

**Acceptance Criteria:**

**Given** the system needs to support multi-group device membership
**When** a database migration is run
**Then** a new `device_group_memberships` table is created with columns:
- `id` (UUID, primary key)
- `device_id` (UUID, FK to devices, NOT NULL)
- `group_id` (UUID, FK to groups, NOT NULL)
- `added_by` (UUID, FK to users, NOT NULL)
- `added_at` (timestamp with timezone, NOT NULL, default NOW())

**And** a unique constraint exists on (`device_id`, `group_id`) to prevent duplicates
**And** indexes exist on `device_id` and `group_id` for efficient queries

**Given** the table exists
**When** querying devices for a group
**Then** the query completes in < 50ms for groups with up to 100 devices

---

### Story UGM-3.2: Add Device to Group

As a **group member**,
I want **to add my device to an authenticated group**,
So that **other group members can see my location**.

**Acceptance Criteria:**

**Given** an authenticated user who is a member of group "Chen Family"
**When** the user calls `POST /api/v1/groups/:groupId/devices` with their device_id
**Then** the device is added to the group
**And** the response includes `group_id`, `device_id`, `added_at`
**And** the operation completes in < 100ms

**Given** a user who is NOT a member of the group
**When** attempting to add a device
**Then** the response is 403 Forbidden with code `authz/not-group-member`

**Given** a user attempting to add a device they don't own
**When** the request is made
**Then** the response is 403 Forbidden with code `authz/not-device-owner`

**Given** a device that is already in the group
**When** attempting to add it again
**Then** the response is 409 Conflict with code `resource/already-exists`

**Given** an invalid group_id or device_id
**When** the request is made
**Then** the response is 404 Not Found with code `resource/not-found`

---

### Story UGM-3.3: Remove Device from Group

As a **device owner or group admin**,
I want **to remove a device from an authenticated group**,
So that **the device is no longer visible to group members**.

**Acceptance Criteria:**

**Given** a device owner who wants to remove their device from a group
**When** calling `DELETE /api/v1/groups/:groupId/devices/:deviceId`
**Then** the device is removed from the group
**And** the response is 204 No Content
**And** the operation completes in < 100ms

**Given** a group admin/owner
**When** removing ANY device from their group
**Then** the device is removed successfully
**And** the response is 204 No Content

**Given** a regular group member (not admin/owner)
**When** attempting to remove another user's device
**Then** the response is 403 Forbidden with code `authz/forbidden`

**Given** a device that is not in the specified group
**When** attempting to remove it
**Then** the response is 404 Not Found with code `resource/not-found`

---

### Story UGM-3.4: List Group Devices

As a **group member**,
I want **to see all devices in my group with their details and locations**,
So that **I can track my family members**.

**Acceptance Criteria:**

**Given** an authenticated user who is a member of a group
**When** calling `GET /api/v1/groups/:groupId/devices`
**Then** the response includes a paginated list of all devices in the group
**And** each device includes: `device_id`, `display_name`, `owner_user_id`, `owner_display_name`, `added_at`, `last_seen_at`
**And** the response completes in < 200ms

**Given** the query parameter `include_location=true`
**When** listing devices
**Then** each device includes `last_location` with latitude, longitude, accuracy, timestamp

**Given** pagination parameters `page=2&per_page=10`
**When** listing devices
**Then** the response includes the correct page of results
**And** pagination metadata shows total, page, per_page, total_pages

**Given** a user who is NOT a member of the group
**When** attempting to list devices
**Then** the response is 403 Forbidden with code `authz/not-group-member`

---

### Story UGM-3.5: View Device's Group Memberships

As a **device owner**,
I want **to see which groups my device belongs to**,
So that **I can manage my device's visibility across groups**.

**Acceptance Criteria:**

**Given** an authenticated user viewing their device details
**When** calling `GET /api/v1/devices/:deviceId/groups`
**Then** the response includes a list of all groups the device belongs to
**And** each group includes: `group_id`, `name`, `role` (user's role in group), `added_at`

**Given** a user viewing a device they don't own
**When** calling `GET /api/v1/devices/:deviceId/groups`
**Then** the response is 403 Forbidden

**Given** a device with no group memberships
**When** calling `GET /api/v1/devices/:deviceId/groups`
**Then** the response includes an empty list

---

### Story UGM-3.6: Enhanced Member List with Device Count

As a **group admin**,
I want **to see all members in my group with their device counts**,
So that **I can understand how many devices each family member has**.

**Acceptance Criteria:**

**Given** an authenticated user who is a member of a group
**When** calling `GET /api/v1/groups/:groupId/members`
**Then** the response includes all members with their details
**And** each member includes a `device_count` field showing how many devices they have in this group

**Given** a member with 2 devices in the group
**When** viewing the member list
**Then** that member's `device_count` is 2

**Given** a member with no devices in the group
**When** viewing the member list
**Then** that member's `device_count` is 0

---

## Epic UGM-4: Backwards Compatibility & Stability

Existing users continue to use the app without disruption - all current functionality remains unchanged.

### Story UGM-4.1: Verify Registration Group Endpoints Unchanged

As an **anonymous user (no account)**,
I want **to continue using registration groups as before**,
So that **I can still share location with family without creating an account**.

**Acceptance Criteria:**

**Given** an unauthenticated device with API key
**When** calling `POST /api/v1/devices/register` with a `group_id` string
**Then** the device is registered to the registration group as before
**And** the response format is unchanged from the existing API

**Given** an unauthenticated device with API key
**When** calling `GET /api/v1/devices?groupId=camping-2025`
**Then** all devices in that registration group are returned
**And** the response format is unchanged from the existing API

**Given** an unauthenticated device
**When** uploading locations via `POST /api/v1/locations`
**Then** the location is stored as before
**And** other devices in the same registration group can see the location

**Given** all existing integration tests for registration group functionality
**When** running the test suite after implementing unified group management
**Then** all existing tests pass without modification

---

### Story UGM-4.2: Verify Authenticated Group Endpoints Unchanged

As an **existing authenticated user**,
I want **all my current group functionality to work exactly as before**,
So that **I don't experience any disruption from the new features**.

**Acceptance Criteria:**

**Given** an authenticated user with existing groups
**When** calling existing group endpoints:
- `GET /api/v1/groups` (list my groups)
- `GET /api/v1/groups/:groupId` (get group details)
- `POST /api/v1/groups` (create new group)
- `PUT /api/v1/groups/:groupId` (update group)
- `DELETE /api/v1/groups/:groupId` (delete group)
**Then** all endpoints work exactly as documented in existing API spec
**And** response formats are unchanged

**Given** existing group invitation functionality
**When** creating and accepting invitations
**Then** the invitation flow works unchanged

**Given** all existing integration tests for authenticated group functionality
**When** running the test suite after implementing unified group management
**Then** all existing tests pass without modification

**Given** the new `device_group_memberships` table exists
**When** existing authenticated group queries are executed
**Then** performance is not degraded (< 10% increase in response time)

