# Project Workflow Analysis

**Date:** 2025-11-25
**Project:** phone-manager-backend
**Analyst:** Martin Janci

## Assessment Results

### Project Classification

- **Project Type:** Backend service/API (Rust Axum)
- **Project Level:** Level 3 (Full Product)
- **Instruction Set:** instructions-lg.md (Large project PRD workflow)

### Scope Summary

- **Brief Description:** Phone Manager Rust backend API server for mobile application. Handles device registration, location tracking, group management, and real-time location sharing for family/friends use cases.
- **Estimated Stories:** 20-30 stories
- **Estimated Epics:** 4 epics (Foundation, Location Tracking, Production Readiness, Future Enhancements)
- **Timeline:** 6 weeks (based on existing spec phases)

### Context

- **Greenfield/Brownfield:** Greenfield (new project)
- **Existing Documentation:** Technical specification (rust-backend-spec.md)
- **Team Size:** Small (1-3 developers)
- **Deployment Intent:** Production deployment with Docker/Kubernetes, Supabase option for minimal deployments

## Recommended Workflow Path

### Primary Outputs

1. **PRD.md** — Product Requirements Document with business context, user personas, success metrics
2. **epics.md** — Epic breakdown with prioritized user stories
3. **Architecture handoff** — Route to 3-solutioning workflow for architecture.md

### Workflow Sequence

1. ✅ Project Assessment (completed)
2. ✅ PRD Creation (completed)
   - Product overview and vision
   - User personas and journeys
   - Functional requirements (26 FRs)
   - Non-functional requirements (17 NFRs)
   - Success metrics and KPIs
3. ✅ Epic Definition (completed)
   - Break down into epics (4 epics)
   - Create user stories per epic (33 stories)
   - Define acceptance criteria
4. ✅ Solution Architecture (completed)
   - Modular monolith architecture
   - Technology stack decisions
   - Component boundaries and integration
   - Implementation guidance

### Next Actions

1. Proceed with PRD workflow (instructions-lg.md)
2. Extract business requirements from existing technical spec
3. Define user personas (family members, parents, etc.)
4. Establish success metrics and KPIs
5. Create epic breakdown aligned with implementation phases

## Special Considerations

- **Existing Technical Spec**: Comprehensive rust-backend-spec.md exists with API design, database schema, and implementation phases. PRD should reference and align with this spec rather than redefine technical details.
- **Mobile Client Dependency**: Backend serves Android client — PRD should consider client-side requirements.
- **Privacy/Security**: Location tracking requires clear privacy policy and data handling documentation.
- **Minimal Stack Option**: Spec includes Supabase option for small deployments — PRD should address deployment tiers.

## Technical Preferences Captured

- **Language**: Rust 1.83+ (Edition 2024)
- **Framework**: Axum 0.8 with Tokio async runtime
- **Database**: PostgreSQL 16 with SQLx (compile-time checked queries)
- **Authentication**: API key-based (SHA-256 hashed)
- **Observability**: Tracing + Prometheus metrics
- **Deployment**: Docker containers, Kubernetes-ready
- **Performance Targets**: <200ms p95 response time, 99.9% uptime, 10K+ concurrent connections

---

_This analysis serves as the routing decision for the adaptive PRD workflow and will be referenced by future orchestration workflows._
