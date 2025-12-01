# Story 9.1: User Authentication Database Schema

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Complete
**Created**: 2025-12-01

---

## User Story

**As a** developer
**I want** database tables for user authentication
**So that** I can store user accounts, sessions, and OAuth integrations

## Prerequisites

- Epic 1 complete (database infrastructure)

## Acceptance Criteria

1. Migration creates `users` table with columns: id (UUID PK), email (VARCHAR(255) UNIQUE NOT NULL), password_hash (VARCHAR(255) NULLABLE), display_name (VARCHAR(100)), is_active (BOOLEAN DEFAULT true), email_verified (BOOLEAN DEFAULT false), created_at (TIMESTAMPTZ), updated_at (TIMESTAMPTZ), last_login_at (TIMESTAMPTZ NULLABLE)
2. Migration creates `oauth_accounts` table with columns: id (UUID PK), user_id (UUID FK), provider (VARCHAR(20) NOT NULL), provider_user_id (VARCHAR(255) NOT NULL), provider_email (VARCHAR(255)), created_at (TIMESTAMPTZ)
3. Migration creates `user_sessions` table with columns: id (UUID PK), user_id (UUID FK), token_hash (VARCHAR(64) NOT NULL), refresh_token_hash (VARCHAR(64) NOT NULL), expires_at (TIMESTAMPTZ NOT NULL), created_at (TIMESTAMPTZ), last_used_at (TIMESTAMPTZ)
4. Unique constraint on oauth_accounts (provider, provider_user_id) for idempotency
5. Unique constraint on user_sessions (token_hash) for quick lookups
6. Foreign keys with ON DELETE CASCADE for data integrity
7. Index on users(email) for fast login lookups
8. Index on user_sessions(token_hash, expires_at) for token validation
9. Index on oauth_accounts(provider, provider_email) for OAuth lookups
10. Trigger for updated_at timestamp automation on users table
11. Migration runs successfully with `sqlx migrate run`

## Technical Notes

- Users can have password_hash NULL (OAuth-only accounts)
- Email must be unique across all authentication methods
- OAuth accounts linked via user_id foreign key
- Sessions track both access token and refresh token hashes
- expires_at used for automatic session cleanup

## Implementation Tasks

- [x] Create migration file for users table
- [x] Create migration file for oauth_accounts table
- [x] Create migration file for user_sessions table
- [x] Add all required indexes
- [x] Add foreign key constraints
- [x] Add unique constraints
- [x] Add check constraints for valid data
- [x] Create UserEntity struct
- [x] Create OAuthAccountEntity struct
- [x] Create UserSessionEntity struct
- [x] Create User domain model
- [x] Create UserRepository with basic CRUD

---

## Dev Notes

- Password hashing will be Argon2id (Story 9.2)
- JWT token generation will be RS256 (Story 9.3)
- Email verification flow deferred to later story
- Password reset flow deferred to later story

---

## Dev Agent Record

### Debug Log
- Starting Story 9.1 implementation
- Created migration 015_user_authentication_schema.sql with users, oauth_accounts, and user_sessions tables
- Added all required indexes for performance
- Created UserEntity, OAuthAccountEntity, UserSessionEntity structs
- Created User, OAuthAccount, UserSession domain models with OAuthProvider enum
- Created UserRepository with CRUD methods for users, OAuth accounts, and sessions
- All tests passing, clippy clean, code formatted

### Completion Notes
Story 9.1 completed successfully. Created complete user authentication database schema:
- users table: Core user accounts with email, password_hash (nullable for OAuth-only), display_name
- oauth_accounts table: Links users to Google/Apple OAuth with unique constraint on (provider, provider_user_id)
- user_sessions table: JWT session tracking with token_hash and refresh_token_hash

All tables have proper indexes, foreign key constraints, and check constraints. Password hash is nullable to support OAuth-only accounts. Ready for password hashing (Story 9.2) and JWT infrastructure (Story 9.3).

---

## File List

- `crates/persistence/src/migrations/015_user_authentication_schema.sql` - New migration
- `crates/persistence/src/entities/user.rs` - Entity structs (User, OAuth, Session)
- `crates/domain/src/models/user.rs` - Domain models with OAuthProvider enum
- `crates/persistence/src/repositories/user.rs` - UserRepository with CRUD
- `crates/domain/src/models/mod.rs` - Added module exports
- `crates/persistence/src/entities/mod.rs` - Added entity exports
- `crates/persistence/src/repositories/mod.rs` - Added repository export

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Story completed |

