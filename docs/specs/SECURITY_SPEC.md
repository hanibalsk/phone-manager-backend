# Security Specification

## Phone Manager - User Management & Device Control

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-01

---

## 1. Overview

This document defines the security architecture for the User Management and Device Control feature, covering authentication, authorization, token management, and data protection.

---

## 2. Authentication Architecture

### 2.1 Authentication Methods

| Method | Use Case | Implementation |
|--------|----------|----------------|
| Email/Password | Primary user authentication | Argon2id password hashing |
| Google OAuth | Social login | OAuth 2.0 with ID token |
| Apple Sign-In | Social login (iOS) | OAuth 2.0 with ID token |
| API Key | Legacy device authentication | SHA-256 hashed, backward compatible |
| Device Token | B2B managed devices | Long-lived JWT |

### 2.2 Authentication Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    AUTHENTICATION FLOWS                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │   Email/    │    │   OAuth     │    │   Device    │         │
│  │  Password   │    │  (Google/   │    │  Enrollment │         │
│  │   Login     │    │   Apple)    │    │    (B2B)    │         │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘         │
│         │                  │                  │                 │
│         ▼                  ▼                  ▼                 │
│  ┌──────────────────────────────────────────────────────┐      │
│  │              Credential Verification                  │      │
│  │  - Password: Argon2id verify                         │      │
│  │  - OAuth: ID token verification via provider         │      │
│  │  - Enrollment: Token + device fingerprint            │      │
│  └───────────────────────┬──────────────────────────────┘      │
│                          │                                      │
│                          ▼                                      │
│  ┌──────────────────────────────────────────────────────┐      │
│  │              Token Generation                         │      │
│  │  - Access Token: RS256 JWT, 1 hour                   │      │
│  │  - Refresh Token: RS256 JWT, 30 days                 │      │
│  │  - Device Token: RS256 JWT, 90 days (B2B)            │      │
│  └───────────────────────┬──────────────────────────────┘      │
│                          │                                      │
│                          ▼                                      │
│  ┌──────────────────────────────────────────────────────┐      │
│  │              Response to Client                       │      │
│  │  - Access + Refresh tokens                           │      │
│  │  - User profile                                      │      │
│  │  - Device binding confirmation                       │      │
│  └──────────────────────────────────────────────────────┘      │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 2.3 Password Security

#### Requirements
- Minimum 8 characters
- At least 1 uppercase letter
- At least 1 lowercase letter
- At least 1 digit
- Maximum 128 characters

#### Hashing Algorithm: Argon2id
```rust
// Configuration
argon2::Config {
    variant: Variant::Argon2id,
    version: Version::Version13,
    mem_cost: 65536,        // 64 MB
    time_cost: 3,           // 3 iterations
    lanes: 4,               // 4 parallel lanes
    secret: &[],
    ad: &[],
    hash_length: 32
}
```

#### Password Reset Flow
1. User requests reset via email
2. Server generates secure token (32 bytes, base64url)
3. Token stored as SHA-256 hash with 1-hour expiry
4. Email sent with reset link
5. User submits new password with token
6. Token validated, password updated, all sessions invalidated

---

## 3. JWT Token Design

### 3.1 Access Token

**Algorithm:** RS256 (RSA-SHA256)
**Lifetime:** 1 hour

```json
{
  "header": {
    "alg": "RS256",
    "typ": "JWT",
    "kid": "key-2025-01"
  },
  "payload": {
    "sub": "user_01HXYZABC123",
    "iss": "https://api.phonemanager.com",
    "aud": ["https://api.phonemanager.com"],
    "iat": 1701388800,
    "exp": 1701392400,
    "jti": "at_unique-id",
    "type": "access",
    "org_id": "org_01ABC",
    "roles": ["user", "group_admin:grp_01DEF"],
    "device_id": "dev_01GHI",
    "scope": "read:profile write:profile read:devices write:devices read:locations"
  }
}
```

### 3.2 Refresh Token

**Algorithm:** RS256
**Lifetime:** 30 days

```json
{
  "payload": {
    "sub": "user_01HXYZABC123",
    "iss": "https://api.phonemanager.com",
    "iat": 1701388800,
    "exp": 1703980800,
    "jti": "rt_unique-id",
    "type": "refresh",
    "device_id": "dev_01GHI",
    "family_id": "fam_01JKL"
  }
}
```

### 3.3 Device Token (B2B)

**Algorithm:** RS256
**Lifetime:** 90 days

```json
{
  "payload": {
    "sub": "dev_01GHI",
    "iss": "https://api.phonemanager.com",
    "iat": 1701388800,
    "exp": 1709164800,
    "jti": "dt_unique-id",
    "type": "device",
    "org_id": "org_01ABC",
    "policy_id": "pol_01MNO"
  }
}
```

### 3.4 Token Rotation

**Refresh Token Rotation:**
1. Client sends refresh token
2. Server validates token and family_id
3. Server issues new access + refresh tokens
4. New refresh token has same family_id
5. Old refresh token marked as used

**Reuse Detection:**
- If used refresh token is submitted again → compromise detected
- Entire token family is invalidated
- User must re-authenticate

### 3.5 Key Management

```yaml
key_rotation:
  algorithm: RS256
  key_size: 2048
  rotation_period: 90 days
  overlap_period: 7 days  # Both keys valid during transition

key_storage:
  backend: HashiCorp Vault / AWS KMS / environment variable
  access: Read-only from application
  backup: Encrypted, stored separately
```

---

## 4. Authorization Model (RBAC)

### 4.1 Role Hierarchy

```
PLATFORM_ADMIN (Internal)
    │
    ├── ORG_OWNER (B2B)
    │       │
    │       ├── ORG_ADMIN
    │       │       │
    │       │       └── GROUP_ADMIN (scoped to assigned groups)
    │       │               │
    │       │               └── ORG_USER
    │       │
    │       └── DEVICE_ONLY (headless managed device)
    │
    └── USER (B2C)
            │
            ├── GROUP_OWNER (groups they created)
            │       │
            │       ├── GROUP_ADMIN
            │       │
            │       └── GROUP_MEMBER
            │               │
            │               └── GROUP_VIEWER
            │
            └── DEVICE (implicit, per-device permissions)
```

### 4.2 Permission Matrix

| Permission | Description |
|------------|-------------|
| `org:manage` | Full organization control |
| `org:read` | View organization details |
| `org:users:manage` | Manage organization users |
| `group:create` | Create new groups |
| `group:manage` | Full group management |
| `group:read` | View group details and members |
| `group:invite` | Create group invitations |
| `device:register` | Register new devices |
| `device:manage` | Manage device settings |
| `device:read` | View device info |
| `device:delete` | Remove/retire devices |
| `location:read:own` | View own device locations |
| `location:read:group` | View group device locations |
| `location:read:org` | View all org device locations |
| `settings:read` | View device settings |
| `settings:write` | Update unlocked settings |
| `settings:lock` | Lock/unlock settings |
| `settings:override` | Override locked settings |
| `policy:manage` | Manage device policies (B2B) |
| `audit:read` | View audit logs |

### 4.3 Role-Permission Mapping

| Role | Permissions |
|------|-------------|
| **PLATFORM_ADMIN** | All permissions |
| **ORG_OWNER** | `org:*`, `group:*`, `device:*`, `location:read:org`, `settings:*`, `policy:*`, `audit:*` |
| **ORG_ADMIN** | `org:read`, `org:users:manage`, `group:*`, `device:*`, `location:read:org`, `settings:*`, `policy:*`, `audit:read` |
| **GROUP_ADMIN** | `group:manage` (scoped), `group:invite`, `device:manage` (group), `location:read:group`, `settings:lock` |
| **GROUP_MEMBER** | `group:read`, `device:read` (group), `location:read:group`, `settings:read`, `settings:write` (own unlocked) |
| **GROUP_VIEWER** | `group:read`, `device:read` (group), `location:read:group` |
| **USER** (B2C) | `group:create`, `device:register`, `device:manage` (own), `location:read:own`, `settings:read`, `settings:write` (own unlocked) |
| **DEVICE_ONLY** | `location:write` (own), `settings:read` (own) |

### 4.4 Authorization Enforcement

```rust
// Middleware pattern
pub async fn authorize<B>(
    auth: AuthContext,
    required_permission: Permission,
    resource: Option<Resource>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    // 1. Check if user has permission
    if !auth.has_permission(&required_permission) {
        return Err(ApiError::Forbidden("insufficient_permissions"));
    }

    // 2. If resource-scoped, check ownership/membership
    if let Some(resource) = resource {
        match resource {
            Resource::Group(group_id) => {
                if !auth.is_member_of(&group_id) {
                    return Err(ApiError::Forbidden("not_group_member"));
                }
            }
            Resource::Device(device_id) => {
                if !auth.can_access_device(&device_id) {
                    return Err(ApiError::Forbidden("no_device_access"));
                }
            }
            Resource::Organization(org_id) => {
                if auth.organization_id != Some(org_id) {
                    return Err(ApiError::Forbidden("wrong_organization"));
                }
            }
        }
    }

    Ok(next.run(request).await)
}
```

---

## 5. Token Storage (Android)

### 5.1 Secure Storage Implementation

```kotlin
class SecureTokenStorage(context: Context) {

    private val masterKey = MasterKey.Builder(context)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()

    private val sharedPreferences = EncryptedSharedPreferences.create(
        context,
        "secure_tokens",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )

    companion object {
        private const val KEY_ACCESS_TOKEN = "access_token"
        private const val KEY_REFRESH_TOKEN = "refresh_token"
        private const val KEY_ACCESS_TOKEN_EXPIRY = "access_token_expiry"
        private const val KEY_DEVICE_TOKEN = "device_token"
        private const val KEY_USER_ID = "user_id"
    }

    fun saveTokens(tokens: AuthTokens) {
        sharedPreferences.edit()
            .putString(KEY_ACCESS_TOKEN, tokens.accessToken)
            .putString(KEY_REFRESH_TOKEN, tokens.refreshToken)
            .putLong(KEY_ACCESS_TOKEN_EXPIRY, tokens.expiresAt.epochSecond)
            .apply()
    }

    fun getAccessToken(): String? =
        sharedPreferences.getString(KEY_ACCESS_TOKEN, null)

    fun isAccessTokenExpired(): Boolean {
        val expiry = sharedPreferences.getLong(KEY_ACCESS_TOKEN_EXPIRY, 0)
        return Instant.now().epochSecond >= expiry - 60 // 60s buffer
    }

    fun clearTokens() {
        sharedPreferences.edit()
            .remove(KEY_ACCESS_TOKEN)
            .remove(KEY_REFRESH_TOKEN)
            .remove(KEY_ACCESS_TOKEN_EXPIRY)
            .remove(KEY_USER_ID)
            .apply()
    }
}
```

### 5.2 Token Refresh Interceptor

```kotlin
class AuthInterceptor(
    private val tokenStorage: SecureTokenStorage,
    private val authApi: AuthApi
) : Interceptor {

    private val tokenMutex = Mutex()

    override fun intercept(chain: Interceptor.Chain): Response {
        val request = chain.request()

        // Skip auth for public endpoints
        if (isPublicEndpoint(request.url.encodedPath)) {
            return chain.proceed(request)
        }

        // Get current token (refresh if needed)
        val token = runBlocking {
            tokenMutex.withLock {
                getValidAccessToken()
            }
        } ?: return chain.proceed(request)

        // Add Authorization header
        val authenticatedRequest = request.newBuilder()
            .header("Authorization", "Bearer $token")
            .build()

        val response = chain.proceed(authenticatedRequest)

        // Handle 401 - token might have been invalidated
        if (response.code == 401) {
            response.close()
            runBlocking {
                tokenMutex.withLock {
                    tokenStorage.clearTokens()
                    // Emit logout event
                }
            }
        }

        return response
    }

    private suspend fun getValidAccessToken(): String? {
        // If access token valid, return it
        if (!tokenStorage.isAccessTokenExpired()) {
            return tokenStorage.getAccessToken()
        }

        // Try to refresh
        val refreshToken = tokenStorage.getRefreshToken() ?: return null

        return try {
            val response = authApi.refreshToken(RefreshTokenRequest(refreshToken))
            tokenStorage.saveAccessToken(response.accessToken, response.expiresIn)
            response.accessToken
        } catch (e: Exception) {
            tokenStorage.clearTokens()
            null
        }
    }
}
```

---

## 6. API Security

### 6.1 Endpoint Authentication Requirements

| Endpoint Pattern | Auth Required | Notes |
|-----------------|---------------|-------|
| `POST /auth/register` | No | Rate limited |
| `POST /auth/login` | No | Rate limited |
| `POST /auth/oauth/*` | No | Rate limited |
| `POST /auth/refresh` | Refresh Token | |
| `POST /auth/logout` | Access Token | |
| `GET /users/me` | Access Token | |
| `* /groups/*` | Access Token | + Group membership |
| `* /devices/*` | Access Token or Device Token | |
| `POST /locations/*` | Access Token or Device Token | |
| `* /admin/*` | Access Token | + Admin role |

### 6.2 Rate Limiting

```yaml
rate_limits:
  # Authentication endpoints (per IP)
  auth_register:
    requests: 10
    window: 1 hour
  auth_login:
    requests: 20
    window: 1 hour
  auth_login_failed:
    requests: 5
    window: 15 minutes
    action: temporary_lockout

  # API endpoints (per user/device)
  api_general:
    requests: 100
    window: 1 minute
  api_location_upload:
    requests: 20
    window: 1 minute
  api_batch_operations:
    requests: 10
    window: 1 minute

  # Admin endpoints (per user)
  admin_general:
    requests: 60
    window: 1 minute
  admin_bulk_operations:
    requests: 5
    window: 1 minute
```

### 6.3 Security Headers

```rust
// Response headers
pub fn security_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        STRICT_TRANSPORT_SECURITY,
        "max-age=31536000; includeSubDomains".parse().unwrap()
    );
    headers.insert(
        X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap()
    );
    headers.insert(
        X_FRAME_OPTIONS,
        "DENY".parse().unwrap()
    );
    headers.insert(
        CONTENT_SECURITY_POLICY,
        "default-src 'none'".parse().unwrap()
    );
    headers.insert(
        CACHE_CONTROL,
        "no-store, no-cache, must-revalidate".parse().unwrap()
    );
    headers.insert(
        "X-Request-Id",
        generate_request_id().parse().unwrap()
    );

    headers
}
```

### 6.4 Input Validation

```rust
// All inputs must be validated before processing
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    // 1. Length check
    if email.len() > 255 {
        return Err(ValidationError::TooLong("email", 255));
    }

    // 2. Format check
    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::InvalidFormat("email"));
    }

    // 3. Normalize
    let normalized = email.to_lowercase().trim();

    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::TooShort("password", 8));
    }
    if password.len() > 128 {
        return Err(ValidationError::TooLong("password", 128));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::MissingUppercase("password"));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::MissingLowercase("password"));
    }
    if !password.chars().any(|c| c.is_digit(10)) {
        return Err(ValidationError::MissingDigit("password"));
    }

    Ok(())
}
```

---

## 7. Data Isolation

### 7.1 Multi-Tenancy Model

