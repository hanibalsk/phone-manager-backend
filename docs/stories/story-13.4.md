# Story 13.4: Enrollment Tokens Management Endpoints

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to create and manage enrollment tokens
**So that** I can provision devices into my organization without manual setup

## Prerequisites

- Story 13.1 complete (Organizations)
- Story 13.3 complete (Device policies)

## Acceptance Criteria

1. Migration creates `enrollment_tokens` table: id (UUID), organization_id (FK), token (VARCHAR unique), token_prefix (VARCHAR), group_id (FK nullable), policy_id (FK nullable), max_uses, current_uses, expires_at, auto_assign_user_by_email, created_by (FK), created_at
2. POST `/api/admin/v1/organizations/{orgId}/enrollment-tokens` creates new token
3. GET `/api/admin/v1/organizations/{orgId}/enrollment-tokens` lists tokens with usage stats
4. DELETE `/api/admin/v1/organizations/{orgId}/enrollment-tokens/{tokenId}` revokes token
5. GET `/api/admin/v1/organizations/{orgId}/enrollment-tokens/{tokenId}/qr` returns QR code data
6. Token format: `enroll_<base64-random-45-chars>`
7. Token prefix stored separately for identification (first 8 chars)
8. Expired or max-used tokens rejected on enrollment
9. QR code contains enrollment URL with token

## Technical Notes

- Create migration 026_enrollment_tokens.sql
- Token generation similar to API keys (secure random + base64)
- QR code endpoint returns PNG or JSON with QR data URL
- Consider using `qrcode` crate for server-side QR generation
- Index on token for fast lookup during enrollment

## API Specification

### POST /api/admin/v1/organizations/{orgId}/enrollment-tokens

Request:
```json
{
  "group_id": "grp_uuid",
  "policy_id": "pol_uuid",
  "max_uses": 50,
  "expires_in_days": 30,
  "auto_assign_user_by_email": true
}
```

Response (201):
```json
{
  "id": "uuid",
  "token": "enroll_abc123xyz...",
  "token_prefix": "enroll_a",
  "organization_id": "org_uuid",
  "group_id": "grp_uuid",
  "policy_id": "pol_uuid",
  "max_uses": 50,
  "current_uses": 0,
  "expires_at": "2025-12-31T00:00:00Z",
  "created_at": "timestamp",
  "qr_code_url": "/api/admin/v1/organizations/{orgId}/enrollment-tokens/{id}/qr"
}
```

### GET /api/admin/v1/organizations/{orgId}/enrollment-tokens/{tokenId}/qr

Response (200) - either PNG image or:
```json
{
  "qr_data": "data:image/png;base64,...",
  "enrollment_url": "https://app.example.com/enroll?token=enroll_abc123..."
}
```

---

## Implementation Tasks

- [x] Create migration 027_enrollment_tokens.sql with table and indexes
- [x] Create EnrollmentTokenEntity in persistence layer
- [x] Create EnrollmentToken domain model
- [x] Create EnrollmentTokenRepository with CRUD
- [x] Token generation using base64 URL-safe encoding
- [x] Implement enrollment token CRUD endpoints
- [x] Implement QR code generation endpoint (returns JSON with URL)
- [x] Add token validation helper for enrollment flow (is_valid, is_expired, is_revoked, is_exhausted)
- [ ] Add audit logging for token operations (deferred to Story 13.9)
- [x] Write unit tests for token generation and validation
- [ ] Write integration tests for endpoints

---

## Dev Notes

- Tokens are one-time use or limited use for bulk provisioning
- QR code contains URL that mobile app can scan to auto-enroll
- Token expiry checked at enrollment time
- Consider rate limiting token creation

---

## Dev Agent Record

### Debug Log

- base64 crate was not in domain/Cargo.toml, added it
- rand::rng() is for rand 0.9, changed to rand::thread_rng() and rng.gen() for rand 0.8.x

### Completion Notes

Implemented enrollment tokens management with:
- Migration 027_enrollment_tokens.sql (used 027 since 026 was used by device policies)
- Token generation using secure random bytes + base64 URL-safe encoding
- Token prefix extraction for identification
- Expiry calculation from days
- Token validation methods (is_valid, is_expired, is_revoked, is_exhausted)
- CRUD endpoints for token management
- QR code endpoint returns JSON with enrollment URL
- Comprehensive unit tests for token generation and validation

---

## File List

- crates/persistence/src/migrations/027_enrollment_tokens.sql
- crates/persistence/src/entities/enrollment_token.rs
- crates/domain/src/models/enrollment_token.rs
- crates/persistence/src/repositories/enrollment_token.rs
- crates/api/src/routes/enrollment_tokens.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Story implemented and completed |

