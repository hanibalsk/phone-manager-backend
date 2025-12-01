# Story 9.7: Logout Endpoint

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As an** authenticated user
**I want** to logout and invalidate my tokens
**So that** my session is securely terminated and cannot be reused

## Prerequisites

- Story 9.3 complete (JWT infrastructure)
- Story 9.4 complete (register endpoint - creates sessions)
- Story 9.6 complete (refresh endpoint - uses sessions)

## Acceptance Criteria

1. POST /api/v1/auth/logout endpoint requires Bearer token authentication
2. Invalidates the current session by deleting it from user_sessions
3. Optionally accepts refresh_token to invalidate specific session
4. Optionally accepts all_devices=true to revoke all user sessions
5. Returns 204 No Content on success
6. Returns 401 for invalid or missing access token

## Technical Notes

- Requires JWT middleware for authentication (Story 9.8)
- For MVP without JWT middleware, accept refresh_token to identify session
- Delete session from user_sessions table
- all_devices=true deletes all sessions for user

## Implementation Tasks

- [x] Add logout method to AuthService
- [x] Create LogoutRequest DTO (optional body)
- [x] Implement logout handler
- [x] Add route to auth routes
- [x] Add unit tests

---

## Dev Notes

- Logout can work without JWT middleware by using refresh_token to identify session
- Once JWT middleware is implemented, can also use access token's JTI
- For MVP, refresh_token approach is sufficient

---

## Dev Agent Record

### Debug Log


### Completion Notes

Implemented logout endpoint with session invalidation:

1. **AuthService.logout()** (`crates/api/src/services/auth.rs`):
   - Validates refresh token to get user_id
   - If all_devices=true, deletes all sessions for the user
   - Otherwise, hashes JTI and deletes specific session
   - Gracefully handles case where session already logged out

2. **Logout Handler** (`crates/api/src/routes/auth.rs`):
   - `POST /api/v1/auth/logout` endpoint
   - LogoutRequest DTO with refresh_token (required) and all_devices (optional, default false)
   - Returns 204 No Content on success
   - Returns 401 for invalid/expired refresh token

3. **Security Features**:
   - Session identified by hashed JTI lookup
   - Support for "logout from all devices" functionality
   - Logs debug message when session already invalidated

---

## File List

- `crates/api/src/services/auth.rs` - Added logout method
- `crates/api/src/routes/auth.rs` - Added LogoutRequest, logout handler
- `crates/api/src/app.rs` - Added logout route

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete - Logout with session invalidation |

