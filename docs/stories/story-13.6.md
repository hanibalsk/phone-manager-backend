# Story 13.6: Policy Resolution Algorithm Implementation

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: To Do
**Created**: 2025-12-01

---

## User Story

**As a** backend system
**I want** to resolve effective settings for a device based on policy hierarchy
**So that** device settings correctly reflect organization, group, and device-level configurations

## Prerequisites

- Story 13.3 complete (Device policies)
- Story 13.5 complete (Device enrollment)
- Story 12.2 complete (Get device settings)

## Acceptance Criteria

1. Policy resolution follows priority order: Organization defaults → Group policy → Device policy → Device custom settings
2. Higher priority policies override lower priority settings
3. Locked settings from any policy level cannot be overridden by lower levels
4. Resolution function returns merged settings + combined locked_keys set
5. GET `/api/v1/devices/{deviceId}/settings` uses resolution algorithm for managed devices
6. Resolution result is cached with TTL for performance
7. Cache invalidated on policy change, group change, or device settings change
8. Resolution handles devices with no organization (backward compatibility)

## Technical Notes

- Implement in `crates/domain/src/services/policy_resolution.rs`
- Use iterative merge strategy with locked key tracking
- Consider memoization for frequently accessed devices
- Cache key: `settings:resolved:{device_id}`

## Algorithm Pseudocode

```
function resolveEffectiveSettings(device):
    settings = {}
    locked_keys = Set()

    # 1. Organization defaults (if managed)
    if device.organization_id:
        org = getOrganization(device.organization_id)
        settings.merge(org.default_settings)

    # 2. Group policy (if in group with policy)
    if device.group_id:
        group = getGroup(device.group_id)
        if group.policy_id:
            policy = getPolicy(group.policy_id)
            settings.merge(policy.settings)
            locked_keys.addAll(policy.locked_settings)

    # 3. Device-specific policy (highest priority)
    if device.policy_id:
        policy = getPolicy(device.policy_id)
        settings.merge(policy.settings)
        locked_keys.addAll(policy.locked_settings)

    # 4. Device custom settings (only non-locked)
    for key, value in device.custom_settings:
        if key not in locked_keys:
            settings[key] = value

    return ResolvedSettings { settings, locked_keys }
```

---

## Implementation Tasks

- [ ] Create PolicyResolutionService in domain layer
- [ ] Implement resolve_effective_settings function
- [ ] Create ResolvedSettings struct with settings map and locked keys
- [ ] Add caching layer with configurable TTL
- [ ] Update get_device_settings to use resolution for managed devices
- [ ] Add cache invalidation triggers
- [ ] Add metrics for resolution timing
- [ ] Write unit tests for resolution scenarios
- [ ] Write integration tests for end-to-end resolution

## Test Scenarios

- Device with no organization (uses direct settings)
- Device with organization defaults only
- Device with group policy overriding org defaults
- Device with device policy overriding group policy
- Locked setting at group level cannot be changed at device level
- Multiple locked settings from different levels combined
- Cache hit vs cache miss performance

---

## Dev Notes

- Resolution is called on every settings fetch for managed devices
- Consider background job to pre-warm cache for active devices
- Lock inheritance is additive - once locked, stays locked
- Non-managed devices skip resolution entirely

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

