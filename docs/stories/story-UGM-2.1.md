# Story UGM-2.1: Create Migration Audit Log Infrastructure

**Status**: Complete ✅

## Story

**As a** system administrator,
**I want** all group migrations to be logged with full context,
**So that** I can troubleshoot issues and maintain compliance records.

**Epic**: UGM-2: Group Migration
**Prerequisites**: None

## Acceptance Criteria

1. [x] Given the system needs to track migration events, when a database migration is run, then a new `migration_audit_logs` table is created with required columns
2. [x] Given the migration audit log table exists, when a migration event occurs (success or failure), then a record is inserted with all relevant context
3. [x] The table includes columns: migration_id (UUID), user_id (UUID FK), registration_group_id (string), authenticated_group_id (UUID FK), devices_migrated (int), device_ids (UUID array), status (enum), error_message (text nullable), created_at (timestamptz)

## Technical Notes

- Database migration file: `056_migration_audit_logs.sql`
- Status enum: `migration_status` with values: success, failed, partial
- Indexes on user_id, registration_group_id, authenticated_group_id, status, created_at
- Repository methods: create, find_by_id, is_already_migrated, get_migration_for_registration_group, list

## Tasks/Subtasks

- [x] 1. Create database migration file
- [x] 2. Create entity struct for migration audit logs
- [x] 3. Create repository with CRUD operations
- [x] 4. Export entities and repository in mod.rs

## File List

### Files Created

- `crates/persistence/src/migrations/056_migration_audit_logs.sql` - Database migration
- `crates/persistence/src/entities/migration_audit.rs` - Entity structs
- `crates/persistence/src/repositories/migration_audit.rs` - Repository implementation
- `docs/stories/story-UGM-2.1.md` - This story file

### Files Modified

- `crates/persistence/src/entities/mod.rs` - Export new entities
- `crates/persistence/src/repositories/mod.rs` - Export new repository

## Implementation Details

### Database Schema
- `migration_audit_logs` table with:
  - Primary key `id` (UUID)
  - `user_id` FK to users table
  - `registration_group_id` (VARCHAR, not FK)
  - `authenticated_group_id` FK to groups table
  - `devices_migrated` count
  - `device_ids` UUID array
  - `status` enum (success/failed/partial)
  - `error_message` optional text
  - `created_at` timestamp

### Entity Structs
- `MigrationStatusDb` - enum for status
- `MigrationAuditLogEntity` - database row mapping
- `MigrationAuditLogWithUserEntity` - includes user email and group name for admin queries

### Repository Methods
- `create()` - insert new audit log
- `find_by_id()` - find by migration ID
- `is_already_migrated()` - check if registration group already migrated
- `get_migration_for_registration_group()` - get successful migration info
- `list()` - paginated list with filters for admin queries

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass (unit tests in workspace)
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created and implemented | Dev Agent |
