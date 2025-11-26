# Story 3.7: Background Job Scheduler Infrastructure

**Status**: Not Started

## Story

**As a** backend system
**I want** a background job scheduler
**So that** I can run periodic maintenance tasks

**Prerequisites**: Epic 1 complete

## Acceptance Criteria

1. [ ] Job scheduler starts on application startup
2. [ ] Supports hourly, daily job frequencies
3. [ ] Jobs run in separate tokio tasks (non-blocking)
4. [ ] Job execution logged with start/end times and results
5. [ ] Failed jobs logged with error details but don't crash application
6. [ ] Graceful shutdown waits for running jobs to complete (with timeout)

## Technical Notes

- Use `tokio::time::interval` for scheduling
- Implement in `crates/api/src/jobs/` module
- Initial job: location cleanup (Story 3.6)

## Tasks/Subtasks

- [ ] 1. Create job scheduler infrastructure
  - [ ] 1.1 Create jobs module
  - [ ] 1.2 Implement scheduler with interval-based jobs
  - [ ] 1.3 Support hourly and daily frequencies
- [ ] 2. Add job execution framework
  - [ ] 2.1 Spawn jobs in separate tokio tasks
  - [ ] 2.2 Log job start/end with duration
  - [ ] 2.3 Handle job failures gracefully
- [ ] 3. Integrate with application lifecycle
  - [ ] 3.1 Start scheduler on app startup
  - [ ] 3.2 Graceful shutdown with timeout
- [ ] 4. Write tests
  - [ ] 4.1 Test job scheduling
  - [ ] 4.2 Test error handling
- [ ] 5. Run linting and formatting checks

## Dev Notes

- Jobs run in background, don't block API requests
- Failures are logged but don't crash the application
- Graceful shutdown waits for jobs with timeout

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
