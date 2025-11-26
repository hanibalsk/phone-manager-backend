# Story 4.2: Rate Limiting per API Key

**Status**: Complete ✅

## Story

**As a** backend system
**I want** rate limits enforced per API key
**So that** no single client can overwhelm the system

**Prerequisites**: Story 1.4 ✅

## Acceptance Criteria

1. [x] Default limit: 100 requests/minute per API key (configurable via `PM__SECURITY__RATE_LIMIT_PER_MINUTE`)
2. [x] Limit enforced using sliding window algorithm
3. [x] Returns 429 Too Many Requests when limit exceeded
4. [x] Response includes `Retry-After` header with seconds until reset
5. [x] Response body: `{"error": "rate_limit_exceeded", "message": "Rate limit of 100 requests/minute exceeded", "retryAfter": <seconds>}`
6. [x] Rate limit state stored in memory (Redis for multi-instance deployments in future)

## Technical Notes

- Use `governor` crate for rate limiting
- Store rate limiter keyed by API key ID
- Consider Redis-backed store for horizontal scaling

## Tasks/Subtasks

- [x] 1. Add governor crate dependency
- [x] 2. Implement rate limit middleware
- [x] 3. Configure per-key rate limiting
- [x] 4. Add 429 error response with Retry-After
- [x] 5. Make limit configurable
- [x] 6. Write tests
- [x] 7. Run linting and formatting checks

## Dev Notes

- In-memory rate limiter for single-instance deployments
- Governor provides efficient sliding window implementation

## Dev Agent Record

### Debug Log

- Implemented using governor crate with keyed rate limiter
- Per-API-key tracking via DashMap
- Configurable via PM__SECURITY__RATE_LIMIT_PER_MINUTE

### Completion Notes

Rate limiting fully functional with per-key limits, 429 responses, and Retry-After headers.

## File List

### Modified Files

- `crates/api/src/app.rs` - rate limit layer integration
- `crates/api/src/config.rs` - rate limit configuration
- `crates/api/src/error.rs` - RateLimitExceeded error

### New Files

- `crates/api/src/middleware/rate_limit.rs` - rate limit middleware

### Deleted Files

(None)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created and implementation complete | Dev Agent |

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
Rate limiting properly implemented with governor crate, per-key tracking, and proper HTTP response headers.

### Key Findings
- **[Info]** Governor provides efficient sliding window algorithm
- **[Info]** DashMap for concurrent per-key state
- **[Info]** Configurable limits via environment

### Acceptance Criteria Coverage
| AC | Status | Evidence |
|----|--------|----------|
| AC1 - 100 req/min default | ✅ | Config with PM__SECURITY__RATE_LIMIT_PER_MINUTE |
| AC2 - Sliding window | ✅ | Governor algorithm |
| AC3 - 429 response | ✅ | RateLimitExceeded error |
| AC4 - Retry-After header | ✅ | Header in response |
| AC5 - JSON error body | ✅ | Structured error response |
| AC6 - In-memory state | ✅ | DashMap storage |

### Test Coverage and Gaps
- Rate limit enforcement tested
- Header presence verified
- No gaps identified

### Architectural Alignment
- ✅ Tower middleware pattern
- ✅ Configurable via environment

### Security Notes
- Prevents DoS attacks via resource exhaustion

### Action Items
None - story approved for completion.

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Senior Developer Review notes appended | AI Reviewer |
