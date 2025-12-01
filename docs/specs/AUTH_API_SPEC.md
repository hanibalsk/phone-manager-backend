# Authentication API Specification

## Phone Manager Backend - User Management & Device Control

**Version:** 1.0.0
**Status:** Design Specification
**Last Updated:** 2025-12-01

---

## 1. Overview

This document defines the authentication API endpoints for user registration, login, OAuth, token management, and password reset.

---

## 2. Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/v1/auth/register` | Create new account | No |
| POST | `/api/v1/auth/login` | Email/password login | No |
| POST | `/api/v1/auth/logout` | Logout (invalidate tokens) | Yes |
| POST | `/api/v1/auth/refresh` | Refresh access token | Refresh Token |
| POST | `/api/v1/auth/oauth/google` | Google OAuth login | No |
| POST | `/api/v1/auth/oauth/apple` | Apple Sign-In | No |
| POST | `/api/v1/auth/forgot-password` | Request password reset | No |
| POST | `/api/v1/auth/reset-password` | Reset password | No |
| POST | `/api/v1/auth/verify-email` | Verify email address | No |
| GET | `/api/v1/auth/me` | Get current user | Yes |
| PUT | `/api/v1/auth/me` | Update current user | Yes |
| PUT | `/api/v1/auth/me/password` | Change password | Yes |

---

## 3. Endpoint Details

### 3.1 POST /api/v1/auth/register

Create a new user account.

**Rate Limit:** 10 requests/hour/IP

#### Request

