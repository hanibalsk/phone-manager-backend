# Story UGM-5.2: API Timeout Handling with Retry-After Header

**Status**: Ready for Development

## Story

**As an** API consumer,
**I want** endpoints that timeout to return appropriate error codes with retry guidance,
**So that** I can implement proper retry logic in my client application.

**Epic**: UGM-5: NFR Compliance
**Prerequisites**: None
**NFRs Covered**: NFR21, NFR22

## Acceptance Criteria

1. [ ] Given any API endpoint, when processing exceeds 30 seconds, then the response is 504 Gateway Timeout
2. [ ] Given a 504 timeout response, then it includes `Retry-After` header with suggested retry delay in seconds
3. [ ] Given a 504 timeout response, then the body includes structured error with `error/timeout` code
4. [ ] Given a timeout error response body, then it includes `retry_after_seconds` field matching the header
5. [ ] Given the migration endpoint specifically, when it times out, then `Retry-After` is set to 60 seconds
6. [ ] Given read-only endpoints (GET), when they timeout, then `Retry-After` is set to 5 seconds
7. [ ] Given write endpoints (POST/PUT/DELETE), when they timeout, then `Retry-After` is set to 30 seconds

## Technical Notes

- Timeout middleware already exists at 30 seconds (`request_timeout_secs` config)
- Need to wrap timeout responses with proper headers and body
- Error response format:
  ```json
  {
    "error": {
      "code": "error/timeout",
      "message": "Request timed out. Please retry after the suggested delay.",
      "details": {
        "retry_after_seconds": 30,
        "endpoint": "/api/v1/groups/migrate"
      }
    }
  }
  ```
- `Retry-After` header format: integer seconds (e.g., `Retry-After: 30`)

## Tasks/Subtasks

- [ ] 1. Create timeout error response type with retry information
- [ ] 2. Create middleware to wrap timeout responses with headers
- [ ] 3. Configure different retry delays by endpoint type
- [ ] 4. Add `Retry-After` header to timeout responses
- [ ] 5. Add integration tests for timeout scenarios
- [ ] 6. Update OpenAPI spec with 504 response schema

## File List

### Files to Modify

- `crates/api/src/middleware/mod.rs` - Add timeout response wrapper
- `crates/api/src/error.rs` - Add timeout error type with retry info
- `crates/api/src/app.rs` - Configure timeout middleware with response wrapper
- `docs/api/openapi.yaml` - Document 504 response with Retry-After

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Timeout responses include proper headers
- [ ] Integration tests pass
- [ ] OpenAPI spec updated
- [ ] Code compiles without warnings
- [ ] Code passes clippy

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created from gap analysis | Dev Agent |
