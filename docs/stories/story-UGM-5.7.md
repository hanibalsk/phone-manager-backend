# Story UGM-5.7: Invite Flow Regression Tests and Performance Guardrails

**Status**: Ready for Development

## Story

**As a** system maintainer,
**I want** comprehensive regression tests for invite flows and performance guardrails,
**So that** UGM changes don't break existing invite functionality or degrade performance.

**Epic**: UGM-5: NFR Compliance
**Prerequisites**: Epic UGM-4 complete
**NFRs Covered**: NFR19, NFR20 (regression), Performance guardrail (<10% slowdown)

## Acceptance Criteria

### Invite Flow Regression
1. [ ] Given an authenticated user creating an invite, when UGM features are enabled, then invite creation works unchanged
2. [ ] Given a user joining via invite code, when UGM features are enabled, then join flow works unchanged
3. [ ] Given an expired invite code, when a user attempts to join, then 410 Gone is returned (NFR20)
4. [ ] Given invite revocation by admin, when UGM features are enabled, then revocation works unchanged
5. [ ] Given invite listing endpoint, when UGM features are enabled, then response format is unchanged

### Performance Guardrails
6. [ ] Given legacy group endpoints (pre-UGM), when measured with UGM tables present, then p95 latency increase is < 10%
7. [ ] Given `GET /api/v1/groups` endpoint, when measured, then latency is within 10% of baseline
8. [ ] Given `GET /api/v1/groups/:groupId` endpoint, when measured, then latency is within 10% of baseline
9. [ ] Given `GET /api/v1/groups/:groupId/members` endpoint, when measured, then latency is within 10% of baseline
10. [ ] Given invite endpoints, when measured with device_group_memberships table populated, then no latency regression

### Baseline Establishment
11. [ ] Given performance test suite, then baseline metrics are recorded before UGM deployment
12. [ ] Given post-deployment metrics, then comparison report shows delta from baseline
13. [ ] Given >10% regression detected, then CI/CD pipeline fails with clear report

## Technical Notes

- Performance baseline should be established with representative data volume
- Regression tests should run in CI on every PR
- Use k6 or similar for load testing
- Baseline metrics stored in `docs/performance/baseline-metrics.json`

**Performance Test Configuration:**
```javascript
// k6 test script
export const options = {
  scenarios: {
    group_list: {
      executor: 'constant-vus',
      vus: 10,
      duration: '30s',
    },
  },
  thresholds: {
    'http_req_duration{endpoint:groups_list}': ['p95<220'], // 200ms baseline + 10%
    'http_req_duration{endpoint:group_detail}': ['p95<220'],
    'http_req_duration{endpoint:group_members}': ['p95<220'],
  },
};
```

**Baseline Metrics Format:**
```json
{
  "recorded_at": "2025-12-18T00:00:00Z",
  "endpoints": {
    "/api/v1/groups": { "p50_ms": 45, "p95_ms": 180, "p99_ms": 250 },
    "/api/v1/groups/:groupId": { "p50_ms": 30, "p95_ms": 120, "p99_ms": 180 },
    "/api/v1/groups/:groupId/members": { "p50_ms": 50, "p95_ms": 200, "p99_ms": 280 }
  }
}
```

## Tasks/Subtasks

- [ ] 1. Create invite flow regression test suite
- [ ] 2. Add test for expired invite error response (NFR20)
- [ ] 3. Establish performance baseline metrics
- [ ] 4. Create k6 load test scripts for group endpoints
- [ ] 5. Add performance guardrail assertions (< 10% regression)
- [ ] 6. Create CI job for performance regression detection
- [ ] 7. Document baseline establishment process

## File List

### Files to Create

- `crates/api/tests/invite_regression_test.rs` - Invite flow regression tests
- `tests/performance/group_endpoints.js` - k6 load test script
- `docs/performance/baseline-metrics.json` - Baseline performance data
- `.github/workflows/performance-regression.yml` - CI job for perf tests

### Files to Modify

- `crates/api/tests/invites_integration.rs` - Add NFR20 expiry test

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Invite regression tests pass
- [ ] Performance baseline established
- [ ] Load tests configured with 10% thresholds
- [ ] CI pipeline includes performance check
- [ ] Code compiles without warnings
- [ ] Code passes clippy

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created from gap analysis | Dev Agent |
