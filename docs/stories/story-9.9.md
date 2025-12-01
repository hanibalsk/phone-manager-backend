# Story 9.9: Password Reset Flow

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** user who forgot my password
**I want** to reset my password via email
**So that** I can regain access to my account

## Prerequisites

- Story 9.1 complete (user database schema)
- Story 9.2 complete (password hashing)

## Acceptance Criteria

1. POST /api/v1/auth/forgot-password accepts email and always returns 200 (no enumeration)
2. Generates secure reset token stored in database with expiry
3. POST /api/v1/auth/reset-password accepts token and new password
4. Validates token exists and not expired
5. Updates password hash and invalidates token
6. Invalidates all existing sessions on password reset
7. Rate limited: 5 requests/hour/IP for forgot-password

## Technical Notes

- Reset token: random 32-byte hex string
- Token expiry: 1 hour
- Store token hash (not plaintext) in database
- Add password_reset_token and password_reset_expires_at to users table
- No actual email sending in MVP - just log the token

## Implementation Tasks

- [x] Add password reset columns to users table (migration)
- [x] Add forgot_password method to AuthService
- [x] Add reset_password method to AuthService
- [x] Create ForgotPasswordRequest and response DTOs
- [x] Create ResetPasswordRequest and response DTOs
- [x] Implement forgot-password handler
- [x] Implement reset-password handler
- [x] Add routes to auth routes
- [x] Add unit tests

---

## Dev Notes

- Email sending will be added in a future story
- For MVP, log the reset token to console for testing
- Always return 200 for forgot-password to prevent email enumeration

---

## Dev Agent Record

### Debug Log


### Completion Notes

Implemented password reset flow with:

1. **Migration** (`016_password_reset_columns.sql`):
   - Added `password_reset_token` VARCHAR(64) column
   - Added `password_reset_expires_at` TIMESTAMPTZ column
   - Added index on reset token for efficient lookup

2. **AuthService Methods**:
   - `forgot_password()`: Generates secure 32-byte token, stores hash in DB, logs token (MVP)
   - `reset_password()`: Validates token hash, checks expiry, updates password, invalidates all sessions
   - `generate_secure_token()`: Helper function using `rand::thread_rng().gen()`

3. **Route Handlers**:
   - POST `/api/v1/auth/forgot-password`: Always returns 200 to prevent email enumeration
   - POST `/api/v1/auth/reset-password`: Validates token and sets new password

4. **DTOs**:
   - `ForgotPasswordRequest`: email
   - `ForgotPasswordResponse`: message
   - `ResetPasswordRequest`: token, newPassword
   - `ResetPasswordResponse`: message

5. **Security Features**:
   - Token stored as SHA-256 hash (not plaintext)
   - 1-hour token expiry
   - All sessions invalidated on password reset
   - No email enumeration (always returns success)
   - Rate limiting applies (5 req/hour/IP specified in story)

---

## File List

- `crates/persistence/src/migrations/016_password_reset_columns.sql` - New migration
- `crates/api/src/services/auth.rs` - Added forgot_password, reset_password, generate_secure_token
- `crates/api/src/routes/auth.rs` - Added DTOs and handlers
- `crates/api/src/app.rs` - Added routes
- `crates/api/Cargo.toml` - Added hex, rand dependencies

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete - password reset flow |

