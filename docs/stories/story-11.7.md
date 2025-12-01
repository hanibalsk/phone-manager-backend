# Story 11.7: RBAC Middleware

**Epic**: Epic 11 - Group Management API
**Status**: Done
**Priority**: High

## User Story

**As a** backend developer
**I want** a reusable RBAC middleware for group-based authorization
**So that** I can easily protect endpoints based on group membership and roles

## Acceptance Criteria

1. Create `require_group_role` middleware that:
   - Extracts group_id from path parameters
   - Verifies user is a member of the group
   - Checks user has the required minimum role
   - Returns 403 if insufficient permissions
   - Returns 404 if group not found or user not a member
2. Middleware should work with role hierarchy (owner > admin > member > viewer)
3. Middleware should be composable with existing auth middleware
4. Add helper macros or functions for common permission checks

## Technical Notes

- Integration with existing `UserAuth` extractor
- Use axum's `from_fn_with_state` for middleware
- Extract group_id from route path (e.g., `:group_id`)
- Store membership info in request extensions for handler use

## Implementation Checklist

- [x] Create `GroupMembershipExtension` struct for passing membership to handlers
- [x] Create `require_group_member` middleware (any role)
- [x] Create `require_group_role` middleware with minimum role parameter
- [x] Create convenience middlewares: require_group_admin, require_group_owner
- [x] Add middleware to appropriate existing routes
- [x] Integration tests

## Middleware Usage Example

```rust
// In route definition
.route("/api/v1/groups/:group_id/sensitive", get(sensitive_handler))
    .route_layer(middleware::from_fn_with_state(
        state.clone(),
        |state, req, next| require_group_role(state, req, next, GroupRole::Admin)
    ))
```

## Permission Hierarchy

| Role | Can View | Can Manage | Can Admin | Can Own |
|------|----------|------------|-----------|---------|
| Viewer | Yes | No | No | No |
| Member | Yes | No | No | No |
| Admin | Yes | Yes | Yes | No |
| Owner | Yes | Yes | Yes | Yes |
