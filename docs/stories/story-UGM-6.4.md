# Story UGM-6.4: Bulk Device Management

**Status**: Backlog (Post-MVP)

## Story

**As a** group admin with many devices,
**I want** to add or remove multiple devices from a group in a single operation,
**So that** I can efficiently manage large groups.

**Epic**: UGM-6: Growth Features (Post-MVP)
**PRD Reference**: Growth Features - Bulk device management

## Acceptance Criteria

### Bulk Add
1. [ ] Given a group admin, when calling `POST /api/v1/groups/:groupId/devices/bulk`, then multiple devices can be added
2. [ ] Given bulk add with 10 devices, when executed, then all 10 are added in a single transaction
3. [ ] Given bulk add with some invalid devices, then valid devices are added and errors reported per-device
4. [ ] Given bulk add exceeding group device limit, then operation fails with clear error

### Bulk Remove
5. [ ] Given a group admin, when calling `DELETE /api/v1/groups/:groupId/devices/bulk`, then multiple devices can be removed
6. [ ] Given bulk remove with 10 devices, when executed, then all 10 are removed in a single transaction
7. [ ] Given bulk remove with some devices not in group, then existing devices are removed and others reported as skipped

### General
8. [ ] Given bulk operations, then maximum batch size is 50 devices
9. [ ] Given bulk operation results, then response includes per-device status (success/failure/skipped)
10. [ ] Given bulk operations, then they are atomic (all succeed or all fail, unless partial mode enabled)

## Technical Notes

**Bulk Add Request:**
```json
POST /api/v1/groups/:groupId/devices/bulk
{
  "device_ids": ["uuid-1", "uuid-2", "uuid-3"],
  "partial": false  // If true, add what's possible and report failures
}
```

**Bulk Add Response:**
```json
{
  "added": 2,
  "failed": 1,
  "results": [
    { "device_id": "uuid-1", "status": "added" },
    { "device_id": "uuid-2", "status": "added" },
    { "device_id": "uuid-3", "status": "failed", "error": "not_owner" }
  ]
}
```

**Bulk Remove Request:**
```json
DELETE /api/v1/groups/:groupId/devices/bulk
{
  "device_ids": ["uuid-1", "uuid-2"]
}
```

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Bulk add endpoint implemented
- [ ] Bulk remove endpoint implemented
- [ ] Transaction handling correct
- [ ] Partial mode works
- [ ] Per-device status reported
- [ ] Integration tests pass

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created for post-MVP backlog | Dev Agent |
