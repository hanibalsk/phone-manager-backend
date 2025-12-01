# Story 13.8: Bulk Device Import Endpoint

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: To Do
**Created**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to import multiple devices at once from a list
**So that** I can efficiently onboard large device fleets

## Prerequisites

- Story 13.7 complete (Fleet management)
- Story 13.3 complete (Device policies)

## Acceptance Criteria

1. POST `/api/admin/v1/organizations/{orgId}/devices/bulk` accepts array of device definitions
2. Maximum 200 devices per request
3. Each device can specify: external_id, display_name, group_id, policy_id, assigned_user_email, metadata
4. Option to update existing devices (matched by external_id or device_uuid)
5. Option to create missing users by email (sends invite)
6. Option to send welcome email to assigned users
7. Response includes counts: processed, created, updated, skipped
8. Response includes array of errors with row number and reason
9. Partial success allowed - some devices may fail while others succeed
10. Transaction per device (not all-or-nothing)

## Technical Notes

- Add external_id column to devices table (unique within org, nullable)
- Add metadata JSONB column to devices table
- Use streaming/batch processing for large imports
- Consider async processing for very large imports (future)

## API Specification

### POST /api/admin/v1/organizations/{orgId}/devices/bulk

Request:
```json
{
  "devices": [
    {
      "external_id": "ASSET-001",
      "display_name": "Field Tablet 1",
      "group_id": "grp_uuid",
      "policy_id": "pol_uuid",
      "assigned_user_email": "john@acme.com",
      "metadata": {
        "asset_tag": "ASSET-001",
        "purchase_date": "2025-01-15",
        "department": "Field Operations"
      }
    }
  ],
  "options": {
    "update_existing": true,
    "create_missing_users": false,
    "send_welcome_email": true
  }
}
```

Response (200):
```json
{
  "processed": 100,
  "created": 85,
  "updated": 10,
  "skipped": 2,
  "errors": [
    {
      "row": 52,
      "external_id": "ASSET-052",
      "error": "User not found: invalid@acme.com"
    },
    {
      "row": 98,
      "external_id": "ASSET-098",
      "error": "Group not found: grp_invalid"
    }
  ]
}
```

### Validation Rules

- external_id: optional, unique within organization
- display_name: required, 2-100 chars
- group_id: optional, must belong to organization
- policy_id: optional, must belong to organization
- assigned_user_email: optional, valid email format
- metadata: optional, max 10KB JSON

---

## Implementation Tasks

- [ ] Add external_id and metadata columns to devices table
- [ ] Create BulkImportService in domain layer
- [ ] Implement bulk device validation
- [ ] Implement per-device import with error handling
- [ ] Add user lookup/creation logic
- [ ] Add welcome email trigger (defer to notification service)
- [ ] Implement POST /api/admin/v1/organizations/{orgId}/devices/bulk
- [ ] Add audit logging for bulk operations
- [ ] Write unit tests for validation and import logic
- [ ] Write integration tests for bulk endpoint

---

## Dev Notes

- Bulk import creates devices in "pending" enrollment status
- Devices need to enroll to become "enrolled"
- Consider CSV import in future (separate endpoint)
- Rate limit: 1 bulk request per minute per organization

---

## Dev Agent Record

### Debug Log


### Completion Notes


---

## File List


---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

