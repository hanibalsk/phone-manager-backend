# Story 3.7: Background Job Scheduler Infrastructure

**Status**: Complete ✅

## Story

**As a** backend system
**I want** a background job scheduler
**So that** I can run periodic maintenance tasks

**Prerequisites**: Epic 1 complete ✅

## Acceptance Criteria

1. [x] Job scheduler starts on application startup
2. [x] Supports hourly, daily job frequencies
3. [x] Jobs run in separate tokio tasks (non-blocking)
4. [x] Job execution logged with start/end times and results
5. [x] Failed jobs logged with error details but don't crash application
6. [x] Graceful shutdown waits for running jobs to complete (with timeout)

## Technical Notes

- Use `tokio::time::interval` for scheduling
- Implement in `crates/api/src/jobs/` module
- Initial job: location cleanup (Story 3.6)

## Tasks/Subtasks

- [x] 1. Create job scheduler infrastructure
  - [x] 1.1 Create jobs module
  - [x] 1.2 Implement scheduler with interval-based jobs
  - [x] 1.3 Support hourly and daily frequencies
- [x] 2. Add job execution framework
  - [x] 2.1 Spawn jobs in separate tokio tasks
  - [x] 2.2 Log job start/end with duration
  - [x] 2.3 Handle job failures gracefully
- [x] 3. Integrate with application lifecycle
  - [x] 3.1 Start scheduler on app startup
  - [x] 3.2 Graceful shutdown with timeout
- [x] 4. Write tests
  - [x] 4.1 Test job scheduling
  - [x] 4.2 Test error handling
- [x] 5. Run linting and formatting checks

## Dev Notes

- Jobs run in background, don't block API requests
- Failures are logged but don't crash the application
- Graceful shutdown waits for jobs with timeout

## Dev Agent Record

### Debug Log

- Implemented scheduler with tokio::time::interval
- Jobs spawned in separate tasks
- Structured logging for job execution
- Graceful shutdown with 30-second timeout

### Completion Notes

Background job scheduler fully functional with hourly/daily frequencies. Proper logging and graceful shutdown implemented.

## File List

### Modified Files

- `crates/api/src/main.rs` - scheduler startup
- `crates/api/src/lib.rs` - jobs module export

### New Files

- `crates/api/src/jobs/mod.rs` - scheduler implementation
- `crates/api/src/jobs/scheduler.rs` - job runner

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
Background job scheduler properly implemented with tokio-based scheduling, structured logging, and graceful shutdown.

### Key Findings
- **[Info]** tokio::time::interval for scheduling
- **[Info]** Separate tasks for non-blocking execution
- **[Info]** Graceful shutdown with timeout

### Acceptance Criteria Coverage
| AC | Status | Evidence |
|----|--------|----------|
| AC1 - Starts on startup | ✅ | main.rs integration |
| AC2 - Hourly/daily frequencies | ✅ | Scheduler configuration |
| AC3 - Separate tokio tasks | ✅ | tokio::spawn |
| AC4 - Logged execution | ✅ | tracing spans |
| AC5 - Failed jobs logged | ✅ | Error handling |
| AC6 - Graceful shutdown | ✅ | Timeout-based shutdown |

### Test Coverage and Gaps
- Scheduler tests
- Error handling tests
- No gaps identified

### Architectural Alignment
- ✅ Async-first design
- ✅ Non-blocking job execution

### Security Notes
- Job failures isolated from API requests

### Action Items
None - story approved for completion.

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Senior Developer Review notes appended | AI Reviewer |
