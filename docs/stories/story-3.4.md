# Story 3.4: Location Validation Logic

**Status**: Not Started

## Story

**As a** backend system
**I want** comprehensive location validation
**So that** invalid data never enters the database

**Prerequisites**: Story 3.1

## Acceptance Criteria

1. [ ] Latitude validation: -90.0 to 90.0 (inclusive), returns error "Latitude must be between -90 and 90"
2. [ ] Longitude validation: -180.0 to 180.0 (inclusive), returns error "Longitude must be between -180 and 180"
3. [ ] Accuracy validation: >= 0.0, returns error "Accuracy must be non-negative"
4. [ ] Bearing validation (if present): 0.0 to 360.0 (inclusive)
5. [ ] Speed validation (if present): >= 0.0
6. [ ] Battery level validation (if present): 0 to 100 (inclusive)
7. [ ] Timestamp validation: not in future, not older than 7 days
8. [ ] Validation errors return 400 with all field errors in single response

## Technical Notes

- Use `validator` crate with custom validators
- Database check constraints provide defense-in-depth
- Unit tests for all validation edge cases

## Tasks/Subtasks

- [ ] 1. Enhance validation logic
  - [ ] 1.1 Add timestamp validation (not future, not older than 7 days)
  - [ ] 1.2 Ensure all validators produce descriptive error messages
- [ ] 2. Add validation to request handlers
  - [ ] 2.1 Validate before any database operations
  - [ ] 2.2 Return all validation errors in single response
- [ ] 3. Write tests
  - [ ] 3.1 Test all validation edge cases
  - [ ] 3.2 Test error message formatting
- [ ] 4. Run linting and formatting checks

## Dev Notes

- Most validation already defined in domain models
- Need to add timestamp range validation
- Validation errors should include all failures, not just first

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
