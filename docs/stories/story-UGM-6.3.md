# Story UGM-6.3: Migration Analytics Dashboard

**Status**: Backlog (Post-MVP)

## Story

**As a** system administrator,
**I want** a dashboard showing migration patterns and statistics,
**So that** I can understand user behavior and identify issues.

**Epic**: UGM-6: Growth Features (Post-MVP)
**PRD Reference**: Growth Features - Migration analytics dashboard

## Acceptance Criteria

1. [ ] Given the admin dashboard, when viewing migration analytics, then total migrations (success/failure) are shown
2. [ ] Given migration analytics, then daily/weekly/monthly migration trends are displayed
3. [ ] Given migration analytics, then average devices per migration is calculated
4. [ ] Given migration analytics, then average migration duration is shown
5. [ ] Given migration analytics, then top 10 largest migrations are listed
6. [ ] Given migration analytics, then failure rate and common error types are displayed
7. [ ] Given migration analytics, then time-to-migration (registration → auth) distribution is shown
8. [ ] Given date range filter, then analytics can be filtered by time period

## Technical Notes

**Admin Endpoint:**
```
GET /api/admin/v1/analytics/migrations
Authorization: X-API-Key (admin)

Query Parameters:
- from: ISO date
- to: ISO date
- granularity: day|week|month

Response:
{
  "summary": {
    "total_migrations": 1250,
    "success_rate": 0.94,
    "avg_devices_per_migration": 3.2,
    "avg_duration_seconds": 0.8
  },
  "trends": [
    { "date": "2025-12-01", "count": 45, "success": 42, "failure": 3 }
  ],
  "top_migrations": [
    { "migration_id": "...", "devices": 25, "date": "..." }
  ],
  "error_breakdown": {
    "already_migrated": 15,
    "no_devices": 8,
    "timeout": 2
  }
}
```

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Admin endpoint implemented
- [ ] Efficient queries (indexed)
- [ ] Date range filtering works
- [ ] Dashboard UI (if applicable)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created for post-MVP backlog | Dev Agent |
