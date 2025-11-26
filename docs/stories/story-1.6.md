# Story 1.6: Structured Logging with Tracing

**Status**: Complete ✅

## Story

**As a** developer
**I want** structured JSON logs with request tracing
**So that** I can debug issues and monitor system behavior

**Prerequisites**: Story 1.2 ✅

## Acceptance Criteria

1. [x] Logs output as structured JSON in production (`PM__LOGGING__FORMAT=json`)
2. [x] Pretty-printed logs in development for readability
3. [x] Log level configurable via `PM__LOGGING__LEVEL` (trace, debug, info, warn, error)
4. [x] Request tracing includes `request_id` from `X-Request-ID` header (auto-generated if missing)
5. [x] All HTTP requests logged with: method, path, status, duration_ms, request_id
6. [x] Database queries logged at debug level with execution time
7. [x] Errors logged with full context and stack traces

## Technical Notes

- Use `tracing` and `tracing-subscriber` crates
- Configure subscriber based on environment
- Include span context for distributed tracing

## Tasks/Subtasks

- [x] 1. Create logging initialization
  - [x] 1.1 Create `init_logging` function
  - [x] 1.2 Support JSON format for production
  - [x] 1.3 Support pretty format for development
  - [x] 1.4 Configure log level from config
- [x] 2. Create request ID middleware
  - [x] 2.1 Create `crates/api/src/middleware/trace_id.rs`
  - [x] 2.2 Extract `X-Request-ID` header or generate UUID
  - [x] 2.3 Add request_id to request extensions
  - [x] 2.4 Add request_id to response headers
  - [x] 2.5 Export from mod.rs
- [x] 3. Create HTTP request logging middleware
  - [x] 3.1 Log method, path, status, duration_ms
  - [x] 3.2 Include request_id in all log entries
- [x] 4. Update app.rs to apply middleware
- [x] 5. Run linting and formatting checks

## Dev Notes

- Logging infrastructure exists in `crates/api/src/middleware/logging.rs`
- Request ID middleware added in `trace_id.rs`
- TraceLayer from tower-http provides base request logging
- Custom trace_id middleware adds request_id correlation

## Dev Agent Record

### Debug Log

**Current State Analysis:**
- `init_logging()` exists and handles JSON/pretty format
- Log level configurable via env filter
- tower-http TraceLayer already added in app.rs
- Missing: request ID middleware for X-Request-ID header

**Implementation Plan:**
1. Create trace_id.rs middleware for request ID handling
2. Enhance logging to include request_id in spans
3. Update app.rs to add trace_id middleware

### Completion Notes

**Story 1.6 Complete - 2025-11-26**

Implemented request ID tracing middleware that:

1. **Request ID Extraction/Generation**: Extracts `X-Request-ID` from incoming headers or generates UUID v4
2. **Request Extensions**: Stores `RequestId` in request extensions for downstream handlers
3. **Response Headers**: Adds `x-request-id` to response headers for client correlation
4. **Tracing Integration**: Creates tracing span with request_id, method, path
5. **Request Logging**: Logs request completion with status and duration_ms

**Middleware Stack Order** (in app.rs):
- CompressionLayer → TimeoutLayer → TraceLayer → trace_id → CORS

**Verification:**
- All 37 tests pass
- Clippy passes with no warnings
- Code formatted with rustfmt

## File List

### Modified Files

- `crates/api/src/app.rs` - Added trace_id middleware layer
- `crates/api/src/middleware/mod.rs` - Added trace_id module and re-exports

### New Files

- `crates/api/src/middleware/trace_id.rs` - Request ID middleware with:
  - `REQUEST_ID_HEADER` constant
  - `RequestId` struct for extension storage
  - `trace_id()` middleware function
  - `get_request_id()` helper for handlers
  - Unit tests for RequestId handling

### Deleted Files

- (none)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |
| 2025-11-26 | Implemented trace_id middleware | Dev Agent |
| 2025-11-26 | Story completed | Dev Agent |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass (37 tests)
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes
