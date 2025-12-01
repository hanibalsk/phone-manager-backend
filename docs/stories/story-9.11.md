# Story 9.11: Current User Profile Endpoints

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** logged-in user
**I want** to view and update my profile information
**So that** I can manage my account details

## Prerequisites

- Story 9.8 complete (JWT middleware)

## Acceptance Criteria

1. GET /api/v1/users/me returns current user profile
2. PUT /api/v1/users/me updates profile fields (display_name, avatar_url)
3. Requires JWT authentication (Bearer token)
4. Returns user info: id, email, displayName, avatarUrl, emailVerified, createdAt, updatedAt
5. Only allows updating own profile (enforced by JWT subject)
6. Returns 401 if not authenticated

## Technical Notes

- Use UserAuth extractor for JWT validation
- Display name validation: 1-100 characters
- Avatar URL validation: valid URL or null
- Email cannot be changed via profile update (separate flow)

## Implementation Tasks

- [x] Create user profile routes module
- [x] Implement GET /me handler
- [x] Implement PUT /me handler
- [x] Create ProfileResponse DTO
- [x] Create UpdateProfileRequest DTO
- [x] Add user repository method for fetching and updating user
- [x] Add routes to app.rs
- [x] Add unit tests

---

## Dev Notes

- Password change would be a separate endpoint (not in this story)
- Profile picture upload would be a separate story (just URL storage here)

---

## Dev Agent Record

### Debug Log

