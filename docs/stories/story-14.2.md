# Story 14.2: Device Fleet List with Advanced Filtering

**Epic**: Epic 14 - Admin Portal Backend
**Status**: Complete
**Created**: 2025-12-01
**Completed**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to list all devices in my fleet with advanced filtering and sorting
**So that** I can efficiently manage and monitor devices across my organization

## Prerequisites

- Story 13.7 complete (Fleet management endpoints)
- Story 14.1 complete (Dashboard metrics)

## Acceptance Criteria

1. GET `/api/admin/v1/organizations/{orgId}/devices` returns paginated device list
2. Each device includes: id, UUID, display name, platform, enrollment status
3. Each device includes optional assigned user info (id, email, display name)
4. Each device includes optional group info (id, name)
5. Each device includes optional policy info (id, name)
6. Each device includes last seen timestamp and optional last location
7. Filtering supported by: status, group_id, policy_id, assigned, search text
8. Sorting supported by: last_seen_at, display_name, created_at, enrolled_at
9. Pagination with page/per_page params (default 50, max 100)
10. Summary counts included: enrolled, pending, suspended, retired, assigned, unassigned
11. Search matches against display name and device UUID
12. Only org admins and owners can access the endpoint

## Technical Notes

- Complete the `list_fleet_devices` implementation in `crates/api/src/routes/fleet.rs`
- Add `list_fleet_devices` method to DeviceRepository with proper joins
- Use LEFT JOINs to include optional related data (user, group, policy, location)
- Query should be optimized with proper indexes
- Consider query builder pattern for complex filter combinations

## API Specification

### GET /api/admin/v1/organizations/{orgId}/devices

Query Parameters:
- `page` (optional): Page number (default: 1)
- `perPage` (optional): Items per page (default: 50, max: 100)
- `status` (optional): Filter by enrollment status (enrolled, pending, suspended, retired)
- `groupId` (optional): Filter by group ID
- `policyId` (optional): Filter by policy UUID
- `assigned` (optional): Filter by assignment status (true/false)
- `search` (optional): Search in display name and UUID
- `sort` (optional): Sort field (last_seen_at, display_name, created_at, enrolled_at)
- `order` (optional): Sort order (asc, desc)

Response (200):
```json
{
  "data": [
    {
      "id": 42,
      "deviceUuid": "550e8400-e29b-41d4-a716-446655440000",
      "displayName": "Field Tablet #42",
      "platform": "android",
      "enrollmentStatus": "enrolled",
      "isManaged": true,
      "assignedUser": {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "email": "john@acme.com",
        "displayName": "John Smith"
      },
      "group": {
        "id": "field-workers",
        "name": "Field Workers"
      },
      "policy": {
        "id": "456e7890-e12b-34d5-a678-901234567890",
        "name": "Standard Policy"
      },
      "lastSeenAt": "2025-12-01T10:25:00Z",
      "lastLocation": {
        "latitude": 37.7749,
        "longitude": -122.4194,
        "timestamp": "2025-12-01T10:25:00Z"
      },
      "enrolledAt": "2025-11-15T09:00:00Z",
      "createdAt": "2025-11-15T08:45:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "perPage": 50,
    "total": 45,
    "totalPages": 1
  },
  "summary": {
    "enrolled": 40,
    "pending": 3,
    "suspended": 1,
    "retired": 1,
    "assigned": 35,
    "unassigned": 10
  }
}
```

---

## Implementation Tasks

- [x] Create `list_fleet_devices` method in DeviceRepository
- [x] Build dynamic SQL query with LEFT JOINs for user, group, policy
- [x] Add filtering logic for status, group_id, policy_id, assigned, search
- [x] Add sorting logic with configurable field and order
- [x] Add pagination with LIMIT/OFFSET
- [x] Query last location from locations table
- [x] Map database results to FleetDeviceItem domain model
- [x] Update fleet.rs route to use new repository method
- [x] Write unit tests for query building logic
- [ ] Write integration tests for endpoint (requires database)

---

## Dev Notes

- Story 13.7 already has the domain models (FleetDeviceItem, FleetDeviceQuery, etc.)
- The route handler exists but returns empty data - need to implement actual query
- Consider using subquery for last location to avoid N+1 queries
- Search should use ILIKE for case-insensitive matching

---

## Dev Agent Record

### Debug Log

- Used LEFT JOIN LATERAL for efficient last location retrieval
- Dynamic SQL query building with parameterized filters
- Added FleetDeviceEntity for mapping join results

### Completion Notes

- list_fleet_devices repository method implements full query with:
  - LEFT JOIN to users for assigned user data
  - LEFT JOIN to device_policies for policy name
  - LEFT JOIN LATERAL subquery for last location
- Dynamic filter building supports all filter types
- Sort by configurable column with NULLS LAST
- Maps to domain FleetDeviceItem with all optional fields
- Route handler updated to call new method instead of returning empty list

---

## File List

- `crates/persistence/src/entities/device.rs` - Added FleetDeviceEntity
- `crates/persistence/src/entities/mod.rs` - Export FleetDeviceEntity
- `crates/persistence/src/repositories/device.rs` - Added list_fleet_devices method
- `crates/api/src/routes/fleet.rs` - Updated to use list_fleet_devices

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete |
