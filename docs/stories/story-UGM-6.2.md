# Story UGM-6.2: Partial Migration (Selective Device Migration)

**Status**: Backlog (Post-MVP)

## Story

**As a** user with multiple devices in a registration group,
**I want** to select which devices to migrate,
**So that** I can leave some devices in the anonymous registration group.

**Epic**: UGM-6: Growth Features (Post-MVP)
**PRD Reference**: Growth Features - Partial migration

## Acceptance Criteria

1. [ ] Given a migration request with `device_ids` array, when executed, then only specified devices are migrated
2. [ ] Given partial migration, then non-selected devices remain in the registration group
3. [ ] Given partial migration, then selected devices are added to the new authenticated group
4. [ ] Given empty `device_ids` array, then migration fails with validation error
5. [ ] Given `device_ids` not in the registration group, then those are ignored with warning
6. [ ] Given partial migration audit log, then it records which devices were included/excluded
7. [ ] Given a registration group after partial migration, when another migration is attempted, then remaining devices can be migrated

## Technical Notes

**Request Format:**
```json
POST /api/v1/groups/migrate
{
  "registration_group_id": "camping-2025",
  "group_name": "Chen Family",
  "device_ids": ["uuid-1", "uuid-2"]  // Optional - if omitted, migrate all
}
```

**Response with partial migration:**
```json
{
  "authenticated_group_id": "...",
  "name": "Chen Family",
  "devices_migrated": 2,
  "devices_remaining_in_registration": 1,
  "migration_id": "..."
}
```

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Partial migration endpoint working
- [ ] Remaining devices queryable in registration group
- [ ] Audit logging includes device selection
- [ ] Integration tests pass

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created for post-MVP backlog | Dev Agent |
