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

- [ ] Create user profile routes module
- [ ] Implement GET /me handler
- [ ] Implement PUT /me handler
- [ ] Create ProfileResponse DTO
- [ ] Create UpdateProfileRequest DTO
- [ ] Add user repository method for fetching and updating user
- [ ] Add routes to app.rs
- [ ] Add unit tests

---

## Dev Notes

- Password change would be a separate endpoint (not in this story)
- Profile picture upload would be a separate story (just URL storage here)

---

## Dev Agent Record

### Debug Log


### Completion Notes


---

## File List


---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

