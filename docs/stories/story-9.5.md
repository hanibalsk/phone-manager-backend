# Story 9.5: Login Endpoint

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** registered user
**I want** to login with email and password
**So that** I can authenticate and access protected resources

## Prerequisites

- Story 9.1 complete (user database schema)
- Story 9.2 complete (password hashing)
- Story 9.3 complete (JWT infrastructure)
- Story 9.4 complete (register endpoint)

## Acceptance Criteria

1. POST /api/v1/auth/login endpoint accepts email and password
2. Validates credentials against stored password hash
3. Returns JWT access and refresh tokens on success
4. Returns 401 for invalid credentials
5. Returns 403 for disabled users
6. Updates last_login_at timestamp on successful login
7. Creates new session record

## Technical Notes

- Use Argon2id verification from shared crate
- Create new session for each login
- Optional device_id and device_name parameters
- Constant-time password comparison (built into Argon2)

## Implementation Tasks

- [x] Add login method to AuthService
- [x] Create LoginRequest DTO
- [x] Implement login handler
- [x] Add route to auth routes
- [x] Update last_login_at on success
- [x] Add unit tests

---

## Dev Notes

- Rate limiting and lockout will be added in a separate story
- Email verification check is optional for MVP

---

## Dev Agent Record

### Debug Log


### Completion Notes

Implemented user login endpoint with credential verification:

1. **AuthService.login()** (`crates/api/src/services/auth.rs`):
   - Fetches user by email (case-insensitive)
   - Checks if user is active (returns UserDisabled error if not)
   - Verifies password using Argon2id constant-time comparison
   - Updates last_login_at timestamp
   - Generates new JWT access and refresh tokens
   - Creates new session record

2. **Login Handler** (`crates/api/src/routes/auth.rs`):
   - `POST /api/v1/auth/login` endpoint
   - LoginRequest DTO with email/password validation
   - Returns 401 for invalid credentials
   - Returns 403 for disabled users
   - Returns user object and token pair on success

3. **Security Features**:
   - Generic error message for invalid credentials (no email enumeration)
   - Constant-time password comparison via Argon2
   - Logs actual errors but returns generic message to client

---

## File List

- `crates/api/src/services/auth.rs` - Added login method and UserRow struct
- `crates/api/src/routes/auth.rs` - Added LoginRequest, LoginResponse, login handler
- `crates/api/src/app.rs` - Added login route

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete - Login endpoint with credential verification |

