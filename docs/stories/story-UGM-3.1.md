# Story UGM-3.1: Device-Group Membership Table

**Status**: Complete ✅

## Story

**As a** system,
**I want** to track which devices belong to which authenticated groups,
**So that** devices can belong to multiple groups simultaneously.

**Epic**: UGM-3: Device-Group Management
**Prerequisites**: None

## Acceptance Criteria

1. [x] Given the system needs to support multi-group device membership, when a database migration is run, then a new `device_group_memberships` table is created with columns:
   - `id` (UUID, primary key)
   - `device_id` (UUID, NOT NULL)
   - `group_id` (UUID, FK to groups, NOT NULL)
   - `added_by` (UUID, FK to users, NOT NULL)
   - `added_at` (timestamp with timezone, NOT NULL, default NOW())
2. [x] A unique constraint exists on (`device_id`, `group_id`) to prevent duplicates
3. [x] Indexes exist on `device_id` and `group_id` for efficient queries
4. [x] Given the table exists, when querying devices for a group, then the query completes efficiently (< 50ms for groups with up to 100 devices)

## Technical Notes

- Migration file: `057_device_group_memberships.sql`
- Repository: `DeviceGroupMembershipRepository`
- Indexes for efficient queries:
  - `idx_device_group_memberships_group_id` - for finding all devices in a group
  - `idx_device_group_memberships_device_id` - for finding all groups a device belongs to
  - `idx_device_group_memberships_added_by` - for audit queries
  - `idx_device_group_memberships_group_added_at` - for ordering within a group

## Tasks/Subtasks

- [x] 1. Create migration file `057_device_group_memberships.sql`
- [x] 2. Create entity `DeviceGroupMembershipEntity`
- [x] 3. Create entities for query results (`DeviceInGroupEntity`, `DeviceInGroupWithLocationEntity`, `DeviceGroupInfoEntity`)
- [x] 4. Create `DeviceGroupMembershipRepository` with CRUD operations
- [x] 5. Export entity and repository from mod.rs files

## File List

### Files Created

- `crates/persistence/src/migrations/057_device_group_memberships.sql` - Migration file
- `crates/persistence/src/entities/device_group_membership.rs` - Entity definitions
- `crates/persistence/src/repositories/device_group_membership.rs` - Repository implementation
- `docs/stories/story-UGM-3.1.md` - This story file

### Files Modified

- `crates/persistence/src/entities/mod.rs` - Export new entity module
- `crates/persistence/src/repositories/mod.rs` - Export new repository module

## Implementation Details

### Database Schema

```sql
CREATE TABLE device_group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    added_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_device_group_membership UNIQUE (device_id, group_id)
);
```

### Repository Methods

| Method | Description |
|--------|-------------|
| `add_device_to_group` | Add a device to a group |
| `remove_device_from_group` | Remove a device from a group |
| `is_device_in_group` | Check if device is in group |
| `get_membership` | Get membership record |
| `list_devices_in_group` | List all devices in a group |
| `list_devices_in_group_with_location` | List devices with last location |
| `count_devices_in_group` | Count devices in a group |
| `list_device_groups` | List all groups a device belongs to |
| `count_device_groups` | Count device's group memberships |
| `count_devices_per_user_in_group` | Count devices per user (for member list) |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass (unit tests in workspace)
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Code passes clippy
- [x] Story file updated with completion notes

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created and implemented | Dev Agent |
