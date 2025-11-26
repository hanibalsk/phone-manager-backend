# Story 3.10: Location Upload Error Handling

**Status**: Complete ✅

## Story

**As a** mobile app
**I want** clear error messages for failed uploads
**So that** I can retry appropriately or alert the user

**Prerequisites**: Story 3.1 ✅, Story 1.7 ✅

## Acceptance Criteria

1. [x] Device not found: 404 with `{"error": "not_found", "message": "Device not found. Please register first."}`
2. [x] Validation errors: 400 with field-level details
3. [x] Database timeout: 503 with `{"error": "service_unavailable", "message": "Database temporarily unavailable"}`
4. [x] Large payload (>1MB): 413 with `{"error": "payload_too_large", "message": "Request exceeds maximum size"}`
5. [x] Rate limit: 429 with `Retry-After` header
6. [x] All errors logged with request_id for tracing

## Technical Notes

- Map SQLx errors to appropriate HTTP status
- Use custom error types from Story 1.7
- Include helpful messages without exposing internals

## Tasks/Subtasks

- [x] 1. Enhance error handling
  - [x] 1.1 Add PayloadTooLarge error variant
  - [x] 1.2 Map database errors appropriately
  - [x] 1.3 Ensure request_id in all error logs
- [x] 2. Add payload size limit
  - [x] 2.1 Configure 1MB limit on location endpoints
  - [x] 2.2 Return 413 for oversized payloads
- [x] 3. Write tests
  - [x] 3.1 Test all error scenarios
  - [x] 3.2 Test error message format
- [x] 4. Run linting and formatting checks

## Dev Notes

- Error framework from Story 1.7 provides base types
- Request ID from middleware for tracing
- Payload limit configurable

## Dev Agent Record

### Debug Log

- All error variants properly mapped
- Request ID included in all error logs
- Payload limit enforced via tower layer
- Rate limiting returns Retry-After header

### Completion Notes

Location upload error handling comprehensive with clear error messages and proper HTTP status codes.

## File List

### Modified Files

- `crates/api/src/error.rs` - PayloadTooLarge variant
- `crates/api/src/routes/locations.rs` - error handling
- `crates/api/src/app.rs` - payload limit layer

### New Files

(None)

### Deleted Files

(None)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |
| 2025-11-26 | Implementation complete | Dev Agent |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes

---

## Senior Developer Review (AI)

### Reviewer: Martin Janci
### Date: 2025-11-26
### Outcome: ✅ Approve

### Summary
Location upload error handling comprehensive with clear messages, proper HTTP status codes, and request ID tracing.

### Key Findings
- **[Info]** Error types from Story 1.7 framework
- **[Info]** Request ID in all error logs
- **[Info]** Rate limiting with Retry-After header

### Acceptance Criteria Coverage
| AC | Status | Evidence |
|----|--------|----------|
| AC1 - 404 for device not found | ✅ | ApiError::NotFound |
| AC2 - 400 with field details | ✅ | ValidationErrors |
| AC3 - 503 for database timeout | ✅ | ServiceUnavailable variant |
| AC4 - 413 for large payload | ✅ | PayloadTooLarge + tower layer |
| AC5 - 429 with Retry-After | ✅ | Rate limit middleware |
| AC6 - request_id in logs | ✅ | Request ID middleware |

### Test Coverage and Gaps
- All error scenarios tested
- Error message format validated
- No gaps identified

### Architectural Alignment
- ✅ Consistent error handling framework
- ✅ Proper HTTP status codes

### Security Notes
- Error messages don't expose internals
- Request ID for audit trail

### Action Items
None - story approved for completion.

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Senior Developer Review notes appended | AI Reviewer |
