# Story UGM-5.6: Enhanced Migration and Device Membership Metrics

**Status**: Ready for Development

## Story

**As a** system operator,
**I want** detailed metrics for migrations and device-group memberships,
**So that** I can monitor system health, identify trends, and troubleshoot issues.

**Epic**: UGM-5: NFR Compliance
**Prerequisites**: Story UGM-2.3 (Basic Migration Metrics)
**PRD Reference**: Metrics section specifying histogram buckets and gauges

## Acceptance Criteria

### Migration Metrics Enhancement
1. [ ] Given the `migration_duration_seconds` histogram, then it includes buckets: [0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]
2. [ ] Given the `migration_duration_seconds` histogram, then it includes label `device_count_bucket` with values: ["1-5", "6-20", "21-50", "51+"]
3. [ ] Given the `migration_total` counter, then it includes labels: `status` (success/failure/conflict), `source` (api/background)
4. [ ] Given a migration with 25 devices, then `migration_devices_migrated` histogram records 25 with appropriate bucket

### Device-Group Membership Metrics (New)
5. [ ] Given the metrics endpoint, then it exposes `device_group_memberships_total` gauge
6. [ ] Given `device_group_memberships_total`, then it has label `group_type` with values: ["authenticated", "registration"]
7. [ ] Given devices added/removed from groups, then `device_group_memberships_total` gauge is updated in real-time
8. [ ] Given the metrics endpoint, then it exposes `devices_per_group` histogram with buckets: [1, 5, 10, 20, 50, 100]

### Latency Metrics
9. [ ] Given migration endpoint calls, then `http_request_duration_seconds` histogram includes label `endpoint="/api/v1/groups/migrate"`
10. [ ] Given device-group endpoints, then latency is tracked with appropriate endpoint labels

### Dashboard Queries (Validation)
11. [ ] Given Prometheus, when querying `histogram_quantile(0.95, migration_duration_seconds)`, then p95 latency is calculable
12. [ ] Given Prometheus, when querying `sum(device_group_memberships_total) by (group_type)`, then membership totals are accurate

## Technical Notes

- Use existing Prometheus metrics infrastructure in `crates/api/src/middleware/metrics.rs`
- Histogram buckets follow Prometheus best practices for latency (powers of 2/10)
- Device count buckets based on typical family group sizes

**Metric Definitions:**
```rust
// Migration duration with device count bucketing
static MIGRATION_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    HistogramVec::new(
        HistogramOpts::new(
            "migration_duration_seconds",
            "Time taken to complete group migration"
        ).buckets(vec![0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]),
        &["device_count_bucket"]
    ).unwrap()
});

// Device-group membership gauge
static DEVICE_GROUP_MEMBERSHIPS: Lazy<IntGaugeVec> = Lazy::new(|| {
    IntGaugeVec::new(
        Opts::new(
            "device_group_memberships_total",
            "Total number of device-group memberships"
        ),
        &["group_type"]
    ).unwrap()
});
```

## Tasks/Subtasks

- [ ] 1. Add histogram buckets to migration duration metric
- [ ] 2. Add device_count_bucket label to migration metrics
- [ ] 3. Create device_group_memberships_total gauge
- [ ] 4. Update gauge on add/remove device operations
- [ ] 5. Add devices_per_group histogram
- [ ] 6. Create Grafana dashboard JSON for UGM metrics
- [ ] 7. Add integration test verifying metrics are exposed
- [ ] 8. Document metric queries in runbook

## File List

### Files to Modify

- `crates/api/src/middleware/metrics.rs` - Add new metrics
- `crates/api/src/routes/groups.rs` - Record metrics on operations
- `crates/persistence/src/repositories/device_group_membership.rs` - Update gauge on DB operations

### Files to Create

- `docs/monitoring/ugm-metrics-dashboard.json` - Grafana dashboard
- `docs/monitoring/ugm-metrics-runbook.md` - Query examples and alerts

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Metrics visible at `/metrics` endpoint
- [ ] Histogram buckets properly configured
- [ ] Gauge updates on membership changes
- [ ] Grafana dashboard created
- [ ] Code compiles without warnings
- [ ] Code passes clippy

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created from gap analysis | Dev Agent |
