# Story 10.1: Device Table Migration for User Binding

**Epic**: Epic 10 - User-Device Binding
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** developer
**I want** to add owner_user_id and organization_id columns to the devices table
**So that** devices can be linked to user accounts and organizations

## Prerequisites

- Epic 9 complete (user authentication schema exists)
- Migration 015 (users table) exists

## Acceptance Criteria

1. Migration adds `owner_user_id` column (UUID, nullable, FK to users)
2. Migration adds `organization_id` column (UUID, nullable, FK to organizations)
3. Migration adds `is_primary` column (boolean, default false) for primary device designation
4. Migration adds `linked_at` column (TIMESTAMPTZ, nullable) for tracking when device was linked
5. Foreign key to users table with ON DELETE SET NULL (device stays but becomes unlinked)
6. Index on owner_user_id for efficient user device queries
7. Index on organization_id for organization device queries
8. Existing devices retain NULL for new columns (backward compatible)
9. Migration is non-blocking (ALTER TABLE ADD COLUMN with NULL default)

## Technical Notes

- Use ADD COLUMN with NULL default for non-blocking migration
- No need to backfill existing data
- FK constraints added after column creation
- Organization ID will be populated via user's organization in future stories

## Implementation Tasks

- [ ] Create migration 018_device_user_binding.sql
- [ ] Add owner_user_id column with FK to users
- [ ] Add organization_id column (no FK yet - organizations table not created)
- [ ] Add is_primary and linked_at columns
- [ ] Add indexes
- [ ] Run migration and verify

---

## Dev Notes

- Organizations table will be created in Epic 13 (B2B Enterprise)
- For now, organization_id is just a placeholder column without FK
- Users table uses UUID id field (from migration 015)

---

## Dev Agent Record

### Debug Log


### Completion Notes

Migration 018_device_user_binding.sql created with:
- owner_user_id UUID column with FK to users table
- organization_id UUID column (placeholder for Epic 13)
- is_primary BOOLEAN column (default false)
- linked_at TIMESTAMPTZ column
- Indexes for efficient queries

---

## File List

- `crates/persistence/src/migrations/018_device_user_binding.sql` - New migration

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

