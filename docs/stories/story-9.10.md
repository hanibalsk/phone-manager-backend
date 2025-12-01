# Story 9.10: Email Verification Endpoint

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** registered user
**I want** to verify my email address
**So that** I can confirm my identity and unlock full account features

## Prerequisites

- Story 9.1 complete (user database schema)
- Story 9.4 complete (register endpoint)

## Acceptance Criteria

1. POST /api/v1/auth/request-verification sends new verification email (if not verified)
2. POST /api/v1/auth/verify-email accepts verification token
3. Generates secure verification token stored in database with expiry
4. Validates token exists and not expired
5. Updates email_verified to true on successful verification
6. Returns appropriate error if already verified or token invalid
7. Rate limited: 3 requests/hour for request-verification

## Technical Notes

- Verification token: random 32-byte hex string
- Token expiry: 24 hours
- Store token hash (not plaintext) in database
- Add email_verification_token and email_verification_expires_at to users table
- No actual email sending in MVP - just log the token
- Resend verification should invalidate previous token

## Implementation Tasks

- [x] Add email verification columns to users table (migration)
- [x] Add request_email_verification method to AuthService
- [x] Add verify_email method to AuthService
- [x] Create RequestVerificationRequest/Response DTOs
- [x] Create VerifyEmailRequest/Response DTOs
- [x] Implement request-verification handler
- [x] Implement verify-email handler
- [x] Add routes to auth routes
- [x] Add unit tests

---

## Dev Notes

- Email sending will be added in a future story
- For MVP, log the verification token to console for testing
- Users can still use the app without verification (email_verified=false)

---

## Dev Agent Record

### Debug Log


### Completion Notes

Implemented email verification flow with:

1. **Migration** (`017_email_verification_columns.sql`):
   - Added `email_verification_token` VARCHAR(64) column
   - Added `email_verification_expires_at` TIMESTAMPTZ column
   - Added index on verification token for efficient lookup

2. **AuthService Methods**:
   - `request_email_verification()`: Generates secure 32-byte token, stores hash in DB, logs token (MVP)
   - `verify_email()`: Validates token hash, checks expiry, sets email_verified=true

3. **Route Handlers**:
   - POST `/api/v1/auth/request-verification`: Requires JWT auth, generates new verification token
   - POST `/api/v1/auth/verify-email`: Public endpoint, validates token and verifies email

4. **DTOs**:
   - `RequestVerificationResponse`: message
   - `VerifyEmailRequest`: token
   - `VerifyEmailResponse`: message, emailVerified

5. **Security Features**:
   - Token stored as SHA-256 hash (not plaintext)
   - 24-hour token expiry
   - Previous token automatically replaced when requesting new one
   - Returns error if email already verified (409 Conflict)
   - Request verification requires authentication (logged-in user)

6. **Error Handling**:
   - `EmailAlreadyVerified` error variant
   - `InvalidVerificationToken` error variant

---

## File List

- `crates/persistence/src/migrations/017_email_verification_columns.sql` - New migration
- `crates/api/src/services/auth.rs` - Added request_email_verification, verify_email, new error variants
- `crates/api/src/routes/auth.rs` - Added DTOs and handlers
- `crates/api/src/app.rs` - Added routes

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete - email verification flow |