- Initial review found that user profile routes already existed in `crates/api/src/routes/users.rs` with GET/PUT handlers
- Routes already registered in `app.rs` at lines 246-248
- ProfileResponse and UpdateProfileRequest DTOs already implemented with validation
- Missing: avatar_url column in users table (handlers reference it but schema didn't include it)
- Created migration 021 to add avatar_url column
- Updated UserEntity and User domain model to include avatar_url field
- Updated UserRepository queries to include avatar_url in SELECT statements
- Fixed COALESCE handling for display_name in profile queries (handle NULL gracefully)
- Fixed pre-existing clippy warnings (redundant_closure, manual_strip)

### Completion Notes

Story 9.11 implementation completed. The user profile endpoints were already implemented but missing the database column for avatar_url.

**Changes made:**
1. Created migration `021_user_avatar_url.sql` - adds avatar_url column to users table
2. Updated `UserEntity` - added avatar_url field
3. Updated `User` domain model - added avatar_url field
4. Updated `UserRepository` queries - included avatar_url in SELECT statements
5. Updated profile route SQL queries - added COALESCE for display_name to handle NULL values
6. Fixed clippy warnings in extractors/user_auth.rs and middleware/user_auth.rs

**Endpoints:**
- GET /api/v1/users/me - Returns ProfileResponse with id, email, displayName, avatarUrl, emailVerified, createdAt, updatedAt
- PUT /api/v1/users/me - Updates display_name and/or avatar_url; validates display_name (1-100 chars) and avatar_url (valid URL)


---

## File List

- `crates/persistence/src/migrations/021_user_avatar_url.sql` (new)
- `crates/persistence/src/entities/user.rs` (modified)
- `crates/persistence/src/repositories/user.rs` (modified)
- `crates/domain/src/models/user.rs` (modified)
- `crates/api/src/routes/users.rs` (modified)
- `crates/api/src/extractors/user_auth.rs` (modified - clippy fix)
- `crates/api/src/middleware/user_auth.rs` (modified - clippy fix)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation completed - added avatar_url migration, updated models and queries |
| 2025-12-01 | Senior Developer Review notes appended |

---

## Senior Developer Review (AI)

### Reviewer
Martin Janci

### Date
2025-12-01

### Outcome
**Approve**

### Summary
Story 9.11 implementation is complete and meets all acceptance criteria. The user profile endpoints (GET/PUT /api/v1/users/me) are properly implemented with JWT authentication, input validation, and appropriate error handling. The implementation follows the project's layered architecture pattern and Rust best practices.

### Key Findings

**Positive Findings:**
1. ✅ **Good validation**: UpdateProfileRequest properly validates display_name (1-100 chars) and avatar_url (valid URL format) using the validator crate
2. ✅ **Proper error handling**: Clear error messages for NotFound, Forbidden, and Validation errors
3. ✅ **Security**: JWT authentication enforced via UserAuth extractor; users can only access/update their own profiles
4. ✅ **NULL handling**: COALESCE used correctly to handle NULL display_name values
5. ✅ **Comprehensive unit tests**: 8 tests covering validation edge cases and serialization

**Low Severity Observations:**
1. [Low] The dynamic query building in `update_current_user` (lines 131-149) works but could benefit from a query builder pattern for maintainability as fields grow
2. [Low] The index on avatar_url (`idx_users_avatar_url`) may be premature optimization - consider if actually needed for query patterns

### Acceptance Criteria Coverage

| AC | Description | Status | Evidence |
|----|-------------|--------|----------|
| 1 | GET /api/v1/users/me returns current user profile | ✅ Met | `get_current_user` handler at lines 49-77 |
| 2 | PUT /api/v1/users/me updates profile fields | ✅ Met | `update_current_user` handler at lines 97-196 |
| 3 | Requires JWT authentication | ✅ Met | `UserAuth` extractor used in both handlers |
| 4 | Returns correct user info fields | ✅ Met | ProfileResponse struct at lines 20-30 |
| 5 | Only allows updating own profile | ✅ Met | JWT subject (user_id) used directly from token |
| 6 | Returns 401 if not authenticated | ✅ Met | Handled by UserAuth extractor |

### Test Coverage and Gaps

**Unit Tests Present:**
- `test_update_profile_request_validation` - Valid request
- `test_update_profile_request_display_name_too_long` - Boundary test
- `test_update_profile_request_display_name_empty` - Empty string validation
- `test_update_profile_request_valid_avatar_url` - Valid URL
- `test_update_profile_request_invalid_avatar_url` - Invalid URL format
- `test_update_profile_request_empty` - No-op case
- `test_profile_response_serialization` - JSON output format
- `test_profile_response_serialization_no_avatar` - Null avatar handling

**Test Coverage Assessment:** Good coverage of validation logic and serialization. Integration tests with actual database would be valuable but are noted as covered separately.

**Suggested Future Tests:**
- Integration test for GET /me with valid JWT
- Integration test for PUT /me with partial updates
- Test for inactive user account handling

### Architectural Alignment

✅ **Follows layered architecture:**
- Routes in `crates/api/src/routes/users.rs`
- Entity in `crates/persistence/src/entities/user.rs`
- Domain model in `crates/domain/src/models/user.rs`
- Repository in `crates/persistence/src/repositories/user.rs`

✅ **Follows project conventions:**
- camelCase JSON serialization via `#[serde(rename_all = "camelCase")]`
- Proper error type usage (`ApiError`)
- Tracing/logging via `tracing::info!`

### Security Notes

1. ✅ Password hash never exposed in profile response (skip_serializing attribute on User model)
2. ✅ Email cannot be changed via profile update (by design)
3. ✅ Avatar URL validated as proper URL format
4. ✅ No SQL injection risk - parameterized queries used throughout
5. ⚠️ Consider adding avatar URL content-type validation or URL scheme restriction (https only) in future iteration

### Best-Practices and References

- [Rust Axum Framework](https://docs.rs/axum/latest/axum/) - Handler patterns followed
- [SQLx](https://docs.rs/sqlx/latest/sqlx/) - Compile-time checked queries pattern used
- [Validator Crate](https://docs.rs/validator/latest/validator/) - Input validation patterns
- OWASP REST Security Guidelines - Profile update follows principle of least privilege

### Action Items

None - implementation is approved for merge.

**Future Enhancements (optional, not blocking):**
- [ ] [Enhancement][Low] Consider avatar URL scheme validation (restrict to https://)
- [ ] [Enhancement][Low] Add integration tests for profile endpoints
- [ ] [TechDebt][Low] Consider using a query builder pattern for dynamic update queries as profile fields grow

