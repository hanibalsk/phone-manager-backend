# Data Models Specification

## Phone Manager - User Management & Device Control

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-01

---

## 1. Overview

This document defines the domain models, API contracts, enums, and validation rules shared between the Android frontend and Rust backend for the User Management and Device Control feature.

---

## 2. Core Domain Models

### 2.1 User

Represents an authenticated user account.

```kotlin
// Android (Kotlin)
data class User(
    val id: String,                      // ULID: user_01HXYZ...
    val email: String,
    val displayName: String,
    val avatarUrl: String?,
    val emailVerified: Boolean,
    val authProvider: AuthProvider,
    val organizationId: String?,         // B2B: Organization membership
    val isActive: Boolean,
    val createdAt: Instant,
    val updatedAt: Instant,
    val lastLoginAt: Instant?
)

enum class AuthProvider {
    EMAIL,      // Email/password authentication
    GOOGLE,     // Google OAuth
    APPLE       // Apple Sign-In
}
```

```rust
// Backend (Rust)
pub struct User {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub auth_provider: AuthProvider,
    pub organization_id: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthProvider {
    Email,
    Google,
    Apple,
}
```

### 2.2 Organization (B2B)

Represents a business/enterprise tenant.

```kotlin
// Android (Kotlin)
data class Organization(
    val id: String,                      // ULID: org_01ABC...
    val name: String,
    val slug: String,                    // URL-safe identifier
    val planType: PlanType,
    val maxUsers: Int,
    val maxDevices: Int,
    val maxGroups: Int,
    val isActive: Boolean,
    val createdAt: Instant,
    val updatedAt: Instant
)

enum class PlanType {
    FREE,
    STARTER,
    BUSINESS,
    ENTERPRISE
}
```

```rust
// Backend (Rust)
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub plan_type: PlanType,
    pub max_users: i32,
    pub max_devices: i32,
    pub max_groups: i32,
    pub settings: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanType {
    Free,
    Starter,
    Business,
    Enterprise,
}
```

### 2.3 Group

Represents a group of users who share location/device access.

```kotlin
// Android (Kotlin)
data class Group(
    val id: String,                      // ULID: grp_01DEF...
    val organizationId: String?,         // B2B: Parent organization
    val name: String,
    val slug: String,
    val description: String?,
    val iconEmoji: String?,              // Group icon/emoji
    val maxDevices: Int,
    val memberCount: Int,
    val deviceCount: Int,
    val isActive: Boolean,
    val createdBy: String,               // User ID of creator
    val createdAt: Instant,
    val updatedAt: Instant
)
```

```rust
// Backend (Rust)
pub struct Group {
    pub id: String,
    pub organization_id: Option<String>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub max_devices: i32,
    pub settings: serde_json::Value,
    pub is_active: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 2.4 GroupMembership

Represents a user's membership in a group with their role.

```kotlin
// Android (Kotlin)
data class GroupMembership(
    val id: String,
    val groupId: String,
    val userId: String,
    val role: GroupRole,
    val invitedBy: String?,
    val joinedAt: Instant
)

