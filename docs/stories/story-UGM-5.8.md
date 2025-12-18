# Story UGM-5.8: Multi-Group Membership Edge Cases

**Status**: Ready for Development

## Story

**As a** user with devices in multiple groups,
**I want** edge cases to be handled correctly,
**So that** my device memberships remain consistent and queryable.

**Epic**: UGM-5: NFR Compliance
**Prerequisites**: Epic UGM-3 complete

## Acceptance Criteria

### Pagination with Multi-Group Devices
1. [ ] Given a device belonging to 5 groups, when calling `GET /api/v1/devices/:deviceId/groups`, then all 5 groups are returned correctly
2. [ ] Given 100 devices each in 3 groups, when listing group devices with pagination, then pagination counts are correct
3. [ ] Given a group with 50 devices, when using `page=2&per_page=20`, then correct devices are returned with accurate `total` count
4. [ ] Given a device in multiple groups, when listing devices for one group, then device appears only once (no duplicates)

### Group Deletion Cascade
5. [ ] Given a group with 10 device memberships, when the group is deleted, then all `device_group_memberships` records are deleted (CASCADE)
6. [ ] Given a device in groups A and B, when group A is deleted, then device remains in group B
7. [ ] Given a device only in group A, when group A is deleted, then device still exists but has no group memberships
8. [ ] Given group deletion, then no orphaned `device_group_memberships` records exist (verified by FK constraint)

### Orphaned Membership Prevention
9. [ ] Given a device being deleted, when it has group memberships, then all memberships are cleaned up
10. [ ] Given the database schema, then `device_group_memberships.device_id` has FK to `devices` with ON DELETE CASCADE
11. [ ] Given the database schema, then `device_group_memberships.group_id` has FK to `groups` with ON DELETE CASCADE
12. [ ] Given a membership record, when its device no longer exists, then the record cannot exist (enforced by DB)

### Constraint Validation
13. [ ] Given an attempt to add same device to same group twice, then 409 Conflict is returned
14. [ ] Given the unique constraint on (device_id, group_id), then database prevents duplicate memberships
15. [ ] Given a device not owned by the user, when attempting to add to group, then 403 Forbidden is returned

## Technical Notes

- Current schema has proper CASCADE constraints - verify they work correctly
- Unique constraint: `UNIQUE (device_id, group_id)` on `device_group_memberships`
- Foreign keys should cascade on delete for both device and group

**Schema Verification:**
```sql
-- Verify constraints exist
SELECT conname, contype, confdeltype
FROM pg_constraint
WHERE conrelid = 'device_group_memberships'::regclass;

-- Expected:
-- device_group_memberships_device_id_fkey | f | c (CASCADE)
-- device_group_memberships_group_id_fkey | f | c (CASCADE)
-- uq_device_group_membership | u | NULL (UNIQUE)
```

## Tasks/Subtasks

- [ ] 1. Verify CASCADE constraints exist in migration
- [ ] 2. Add integration test: pagination with multi-group devices
- [ ] 3. Add integration test: group deletion cascades memberships
- [ ] 4. Add integration test: device deletion cascades memberships
- [ ] 5. Add integration test: no duplicate memberships
- [ ] 6. Add database constraint verification test
- [ ] 7. Add edge case: device in 10+ groups

## File List

### Files to Modify

- `crates/persistence/src/migrations/057_device_group_memberships.sql` - Verify CASCADE constraints

### Files to Create

- `crates/api/tests/multi_group_edge_cases_test.rs` - Edge case integration tests

## Test Scenarios

```rust
#[tokio::test]
async fn test_group_deletion_cascades_memberships() {
    // Create group with 10 device memberships
    // Delete group
    // Assert: no device_group_memberships for that group_id exist
}

#[tokio::test]
async fn test_device_in_multiple_groups_pagination() {
    // Create device in 5 groups
    // List device's groups
    // Assert: all 5 returned, pagination metadata correct
}

#[tokio::test]
async fn test_no_orphaned_memberships_after_device_delete() {
    // Create device in 3 groups
    // Delete device
    // Assert: no device_group_memberships for that device_id exist
}
```

## Definition of Done

- [ ] All acceptance criteria met
- [ ] CASCADE constraints verified
- [ ] Edge case integration tests pass
- [ ] No orphaned memberships possible
- [ ] Code compiles without warnings
- [ ] Code passes clippy

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created from gap analysis | Dev Agent |
