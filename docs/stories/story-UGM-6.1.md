# Story UGM-6.1: Migration Rollback Capability

**Status**: Backlog (Post-MVP)

## Story

**As a** user who migrated a registration group,
**I want** to roll back the migration if issues are detected,
**So that** I can recover from problematic migrations without data loss.

**Epic**: UGM-6: Growth Features (Post-MVP)
**PRD Reference**: Growth Features - Migration rollback

## Acceptance Criteria

1. [ ] Given a completed migration, when calling `POST /api/v1/groups/:groupId/rollback`, then the authenticated group is deleted
2. [ ] Given a rollback request, when executed, then all devices are returned to their original registration group
3. [ ] Given a rollback request, when executed, then device_group_memberships are deleted
4. [ ] Given a rollback request, then the original registration_group_id is restored to devices
5. [ ] Given a migration older than 7 days, when rollback is attempted, then it is rejected (time limit)
6. [ ] Given a group with new members added post-migration, when rollback is attempted, then it fails (group modified)
7. [ ] Given a successful rollback, then it is logged in migration_audit_logs with status "rolled_back"

## Technical Notes

- Rollback window: 7 days from migration
- Rollback blocked if group has been modified (new members, settings changed)
- Requires preserving original registration_group_id in migration audit log

**Endpoint:**
```
POST /api/v1/groups/:groupId/rollback
Authorization: Bearer <jwt>

Response (200 OK):
{
  "rolled_back": true,
  "migration_id": "...",
  "registration_group_id": "camping-2025",
  "devices_restored": 3
}
```

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Rollback endpoint implemented
- [ ] Time limit enforced
- [ ] Modification detection implemented
- [ ] Audit logging for rollbacks
- [ ] Integration tests pass

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created for post-MVP backlog | Dev Agent |
