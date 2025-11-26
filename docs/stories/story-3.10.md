# Story 3.10: Location Upload Error Handling

**Status**: Not Started

## Story

**As a** mobile app
**I want** clear error messages for failed uploads
**So that** I can retry appropriately or alert the user

**Prerequisites**: Story 3.1, Story 1.7

## Acceptance Criteria

1. [ ] Device not found: 404 with `{"error": "not_found", "message": "Device not found. Please register first."}`
2. [ ] Validation errors: 400 with field-level details
3. [ ] Database timeout: 503 with `{"error": "service_unavailable", "message": "Database temporarily unavailable"}`
4. [ ] Large payload (>1MB): 413 with `{"error": "payload_too_large", "message": "Request exceeds maximum size"}`
5. [ ] Rate limit: 429 with `Retry-After` header
6. [ ] All errors logged with request_id for tracing

## Technical Notes

- Map SQLx errors to appropriate HTTP status
- Use custom error types from Story 1.7
- Include helpful messages without exposing internals

## Tasks/Subtasks

- [ ] 1. Enhance error handling
  - [ ] 1.1 Add PayloadTooLarge error variant
  - [ ] 1.2 Map database errors appropriately
  - [ ] 1.3 Ensure request_id in all error logs
- [ ] 2. Add payload size limit
  - [ ] 2.1 Configure 1MB limit on location endpoints
  - [ ] 2.2 Return 413 for oversized payloads
- [ ] 3. Write tests
  - [ ] 3.1 Test all error scenarios
  - [ ] 3.2 Test error message format
- [ ] 4. Run linting and formatting checks

## Dev Notes

- Error framework from Story 1.7 provides base types
- Request ID from middleware for tracing
- Payload limit configurable

## Dev Agent Record

### Debug Log

(Implementation notes will be added here)

### Completion Notes

(To be filled upon completion)

## File List

### Modified Files

(To be filled)

### New Files

(To be filled)

### Deleted Files

(None expected)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Code formatted with rustfmt
- [ ] Story file updated with completion notes
