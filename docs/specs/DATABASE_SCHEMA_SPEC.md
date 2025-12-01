# Database Schema Specification

## Phone Manager Backend - User Management & Device Control

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-01
**Database:** PostgreSQL 14+

---

## 1. Overview

This document defines the complete database schema for the User Management and Device Control feature, including new tables, modifications to existing tables, indexes, constraints, and migration strategy.

---

## 2. Schema Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           DATABASE SCHEMA                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐                          ┌─────────────────┐           │
│  │  organizations  │──────────────────────────│    org_users    │           │
│  │  (B2B tenants)  │                          │  (org admins)   │           │
│  └────────┬────────┘                          └─────────────────┘           │
│           │                                                                  │
│           │ 1:N                                                              │
│           ▼                                                                  │
│  ┌─────────────────┐         ┌─────────────────┐                            │
│  │     users       │◄────────│  oauth_accounts │                            │
│  │  (all users)    │         │  (social login) │                            │
│  └────────┬────────┘         └─────────────────┘                            │
│           │                                                                  │
│           │ 1:N                    1:N                                       │
│           ▼                         │                                        │
│  ┌─────────────────┐    ┌──────────▼────────┐    ┌─────────────────┐        │
│  │  user_sessions  │    │      groups       │◄───│  group_invites  │        │
│  │ (refresh tokens)│    │  (sharing groups) │    │  (invite codes) │        │
│  └─────────────────┘    └────────┬──────────┘    └─────────────────┘        │
│                                  │                                           │
│                                  │ M:N                                       │
│                                  ▼                                           │
│                         ┌─────────────────┐                                  │
│                         │group_memberships│                                  │
│                         │  (with roles)   │                                  │
│                         └────────┬────────┘                                  │
│                                  │                                           │
│           ┌──────────────────────┼──────────────────────┐                   │
│           │                      │                      │                   │
│           ▼                      ▼                      ▼                   │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐         │
│  │     devices     │    │ device_settings │    │ device_policies │         │
│  │   (extended)    │    │  (with locks)   │    │  (B2B templates)│         │
│  └────────┬────────┘    └─────────────────┘    └─────────────────┘         │
│           │                                                                  │
│           │                      ┌─────────────────┐                        │
│           └──────────────────────│enrollment_tokens│                        │
│                                  │ (B2B provisioning)                       │
│                                  └─────────────────┘                        │
│                                                                              │
│  ┌─────────────────┐    ┌─────────────────┐                                 │
│  │   audit_logs    │    │ unlock_requests │                                 │
│  │ (admin actions) │    │(setting unlocks)│                                 │
│  └─────────────────┘    └─────────────────┘                                 │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. New Tables

### 3.1 organizations

B2B tenant organizations.

```sql
CREATE TABLE organizations (
    id              VARCHAR(26) PRIMARY KEY,  -- ULID: org_01HXYZ...
    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(100) NOT NULL UNIQUE,
    billing_email   VARCHAR(255),
    plan_type       VARCHAR(50) NOT NULL DEFAULT 'free',
    max_users       INTEGER NOT NULL DEFAULT 10,
    max_devices     INTEGER NOT NULL DEFAULT 50,
    max_groups      INTEGER NOT NULL DEFAULT 10,
    settings        JSONB NOT NULL DEFAULT '{}',
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_organizations_slug ON organizations(slug);
CREATE INDEX idx_organizations_active ON organizations(is_active) WHERE is_active = TRUE;

-- Constraints
ALTER TABLE organizations ADD CONSTRAINT chk_plan_type
    CHECK (plan_type IN ('free', 'starter', 'business', 'enterprise'));
```

### 3.2 users

User accounts for authentication.

```sql
CREATE TABLE users (
    id                  VARCHAR(26) PRIMARY KEY,  -- ULID: user_01HXYZ...
    organization_id     VARCHAR(26) REFERENCES organizations(id) ON DELETE SET NULL,
    email               VARCHAR(255) NOT NULL,
    email_verified      BOOLEAN NOT NULL DEFAULT FALSE,
    password_hash       VARCHAR(255),  -- NULL for OAuth-only users
    display_name        VARCHAR(100) NOT NULL,
    avatar_url          VARCHAR(500),
    auth_provider       VARCHAR(50) NOT NULL DEFAULT 'email',
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at       TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_users_email UNIQUE(email)
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_organization ON users(organization_id);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = TRUE;

-- Constraints
ALTER TABLE users ADD CONSTRAINT chk_auth_provider
    CHECK (auth_provider IN ('email', 'google', 'apple'));
```

