# Story 2.3: Group Membership Validation

**Status**: Ready for Review

## Story

**As a** backend system
**I want** to validate group membership on device operations
**So that** business rules are enforced consistently

**Prerequisites**: Story 2.1 ✅

## Acceptance Criteria

1. [x] Before device registration/update, count active devices in target group
2. [x] Reject registration if group has 20 active devices (active=true)
3. [x] Allow registration if group has <20 devices or device is updating within same group
4. [x] Group count query executes in <50ms
5. [x] Validation errors return 409 Conflict with message: "Group has reached maximum device limit (20)"
6. [x] Inactive devices (active=false) don't count toward limit

## Technical Notes

- Use efficient `COUNT(*)` query with `WHERE active=true AND group_id=?`
- Consider caching group counts in Redis for high-traffic scenarios (future optimization)

## Tasks/Subtasks

- [x] 1. Implement group count repository method
  - [x] 1.1 Add `count_active_devices_in_group` to device repository
  - [x] 1.2 Ensure query is efficient with proper index usage
- [x] 2. Add validation to registration flow
  - [x] 2.1 Check group capacity before insert
  - [x] 2.2 Allow same-group updates without capacity check
  - [x] 2.3 Return 409 Conflict for full groups
- [x] 3. Write tests
  - [x] 3.1 Test group at capacity rejection
  - [x] 3.2 Test same-group update allowed
  - [x] 3.3 Test inactive devices not counted
- [x] 4. Run linting and formatting checks

## Dev Notes

- Group limit configurable via PM__LIMITS__MAX_DEVICES_PER_GROUP (default 20)
- Index on (group_id, active) should make count query efficient

## Dev Agent Record

### Debug Log

- Implemented `count_active_devices_in_group` with COUNT(*) and active=true filter
- Handler checks capacity for new devices and group changes
- Same-group updates bypass capacity check via is_changing_group flag

### Completion Notes

Group membership validation fully integrated into registration flow. Capacity is checked when device is new or changing groups.

## File List

### Modified Files

- `crates/persistence/src/repositories/device.rs` - count_active_devices_in_group method
- `crates/api/src/routes/devices.rs` - Capacity validation logic

### New Files

(None)

### Deleted Files

(None)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |
| 2025-11-26 | Implementation complete | Dev Agent |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes
