# Story 13.9: Audit Logging System

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: To Do
**Created**: 2025-12-01

---

## User Story

**As a** security administrator
**I want** comprehensive audit logs for all administrative actions
**So that** I can track who did what and when for compliance and security

## Prerequisites

- Story 13.1 complete (Organizations)
- Story 13.2 complete (Organization users)

## Acceptance Criteria

1. Migration creates `audit_logs` table: id (UUID), organization_id (FK), timestamp, actor_id, actor_type, actor_email, action, resource_type, resource_id, resource_name, changes (JSONB), metadata (JSONB), ip_address, user_agent
2. All admin endpoints automatically log operations
3. Audit entries capture before/after state for mutations
4. Actor can be: user, system, api_key
5. Actions follow format: `resource.operation` (e.g., device.assign, policy.update)
6. Changes capture old and new values for modified fields
7. Metadata includes request context (IP, user agent, request_id)
8. Logs are immutable (no updates or deletes via API)
9. Audit logging does not block request processing (async)

## Technical Notes

- Create migration 028_audit_logs.sql
- Implement as Axum middleware layer
- Use tokio::spawn for async log insertion
- Index on (organization_id, timestamp DESC) for efficient querying
- Consider partitioning by month for large-scale deployments

## Audit Log Schema

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor_id UUID,
    actor_type VARCHAR(20) NOT NULL, -- user, system, api_key
    actor_email VARCHAR(255),
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(30) NOT NULL,
    resource_id UUID,
    resource_name VARCHAR(255),
    changes JSONB,
    metadata JSONB,
    ip_address INET,
    user_agent TEXT
);

CREATE INDEX idx_audit_logs_org_timestamp
    ON audit_logs(organization_id, timestamp DESC);
CREATE INDEX idx_audit_logs_actor
    ON audit_logs(organization_id, actor_id);
CREATE INDEX idx_audit_logs_action
    ON audit_logs(organization_id, action);
CREATE INDEX idx_audit_logs_resource
    ON audit_logs(organization_id, resource_type, resource_id);
```

## Audited Actions

| Category | Actions |
|----------|---------|
| Organization | org.create, org.update, org.delete, org.settings_change |
| User | user.create, user.update, user.delete, user.role_change, user.invite |
| Device | device.register, device.enroll, device.assign, device.unassign, device.suspend, device.retire, device.wipe, device.settings_change |
| Policy | policy.create, policy.update, policy.delete, policy.apply, policy.unapply |
| Group | group.create, group.update, group.delete, group.member_add, group.member_remove |
| Settings | settings.lock, settings.unlock, settings.override, settings.bulk_update |
| Token | token.create, token.revoke, token.use |

---

## Implementation Tasks

- [ ] Create migration 028_audit_logs.sql with table and indexes
- [ ] Create AuditLogEntity in persistence layer
- [ ] Create AuditLog domain model
- [ ] Create AuditLogRepository with insert and query operations
- [ ] Create AuditService with async logging
- [ ] Create audit logging middleware/helper
- [ ] Add audit calls to all admin endpoints
- [ ] Capture changes (before/after) for mutations
- [ ] Add request context extraction (IP, user agent)
- [ ] Write unit tests for audit service
- [ ] Write integration tests for logging

---

## Dev Notes

- Audit logs are write-heavy, optimize for inserts
- Consider retention policy (e.g., 1 year) with background cleanup
- Changes JSON format: `{"field": {"old": "val1", "new": "val2"}}`
- Actor identification from JWT claims or API key

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