### 3.3 oauth_accounts

OAuth provider account links.

```sql
CREATE TABLE oauth_accounts (
    id                  VARCHAR(26) PRIMARY KEY,
    user_id             VARCHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider            VARCHAR(50) NOT NULL,  -- 'google', 'apple'
    provider_user_id    VARCHAR(255) NOT NULL,
    email               VARCHAR(255),
    access_token        TEXT,  -- Encrypted
    refresh_token       TEXT,  -- Encrypted
    token_expires_at    TIMESTAMPTZ,
    raw_data            JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_oauth_provider_user UNIQUE(provider, provider_user_id)
);

-- Indexes
CREATE INDEX idx_oauth_user ON oauth_accounts(user_id);
CREATE INDEX idx_oauth_provider ON oauth_accounts(provider, provider_user_id);
```

### 3.4 user_sessions

Refresh token tracking for token rotation.

```sql
CREATE TABLE user_sessions (
    id                  VARCHAR(26) PRIMARY KEY,
    user_id             VARCHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id           VARCHAR(26),  -- Optional device binding
    refresh_token_hash  VARCHAR(64) NOT NULL,  -- SHA-256 hash
    token_family_id     VARCHAR(26) NOT NULL,  -- For rotation detection
    ip_address          INET,
    user_agent          TEXT,
    last_used_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at          TIMESTAMPTZ NOT NULL,
    revoked_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_sessions_user ON user_sessions(user_id);
CREATE INDEX idx_sessions_token ON user_sessions(refresh_token_hash);
CREATE INDEX idx_sessions_family ON user_sessions(token_family_id);
CREATE INDEX idx_sessions_expires ON user_sessions(expires_at) WHERE revoked_at IS NULL;
```

### 3.5 groups

User groups for sharing and permissions.

```sql
CREATE TABLE groups (
    id                  VARCHAR(26) PRIMARY KEY,  -- ULID: grp_01DEF...
    organization_id     VARCHAR(26) REFERENCES organizations(id) ON DELETE CASCADE,
    name                VARCHAR(100) NOT NULL,
    slug                VARCHAR(100) NOT NULL,
    description         TEXT,
    icon_emoji          VARCHAR(10),
    max_devices         INTEGER NOT NULL DEFAULT 20,
    settings            JSONB NOT NULL DEFAULT '{}',
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    created_by          VARCHAR(26) REFERENCES users(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_group_org_slug UNIQUE(organization_id, slug)
);

-- Indexes
CREATE INDEX idx_groups_organization ON groups(organization_id);
CREATE INDEX idx_groups_slug ON groups(slug);
CREATE INDEX idx_groups_created_by ON groups(created_by);
CREATE INDEX idx_groups_active ON groups(is_active) WHERE is_active = TRUE;
```

### 3.6 group_memberships

Group membership with roles.

```sql
CREATE TABLE group_memberships (
    id                  VARCHAR(26) PRIMARY KEY,
    group_id            VARCHAR(26) NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id             VARCHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role                VARCHAR(50) NOT NULL DEFAULT 'member',
    invited_by          VARCHAR(26) REFERENCES users(id) ON DELETE SET NULL,
    joined_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_group_user UNIQUE(group_id, user_id)
);

-- Indexes
CREATE INDEX idx_memberships_group ON group_memberships(group_id);
CREATE INDEX idx_memberships_user ON group_memberships(user_id);
CREATE INDEX idx_memberships_role ON group_memberships(role);

-- Constraints
ALTER TABLE group_memberships ADD CONSTRAINT chk_role
    CHECK (role IN ('owner', 'admin', 'member', 'viewer'));
```

### 3.7 group_invites

Group invitation codes.

```sql
CREATE TABLE group_invites (
    id                  VARCHAR(26) PRIMARY KEY,
    group_id            VARCHAR(26) NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    code                VARCHAR(20) NOT NULL UNIQUE,  -- ABC-123-XYZ
    preset_role         VARCHAR(50) NOT NULL DEFAULT 'member',
    max_uses            INTEGER NOT NULL DEFAULT 1,
    current_uses        INTEGER NOT NULL DEFAULT 0,
    expires_at          TIMESTAMPTZ NOT NULL,
    created_by          VARCHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_invites_code ON group_invites(code);
CREATE INDEX idx_invites_group ON group_invites(group_id);
CREATE INDEX idx_invites_expires ON group_invites(expires_at);
```

### 3.8 device_policies

B2B device policy templates.

```sql
CREATE TABLE device_policies (
    id                  VARCHAR(26) PRIMARY KEY,  -- ULID: pol_01MNO...
    organization_id     VARCHAR(26) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name                VARCHAR(100) NOT NULL,
    description         TEXT,
    is_default          BOOLEAN NOT NULL DEFAULT FALSE,
    settings            JSONB NOT NULL DEFAULT '{}',  -- Default setting values
    locked_settings     TEXT[] NOT NULL DEFAULT '{}',  -- Keys that are locked
    priority            INTEGER NOT NULL DEFAULT 0,
    created_by          VARCHAR(26) REFERENCES users(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_policy_org_name UNIQUE(organization_id, name)
);

-- Indexes
CREATE INDEX idx_policies_organization ON device_policies(organization_id);
CREATE INDEX idx_policies_default ON device_policies(is_default) WHERE is_default = TRUE;
```

### 3.9 device_settings

Per-device settings with lock status.

```sql
CREATE TABLE device_settings (
    id                  VARCHAR(26) PRIMARY KEY,
    device_id           VARCHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    setting_key         VARCHAR(100) NOT NULL,
    setting_value       JSONB NOT NULL,
    is_locked           BOOLEAN NOT NULL DEFAULT FALSE,
    locked_by           VARCHAR(26) REFERENCES users(id) ON DELETE SET NULL,
    locked_at           TIMESTAMPTZ,
    lock_reason         TEXT,
    updated_by          VARCHAR(26) REFERENCES users(id) ON DELETE SET NULL,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_device_setting UNIQUE(device_id, setting_key)
);

-- Indexes
CREATE INDEX idx_device_settings_device ON device_settings(device_id);
CREATE INDEX idx_device_settings_locked ON device_settings(is_locked) WHERE is_locked = TRUE;
```

### 3.10 setting_definitions

Catalog of lockable settings.

```sql
CREATE TABLE setting_definitions (
    key                 VARCHAR(100) PRIMARY KEY,
    display_name        VARCHAR(100) NOT NULL,
    description         TEXT,
    data_type           VARCHAR(50) NOT NULL,
    default_value       JSONB NOT NULL,
    validation          JSONB,  -- Validation rules
    is_lockable         BOOLEAN NOT NULL DEFAULT TRUE,
    category            VARCHAR(50) NOT NULL DEFAULT 'general',
    sort_order          INTEGER NOT NULL DEFAULT 0
);

-- Seed data
INSERT INTO setting_definitions (key, display_name, description, data_type, default_value, is_lockable, category, sort_order) VALUES
    ('tracking_enabled', 'Location Tracking', 'Enable or disable location tracking', 'boolean', 'true', TRUE, 'tracking', 1),
    ('tracking_interval_minutes', 'Tracking Interval', 'Minutes between location updates', 'integer', '5', TRUE, 'tracking', 2),
    ('high_accuracy_mode', 'High Accuracy Mode', 'Use GPS for higher accuracy', 'boolean', 'true', TRUE, 'tracking', 3),
    ('secret_mode_enabled', 'Secret Mode', 'Minimize visibility of tracking', 'boolean', 'false', TRUE, 'privacy', 10),
    ('show_weather_notification', 'Weather in Notification', 'Show weather in tracking notification', 'boolean', 'true', FALSE, 'notifications', 20),
    ('movement_detection_enabled', 'Movement Detection', 'Detect transportation mode changes', 'boolean', 'true', TRUE, 'movement', 30),
    ('trip_detection_enabled', 'Trip Detection', 'Automatically detect and track trips', 'boolean', 'true', TRUE, 'movement', 31),
    ('geofence_enabled', 'Geofencing', 'Enable geofence alerts', 'boolean', 'true', TRUE, 'geofence', 40),
    ('proximity_alerts_enabled', 'Proximity Alerts', 'Enable proximity alerts', 'boolean', 'true', TRUE, 'alerts', 50);
```

### 3.11 unlock_requests

Requests to unlock settings.

```sql
CREATE TABLE unlock_requests (
    id                  VARCHAR(26) PRIMARY KEY,
    device_id           VARCHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    setting_key         VARCHAR(100) NOT NULL,
    requested_by        VARCHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason              TEXT,
    status              VARCHAR(50) NOT NULL DEFAULT 'pending',
    decided_by          VARCHAR(26) REFERENCES users(id) ON DELETE SET NULL,
    decided_at          TIMESTAMPTZ,
    decision_note       TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_unlock_requests_device ON unlock_requests(device_id);
CREATE INDEX idx_unlock_requests_status ON unlock_requests(status);
CREATE INDEX idx_unlock_requests_pending ON unlock_requests(status, created_at) WHERE status = 'pending';

-- Constraints
ALTER TABLE unlock_requests ADD CONSTRAINT chk_status
    CHECK (status IN ('pending', 'approved', 'denied'));
```

### 3.12 enrollment_tokens

B2B device provisioning tokens.

```sql
CREATE TABLE enrollment_tokens (
    id                  VARCHAR(26) PRIMARY KEY,
    organization_id     VARCHAR(26) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    group_id            VARCHAR(26) REFERENCES groups(id) ON DELETE SET NULL,
    policy_id           VARCHAR(26) REFERENCES device_policies(id) ON DELETE SET NULL,
    token_hash          VARCHAR(64) NOT NULL UNIQUE,  -- SHA-256 hash
    token_prefix        VARCHAR(8) NOT NULL,  -- For identification
    max_uses            INTEGER NOT NULL DEFAULT 1,
    current_uses        INTEGER NOT NULL DEFAULT 0,
    expires_at          TIMESTAMPTZ NOT NULL,
    created_by          VARCHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_enrollment_tokens_hash ON enrollment_tokens(token_hash);
CREATE INDEX idx_enrollment_tokens_org ON enrollment_tokens(organization_id);
CREATE INDEX idx_enrollment_tokens_expires ON enrollment_tokens(expires_at);
```

### 3.13 device_tokens

Long-lived tokens for B2B managed devices.

```sql
CREATE TABLE device_tokens (
    id                  VARCHAR(26) PRIMARY KEY,
    device_id           VARCHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    token_hash          VARCHAR(64) NOT NULL,  -- SHA-256 hash
    token_prefix        VARCHAR(8) NOT NULL,
    last_used_at        TIMESTAMPTZ,
    expires_at          TIMESTAMPTZ NOT NULL,
    revoked_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_device_tokens_hash ON device_tokens(token_hash);
CREATE INDEX idx_device_tokens_device ON device_tokens(device_id);
CREATE INDEX idx_device_tokens_expires ON device_tokens(expires_at) WHERE revoked_at IS NULL;
```

### 3.14 audit_logs

Audit trail for admin actions.

```sql
CREATE TABLE audit_logs (
    id                  VARCHAR(26) PRIMARY KEY,
    organization_id     VARCHAR(26) REFERENCES organizations(id) ON DELETE SET NULL,
    actor_id            VARCHAR(26),
    actor_type          VARCHAR(50) NOT NULL,  -- 'user', 'device', 'system', 'admin'
    actor_email         VARCHAR(255),
    action              VARCHAR(100) NOT NULL,  -- 'device.register', 'settings.lock'
    resource_type       VARCHAR(100) NOT NULL,  -- 'device', 'group', 'user'
    resource_id         VARCHAR(26),
    resource_name       VARCHAR(255),
    changes             JSONB,  -- Before/after values
    metadata            JSONB DEFAULT '{}',
    ip_address          INET,
    user_agent          TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_audit_organization ON audit_logs(organization_id);
CREATE INDEX idx_audit_actor ON audit_logs(actor_id);
CREATE INDEX idx_audit_action ON audit_logs(action);
CREATE INDEX idx_audit_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);

-- Partition by month for large deployments (optional)
-- CREATE TABLE audit_logs_2025_01 PARTITION OF audit_logs
--     FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
```

### 3.15 org_users

Organization admin users (separate from regular users).

```sql
CREATE TABLE org_users (
    id                  VARCHAR(26) PRIMARY KEY,
    organization_id     VARCHAR(26) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id             VARCHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role                VARCHAR(50) NOT NULL DEFAULT 'admin',
    permissions         TEXT[] NOT NULL DEFAULT '{}',
    granted_by          VARCHAR(26) REFERENCES users(id) ON DELETE SET NULL,
    granted_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_org_user UNIQUE(organization_id, user_id)
);

-- Indexes
CREATE INDEX idx_org_users_organization ON org_users(organization_id);
CREATE INDEX idx_org_users_user ON org_users(user_id);

-- Constraints
ALTER TABLE org_users ADD CONSTRAINT chk_org_role
    CHECK (role IN ('owner', 'admin', 'viewer'));
```

---

## 4. Modified Existing Tables

### 4.1 devices (Extended)

Add columns to existing devices table.