enum class GroupRole {
    OWNER,      // Full control, can delete group
    ADMIN,      // Can manage members and settings
    MEMBER,     // Can view locations, manage own devices
    VIEWER      // Read-only access to locations
}
```

```rust
// Backend (Rust)
pub struct GroupMembership {
    pub id: String,
    pub group_id: String,
    pub user_id: String,
    pub role: GroupRole,
    pub invited_by: Option<String>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupRole {
    Owner,
    Admin,
    Member,
    Viewer,
}
```

### 2.5 GroupMember (View Model)

Combined view of a group member for display purposes.

```kotlin
// Android (Kotlin)
data class GroupMember(
    val user: User,
    val membership: GroupMembership,
    val devices: List<DeviceSummary>
)

data class DeviceSummary(
    val id: String,
    val deviceId: String,               // UUID
    val displayName: String,
    val platform: String,
    val lastSeenAt: Instant?,
    val lastLocation: LastLocation?,
    val isOnline: Boolean
)

data class LastLocation(
    val latitude: Double,
    val longitude: Double,
    val accuracy: Float,
    val timestamp: Instant
)
```

### 2.6 GroupInvite

Invitation to join a group.

```kotlin
// Android (Kotlin)
data class GroupInvite(
    val id: String,
    val groupId: String,
    val code: String,                   // Invite code: ABC-123-XYZ
    val presetRole: GroupRole,          // Role when user joins
    val maxUses: Int,
    val currentUses: Int,
    val expiresAt: Instant,
    val createdBy: String,
    val createdAt: Instant,
    val isExpired: Boolean,
    val isFullyUsed: Boolean
)
```

```rust
// Backend (Rust)
pub struct GroupInvite {
    pub id: String,
    pub group_id: String,
    pub code: String,
    pub preset_role: GroupRole,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: DateTime<Utc>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}
```

### 2.7 Device (Extended)

Extended device model with user ownership and organization binding.

```kotlin
// Android (Kotlin)
data class Device(
    val id: String,                      // ULID: dev_01GHI...
    val deviceUuid: String,              // Hardware UUID
    val displayName: String,
    val platform: String,                // "android", "ios"
    val deviceInfo: DeviceInfo,

    // Ownership
    val ownerUserId: String?,            // Primary owner
    val organizationId: String?,         // B2B: Organization

    // Management
    val isManaged: Boolean,              // B2B: Company-managed device
    val policyId: String?,               // Applied policy
    val enrollmentStatus: EnrollmentStatus,

    // Location
    val lastSeenAt: Instant?,
    val lastLatitude: Double?,
    val lastLongitude: Double?,

    // Status
    val isActive: Boolean,
    val createdAt: Instant,
    val updatedAt: Instant
)

data class DeviceInfo(
    val manufacturer: String?,
    val model: String?,
    val osVersion: String?,
    val appVersion: String?
)

enum class EnrollmentStatus {
    PENDING,        // Awaiting enrollment completion
    ENROLLED,       // Successfully enrolled
    SUSPENDED,      // Temporarily suspended
    RETIRED         // Permanently deactivated
}
```

```rust
// Backend (Rust)
pub struct Device {
    pub id: String,
    pub device_uuid: Uuid,
    pub display_name: String,
    pub platform: String,
    pub device_info: serde_json::Value,
    pub owner_user_id: Option<String>,
    pub organization_id: Option<String>,
    pub is_managed: bool,
    pub policy_id: Option<String>,
    pub enrollment_status: EnrollmentStatus,
    pub fcm_token: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub last_latitude: Option<f64>,
    pub last_longitude: Option<f64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentStatus {
    Pending,
    Enrolled,
    Suspended,
    Retired,
}
```

### 2.8 DeviceSettings

Device settings with lock status.

```kotlin
// Android (Kotlin)
data class DeviceSettings(
    val deviceId: String,
    val settings: Map<String, SettingValue>,
    val locks: Map<String, SettingLock>,
    val managedByGroupId: String?,       // If managed by a group admin
    val policyId: String?,               // B2B: Applied policy
    val lastSyncedAt: Instant
)

data class SettingValue(
    val key: String,
    val value: Any,                      // Boolean, Int, String, etc.
    val dataType: SettingDataType,
    val updatedAt: Instant,
    val updatedBy: String?               // User ID who last changed
)

data class SettingLock(
    val key: String,
    val isLocked: Boolean,
    val lockedBy: String?,               // User ID who locked
    val lockedAt: Instant?,
    val reason: String?                  // Optional lock reason
)

enum class SettingDataType {
    BOOLEAN,
    INTEGER,
    STRING,
    ENUM
}
```

```rust
// Backend (Rust)
pub struct DeviceSettings {
    pub device_id: String,
    pub settings: HashMap<String, SettingValue>,
    pub locks: HashMap<String, SettingLock>,
    pub managed_by_group_id: Option<String>,
    pub policy_id: Option<String>,
    pub last_synced_at: DateTime<Utc>,
}

pub struct SettingValue {
    pub key: String,
    pub value: serde_json::Value,
    pub data_type: SettingDataType,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<String>,
}

pub struct SettingLock {
    pub key: String,
    pub is_locked: bool,
    pub locked_by: Option<String>,
    pub locked_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingDataType {
    Boolean,
    Integer,
    String,
    Enum,
}
```

### 2.9 DevicePolicy (B2B)

Policy template for managed devices.

```kotlin
// Android (Kotlin)
data class DevicePolicy(
    val id: String,
    val organizationId: String,
    val name: String,
    val description: String?,
    val isDefault: Boolean,
    val settings: Map<String, Any>,      // Default setting values
    val lockedSettings: Set<String>,     // Keys that are locked
    val createdBy: String,
    val createdAt: Instant,
    val updatedAt: Instant
)
```

```rust
// Backend (Rust)
pub struct DevicePolicy {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub settings: serde_json::Value,
    pub locked_settings: Vec<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 2.10 UnlockRequest

Request from user to unlock a setting.

```kotlin
// Android (Kotlin)
data class UnlockRequest(
    val id: String,
    val deviceId: String,
    val settingKey: String,
    val requestedBy: String,             // User ID
    val reason: String?,
    val status: UnlockRequestStatus,
    val decidedBy: String?,              // Admin who decided
    val decidedAt: Instant?,
    val createdAt: Instant
)

enum class UnlockRequestStatus {
    PENDING,
    APPROVED,
    DENIED
}
```

---

## 3. Authentication Models

### 3.1 Auth Tokens

```kotlin
// Android (Kotlin)
data class AuthTokens(
    val accessToken: String,
    val refreshToken: String,
    val tokenType: String,               // "Bearer"
    val expiresIn: Long,                 // Seconds until expiry
    val expiresAt: Instant               // Computed expiry time
)

data class DeviceToken(
    val token: String,
    val expiresAt: Instant
)
```

### 3.2 User Session

```kotlin
// Android (Kotlin)
data class UserSession(
    val user: User,
    val tokens: AuthTokens,
    val deviceId: String,
    val isAuthenticated: Boolean
)
```

---

## 4. API Request/Response Contracts

### 4.1 Authentication

#### Register

```kotlin
// Request
data class RegisterRequest(
    val email: String,
    val password: String,
    val displayName: String,
    val deviceId: String,
    val deviceName: String
)

// Response
data class RegisterResponse(
    val user: User,
    val tokens: AuthTokens,
    val requiresEmailVerification: Boolean
)
```

#### Login

```kotlin
// Request
data class LoginRequest(
    val email: String,
    val password: String,
    val deviceId: String,
    val deviceName: String
)

// Response
data class LoginResponse(
    val user: User,
    val tokens: AuthTokens
)
```

#### OAuth

```kotlin
// Request
data class OAuthRequest(
    val provider: AuthProvider,
    val idToken: String,                 // OAuth ID token
    val deviceId: String,
    val deviceName: String
)

// Response: Same as LoginResponse
```

#### Refresh Token

```kotlin
// Request
data class RefreshTokenRequest(
    val refreshToken: String
)

// Response
data class RefreshTokenResponse(
    val accessToken: String,
    val expiresIn: Long
)
```

### 4.2 User Management

#### Update Profile

```kotlin
// Request
data class UpdateProfileRequest(
    val displayName: String?,
    val avatarUrl: String?
)

// Response: User
```

#### Link Device

```kotlin
// Request
data class LinkDeviceRequest(
    val deviceId: String,
    val displayName: String
)

// Response
data class LinkDeviceResponse(
    val device: Device,
    val linked: Boolean
)
```

### 4.3 Group Management

#### Create Group

```kotlin
// Request
data class CreateGroupRequest(
    val name: String,
    val description: String?,
    val iconEmoji: String?
)

// Response: Group
```

#### Create Invite

```kotlin
// Request
data class CreateInviteRequest(
    val presetRole: GroupRole,
    val maxUses: Int,                    // Default: 1
    val expiresInHours: Int              // Default: 24
)

// Response: GroupInvite
```

#### Join Group

```kotlin
// Request
data class JoinGroupRequest(
    val inviteCode: String
)

// Response
data class JoinGroupResponse(
    val group: Group,
    val membership: GroupMembership
)
```

#### Update Member Role

```kotlin
// Request
data class UpdateMemberRoleRequest(
    val role: GroupRole
)

// Response: GroupMembership
```

### 4.4 Device Settings

#### Get Settings

```kotlin
// Response
data class GetSettingsResponse(
    val settings: DeviceSettings,
    val availableSettings: List<SettingDefinition>
)

data class SettingDefinition(
    val key: String,
    val displayName: String,
    val description: String?,
    val dataType: SettingDataType,
    val defaultValue: Any,
    val isLockable: Boolean,
    val category: String,
    val validValues: List<Any>?          // For enum types
)
```

#### Update Settings

```kotlin
// Request
data class UpdateSettingsRequest(
    val settings: Map<String, Any>
)

// Response: DeviceSettings
```

#### Update Locks

```kotlin
// Request
data class UpdateLocksRequest(
    val locks: Map<String, Boolean>,     // key -> isLocked
    val reason: String?
)

// Response: DeviceSettings
```

#### Request Unlock

```kotlin
// Request
data class RequestUnlockRequest(
    val settingKey: String,
    val reason: String?
)

// Response: UnlockRequest
```

---

## 5. Enums Reference

### 5.1 Complete Enum List

| Enum | Values | Description |
|------|--------|-------------|
| `AuthProvider` | EMAIL, GOOGLE, APPLE | Authentication method |
| `PlanType` | FREE, STARTER, BUSINESS, ENTERPRISE | B2B subscription tiers |
| `GroupRole` | OWNER, ADMIN, MEMBER, VIEWER | Group membership roles |
| `EnrollmentStatus` | PENDING, ENROLLED, SUSPENDED, RETIRED | Device enrollment state |
| `SettingDataType` | BOOLEAN, INTEGER, STRING, ENUM | Setting value types |
| `UnlockRequestStatus` | PENDING, APPROVED, DENIED | Unlock request state |

### 5.2 Setting Categories

| Category | Description |
|----------|-------------|
| `tracking` | Location tracking settings |
| `privacy` | Privacy and visibility settings |
| `notifications` | Notification preferences |
| `battery` | Battery optimization settings |
| `geofence` | Geofencing settings |
| `alerts` | Alert and proximity settings |
| `movement` | Movement detection settings |
| `trip` | Trip tracking settings |

---

## 6. Validation Rules

### 6.1 User Validation

| Field | Rules |
|-------|-------|
| `email` | Valid email format, max 255 chars, lowercase |
| `password` | Min 8 chars, 1 uppercase, 1 lowercase, 1 digit |
| `displayName` | 1-100 chars, no leading/trailing whitespace |

### 6.2 Group Validation

| Field | Rules |
|-------|-------|
| `name` | 1-100 chars, unique within organization |
| `slug` | Auto-generated, URL-safe, unique within organization |
| `description` | Max 500 chars |
| `maxDevices` | 1-100, default 20 |

### 6.3 Invite Validation

| Field | Rules |
|-------|-------|
| `code` | Format: XXX-XXX-XXX (auto-generated) |
| `maxUses` | 1-100, default 1 |
| `expiresInHours` | 1-168 (max 7 days), default 24 |

### 6.4 Device Validation

| Field | Rules |
|-------|-------|
| `displayName` | 1-100 chars |
| `deviceUuid` | Valid UUID format |

---

## 7. Error Codes

### 7.1 Authentication Errors

| Code | Message | HTTP Status |
|------|---------|-------------|
| `auth/invalid-credentials` | Invalid email or password | 401 |
| `auth/email-not-verified` | Email not verified | 403 |
| `auth/token-expired` | Access token expired | 401 |
| `auth/invalid-refresh-token` | Invalid or expired refresh token | 401 |
| `auth/user-disabled` | User account is disabled | 403 |
| `auth/email-already-exists` | Email already registered | 409 |

### 7.2 Authorization Errors

| Code | Message | HTTP Status |
|------|---------|-------------|
| `authz/insufficient-permissions` | Insufficient permissions | 403 |
| `authz/not-group-member` | Not a member of this group | 403 |
| `authz/not-group-admin` | Admin role required | 403 |
| `authz/setting-locked` | Setting is locked by admin | 403 |

### 7.3 Resource Errors

| Code | Message | HTTP Status |
|------|---------|-------------|
| `resource/not-found` | Resource not found | 404 |
| `resource/already-exists` | Resource already exists | 409 |
| `resource/limit-exceeded` | Resource limit exceeded | 429 |

### 7.4 Validation Errors

| Code | Message | HTTP Status |
|------|---------|-------------|
| `validation/invalid-email` | Invalid email format | 400 |
| `validation/weak-password` | Password does not meet requirements | 400 |
| `validation/invalid-invite-code` | Invalid or expired invite code | 400 |

---

## 8. Serialization Notes

### 8.1 JSON Conventions

- **Field naming**: `snake_case` for JSON
- **Dates**: ISO 8601 format (`2025-12-01T10:30:00Z`)
- **Enums**: Lowercase snake_case
- **Nulls**: Omit null fields in responses
- **ULIDs**: String format with type prefix (e.g., `user_01HXYZ...`)

### 8.2 Kotlin Serialization

```kotlin
@Serializable
data class User(
    val id: String,
    @SerialName("display_name")
    val displayName: String,
    @SerialName("created_at")
    val createdAt: Instant
)
```

### 8.3 Rust Serialization

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct User {
    pub id: String,
    pub display_name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}
```

---

## 9. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-01 | Initial specification |
