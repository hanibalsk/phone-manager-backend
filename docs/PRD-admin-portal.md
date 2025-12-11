# Product Requirements Document: Admin Portal Backend API

**Version:** 1.0
**Date:** 2025-12-10
**Author:** Martin Janci
**Project:** phone-manager-backend (Admin Portal Backend API)
**Classification:** Level 4 (Platform/Enterprise)

---

## Executive Summary

This PRD defines the comprehensive Admin Portal Backend API implementation for the Phone Manager platform. The admin portal provides enterprise-grade administrative capabilities including RBAC, organization management, user administration, device fleet management, webhooks, analytics, and compliance features.

**Current State:** 31% complete (36 of 115 endpoints implemented)
**Target State:** 100% endpoint coverage with full enterprise feature set

---

## 1. Strategic Goals

| ID | Goal | Success Metric |
|----|------|----------------|
| SG-1 | Complete Admin API Coverage | 115/115 endpoints operational |
| SG-2 | Enterprise Security Compliance | RBAC, MFA, audit logging fully operational |
| SG-3 | Operational Excellence | < 200ms p95 response time, 99.9% uptime |
| SG-4 | Developer Experience | 100% OpenAPI documentation coverage |
| SG-5 | Compliance Readiness | GDPR data subject request handling |
| SG-6 | Scalable Multi-tenancy | Support 10,000+ organizations |

---

## 2. Problem Statement

### The Problem
Organizations using the Phone Manager platform lack comprehensive administrative tools to manage users, devices, policies, and compliance at enterprise scale. The current implementation covers only 31% of the required administrative functionality.

### Current Situation
- **36 endpoints implemented** out of 115 specified
- **Two complete domains missing**: App Usage (AP-8) and Admin Location Management (AP-6)
- **Critical gaps** in: User session management, MFA administration, bulk operations, GDPR compliance, audit logging
- **Architecture decision needed**: Global vs organization-scoped API paths

### Why Now
- Enterprise customers require complete admin functionality for production deployment
- Compliance requirements (GDPR, audit trails) are blocking enterprise adoption
- Competitive pressure from platforms with mature admin capabilities
- Technical debt from incomplete implementation creates maintenance burden

### Impact of Not Solving
- Lost enterprise customers due to incomplete admin features
- Compliance violations and potential regulatory penalties
- Increased support burden from manual administrative tasks
- Security risks from incomplete access control and audit capabilities

---

## 3. Functional Requirements

### 3.1 RBAC & Access Control (AP-1)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | List all available system permissions with categories | High |
| FR-1.2 | Create custom roles with permission assignments | High |
| FR-1.3 | Delete custom roles (prevent deletion of system roles) | High |

### 3.2 Organization Management (AP-2)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | Full CRUD operations for organizations | High |
| FR-2.2 | Organization usage metrics and quota tracking | High |
| FR-2.3 | Organization settings management (branding, defaults) | Medium |
| FR-2.4 | Feature toggle management per organization | Medium |
| FR-2.5 | Organization suspension and reactivation | High |

### 3.3 User Administration (AP-3)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | List users with filtering (status, role, search) | High |
| FR-3.2 | Get user details including activity summary | High |
| FR-3.3 | Create users with role assignment | High |
| FR-3.4 | Update user profile and permissions | High |
| FR-3.5 | Suspend user accounts (preserve data) | High |
| FR-3.6 | Reactivate suspended users | High |
| FR-3.7 | Trigger password reset emails | High |
| FR-3.8 | Get user MFA status | High |
| FR-3.9 | Force MFA enrollment for users | High |
| FR-3.10 | Reset user MFA (remove current method) | High |
| FR-3.11 | List user active sessions | High |
| FR-3.12 | Revoke specific user session | High |
| FR-3.13 | Revoke all user sessions (force re-login) | High |

### 3.4 Device Fleet Administration (AP-4)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | List devices with advanced filtering | High |
| FR-4.2 | Get device details with status and assignment | High |
| FR-4.3 | Update device metadata and assignment | High |
| FR-4.4 | Bulk update multiple devices | High |
| FR-4.5 | Deactivate device (soft delete) | High |
| FR-4.6 | Get device command history | Medium |
| FR-4.7 | Issue commands to devices (lock, wipe, locate) | High |