```sql
-- New columns
ALTER TABLE devices ADD COLUMN IF NOT EXISTS owner_user_id VARCHAR(26) REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE devices ADD COLUMN IF NOT EXISTS organization_id VARCHAR(26) REFERENCES organizations(id) ON DELETE SET NULL;
ALTER TABLE devices ADD COLUMN IF NOT EXISTS policy_id VARCHAR(26) REFERENCES device_policies(id) ON DELETE SET NULL;
ALTER TABLE devices ADD COLUMN IF NOT EXISTS is_managed BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE devices ADD COLUMN IF NOT EXISTS enrollment_status VARCHAR(50) NOT NULL DEFAULT 'enrolled';
ALTER TABLE devices ADD COLUMN IF NOT EXISTS effective_settings JSONB DEFAULT '{}';
ALTER TABLE devices ADD COLUMN IF NOT EXISTS last_settings_sync_at TIMESTAMPTZ;

-- New indexes
CREATE INDEX IF NOT EXISTS idx_devices_owner ON devices(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_devices_organization ON devices(organization_id);
CREATE INDEX IF NOT EXISTS idx_devices_managed ON devices(is_managed) WHERE is_managed = TRUE;

-- Constraints
ALTER TABLE devices ADD CONSTRAINT chk_enrollment_status
    CHECK (enrollment_status IN ('pending', 'enrolled', 'suspended', 'retired'));
```

### 4.2 api_keys (Extended)

Add user/organization scoping to existing api_keys.

```sql
-- New columns
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS user_id VARCHAR(26) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS organization_id VARCHAR(26) REFERENCES organizations(id) ON DELETE CASCADE;
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS scopes TEXT[] NOT NULL DEFAULT '{}';

-- New indexes
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_organization ON api_keys(organization_id);
```

---

## 5. Migration Strategy

### 5.1 Migration Order

```
1. organizations (no dependencies)
2. users (depends on organizations)
3. oauth_accounts (depends on users)
4. user_sessions (depends on users)
5. groups (depends on organizations, users)
6. group_memberships (depends on groups, users)
7. group_invites (depends on groups, users)
8. device_policies (depends on organizations, users)
9. devices modifications (depends on users, organizations, device_policies)
10. device_settings (depends on devices, users)
11. setting_definitions (no dependencies)
12. unlock_requests (depends on devices, users)
13. enrollment_tokens (depends on organizations, groups, device_policies, users)
14. device_tokens (depends on devices)
15. audit_logs (depends on organizations)
16. org_users (depends on organizations, users)
17. api_keys modifications (depends on users, organizations)
```

### 5.2 Migration Files

```
migrations/
├── 015_create_organizations.sql
├── 016_create_users.sql
├── 017_create_oauth_accounts.sql
├── 018_create_user_sessions.sql
├── 019_create_groups.sql
├── 020_create_group_memberships.sql
├── 021_create_group_invites.sql
├── 022_create_device_policies.sql
├── 023_alter_devices.sql
├── 024_create_device_settings.sql
├── 025_create_setting_definitions.sql
├── 026_create_unlock_requests.sql
├── 027_create_enrollment_tokens.sql
├── 028_create_device_tokens.sql
├── 029_create_audit_logs.sql
├── 030_create_org_users.sql
└── 031_alter_api_keys.sql
```

### 5.3 Backward Compatibility

- Existing devices continue to work with `owner_user_id = NULL`
- API key authentication remains functional alongside JWT
- No data migration required for existing devices
- Groups replace implicit `group_id` string on devices

---

## 6. Indexes Summary

| Table | Index | Columns | Purpose |
|-------|-------|---------|---------|
| users | idx_users_email | email | Login lookup |
| users | idx_users_organization | organization_id | Org queries |
| user_sessions | idx_sessions_token | refresh_token_hash | Token validation |
| groups | idx_groups_slug | slug | URL lookup |
| group_memberships | idx_memberships_user | user_id | User's groups |
| device_settings | idx_device_settings_device | device_id | Settings lookup |
| audit_logs | idx_audit_created | created_at DESC | Recent logs |

---

## 7. Constraints Summary

| Table | Constraint | Type | Description |
|-------|------------|------|-------------|
| users | uq_users_email | UNIQUE | One account per email |
| groups | uq_group_org_slug | UNIQUE | Unique slug per org |
| group_memberships | uq_group_user | UNIQUE | One membership per group |
| device_settings | uq_device_setting | UNIQUE | One value per setting per device |
| users | chk_auth_provider | CHECK | Valid auth providers |
| group_memberships | chk_role | CHECK | Valid roles |

---

## 8. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-01 | Initial schema specification |
