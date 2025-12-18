# Story UGM-2.4: Migration History Query (Admin)

**Status**: Complete ✅

## Story

**As an** administrator,
**I want** to query migration history,
**So that** I can troubleshoot user issues and analyze migration patterns.

**Epic**: UGM-2: Group Migration
**Prerequisites**: Story UGM-2.1: Create Migration Audit Log Infrastructure

## Acceptance Criteria

1. [x] Given an admin user with appropriate permissions, when calling `GET /api/admin/v1/migrations`, then the response includes a paginated list of migration records
2. [x] Each record includes: migration_id, user_id, user_email, registration_group_id, authenticated_group_id, group_name, devices_migrated, status, created_at
3. [x] Given a query parameter `status=failed`, when the admin queries migrations, then only failed migrations are returned
4. [x] Given a query parameter `user_id=<uuid>`, when the admin queries migrations, then only migrations for that user are returned
5. [x] Given pagination parameters `page=2&per_page=10`, when the admin queries migrations, then the response includes the correct page of results
6. [x] Pagination metadata shows total, page, per_page, total_pages

## Technical Notes

- Endpoint: `GET /api/admin/v1/migrations`
- Requires admin API key authentication (ApiKeyAuth extractor)
- Query parameters: `user_id`, `status`, `registration_group_id`, `page`, `per_page`
- Status values: `success`, `failed`, `partial`
- Default pagination: page=1, per_page=20
- Max per_page: 100

## Tasks/Subtasks

- [x] 1. Create admin_migrations.rs route module
- [x] 2. Add ListMigrationsQuery struct for query parameters
- [x] 3. Add MigrationRecord response struct
- [x] 4. Implement list_migrations handler
- [x] 5. Register module in routes/mod.rs
- [x] 6. Add route in app.rs

## File List

### Files Created

- `crates/api/src/routes/admin_migrations.rs` - Admin migrations route handler
- `docs/stories/story-UGM-2.4.md` - This story file

### Files Modified

- `crates/api/src/routes/mod.rs` - Export admin_migrations module
- `crates/api/src/app.rs` - Register admin migrations route

## Implementation Details

### Request Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `user_id` | UUID | - | Filter by user ID |
| `status` | string | - | Filter by status (success/failed/partial) |
| `registration_group_id` | string | - | Filter by registration group ID |
| `page` | i64 | 1 | Page number (1-based) |
| `per_page` | i64 | 20 | Items per page (1-100) |

### Response Format

```json
{
  "data": [
    {
      "migration_id": "uuid",
      "user_id": "uuid",
      "user_email": "user@example.com",
      "registration_group_id": "camping-2025",
      "authenticated_group_id": "uuid",
      "group_name": "Camping Trip",
      "devices_migrated": 3,
      "device_ids": ["uuid1", "uuid2", "uuid3"],
      "status": "success",
      "error_message": null,
      "created_at": "2025-12-18T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

### Authentication

Requires admin API key via `X-API-Key` header.

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
