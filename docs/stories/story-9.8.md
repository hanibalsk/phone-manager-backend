# Story 9.8: JWT Middleware

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** backend developer
**I want** middleware that validates JWT access tokens
**So that** protected routes can authenticate users and access user context

## Prerequisites

- Story 9.3 complete (JWT infrastructure)
- Story 9.4 complete (register endpoint - issues tokens)
- Story 9.5 complete (login endpoint - issues tokens)

## Acceptance Criteria

1. Middleware extracts Bearer token from Authorization header
2. Validates token signature using public key
3. Validates token expiration
4. Extracts user_id from token claims
5. Makes user_id available to handlers via request extensions
6. Returns 401 Unauthorized for missing or invalid tokens
7. Optional variant for routes where auth is optional

## Technical Notes

- Use JwtConfig.validate_access_token() for validation
- Extract user_id from claims.sub
- Create UserAuth extractor similar to ApiKeyAuth
- Support both required and optional authentication

## Implementation Tasks

- [x] Create require_user_auth middleware
- [x] Create UserAuth extractor
- [x] Create OptionalUserAuth extractor
- [x] Add unit tests
- [ ] Add middleware to app.rs for user-authenticated routes (deferred - no user routes yet)

---

## Dev Notes

- Similar pattern to existing require_auth (API key) middleware
- Will be used alongside API key auth for dual-auth support in future
- User routes will use this middleware

---

## Dev Agent Record

### Debug Log


### Completion Notes

Implemented JWT authentication middleware and extractors:

1. **Middleware** (`crates/api/src/middleware/user_auth.rs`):
   - `require_user_auth`: Validates Bearer token and stores UserAuth in extensions
   - `optional_user_auth`: Attempts validation but allows unauthenticated requests
   - UserAuth struct with user_id and jti fields
   - Helper methods for JWT config creation and token validation

2. **Extractors** (`crates/api/src/extractors/user_auth.rs`):
   - `UserAuth`: Axum extractor that validates JWT and provides user info
   - `OptionalUserAuth`: Optional variant for routes where auth is optional
   - Both check extensions first (from middleware) then validate directly

3. **Features**:
   - Extracts Bearer token from Authorization header
   - Validates token signature with RS256 public key
   - Validates token expiration
   - Extracts user_id (UUID) and jti from claims
   - Returns 401 Unauthorized for missing/invalid tokens
   - Stores auth info in request extensions for handlers

Note: Middleware not yet wired to routes - will be used when user-specific
routes are added in Epic 10 and beyond.

---

## File List

- `crates/api/src/middleware/user_auth.rs` - New middleware module
- `crates/api/src/middleware/mod.rs` - Added user_auth exports
- `crates/api/src/extractors/user_auth.rs` - New extractor module
- `crates/api/src/extractors/mod.rs` - Added user_auth exports

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete - JWT middleware and extractors |

