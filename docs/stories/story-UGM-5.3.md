# Story UGM-5.3: Migration Progress Indication for Long Operations

**Status**: Ready for Development

## Story

**As a** user migrating a large registration group,
**I want** progress indication when migration takes longer than expected,
**So that** I know the operation is still in progress and approximately how long to wait.

**Epic**: UGM-5: NFR Compliance
**Prerequisites**: Story UGM-2.2 (Migration Endpoint)
**NFRs Covered**: NFR23

## Acceptance Criteria

1. [ ] Given a migration request, when processing exceeds 2 seconds, then the response is 202 Accepted with progress information
2. [ ] Given a 202 Accepted response, then it includes `migration_id` for status polling
3. [ ] Given a 202 Accepted response, then it includes `status_url` endpoint to check progress
4. [ ] Given a 202 Accepted response, then it includes `estimated_completion_seconds` based on device count
5. [ ] Given the status polling endpoint, when migration is in progress, then response includes `progress_percent` (0-100)
6. [ ] Given the status polling endpoint, when migration completes, then response includes final result with `status: completed`
7. [ ] Given the status polling endpoint, when migration fails, then response includes `status: failed` with error details
8. [ ] Given a small migration (<10 devices), when it completes within 2 seconds, then return 201 Created directly (no polling needed)

## Technical Notes

- Threshold for async processing: 2 seconds OR >50 devices (whichever triggers first)
- Progress estimation: ~100ms per device
- Status polling endpoint: `GET /api/v1/groups/migrations/:migration_id/status`
- Response formats:

**202 Accepted (long-running):**
```json
{
  "migration_id": "660e8400-e29b-41d4-a716-446655440001",
  "status": "in_progress",
  "status_url": "/api/v1/groups/migrations/660e8400.../status",
  "estimated_completion_seconds": 5,
  "devices_total": 50,
  "devices_processed": 0
}
```

**Status polling response:**
```json
{
  "migration_id": "660e8400-e29b-41d4-a716-446655440001",
  "status": "in_progress",
  "progress_percent": 60,
  "devices_total": 50,
  "devices_processed": 30,
  "estimated_remaining_seconds": 2
}
```

**Completion response:**
```json
{
  "migration_id": "660e8400-e29b-41d4-a716-446655440001",
  "status": "completed",
  "authenticated_group_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Chen Family",
  "devices_migrated": 50,
  "completed_at": "2025-12-18T10:30:05Z"
}
```

## Tasks/Subtasks

- [ ] 1. Add `migration_status` column to `migration_audit_logs` table
- [ ] 2. Add `devices_processed` tracking column
- [ ] 3. Create status polling endpoint handler
- [ ] 4. Modify migration endpoint to return 202 for large/slow migrations
- [ ] 5. Implement background processing for large migrations
- [ ] 6. Add progress calculation logic
- [ ] 7. Add integration tests for progress scenarios
- [ ] 8. Update OpenAPI spec with 202 response and status endpoint

## File List

### Files to Create

- `crates/api/src/routes/migration_status.rs` - Status polling endpoint

### Files to Modify

- `crates/persistence/src/migrations/` - Add migration status columns
- `crates/api/src/routes/groups.rs` - Update migration endpoint for 202 response
- `crates/domain/src/models/migration.rs` - Add progress-related types
- `crates/api/src/app.rs` - Register status endpoint
- `docs/api/openapi.yaml` - Document 202 response and status endpoint

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Small migrations complete synchronously (201)
- [ ] Large migrations return 202 with polling
- [ ] Status endpoint shows accurate progress
- [ ] Integration tests pass
- [ ] OpenAPI spec updated
- [ ] Code compiles without warnings
- [ ] Code passes clippy

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created from gap analysis | Dev Agent |
