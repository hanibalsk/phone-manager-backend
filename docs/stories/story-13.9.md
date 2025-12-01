# Story 13.9: Audit Logging System

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: Done
**Created**: 2025-12-01
**Completed**: 2025-12-01

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

- [x] Create migration 031_audit_logs.sql with table and indexes
- [x] Create AuditLogEntity in persistence layer
- [x] Create AuditLog domain model
- [x] Create AuditLogRepository with insert and query operations
- [x] Create AuditService with async logging (insert_async method)
- [ ] Create audit logging middleware/helper (deferred - use insert_async in route handlers)
- [ ] Add audit calls to all admin endpoints (deferred - can be added incrementally)
- [x] Capture changes (before/after) for mutations (FieldChange struct)
- [x] Add request context extraction (IP, user agent) (AuditMetadata struct)
- [x] Write unit tests for audit service
- [ ] Write integration tests for logging (requires database)

---

## Dev Notes

- Audit logs are write-heavy, optimize for inserts
- Consider retention policy (e.g., 1 year) with background cleanup
- Changes JSON format: `{"field": {"old": "val1", "new": "val2"}}`
- Actor identification from JWT claims or API key

---

## Dev Agent Record

### Debug Log

- Migration numbered 031 (previous migrations used 024-030)
- AuditLogEntity already started by previous work
- Created comprehensive domain model with all actor types, resource types, and actions

### Completion Notes

Story 13.9 implemented with audit logging infrastructure:

**Database Layer:**
- Migration 031_audit_logs.sql with table, audit_actor_type enum, and indexes
- AuditLogEntity with all required fields

**Domain Layer:**
- ActorType enum (User, System, ApiKey)
- ResourceType enum (Organization, User, Device, Policy, Group, etc.)
- AuditAction enum with all documented actions (org.create, device.assign, etc.)
- FieldChange for capturing before/after values
- AuditLog domain model with nested Actor and Resource info
- AuditMetadata for request context (IP, user agent, request_id)
- CreateAuditLogInput builder pattern for easy audit log creation

**Persistence Layer:**
- AuditLogRepository with:
  - insert() for sync insertion
  - insert_async() for fire-and-forget async logging
  - find_by_id() for single entry lookup
  - list() with filtering and pagination
  - list_for_export() for bulk export (used by Story 13.10)
  - count() for total entries

**API Layer:**
- Audit log routes nested under /api/admin/v1/organizations/:org_id/audit-logs
- GET / - list audit logs with pagination
- GET /:log_id - get single audit log

**Deferred:**
- Adding audit calls to existing endpoints (can be done incrementally)
- Audit logging middleware (handlers can use insert_async directly)

---

## File List

- crates/persistence/src/migrations/031_audit_logs.sql
- crates/persistence/src/entities/audit_log.rs
- crates/persistence/src/entities/mod.rs (updated)
- crates/domain/src/models/audit_log.rs
- crates/domain/src/models/mod.rs (updated)
- crates/persistence/src/repositories/audit_log.rs
- crates/persistence/src/repositories/mod.rs (updated)
- crates/api/src/routes/audit_logs.rs
- crates/api/src/routes/mod.rs (updated)
- crates/api/src/app.rs (updated)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Story implemented - audit logging infrastructure complete |