```
┌─────────────────────────────────────────────────────────────────┐
│                     DATA ISOLATION MODEL                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  B2B Organizations                    B2C Users                  │
│  ┌─────────────────┐                 ┌─────────────────┐        │
│  │  Organization A │                 │    User X       │        │
│  │  ┌───────────┐  │                 │  ┌───────────┐  │        │
│  │  │  Users    │  │                 │  │  Groups   │  │        │
│  │  │  Groups   │  │                 │  │  Devices  │  │        │
│  │  │  Devices  │  │                 │  └───────────┘  │        │
│  │  │  Policies │  │                 └─────────────────┘        │
│  │  └───────────┘  │                                            │
│  └─────────────────┘                                            │
│           │                                                      │
│           │ COMPLETE ISOLATION                                   │
│           │                                                      │
│  ┌─────────────────┐                                            │
│  │  Organization B │   No cross-org access                      │
│  │  ┌───────────┐  │   No cross-user access (B2C)               │
│  │  │  ...      │  │   Groups provide controlled sharing        │
│  │  └───────────┘  │                                            │
│  └─────────────────┘                                            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 7.2 Query Isolation

```rust
// All queries MUST include organization/user scope
pub async fn get_devices(
    pool: &PgPool,
    auth: &AuthContext,
) -> Result<Vec<Device>, DbError> {
    // B2B: Scope to organization
    if let Some(org_id) = &auth.organization_id {
        sqlx::query_as!(
            Device,
            r#"
            SELECT * FROM devices
            WHERE organization_id = $1 AND is_active = true
            "#,
            org_id
        )
        .fetch_all(pool)
        .await
    }
    // B2C: Scope to user's groups
    else {
        sqlx::query_as!(
            Device,
            r#"
            SELECT d.* FROM devices d
            JOIN group_memberships gm ON d.group_id = gm.group_id
            WHERE gm.user_id = $1 AND d.is_active = true
            "#,
            &auth.user_id
        )
        .fetch_all(pool)
        .await
    }
}
```

### 7.3 Audit Logging

```rust
pub struct AuditLog {
    pub id: String,
    pub organization_id: Option<String>,
    pub actor_id: String,
    pub actor_type: ActorType,  // User, Device, System, Admin
    pub action: String,         // "device.register", "settings.lock"
    pub resource_type: String,  // "device", "group", "user"
    pub resource_id: String,
    pub changes: Option<serde_json::Value>,  // Before/after
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Log all security-relevant actions
pub async fn audit_log(
    pool: &PgPool,
    auth: &AuthContext,
    action: &str,
    resource: &Resource,
    changes: Option<&Changes>,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        INSERT INTO audit_logs (
            id, organization_id, actor_id, actor_type, action,
            resource_type, resource_id, changes, ip_address, user_agent
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        generate_ulid("log"),
        auth.organization_id,
        auth.actor_id(),
        auth.actor_type().as_str(),
        action,
        resource.type_name(),
        resource.id(),
        changes.map(|c| serde_json::to_value(c).unwrap()),
        auth.ip_address.map(|ip| ip.to_string()),
        auth.user_agent.as_deref()
    )
    .execute(pool)
    .await?;

    Ok(())
}
```

---

## 8. Backward Compatibility

### 8.1 Legacy API Key Support

```rust
// Support both Bearer tokens and X-API-Key
pub async fn authenticate(
    headers: &HeaderMap,
) -> Result<AuthContext, ApiError> {
    // Try Bearer token first (new auth)
    if let Some(auth_header) = headers.get("Authorization") {
        let token = auth_header
            .to_str()
            .map_err(|_| ApiError::Unauthorized)?
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        return validate_jwt(token).await;
    }

    // Fall back to X-API-Key (legacy)
    if let Some(api_key) = headers.get("X-API-Key") {
        let key = api_key.to_str().map_err(|_| ApiError::Unauthorized)?;
        return validate_api_key(key).await;
    }

    Err(ApiError::Unauthorized)
}
```

### 8.2 Device Migration Path

```
Existing Device (X-API-Key)
         │
         ▼
┌─────────────────────────────────┐
│  Option 1: Continue as-is      │  Works forever (backward compat)
│  - No account required         │
│  - Limited features            │
└─────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│  Option 2: Link to Account     │  User creates account, links device
│  - Full feature access         │
│  - Groups and sharing          │
│  - Settings sync               │
└─────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│  Option 3: B2B Enrollment      │  Company enrolls device
│  - Managed device              │
│  - Policy enforcement          │
│  - Remote control              │
└─────────────────────────────────┘
```

---

## 9. Security Checklist

### 9.1 Authentication
- [ ] Passwords hashed with Argon2id
- [ ] JWT tokens signed with RS256
- [ ] Refresh token rotation implemented
- [ ] Token reuse detection enabled
- [ ] Password reset uses secure tokens
- [ ] OAuth ID tokens verified with provider
- [ ] Rate limiting on auth endpoints

### 9.2 Authorization
- [ ] RBAC permissions enforced
- [ ] Resource ownership verified
- [ ] Organization isolation enforced
- [ ] Group membership validated
- [ ] Admin actions require admin role

### 9.3 Data Protection
- [ ] All communication over HTTPS
- [ ] Tokens stored encrypted (Android)
- [ ] PII encrypted at rest
- [ ] Audit logging enabled
- [ ] Data isolation verified

### 9.4 API Security
- [ ] Input validation on all endpoints
- [ ] Rate limiting configured
- [ ] Security headers set
- [ ] CORS properly configured
- [ ] Error messages don't leak info

---

## 10. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-01 | Initial specification |
