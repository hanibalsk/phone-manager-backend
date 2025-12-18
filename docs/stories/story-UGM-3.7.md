# Story UGM-3.7: Device Assignment Indicator in Group Listing

**Status**: Complete ✅

## Story

**As a** user viewing their groups,
**I want** to see which groups contain my current device,
**So that** I can quickly identify device-group relationships.

**Epic**: UGM-3: Multi-Group Device Management
**PRD Reference**: AC 3 - Device assignment indication

## Acceptance Criteria

### API Enhancement
1. [x] Given `GET /api/v1/groups/mine` is called, then each group in response includes `has_current_device` boolean field
2. [x] Given a group containing the requesting user's device, then `has_current_device` is `true`
3. [x] Given a group NOT containing the requesting user's device, then `has_current_device` is `false`
4. [x] Given user has no devices, then `has_current_device` is `false` for all groups

### Query Parameters
5. [x] Given `device_id` query parameter is provided, then check assignment for that specific device
6. [x] Given no `device_id` parameter, then check if ANY user device is in each group

## Technical Notes

**Updated Response Format:**
```json
{
  "groups": [
    {
      "id": "uuid",
      "name": "Family",
      "slug": "family-abc123",
      "icon_emoji": "👨‍👩‍👧‍👦",
      "member_count": 4,
      "device_count": 6,
      "your_role": "admin",
      "joined_at": "2025-01-15T10:30:00Z",
      "has_current_device": true
    }
  ]
}
```

**Implementation Approach:**
- Extend `find_user_groups` repository query with LEFT JOIN on `device_group_memberships`
- Add optional `device_id` parameter to filter specific device
- If no `device_id`, check if ANY device owned by user is in each group

**SQL Enhancement (actual implementation):**
```sql
EXISTS (
    SELECT 1 FROM device_group_memberships dgm
    JOIN devices d ON dgm.device_id = d.device_id
    WHERE dgm.group_id = g.id
    AND d.owner_user_id = $1
    AND d.active = true
    AND (dgm.device_id = $2 OR $2 IS NULL)
) as has_current_device
```

## Definition of Done

- [x] All acceptance criteria met
- [x] `has_current_device` field added to GroupSummary
- [x] Repository query updated with device check
- [x] Handler accepts optional device_id parameter
- [x] Integration tests pass
- [x] No breaking changes to existing API consumers

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created based on frontend requirements | Dev Agent |
| 2025-12-18 | Implementation complete - all tests passing | Dev Agent |
