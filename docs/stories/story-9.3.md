# Story 9.3: JWT Infrastructure with RS256

**Epic**: Epic 9 - Authentication Foundation (Backend)
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** backend system
**I want** JWT token infrastructure using RS256 algorithm
**So that** I can issue and validate secure authentication tokens

## Prerequisites

- Story 9.1 complete (user database schema)

## Acceptance Criteria

1. Generate RS256 key pair (private key for signing, public key for verification)
2. Issue access tokens with configurable expiration (default 15 minutes)
3. Issue refresh tokens with configurable expiration (default 7 days)
4. Token payload includes: sub (user_id), exp, iat, jti (token id)
5. Validate tokens and extract claims
6. Support key rotation (load key from file or environment)
7. Configuration: `PM__JWT__SECRET_KEY`, `PM__JWT__ACCESS_TOKEN_EXPIRY_SECS`, `PM__JWT__REFRESH_TOKEN_EXPIRY_SECS`
8. Token validation returns clear errors for expired, invalid, or malformed tokens

## Technical Notes

- Use `jsonwebtoken` crate for JWT operations
- RS256 (RSA-SHA256) for asymmetric signing
- For development, can generate keys with OpenSSL
- Store private key securely (env var or secrets management)
- Public key can be exposed for verification by other services

## Implementation Tasks

- [x] Add jsonwebtoken dependency
- [x] Create JWT configuration struct
- [x] Implement key loading from PEM or environment
- [x] Implement access token generation
- [x] Implement refresh token generation
- [x] Implement token validation
- [x] Add JWT-related configuration to app config (completed in Story 9.4)
- [x] Add comprehensive unit tests

---

## Dev Notes

- RS256 allows public key distribution for token verification
- Access tokens are short-lived, refresh tokens are long-lived
- jti claim enables token revocation via blacklist
- Consider key rotation strategy for production

---

## Dev Agent Record

### Debug Log


### Completion Notes

Implemented JWT token infrastructure with RS256 algorithm support:

1. **JwtConfig struct**: Holds encoding/decoding keys and configurable expiry times
   - `new()` constructor accepts PEM-formatted RSA keys
   - `new_for_testing()` for unit tests with HS256 symmetric key

2. **Claims struct**: Standard JWT claims with:
   - `sub`: User ID (UUID as string)
   - `exp`: Expiration timestamp
   - `iat`: Issued at timestamp
   - `jti`: Unique token ID for revocation support
   - `token_type`: Access or Refresh

3. **Token generation**:
   - `generate_access_token()`: Returns (token, jti) tuple
   - `generate_refresh_token()`: Returns (token, jti) tuple
   - Default expiry: 15 minutes for access, 7 days for refresh

4. **Token validation**:
   - `validate_token()`: Generic validation with expiry check
   - `validate_access_token()`: Validates and checks token type
   - `validate_refresh_token()`: Validates and checks token type
   - Leeway set to 0 for strict expiration checking

5. **Error handling**:
   - `JwtError` enum with clear error types
   - TokenExpired, InvalidToken, DecodingError, EncodingError, InvalidKey

14 unit tests covering all functionality.

---

## File List

- `crates/shared/src/jwt.rs` - JWT token infrastructure
- `crates/shared/src/lib.rs` - Module export
- `crates/shared/Cargo.toml` - Added jsonwebtoken dependency
- `Cargo.toml` - Added jsonwebtoken to workspace dependencies

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete - JWT infrastructure with RS256, 14 tests |

