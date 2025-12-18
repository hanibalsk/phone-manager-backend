# Story UGM-5.5: Migration Concurrency and Atomicity

**Status**: Ready for Development

## Story

**As a** system administrator,
**I want** concurrent migration requests to be handled safely,
**So that** data integrity is maintained and no duplicate groups are created.

**Epic**: UGM-5: NFR Compliance
**Prerequisites**: Story UGM-2.2 (Migration Endpoint)
**NFRs Covered**: NFR5, NFR13, NFR17

## Acceptance Criteria

### Atomicity (NFR13)
1. [ ] Given a migration in progress, when a database error occurs mid-migration, then all changes are rolled back (no partial state)
2. [ ] Given a migration with 10 devices, when device 5 fails to migrate, then none of the devices are migrated (all-or-nothing)
3. [ ] Given a successful migration, then the authenticated group AND all device memberships are created in a single transaction

### Concurrency (NFR5, NFR17)
4. [ ] Given 10 concurrent migration requests for the SAME registration group, when all requests complete, then exactly ONE migration succeeds and 9 return 409 Conflict
5. [ ] Given 10 concurrent migration requests for DIFFERENT registration groups, when all requests complete, then all 10 migrations succeed independently
6. [ ] Given a migration in progress (locked), when another request arrives for same group, then it returns 409 immediately (doesn't wait)

### Performance Under Load (NFR5)
7. [ ] Given 100 concurrent migration requests (mix of same/different groups), when all complete, then p95 latency is < 500ms
8. [ ] Given migration endpoint under concurrent load, when measured, then no deadlocks occur

### Transaction Isolation
9. [ ] Given migration uses PostgreSQL transaction, then isolation level is SERIALIZABLE or uses advisory locks
10. [ ] Given concurrent access to same registration_group_id, then database-level locking prevents race conditions

## Technical Notes

- Use PostgreSQL advisory locks on registration_group_id hash for concurrency control
- Transaction isolation: SERIALIZABLE or explicit row locking
- Advisory lock example:
  ```sql
  SELECT pg_advisory_xact_lock(hashtext($1)); -- Lock on registration_group_id
  ```
- Return 409 Conflict with clear message:
  ```json
  {
    "error": {
      "code": "migration/in-progress",
      "message": "Migration for this registration group is already in progress",
      "details": {
        "registration_group_id": "camping-2025"
      }
    }
  }
  ```

## Tasks/Subtasks

- [ ] 1. Add advisory lock acquisition at start of migration
- [ ] 2. Ensure all migration operations are in single transaction
- [ ] 3. Add 409 response for concurrent migration attempts
- [ ] 4. Create concurrency stress test (10 threads, same group)
- [ ] 5. Create concurrency stress test (10 threads, different groups)
- [ ] 6. Create atomicity test (simulate mid-migration failure)
- [ ] 7. Add p95 latency measurement under load
- [ ] 8. Verify no deadlocks under concurrent access

## File List

### Files to Modify

- `crates/api/src/routes/groups.rs` - Add advisory lock to migration
- `crates/persistence/src/repositories/group.rs` - Add lock acquisition
- `crates/api/src/error.rs` - Add migration-in-progress error type

### Files to Create

- `crates/api/tests/migration_concurrency_test.rs` - Concurrency stress tests

## Test Scenarios

```rust
#[tokio::test]
async fn test_concurrent_migration_same_group() {
    // Spawn 10 concurrent migration requests for same registration_group_id
    // Assert: exactly 1 succeeds, 9 return 409
}

#[tokio::test]
async fn test_concurrent_migration_different_groups() {
    // Spawn 10 concurrent migration requests for different registration_group_ids
    // Assert: all 10 succeed
}

#[tokio::test]
async fn test_migration_atomicity_on_failure() {
    // Simulate failure after 5 of 10 devices migrated
    // Assert: no devices migrated, no group created
}
```

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Concurrency stress tests pass consistently (10 runs)
- [ ] No deadlocks detected under load
- [ ] p95 latency < 500ms under concurrent load
- [ ] Code compiles without warnings
- [ ] Code passes clippy

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created from gap analysis | Dev Agent |
