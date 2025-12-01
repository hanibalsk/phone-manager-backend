# JWT Key Rotation Procedure

This document describes the procedure for rotating RSA keys used for JWT token signing and validation in the Phone Manager backend.

## Overview

The Phone Manager API uses RS256 (RSA Signature with SHA-256) for JWT token signing. This requires:
- **Private Key**: Used to sign access and refresh tokens
- **Public Key**: Used to validate tokens

Keys should be rotated:
- Periodically (recommended: every 90 days)
- When a key may have been compromised
- When a team member with key access leaves the organization
- After a security incident

## Key Format

Keys must be in PEM format:

```
-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA...
-----END RSA PRIVATE KEY-----

-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG...
-----END PUBLIC KEY-----
```

## Key Generation

### Using OpenSSL (Recommended)

```bash
# Generate a 2048-bit RSA private key
openssl genrsa -out jwt_private.pem 2048

# Extract the public key from the private key
openssl rsa -in jwt_private.pem -pubout -out jwt_public.pem

# Verify the key pair
openssl rsa -in jwt_private.pem -check -noout
```

### Using Rust (Alternative)

```rust
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};

fn generate_keys() {
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    let public_key = RsaPublicKey::from(&private_key);

    let private_pem = private_key.to_pkcs1_pem(Default::default()).unwrap();
    let public_pem = public_key.to_pkcs1_pem(Default::default()).unwrap();

    println!("Private Key:\n{}", private_pem);
    println!("Public Key:\n{}", public_pem);
}
```

## Zero-Downtime Rotation Procedure

### Step 1: Generate New Key Pair

```bash
# Create a directory for the new keys
mkdir -p keys/new

# Generate new keys
openssl genrsa -out keys/new/jwt_private.pem 2048
openssl rsa -in keys/new/jwt_private.pem -pubout -out keys/new/jwt_public.pem
```

### Step 2: Update Secret Management

Store the new keys in your secrets manager (e.g., AWS Secrets Manager, Vault):

```bash
# Example with AWS Secrets Manager
aws secretsmanager update-secret \
  --secret-id phone-manager/jwt-keys \
  --secret-string "$(cat keys/new/jwt_private.pem)"
```

### Step 3: Update Environment Variables

Update the following environment variables:

```bash
# Environment variables (set via your deployment system)
PM__JWT__PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----
...new key content...
-----END RSA PRIVATE KEY-----"

PM__JWT__PUBLIC_KEY="-----BEGIN PUBLIC KEY-----
...new key content...
-----END PUBLIC KEY-----"
```

### Step 4: Deploy New Keys

1. **Rolling Deployment**: Deploy the new configuration to your servers
2. **Grace Period**: The default 30-second leeway allows for brief clock skew, but not key rotation

### Step 5: Handle Active Sessions

After key rotation, existing tokens will become invalid. Choose one of these strategies:

#### Option A: Immediate Invalidation (Recommended for Security)
- All users will need to re-authenticate
- Suitable for security incidents

#### Option B: Gradual Migration (Recommended for Routine Rotation)
1. Deploy with both old and new public keys
2. Sign new tokens with new private key
3. Accept tokens signed with either key
4. After max token lifetime (30 days), remove old public key

Implementation for Option B requires code changes:

```rust
// Example multi-key validation (pseudo-code)
fn validate_token(token: &str, public_keys: &[&str]) -> Result<Claims, JwtError> {
    for public_key in public_keys {
        if let Ok(claims) = jwt_config.validate(token, public_key) {
            return Ok(claims);
        }
    }
    Err(JwtError::InvalidToken)
}
```

### Step 6: Verify Rotation Success

```bash
# 1. Check API health
curl https://api.example.com/api/health

# 2. Test authentication flow
curl -X POST https://api.example.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"testpass"}'

# 3. Test token validation
curl https://api.example.com/api/v1/users/me \
  -H "Authorization: Bearer <new_token>"
```

### Step 7: Clean Up

```bash
# Remove old keys from local storage
rm -rf keys/old
mv keys/new keys/old  # Keep for rollback if needed

# After successful rotation (7+ days)
rm -rf keys/old
```

## Emergency Key Rotation

In case of a security incident:

1. **Immediately generate new keys** (see Step 1)
2. **Deploy immediately** without grace period
3. **Invalidate all sessions**:
   ```sql
   -- Force logout all users
   TRUNCATE TABLE user_sessions;
   ```
4. **Notify users** about required re-authentication
5. **Audit logs** for unauthorized access

## Configuration Reference

### JWT Configuration Options

| Setting | Environment Variable | Default | Description |
|---------|---------------------|---------|-------------|
| Private Key | `PM__JWT__PRIVATE_KEY` | (required) | RSA private key in PEM format |
| Public Key | `PM__JWT__PUBLIC_KEY` | (required) | RSA public key in PEM format |
| Access Token Expiry | `PM__JWT__ACCESS_TOKEN_EXPIRY_SECS` | 3600 (1 hour) | Access token lifetime |
| Refresh Token Expiry | `PM__JWT__REFRESH_TOKEN_EXPIRY_SECS` | 2592000 (30 days) | Refresh token lifetime |
| Leeway | `PM__JWT__LEEWAY_SECS` | 30 | Clock skew tolerance |

### Security Recommendations

1. **Key Size**: Use at least 2048-bit RSA keys (4096-bit recommended for high security)
2. **Key Storage**: Never commit keys to version control
3. **Access Control**: Limit key access to essential personnel only
4. **Audit Trail**: Log all key rotation events
5. **Rotation Schedule**: Rotate keys every 90 days minimum
6. **Backup**: Maintain secure backup of keys for disaster recovery

## Monitoring

Set up alerts for:
- JWT validation errors spike (may indicate key mismatch)
- Authentication failures increase after rotation
- Token refresh errors

```rust
// Monitoring metrics (already implemented)
counter!("auth_jwt_validation_errors").increment(1);
counter!("auth_login_failures").increment(1);
```

## Rollback Procedure

If issues occur after rotation:

1. **Restore Old Keys**: Deploy previous key configuration
2. **Clear Sessions**: Truncate user_sessions table
3. **Monitor**: Watch for authentication errors
4. **Investigate**: Review logs for root cause

```bash
# Quick rollback (if old keys are available)
export PM__JWT__PRIVATE_KEY="$(cat keys/old/jwt_private.pem)"
export PM__JWT__PUBLIC_KEY="$(cat keys/old/jwt_public.pem)"
# Restart services
```

## Compliance Notes

For compliance with security standards (SOC 2, HIPAA, PCI-DSS):

1. Document all key rotation events
2. Maintain key rotation audit log
3. Store rotation records for required retention period
4. Include key rotation in security incident response plan
