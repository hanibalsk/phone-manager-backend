# Story 13.10: Audit Query and Export Endpoints

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: To Do
**Created**: 2025-12-01

---

## User Story

**As a** security administrator
**I want** to query and export audit logs
**So that** I can investigate incidents and generate compliance reports

## Prerequisites

- Story 13.9 complete (Audit logging system)

## Acceptance Criteria

1. GET `/api/admin/v1/organizations/{orgId}/audit-logs` returns paginated audit entries
2. Supports filtering by: actor_id, action, resource_type, resource_id, date range (from/to)
3. GET `/api/admin/v1/organizations/{orgId}/audit-logs/export` exports logs as CSV or JSON
4. Export supports same filters as list endpoint
5. Export limited to 10,000 records per request
6. Large exports (>1000 records) return async job ID
7. Export file downloadable via GET `/api/admin/v1/organizations/{orgId}/audit-logs/export/{jobId}`
8. Export jobs expire after 24 hours
9. Response includes pagination metadata

## Technical Notes

- Use keyset pagination for efficient querying of large logs
- Export jobs stored in Redis or database with TTL
- Generate CSV using streaming for memory efficiency
- Consider background job for large exports

## API Specification

### GET /api/admin/v1/organizations/{orgId}/audit-logs

Query Parameters:
- page: page number (default 1)
- per_page: items per page (default 50, max 100)
- actor_id: filter by actor UUID
- action: filter by action (e.g., "device.assign")
- resource_type: filter by resource type (device, user, policy, etc.)
- resource_id: filter by specific resource UUID
- from: start date (ISO 8601)
- to: end date (ISO 8601)

Response (200):
```json
{
  "data": [
    {
      "id": "uuid",
      "timestamp": "2025-12-01T10:30:00Z",
      "actor": {
        "id": "user_uuid",
        "type": "user",
        "email": "admin@acme.com"
      },
      "action": "device.assign",
      "resource": {
        "type": "device",
        "id": "dev_uuid",
        "name": "Field Tablet #42"
      },
      "changes": {
        "assigned_user_id": {
          "old": null,
          "new": "user_uuid"
        }
      },
      "metadata": {
        "ip_address": "192.168.1.100",
        "user_agent": "Mozilla/5.0...",
        "request_id": "req_abc123"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 1250,
    "total_pages": 25
  }
}
```

### GET /api/admin/v1/organizations/{orgId}/audit-logs/export

Query Parameters:
- format: csv or json (default json)
- Same filters as list endpoint

Response (200) for small exports:
```json
{
  "format": "csv",
  "record_count": 500,
  "download_url": "data:text/csv;base64,..."
}
```

Response (202) for large exports:
```json
{
  "job_id": "export_abc123",
  "status": "processing",
  "estimated_records": 5000,
  "check_url": "/api/admin/v1/organizations/{orgId}/audit-logs/export/export_abc123"
}
```

### GET /api/admin/v1/organizations/{orgId}/audit-logs/export/{jobId}

Response (200) when complete:
```json
{
  "job_id": "export_abc123",
  "status": "completed",
  "record_count": 5000,
  "download_url": "https://storage.example.com/exports/export_abc123.csv",
  "expires_at": "2025-12-02T10:30:00Z"
}
```

---

## Implementation Tasks

- [ ] Create AuditLogQueryService in domain layer
- [ ] Implement paginated list endpoint with filters
- [ ] Create export job tracking (database or Redis)
- [ ] Implement sync export for small datasets
- [ ] Implement async export with job tracking
- [ ] Create CSV generation utility
- [ ] Create JSON streaming export
- [ ] Implement export download endpoint
- [ ] Add export job cleanup background task
- [ ] Write unit tests for query filters
- [ ] Write integration tests for export flow

---

## Dev Notes

- Audit logs are read-heavy for queries, ensure indexes are effective
- Consider full-text search on action/resource_name for future
- Export files should be securely stored (signed URLs)
- Rate limit: 10 exports per hour per organization

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