### 3.5 Groups Administration (AP-5)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1 | List groups with member counts | Medium |
| FR-5.2 | Get group details with full membership | Medium |
| FR-5.3 | Create groups with initial settings | Medium |
| FR-5.4 | Update group settings and metadata | Medium |
| FR-5.5 | Delete groups (with member handling) | Medium |
| FR-5.6 | Add/remove group members | Medium |
| FR-5.7 | Manage group invitations | Low |

### 3.6 Location & Geofence Administration (AP-6)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.1 | List admin-defined geofences | Low |
| FR-6.2 | Create organization-wide geofences | Low |
| FR-6.3 | Update geofence parameters | Low |
| FR-6.4 | Delete admin geofences | Low |
| FR-6.5 | Get location data for admin devices | Low |
| FR-6.6 | Location history queries for compliance | Medium |

### 3.7 Webhook Administration (AP-7)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-7.1 | List webhooks with delivery statistics | High |
| FR-7.2 | Create webhooks with event subscriptions | High |
| FR-7.3 | Update webhook configuration | High |
| FR-7.4 | Delete webhooks | High |
| FR-7.5 | Test webhook endpoint connectivity | High |
| FR-7.6 | Get webhook delivery logs | High |
| FR-7.7 | Retry failed webhook deliveries | High |
| FR-7.8 | Get webhook delivery statistics | Medium |

### 3.8 App Usage & Unlock Requests (AP-8)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-8.1 | Get app usage summary per device | Low |
| FR-8.2 | Get detailed app usage history | Low |
| FR-8.3 | List pending unlock requests | Low |
| FR-8.4 | Approve unlock requests | Low |
| FR-8.5 | Deny unlock requests | Low |
| FR-8.6 | Bulk process unlock requests | Low |
| FR-8.7 | App usage analytics aggregation | Low |

### 3.9 System Configuration (AP-9)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-9.1 | Get system settings | Medium |
| FR-9.2 | Update system settings | Medium |
| FR-9.3 | Manage notification templates | Medium |
| FR-9.4 | Configure rate limits | Medium |
| FR-9.5 | Manage feature flags | Medium |
| FR-9.6 | Email template management | Medium |
| FR-9.7 | System maintenance mode toggle | Medium |

### 3.10 Dashboard & Analytics (AP-10)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-10.1 | Dashboard overview metrics | Medium |
| FR-10.2 | User analytics (active, growth, retention) | Medium |
| FR-10.3 | Device analytics (enrollment, status distribution) | Medium |
| FR-10.4 | API usage analytics | Medium |
| FR-10.5 | Generate user reports | Medium |
| FR-10.6 | Generate device reports | Medium |
| FR-10.7 | Generate compliance reports | High |
| FR-10.8 | Check async report generation status | Medium |
| FR-10.9 | Download generated reports | Medium |

### 3.11 Audit & Compliance (AP-11)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-11.1 | List audit logs with filtering | High |
| FR-11.2 | Get audit log entry details | High |
| FR-11.3 | Export audit logs (CSV/JSON) | High |
| FR-11.4 | List data subject requests (GDPR) | High |
| FR-11.5 | Create data subject request | High |
| FR-11.6 | Process data subject request | High |
| FR-11.7 | Get compliance status dashboard | High |
| FR-11.8 | Generate compliance report | High |

---

## 4. Non-Functional Requirements

### 4.1 Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1 | API Response Time | < 200ms (p95) for standard operations |
| NFR-2 | Complex Queries | < 500ms (p95) for analytics and reports |
| NFR-3 | Bulk Operations | Process 100+ items within 5 seconds |
| NFR-4 | Dashboard Load | < 1 second for metrics aggregation |
| NFR-5 | Async Reports | Complete within 60 seconds for standard queries |

### 4.2 Scalability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-6 | Concurrent Sessions | Support 1,000+ concurrent admin sessions |
| NFR-7 | Organization Scale | Handle 10,000+ organizations |
| NFR-8 | Audit Log Volume | Retain 90 days with efficient querying |
| NFR-9 | Horizontal Scaling | Stateless design supporting multiple instances |

### 4.3 Availability & Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-10 | System Uptime | 99.9% availability |
| NFR-11 | Data Durability | Zero data loss for audit and compliance records |
| NFR-12 | Graceful Degradation | Non-critical features fail independently |
| NFR-13 | Recovery Time | < 5 minutes for critical service recovery |

### 4.4 Security

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-14 | Authentication | JWT RS256 with < 24 hour session timeout |
| NFR-15 | MFA Support | TOTP-based MFA for admin accounts |
| NFR-16 | Rate Limiting | Per-user limits on sensitive operations |
| NFR-17 | Encryption | TLS 1.3 in transit, AES-256 at rest |
| NFR-18 | API Key Security | SHA-256 hashed storage, prefix identification |

### 4.5 Compliance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-19 | GDPR Compliance | Data subject requests processed within 30 days |
| NFR-20 | Audit Trail | Immutable audit logs for all admin actions |
| NFR-21 | Data Retention | Configurable retention policies per organization |
| NFR-22 | Access Logging | All API access logged with user context |

### 4.6 Observability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-23 | Metrics | Prometheus metrics for all critical paths |
| NFR-24 | Tracing | Distributed tracing with correlation IDs |
| NFR-25 | Alerting | Real-time alerts for security and system anomalies |
| NFR-26 | Health Checks | Liveness and readiness probes for orchestration |

---

## 5. User Interaction & Design

### 5.1 API Design Principles

| Principle | Description |
|-----------|-------------|
| RESTful Consistency | All endpoints follow REST conventions |
| JSON Snake Case | All fields use snake_case naming |
| Cursor Pagination | Cursor-based pagination for all list endpoints |
| Structured Errors | Error responses include code, message, and details |
| Idempotency | Support idempotency keys for mutations |
| API Versioning | `/api/admin/v1/` prefix for all admin endpoints |

### 5.2 Response Formats

**Success Response:**
```json
{
  "data": { ... },
  "meta": {
    "request_id": "uuid",
    "timestamp": "ISO8601"
  }
}
```

**List Response:**
```json
{
  "data": [ ... ],
  "meta": {
    "total": 100,
    "cursor": "next_page_token",
    "has_more": true
  }
}
```