```json
{
  "email": "user@example.com",
  "password": "SecureP@ssw0rd!",
  "display_name": "John Doe",
  "device_id": "dev_01HXYZABC123",
  "device_name": "John's Pixel 8"
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| email | string | Yes | Valid email, max 255 chars |
| password | string | Yes | Min 8 chars, 1 upper, 1 lower, 1 digit |
| display_name | string | Yes | 1-100 chars |
| device_id | string | No | Device to link after registration |
| device_name | string | No | Required if device_id provided |

#### Response (201 Created)

```json
{
  "user": {
    "id": "user_01HXYZABC123",
    "email": "user@example.com",
    "display_name": "John Doe",
    "avatar_url": null,
    "email_verified": false,
    "auth_provider": "email",
    "organization_id": null,
    "created_at": "2025-12-01T10:30:00Z"
  },
  "tokens": {
    "access_token": "eyJhbGciOiJSUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
    "token_type": "Bearer",
    "expires_in": 3600
  },
  "device_linked": true,
  "requires_email_verification": true
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | validation/invalid-email | Invalid email format |
| 400 | validation/weak-password | Password doesn't meet requirements |
| 409 | auth/email-already-exists | Email already registered |
| 429 | rate-limit/exceeded | Too many registration attempts |

---

### 3.2 POST /api/v1/auth/login

Login with email and password.

**Rate Limit:** 20 requests/hour/IP, 5 failed attempts/15min lockout

#### Request

```json
{
  "email": "user@example.com",
  "password": "SecureP@ssw0rd!",
  "device_id": "dev_01HXYZABC123",
  "device_name": "John's Pixel 8"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| email | string | Yes | User's email |
| password | string | Yes | User's password |
| device_id | string | No | Device making the request |
| device_name | string | No | Human-readable device name |

#### Response (200 OK)

```json
{
  "user": {
    "id": "user_01HXYZABC123",
    "email": "user@example.com",
    "display_name": "John Doe",
    "avatar_url": "https://...",
    "email_verified": true,
    "auth_provider": "email",
    "organization_id": null,
    "created_at": "2025-12-01T10:30:00Z"
  },
  "tokens": {
    "access_token": "eyJhbGciOiJSUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
    "token_type": "Bearer",
    "expires_in": 3600
  }
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 401 | auth/invalid-credentials | Invalid email or password |
| 403 | auth/email-not-verified | Email not verified (if required) |
| 403 | auth/user-disabled | User account is disabled |
| 423 | auth/account-locked | Too many failed attempts |
| 429 | rate-limit/exceeded | Rate limit exceeded |

---

### 3.3 POST /api/v1/auth/logout

Logout and invalidate refresh token.

**Authentication:** Bearer Token

#### Request Headers

```
Authorization: Bearer eyJhbGciOiJSUzI1NiIs...
```

#### Request Body (Optional)

```json
{
  "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
  "all_devices": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| refresh_token | string | No | Specific token to revoke |
| all_devices | boolean | No | Revoke all sessions |

#### Response (204 No Content)

No response body.

---

### 3.4 POST /api/v1/auth/refresh

Refresh access token using refresh token.

**Rate Limit:** 60 requests/hour

#### Request

```json
{
  "refresh_token": "eyJhbGciOiJSUzI1NiIs..."
}
```

#### Response (200 OK)

```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

**Note:** A new refresh token is issued (rotation). The old refresh token is invalidated.

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 401 | auth/invalid-refresh-token | Invalid or expired refresh token |
| 401 | auth/token-reuse-detected | Token reuse detected, family invalidated |

---

### 3.5 POST /api/v1/auth/oauth/google

Login or register with Google OAuth.

**Rate Limit:** 20 requests/hour/IP

#### Request

```json
{
  "id_token": "eyJhbGciOiJSUzI1NiIs...",
  "device_id": "dev_01HXYZABC123",
  "device_name": "John's Pixel 8"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id_token | string | Yes | Google ID token from client |
| device_id | string | No | Device making the request |
| device_name | string | No | Human-readable device name |

#### Response (200 OK)

```json
{
  "user": {
    "id": "user_01HXYZABC123",
    "email": "user@gmail.com",
    "display_name": "John Doe",
    "avatar_url": "https://lh3.googleusercontent.com/...",
    "email_verified": true,
    "auth_provider": "google",
    "organization_id": null,
    "created_at": "2025-12-01T10:30:00Z"
  },
  "tokens": {
    "access_token": "eyJhbGciOiJSUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
    "token_type": "Bearer",
    "expires_in": 3600
  },
  "is_new_user": true
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | auth/invalid-id-token | Invalid Google ID token |
| 403 | auth/user-disabled | User account is disabled |

---

### 3.6 POST /api/v1/auth/oauth/apple

Login or register with Apple Sign-In.

**Rate Limit:** 20 requests/hour/IP

#### Request

```json
{
  "id_token": "eyJhbGciOiJSUzI1NiIs...",
  "authorization_code": "c1234567890...",
  "user_info": {
    "name": "John Doe",
    "email": "user@privaterelay.appleid.com"
  },
  "device_id": "dev_01HXYZABC123",
  "device_name": "John's iPhone"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id_token | string | Yes | Apple ID token |
| authorization_code | string | No | Authorization code for token exchange |
| user_info | object | No | User info (only on first sign-in) |
| device_id | string | No | Device making the request |
| device_name | string | No | Human-readable device name |

#### Response (200 OK)

Same format as Google OAuth response.

---

### 3.7 POST /api/v1/auth/forgot-password

Request password reset email.

**Rate Limit:** 5 requests/hour/IP

#### Request

```json
{
  "email": "user@example.com"
}
```

#### Response (200 OK)

```json
{
  "message": "If the email exists, a reset link has been sent"
}
```

**Note:** Always returns 200 to prevent email enumeration.

---

### 3.8 POST /api/v1/auth/reset-password

Reset password using token from email.

#### Request

```json
{
  "token": "reset_abc123xyz...",
  "password": "NewSecureP@ss1!"
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| token | string | Yes | Reset token from email |
| password | string | Yes | Min 8 chars, 1 upper, 1 lower, 1 digit |

#### Response (200 OK)

```json
{
  "message": "Password reset successful"
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | auth/invalid-reset-token | Invalid or expired reset token |
| 400 | validation/weak-password | Password doesn't meet requirements |

---

### 3.9 POST /api/v1/auth/verify-email

Verify email address using token.

#### Request

```json
{
  "token": "verify_abc123xyz..."
}
```

#### Response (200 OK)

```json
{
  "message": "Email verified successfully",
  "user": {
    "id": "user_01HXYZABC123",
    "email": "user@example.com",
    "email_verified": true
  }
}
```

---

### 3.10 GET /api/v1/auth/me

Get current authenticated user.

**Authentication:** Bearer Token

#### Response (200 OK)

```json
{
  "id": "user_01HXYZABC123",
  "email": "user@example.com",
  "display_name": "John Doe",
  "avatar_url": "https://...",
  "email_verified": true,
  "auth_provider": "email",
  "organization_id": null,
  "is_active": true,
  "last_login_at": "2025-12-01T10:30:00Z",
  "created_at": "2025-11-01T08:00:00Z",
  "updated_at": "2025-12-01T10:30:00Z"
}
```

---

### 3.11 PUT /api/v1/auth/me

Update current user profile.

**Authentication:** Bearer Token

#### Request

```json
{
  "display_name": "John D.",
  "avatar_url": "https://..."
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| display_name | string | No | New display name (1-100 chars) |
| avatar_url | string | No | New avatar URL |

#### Response (200 OK)

Updated user object.

---

### 3.12 PUT /api/v1/auth/me/password

Change current user's password.

**Authentication:** Bearer Token

#### Request

```json
{
  "current_password": "OldP@ssw0rd!",
  "new_password": "NewP@ssw0rd!"
}
```

#### Response (200 OK)

```json
{
  "message": "Password changed successfully"
}
```

#### Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | auth/invalid-password | Current password is incorrect |
| 400 | validation/weak-password | New password doesn't meet requirements |
| 400 | validation/same-password | New password same as current |

---

## 4. Token Specifications

### 4.1 Access Token

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
    "org_id": null,
    "roles": ["user"],
    "device_id": "dev_01GHI",
    "scope": "read:profile write:profile read:devices write:devices read:locations"
  }
}
```

**Lifetime:** 1 hour

### 4.2 Refresh Token

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

**Lifetime:** 30 days

---

## 5. Error Response Format

All errors follow this format:

```json
{
  "error": {
    "code": "auth/invalid-credentials",
    "message": "Invalid email or password",
    "details": null
  }
}
```

---

## 6. Security Considerations

### 6.1 Password Security
- Hashed with Argon2id
- Never stored in plaintext
- Never returned in API responses

### 6.2 Token Security
- Access tokens are short-lived (1 hour)
- Refresh tokens are rotated on each use
- Token reuse is detected and family invalidated
- Tokens signed with RS256

### 6.3 Rate Limiting
- Registration: 10/hour/IP
- Login: 20/hour/IP, lockout after 5 failures
- Password reset: 5/hour/IP
- Token refresh: 60/hour

### 6.4 OAuth Security
- ID tokens verified with provider's public keys
- Nonce validation for replay protection
- Account linking prevents duplicate accounts

---

## 7. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-01 | Initial specification |
