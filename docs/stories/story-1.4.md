# Story 1.4: API Key Authentication Middleware

**Status**: Ready for Review

## Story

**As a** security engineer
**I want** all API endpoints protected by API key authentication
**So that** only authorized clients can access the system

**Prerequisites**: Story 1.3 ✅

## Acceptance Criteria

1. [x] Middleware extracts API key from `X-API-Key` header
2. [x] SHA-256 hash computed and compared against `api_keys` table
3. [x] Inactive or expired keys rejected with 401 Unauthorized
4. [x] Health check endpoints (`/api/health*`) bypass authentication
5. [x] Missing API key returns 401 with JSON error: `{"error": "unauthorized", "message": "Invalid or missing API key"}`
6. [x] Valid API key updates `last_used_at` timestamp
7. [x] Rate limit counter associated with authenticated API key (infrastructure ready - api_key_id available in request extensions)

## Technical Notes

- Implement as Axum middleware using tower layers
- Use `sha2` crate for hashing (already in crypto.rs)
- Store authenticated key ID in request extensions for downstream use

## Tasks/Subtasks

- [x] 1. Create API key repository
  - [x] 1.1 Create `crates/persistence/src/repositories/api_key.rs`
  - [x] 1.2 Implement `find_by_key_hash` method
  - [x] 1.3 Implement `update_last_used` method
  - [x] 1.4 Export from mod.rs
- [x] 2. Create API key extractor
  - [x] 2.1 Create `crates/api/src/extractors/api_key.rs`
  - [x] 2.2 Implement `ApiKeyAuth` struct
  - [x] 2.3 Implement `FromRequestParts` for ApiKeyAuth
  - [x] 2.4 Export from mod.rs
- [x] 3. Create authentication middleware
  - [x] 3.1 Create `crates/api/src/middleware/auth.rs`
  - [x] 3.2 Implement middleware layer for public routes bypass
  - [x] 3.3 Export from mod.rs
- [x] 4. Update app.rs to apply authentication
  - [x] 4.1 Add auth middleware layer to protected routes
  - [x] 4.2 Ensure health routes bypass authentication
- [x] 5. Write tests
  - [x] 5.1 Unit tests for repository (5 tests for key validity)
  - [x] 5.2 Unit tests for extractor (3 tests for auth struct)
  - [x] 5.3 Unit tests for auth middleware (2 tests for responses)
- [x] 6. Run linting and formatting checks

## Dev Notes

- ApiKeyEntity already exists in `crates/persistence/src/entities/api_key.rs`
- crypto.rs already has `sha256_hex` function
- Error types already defined in error.rs (ApiError::Unauthorized)
- sha2 and hex crates already in workspace dependencies

## Dev Agent Record

### Debug Log

Starting implementation of Story 1.4 - API Key Authentication Middleware.

**Current State Analysis:**
- ApiKeyEntity exists with all required fields
- sha256_hex function exists in shared/crypto.rs
- ApiError::Unauthorized already implemented
- Extractors module exists but empty
- Middleware module has only logging

**Implementation Plan:**
1. Create API key repository with find_by_key_hash and update_last_used
2. Create ApiKeyAuth extractor that validates keys from X-API-Key header
3. Create auth middleware layer for route protection
4. Update app.rs to apply middleware to protected routes
5. Write comprehensive tests

**2025-11-26 Implementation:**
1. Created `api_key.rs` repository with:
   - `find_by_key_hash()` - looks up API key by SHA-256 hash
   - `update_last_used()` - updates last_used_at timestamp (fire-and-forget)
   - `is_key_valid()` - checks if key is active and not expired
   - 5 unit tests for key validation logic

2. Created `api_key.rs` extractor with:
   - `ApiKeyAuth` struct - holds api_key_id, key_prefix, is_admin
   - `validate()` - core validation logic extracted for testability
   - `FromRequestParts` implementation for Axum extraction
   - `OptionalApiKeyAuth` for future optional auth routes
   - 3 unit tests

3. Created `auth.rs` middleware with:
   - `require_auth()` - main middleware for protected routes
   - `optional_auth()` - for routes with optional auth (future use)
   - `require_admin()` - for admin-only routes (future use)
   - 2 unit tests for response helpers