**Error Response:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Human readable message",
    "details": [ ... ]
  },
  "meta": {
    "request_id": "uuid"
  }
}
```

### 5.3 Authentication Headers

| Header | Purpose |
|--------|---------|
| `Authorization: Bearer <jwt>` | User authentication |
| `X-API-Key` | API key authentication |
| `X-Request-ID` | Request correlation |
| `X-Idempotency-Key` | Mutation idempotency |

---

## 6. Technical Constraints

### 6.1 Technology Stack

| Component | Constraint |
|-----------|------------|
| Language | Rust 1.83+ (Edition 2024) |
| Framework | Axum 0.8 with async handlers |
| Database | PostgreSQL 16 with SQLx |
| Auth | Existing JWT RS256 + API Key middleware |
| Serialization | Serde with snake_case |

### 6.2 Architecture Constraints

| Constraint | Description |
|------------|-------------|
| Organization-Scoped APIs | Use `/api/admin/v1/organizations/:org_id/*` pattern |
| Feature Toggles | B2B features gated by `PM__FEATURES__B2B_ENABLED` |
| SQLx Offline Mode | All queries must support `cargo sqlx prepare` |
| Backward Compatibility | Existing 36 endpoints must not break |
| Layered Architecture | Routes → Middleware → Services → Repositories |

### 6.3 Integration Dependencies

| Dependency | Impact |
|------------|--------|
| FCM | Push notifications require Firebase config |
| Email Service | Password reset and invitations require SMTP |
| Webhook Delivery | Existing circuit breaker pattern |

---

## 7. Architecture Decisions

### Decision 1: API Path Structure
**Decision:** Keep organization-scoped paths (`/api/admin/v1/organizations/:org_id/*`)
**Rationale:** Matches existing implementation, clearer multi-tenant semantics, simpler authorization

### Decision 2: Session Management
**Decision:** JWT + session table for revocation support
**Rationale:** Enables session revocation without token blacklisting complexity

### Decision 3: MFA Storage
**Decision:** Separate MFA table from users
**Rationale:** Supports multiple MFA methods, cleaner data model

### Decision 4: Report Generation
**Decision:** Async with job queue
**Rationale:** Prevents timeout for large reports, better UX

### Decision 5: Audit Log Storage
**Decision:** Same database initially, dedicated store as volume grows
**Rationale:** Simplifies initial implementation, clear migration path

---

## 8. Epic Overview

| Epic | Name | Endpoints | Priority | Current |
|------|------|-----------|----------|---------|
| AP-1 | RBAC & Access Control | 3 | High | 33% |
| AP-2 | Organization Management | 7 | High | 71% |
| AP-3 | User Administration | 13 | High | 31% |
| AP-4 | Device Fleet Administration | 15 | High | 53% |
| AP-5 | Groups Administration | 10 | Medium | 50% |
| AP-6 | Location & Geofence Admin | 13 | Low | 0% |
| AP-7 | Webhook Administration | 9 | High | 56% |
| AP-8 | App Usage & Unlock Requests | 13 | Low | 0% |
| AP-9 | System Configuration | 14 | Medium | 36% |
| AP-10 | Dashboard & Analytics | 10 | Medium | 30% |
| AP-11 | Audit & Compliance | 8 | High | 0% |

**Total:** 115 endpoints across 11 epics

---

## 9. Implementation Priority

### Phase 1: Core Functionality (High Priority)
1. **AP-11** Audit & Compliance - Foundation for compliance
2. **AP-3** User Administration - Critical user management
3. **AP-1** RBAC & Access Control - Security foundation
4. **AP-7** Webhook Administration - Complete webhook features

### Phase 2: Enhanced Features (Medium Priority)
5. **AP-4** Device Fleet Administration
6. **AP-2** Organization Management
7. **AP-10** Dashboard & Analytics
8. **AP-9** System Configuration
9. **AP-5** Groups Administration

### Phase 3: New Domains (Lower Priority)
10. **AP-6** Location & Geofence Admin - New domain
11. **AP-8** App Usage & Unlock Requests - New domain

---

## 10. Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| API Coverage | 100% (115/115 endpoints) | Endpoint count |
| Test Coverage | ≥80% unit, ≥70% integration | cargo tarpaulin |
| Documentation | 100% OpenAPI coverage | Spec completeness |
| Performance | All NFRs met | Load testing |
| Security | Zero high/critical vulnerabilities | Security audit |
| Compliance | GDPR readiness certification | Compliance review |

---

## 11. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Architecture mismatch with spec | Medium | Document decision, update spec to match impl |
| MFA infrastructure complexity | High | Phased rollout, start with TOTP only |
| Report generation performance | Medium | Async processing with progress tracking |
| Audit log volume growth | Medium | Partitioning strategy, retention policies |
| Breaking existing integrations | High | Strict backward compatibility, versioning |

---

## 12. Dependencies

| Dependency | Type | Status |
|------------|------|--------|
| PostgreSQL 16 | Infrastructure | Available |
| JWT RS256 keys | Configuration | Configured |
| Email service | External | Optional |
| FCM credentials | External | Optional |
| Existing auth middleware | Code | Available |
| Existing webhook delivery | Code | Available |

---

## Appendix A: Related Documents

### Uncommitted Project Documents (This PR)

| Document | Purpose | Status |
|----------|---------|--------|
| `docs/admin-api-gap-analysis.md` | Detailed endpoint gap analysis comparing spec to implementation | New |
| `docs/project-workflow-analysis.md` | Project classification and workflow routing | Modified |
| `docs/admin-portal-epics.md` | Detailed epic definitions with user stories (AP-1 to AP-11) | New |

### Existing Project Documents

| Document | Purpose |
|----------|---------|
| `CLAUDE.md` | Project technical context and conventions |
| `docs/epics.md` | Core backend epics (1-8) and extensions (15) |
| `docs/rust-backend-spec.md` | Backend API specification |
| `docs/PRD.md` | Core product requirements |
| `docs/PRD-movement-tracking.md` | Movement tracking feature PRD |

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| RBAC | Role-Based Access Control |
| MFA | Multi-Factor Authentication |
| GDPR | General Data Protection Regulation |
| DSR | Data Subject Request |
| JWT | JSON Web Token |
| FCM | Firebase Cloud Messaging |
| TOTP | Time-based One-Time Password |

---

*Document Status: APPROVED*
*Next Step: Create epics.md with detailed user stories*
