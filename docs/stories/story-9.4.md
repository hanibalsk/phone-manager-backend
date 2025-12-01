# Story 9.4: Register Endpoint

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** new user
**I want** to register with email and password
**So that** I can create an account and access the system

## Prerequisites

- Story 9.1 complete (user database schema)
- Story 9.2 complete (password hashing)
- Story 9.3 complete (JWT infrastructure)

## Acceptance Criteria

1. POST /api/v1/auth/register endpoint accepts email, password, display_name
2. Password validated: min 8 chars, 1 uppercase, 1 lowercase, 1 digit
3. Email validated for format and uniqueness
4. Password hashed with Argon2id before storage
5. JWT access and refresh tokens returned on success
6. Returns 201 Created with user object and tokens
7. Returns 400 for validation errors
8. Returns 409 for duplicate email
9. Optional device_id and device_name for linking on registration

## Technical Notes

- Use validator crate for input validation
- Integrate with password hashing from Story 9.2
- Integrate with JWT from Story 9.3
- Create user_sessions record for the new token
- Store token JTI (jti) for revocation support
- JWT config needs to be loaded from environment

## Implementation Tasks

- [x] Add JWT configuration to app config
- [x] Create RegisterRequest and RegisterResponse DTOs
- [x] Create AuthService with register method
- [x] Implement password validation rules
- [x] Implement register handler in routes
- [x] Create auth routes module with /auth prefix
- [x] Add user session creation on registration
- [ ] Add comprehensive integration tests (deferred)

---

## Dev Notes

- Rate limiting will be added in a later story
- Email verification is optional for MVP
- Device linking on registration is optional

---

## Dev Agent Record

### Debug Log


### Completion Notes

Implemented user registration endpoint with full authentication flow:

1. **JWT Configuration** (`crates/api/src/config.rs`):
   - Added `JwtAuthConfig` struct with RSA key paths and token expiry settings
   - Environment variables: `PM__JWT__PRIVATE_KEY`, `PM__JWT__PUBLIC_KEY`
   - Default expiry: 1 hour (access), 30 days (refresh)

2. **AuthService** (`crates/api/src/services/auth.rs`):
   - `register()` method: validates password, hashes with Argon2id, creates user
   - Password validation: min 8 chars, 1 uppercase, 1 lowercase, 1 digit
   - Token generation using JwtConfig from shared crate
   - Session creation with hashed JTIs for revocation support

3. **Auth Routes** (`crates/api/src/routes/auth.rs`):
   - `POST /api/v1/auth/register` endpoint
   - Request validation with validator crate
   - Response includes user object and token pair
   - Error mapping for duplicate email, weak password, etc.

4. **Configuration updates**:
   - Updated `config/default.toml` with JWT section
   - Updated test config defaults

---

## File List

- `crates/api/src/config.rs` - Added JwtAuthConfig
- `crates/api/src/services/auth.rs` - AuthService with register method
- `crates/api/src/services/mod.rs` - Module export
- `crates/api/src/routes/auth.rs` - Register endpoint handler
- `crates/api/src/routes/mod.rs` - Module export
- `crates/api/src/app.rs` - Added auth routes to router
- `config/default.toml` - Added JWT configuration section

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete - Register endpoint with JWT tokens |