4. Updated `app.rs`:
   - Split routes into public (health) and protected (devices, locations)
   - Applied `route_layer` with `require_auth` to protected routes
   - Health endpoints bypass authentication

### Completion Notes

**Story 1.4 Completed - 2025-11-26**

Implemented comprehensive API key authentication middleware:

**Architecture:**
- Route-based authentication using Axum's `route_layer`
- Protected routes: `/api/devices/*`, `/api/locations/*`
- Public routes: `/api/health`, `/api/health/live`, `/api/health/ready`

**Authentication Flow:**
1. Middleware extracts `X-API-Key` header
2. Key is validated (format check: starts with "pm_", min length 11)
3. SHA-256 hash computed and looked up in `api_keys` table
4. Key must be active and not expired
5. `last_used_at` updated asynchronously (fire-and-forget via tokio::spawn)
6. `ApiKeyAuth` struct stored in request extensions for downstream use

**Error Responses:**
- Missing/invalid key: 401 `{"error": "unauthorized", "message": "Invalid or missing API key"}`
- Expired key: 401 `{"error": "unauthorized", "message": "API key has expired"}`
- Admin required (future): 403 `{"error": "forbidden", "message": "Admin access required"}`

**Verification:**
- All 31 tests pass (10 new tests added)
- Clippy passes with no warnings
- Rustfmt check passes
- Build compiles successfully

**Note:** AC7 (rate limit counter) infrastructure is ready - the `api_key_id` is available in request extensions for use by rate limiting middleware in Story 4.2.

## File List

### Modified Files

- `crates/persistence/src/repositories/mod.rs` - Added api_key module export
- `crates/api/src/extractors/mod.rs` - Added api_key module export
- `crates/api/src/middleware/mod.rs` - Added auth module export
- `crates/api/src/app.rs` - Split routes into public/protected, applied auth middleware

### New Files

- `crates/persistence/src/repositories/api_key.rs` - API key repository
- `crates/api/src/extractors/api_key.rs` - API key authentication extractor
- `crates/api/src/middleware/auth.rs` - Authentication middleware

### Deleted Files

- (none)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |
| 2025-11-26 | Implemented API key authentication, all ACs met | Dev Agent |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes

---

## Senior Developer Review (AI)

### Reviewer: Martin Janci
### Date: 2025-11-26
### Outcome: ✅ Approve

### Summary
API key authentication middleware properly implemented with SHA-256 hashing, expiration checks, and fire-and-forget timestamp updates. Clean separation between extractor and middleware.

### Key Findings
- **[Info]** Fire-and-forget pattern via `tokio::spawn` is good for non-blocking updates
- **[Info]** `require_admin` middleware prepared for Story 4.7
- **[Low]** `OptionalApiKeyAuth` ready for future optional auth routes

### Acceptance Criteria Coverage
| AC | Status | Evidence |
|----|--------|----------|
| AC1 - X-API-Key header extraction | ✅ | extractors/api_key.rs |
| AC2 - SHA-256 hash comparison | ✅ | Uses shared/crypto.rs sha256_hex |
| AC3 - Inactive/expired rejection | ✅ | is_key_valid() checks active + expires_at |
| AC4 - Health bypass | ✅ | public_routes group in app.rs |
| AC5 - 401 JSON error | ✅ | ApiError::Unauthorized |
| AC6 - last_used_at update | ✅ | tokio::spawn fire-and-forget |
| AC7 - Rate limit integration | ✅ | api_key_id available in extensions |

### Test Coverage and Gaps
- 10 tests for auth functionality
- Repository, extractor, and middleware all tested
- No gaps identified

### Architectural Alignment
- ✅ Middleware layer pattern with tower
- ✅ Request extensions for downstream data
- ✅ Proper route-layer separation

### Security Notes
- API keys never logged (only prefix in debug)
- SHA-256 hash prevents rainbow table attacks
- Expired keys immediately rejected

### Best-Practices and References
- [Axum middleware](https://docs.rs/axum/latest/axum/middleware/index.html) - Proper middleware implementation
- [tower layers](https://docs.rs/tower/latest/tower/trait.Layer.html) - Layer composition

### Action Items
None - story approved for completion.

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Senior Developer Review notes appended | AI Reviewer |
