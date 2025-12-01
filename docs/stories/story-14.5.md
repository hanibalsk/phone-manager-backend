# Story 14.5: Policy Apply/Unapply Endpoints

**Epic**: Epic 14 - Admin Portal Backend
**Status**: Complete
**Completed**: 2025-12-01
**Created**: 2025-12-01

---

## User Story

**As an** organization administrator
**I want** to apply and unapply device policies to/from devices and groups
**So that** I can manage device configurations at scale and reset devices to no policy

## Prerequisites

- Story 13.3 complete (Device policies CRUD)
- Story 13.6 complete (Policy resolution)

## Acceptance Criteria

1. POST `/api/admin/v1/organizations/{orgId}/policies/{policyId}/apply` already exists
2. POST `/api/admin/v1/organizations/{orgId}/policies/{policyId}/unapply` removes policy from devices/groups
3. Unapply can target specific devices or groups
4. Returns count of devices affected
5. Only org admins and owners can access these endpoints

## Technical Notes

- Apply endpoint already exists in device_policies.rs
- Add unapply endpoint to remove policy from devices
- Add repository methods for unapply operations

## API Specification

### POST /api/admin/v1/organizations/{orgId}/policies/{policyId}/unapply

Request Body:
```json
{
  "targets": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "targetType": "device"
    },
    {
      "id": "group-uuid",
      "targetType": "group"
    }
  ]
}
```

Response (200):
```json
{
  "policyId": "policy-uuid",
  "unappliedFrom": {
    "devices": 5,
    "groups": 1
  },
  "totalDevicesAffected": 12
}
```

---

## Implementation Tasks

- [x] Add unapply_from_devices repository method
- [x] Add unapply_from_group repository method
- [x] Add unapply_policy route handler
- [x] Create domain models for unapply request/response
- [x] Add route to app.rs
- [x] Write unit tests

---

## Dev Notes

- Unapply sets policy_id to NULL on devices
- Device count triggers will automatically update

---

## Dev Agent Record

### Debug Log

- Apply endpoint already existed in device_policies.rs from Story 13.3
- Added UnapplyPolicyRequest and UnapplyPolicyResponse domain models
- Added unapply_from_devices and unapply_from_group repository methods
- Unapply sets policy_id to NULL only for devices that have the specified policy

### Completion Notes

- Policy unapply removes policy assignment from devices and groups
- Device count triggers automatically update the policy's device_count
- Reuses PolicyTarget and AppliedToCount types from apply endpoint

---

## File List

- `crates/domain/src/models/device_policy.rs` - Added unapply request/response models
- `crates/domain/src/models/mod.rs` - Export new models
- `crates/persistence/src/repositories/device_policy.rs` - Added unapply repository methods
- `crates/api/src/routes/device_policies.rs` - Added unapply_policy route handler
- `crates/api/src/app.rs` - Added unapply route

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Implementation complete |
| 2025-12-01 | Senior developer review: APPROVED |

---

## Senior Developer Review

**Reviewer**: Martin Janci
**Date**: 2025-12-01
**Outcome**: ✅ APPROVED

### Summary
Policy apply/unapply endpoints implementation meets all acceptance criteria. Unapply properly sets policy_id to NULL only for devices that have the specified policy.

### Findings
- **Positive**: Reuses PolicyTarget and AppliedToCount types from apply endpoint
- **Positive**: Device count triggers automatically update policy's device_count
- **Positive**: Proper targeting (device or group)
- **Note**: Apply endpoint already existed from Story 13.3

### Acceptance Criteria Verification
| AC | Status |
|----|--------|
| Apply endpoint exists | ✅ (Story 13.3) |
| Unapply removes policy from devices/groups | ✅ |
| Target specific devices or groups | ✅ |
| Returns affected device counts | ✅ |
| Admin/Owner access only | ✅ |

### Security
- JWT authentication enforced
- Organization isolation verified
- Policy ownership validated before operations

### Action Items
None
