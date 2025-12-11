# Story AP-3.6: Reactivate User

**Status**: Completed

## Story

**As an** organization admin
**I want to** reactivate suspended users
**So that** they can resume platform access

**Epic**: AP-3: User Administration
**Priority**: High

## Acceptance Criteria

1. [x] POST `/api/admin/v1/organizations/:org_id/users/:user_id/reactivate` reactivates
2. [x] User can login after reactivation
3. [x] Previous assignments restored (preserved during suspension)
4. [x] Audit log entry created (via tracing)

## Technical Notes

- Clears suspended_at, suspended_by, suspension_reason fields from org_users table
- Idempotent operation (success if already active)
- Previous permissions and role preserved during suspension

## Tasks/Subtasks

- [x] 1. Create reactivate endpoint handler
- [x] 2. Update OrgUser repository with reactivate method
- [x] 3. Add audit logging via tracing
- [x] 4. Write unit tests

## Dev Notes

- Depends on AP-3.5 for suspension fields
- User's role and permissions are preserved during suspension
- Admins cannot reactivate other admins or owners (matching suspend permissions)

## Dev Agent Record

### Debug Log

- Added reactivate repository method to OrgUserRepository
- Created reactivate_user route handler
- Added ReactivateOrgUserResponse DTO
- Implemented idempotent behavior (returns success if user already active)

### Completion Notes

Implementation complete. Reactivation clears all suspension fields and logs the action.

## File List

### Modified Files

- `crates/domain/src/models/org_user.rs` - Added ReactivateOrgUserResponse DTO
- `crates/domain/src/models/mod.rs` - Exported ReactivateOrgUserResponse
- `crates/persistence/src/repositories/org_user.rs` - Added reactivate method
- `crates/api/src/routes/admin_users.rs` - Added reactivate endpoint

### New Files

(None - shared migration with AP-3.5)

### Deleted Files

(None)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-11 | Story created from AP-3.6 epic | Dev Agent |
| 2025-12-11 | Implementation completed | Dev Agent |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes
