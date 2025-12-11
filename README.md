# Phone Manager Backend

A high-performance Rust backend API for the Phone Manager mobile application. Handles device registration, real-time location tracking, and group management for family/friends location sharing.

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- **Device Management** - Register devices with UUID identifiers, manage display names, and organize into groups
- **Location Tracking** - Real-time single and batch location uploads with 30-day retention
- **Group Coordination** - Share locations within family/friend groups (up to 20 devices per group)
- **Geofences & Events** - Per-device circular geofences with enter/exit/dwell event tracking
- **Webhooks** - Event delivery to external endpoints with HMAC-SHA256 signatures and circuit breaker
- **API Key Authentication** - Secure SHA-256 hashed API key authentication with admin roles
- **JWT Authentication** - RS256 user authentication with OAuth social login (Google, Apple)
- **Configuration Toggles** - Runtime feature flags for registration, OAuth-only, invite-only modes
- **Admin Bootstrap** - Automatic first admin creation via environment variables
- **Rate Limiting** - Per-API-key rate limiting using sliding window algorithm
- **Prometheus Metrics** - Production-ready observability with request counters and latency histograms
- **GDPR Compliance** - Data export and hard deletion for privacy compliance
- **Kubernetes Ready** - Complete deployment manifests with HPA auto-scaling
- **Admin Frontend Serving** - Serve Next.js static admin UI with hostname-based environment selection

## Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 1.83+ |
| Web Framework | Axum | 0.8 |
| Async Runtime | Tokio | 1.42 |
| Database | PostgreSQL + SQLx | 0.8 |
| Rate Limiting | Governor | 0.7 |
| Metrics | Prometheus | 0.24 |

## Quick Start

### Prerequisites

- Rust 1.83+ (`rustup update stable`)
- PostgreSQL 14+
- SQLx CLI (`cargo install sqlx-cli --no-default-features --features postgres`)

### Setup

1. **Clone and configure:**
   ```bash
   git clone <repository-url>
   cd phone-manager-backend
   cp .env.example .env
   ```

2. **Set database URL:**
   ```bash
   # In .env file
   PM__DATABASE__URL=postgres://user:password@localhost:5432/phone_manager
   ```

3. **Create database and run migrations:**
   ```bash
   sqlx database create
   sqlx migrate run --source crates/persistence/src/migrations
   ```

4. **Generate an API key:**
   ```bash
   ./scripts/manage-api-key.sh create --name "Development Key"
   # Save the generated key - it won't be shown again!
   ```

5. **Run the server:**
   ```bash
   cargo run --bin phone-manager
   ```

The API will be available at `http://localhost:8080`.

## API Reference

All protected endpoints require the `X-API-Key` header.

### Health & Metrics

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/health` | GET | No | Basic health check |
| `/api/health/live` | GET | No | Kubernetes liveness probe |
| `/api/health/ready` | GET | No | Kubernetes readiness probe |
| `/metrics` | GET | No | Prometheus metrics |

### Device Management

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/v1/devices/register` | POST | Yes | Register or update a device |
| `/api/v1/devices?groupId={id}` | GET | Yes | List devices in a group with last location |
| `/api/v1/devices/:device_id` | DELETE | Yes | Soft delete (deactivate) a device |

**Register Device Request:**
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "John's Phone",
  "group_id": "family-smith",
  "platform": "android",
  "fcm_token": "optional-firebase-token"
}
```

### Location Tracking

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/v1/locations` | POST | Yes | Upload single location |
| `/api/v1/locations/batch` | POST | Yes | Upload batch locations (max 50) |

**Single Location Request:**
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "latitude": 37.7749,
  "longitude": -122.4194,
  "accuracy": 10.0,
  "altitude": 15.5,
  "bearing": 180.0,
  "speed": 5.5,
  "provider": "gps",
  "batteryLevel": 85,
  "networkType": "wifi",
  "capturedAt": "2024-01-15T10:30:00Z"
}
```

**Batch Location Request:**
```json
{
  "locations": [
    { "device_id": "...", "latitude": 37.7749, "longitude": -122.4194, ... },
    { "device_id": "...", "latitude": 37.7750, "longitude": -122.4195, ... }
  ]
}
```

### Geofences

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/v1/geofences` | POST | API Key | Create a geofence |
| `/api/v1/geofences?deviceId={id}` | GET | API Key | List device geofences |
| `/api/v1/geofences/:geofence_id` | GET | API Key | Get a geofence |
| `/api/v1/geofences/:geofence_id` | PATCH | API Key | Update a geofence |
| `/api/v1/geofences/:geofence_id` | DELETE | API Key | Delete a geofence |

**Create Geofence Request:**
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Home",
  "latitude": 37.7749,
  "longitude": -122.4194,
  "radius": 100,
  "metadata": { "color": "#FF5733" }
}
```

**Limits:**
- Radius: 20-50,000 meters
- Max 50 geofences per device

### Geofence Events

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/v1/geofence-events` | POST | API Key | Create a geofence event |
| `/api/v1/geofence-events?deviceId={id}` | GET | API Key | List device events |
| `/api/v1/geofence-events/:event_id` | GET | API Key | Get an event |

**Create Geofence Event Request:**
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "geofence_id": "660e8400-e29b-41d4-a716-446655440001",
  "event_type": "enter",
  "timestamp": "1701878400000",
  "latitude": 37.7749,
  "longitude": -122.4194
}
```

**Event Types:** `enter`, `exit`, `dwell`

Creating an event automatically triggers webhook delivery to all enabled webhooks for the device.

### Webhooks

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/v1/webhooks` | POST | API Key | Create a webhook |
| `/api/v1/webhooks?ownerDeviceId={id}` | GET | API Key | List device webhooks |
| `/api/v1/webhooks/:webhook_id` | GET | API Key | Get a webhook |
| `/api/v1/webhooks/:webhook_id` | PUT | API Key | Update a webhook |
| `/api/v1/webhooks/:webhook_id` | DELETE | API Key | Delete a webhook |

**Create Webhook Request:**
```json
{
  "owner_device_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Home Assistant",
  "target_url": "https://homeassistant.local/api/webhook/geofence",
  "secret": "my-secret-key-for-hmac-signing"
}
```

**Webhook Delivery:**
- HTTPS required for target URLs
- HMAC-SHA256 signature in `X-Webhook-Signature` header
- Automatic retries with exponential backoff (0s, 60s, 300s, 900s)
- Max 4 retry attempts per delivery
- Circuit breaker opens after 5 consecutive failures (5-minute cooldown)

**Limits:**
- Max 10 webhooks per device
- Secret: 16-256 characters

### Proximity Alerts

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/v1/proximity-alerts` | POST | API Key | Create proximity alert |
| `/api/v1/proximity-alerts?sourceDeviceId={id}` | GET | API Key | List alerts |
| `/api/v1/proximity-alerts/:alert_id` | GET | API Key | Get alert |
| `/api/v1/proximity-alerts/:alert_id` | PATCH | API Key | Update alert |
| `/api/v1/proximity-alerts/:alert_id` | DELETE | API Key | Delete alert |

**Create Proximity Alert Request:**
```json
{
  "source_device_id": "550e8400-e29b-41d4-a716-446655440000",
  "target_device_id": "660e8400-e29b-41d4-a716-446655440001",
  "radius_meters": 500
}
```

**Limits:**
- Radius: 50-100,000 meters
- Max 20 alerts per source device
- Devices must be in the same group

### Privacy (GDPR)

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/v1/devices/:device_id/data-export` | GET | Yes | Export all device data (Article 20) |
| `/api/v1/devices/:device_id/data` | DELETE | Yes | Hard delete device and all data (Article 17) |

### Admin Operations

Admin endpoints require an API key with `is_admin = true`.

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/v1/admin/devices/inactive?older_than_days=30` | DELETE | Admin | Delete inactive devices |
| `/api/v1/admin/devices/:device_id/reactivate` | POST | Admin | Reactivate soft-deleted device |
| `/api/v1/admin/stats` | GET | Admin | Get system statistics |

### Legacy Routes

Legacy routes (without `/v1/`) return `301 Moved Permanently` redirects to v1 endpoints.

## Configuration

Configuration uses TOML files with environment variable overrides.

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PM__DATABASE__URL` | Yes | - | PostgreSQL connection string |
| `PM__SERVER__HOST` | No | `0.0.0.0` | Server bind address |
| `PM__SERVER__PORT` | No | `8080` | Server port |
| `PM__LOGGING__LEVEL` | No | `info` | Log level (trace/debug/info/warn/error) |
| `PM__LOGGING__FORMAT` | No | `json` | Log format (json/pretty) |
| `PM__SECURITY__CORS_ORIGINS` | No | `[]` | Allowed CORS origins |
| `PM__SECURITY__RATE_LIMIT_PER_MINUTE` | No | `100` | Rate limit per API key |
| `PM__LIMITS__MAX_DEVICES_PER_GROUP` | No | `20` | Max devices per group |
| `PM__LIMITS__MAX_BATCH_SIZE` | No | `50` | Max locations per batch |
| `PM__LIMITS__LOCATION_RETENTION_DAYS` | No | `30` | Days to retain location data |
| `PM__DATABASE__MAX_CONNECTIONS` | No | `20` | DB connection pool max |
| `PM__DATABASE__MIN_CONNECTIONS` | No | `5` | DB connection pool min |
| `PM__FRONTEND__ENABLED` | No | `false` | Enable static frontend serving |
| `PM__FRONTEND__BASE_DIR` | No | `/app/frontend` | Base directory for frontend files |
| `PM__FRONTEND__STAGING_HOSTNAME` | No | - | Hostname for staging environment |
| `PM__FRONTEND__PRODUCTION_HOSTNAME` | No | - | Hostname for production environment |
| `PM__FRONTEND__DEFAULT_ENVIRONMENT` | No | `production` | Default when hostname doesn't match |
| `PM__AUTH__REGISTRATION_ENABLED` | No | `true` | Enable user registration |
| `PM__AUTH__INVITE_ONLY` | No | `false` | Require invite token for registration |
| `PM__AUTH__OAUTH_ONLY` | No | `false` | Disable password auth, require OAuth |
| `PM__FEATURES__GEOFENCES_ENABLED` | No | `true` | Enable geofence features |
| `PM__FEATURES__PROXIMITY_ALERTS_ENABLED` | No | `true` | Enable proximity alert features |
| `PM__FEATURES__WEBHOOKS_ENABLED` | No | `true` | Enable webhook features |
| `PM__FEATURES__MOVEMENT_TRACKING_ENABLED` | No | `true` | Enable trips and movement events |
| `PM__FEATURES__B2B_ENABLED` | No | `true` | Enable B2B/organization features |
| `PM__ADMIN__BOOTSTRAP_EMAIL` | No | - | Email for first admin user (one-time) |
| `PM__ADMIN__BOOTSTRAP_PASSWORD` | No | - | Password for first admin (remove after setup!) |
| `PM__REPORTS__REPORTS_DIR` | No | `./reports` | Directory for generated report files |
| `PM__REPORTS__BATCH_SIZE` | No | `5` | Reports to process per background job batch |
| `PM__REPORTS__EXPIRATION_DAYS` | No | `7` | Days before generated reports expire |

### Configuration Files

- `config/default.toml` - Default configuration
- `config/minimal.toml` - Minimal configuration for testing

## API Key Management

Use the CLI tool to manage API keys:

```bash
# Create a standard API key
./scripts/manage-api-key.sh create --name "Production App"

# Create an admin API key
./scripts/manage-api-key.sh create --name "Admin Key" --admin

# Create a key with expiration
./scripts/manage-api-key.sh create --name "Temp Key" --expires 30

# List all keys (requires DATABASE_URL)
DATABASE_URL=postgres://... ./scripts/manage-api-key.sh list

# Rotate a key
DATABASE_URL=postgres://... ./scripts/manage-api-key.sh rotate --prefix pm_aBcDeFgH

# Deactivate a key
DATABASE_URL=postgres://... ./scripts/manage-api-key.sh deactivate --prefix pm_aBcDeFgH

# Get key info
DATABASE_URL=postgres://... ./scripts/manage-api-key.sh info --prefix pm_aBcDeFgH
```

## Development

### Build

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --release --bin phone-manager
```

### Test

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_name
```

### Lint & Format

```bash
# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --workspace -- -D warnings

# Check SQLx queries (requires DATABASE_URL)
cargo sqlx prepare --workspace --check
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add -r <migration_name> --source crates/persistence/src/migrations

# Run migrations
sqlx migrate run --source crates/persistence/src/migrations

# Revert last migration
sqlx migrate revert --source crates/persistence/src/migrations

# Generate offline query data
cargo sqlx prepare --workspace
```

## Kubernetes Deployment

Complete Kubernetes manifests are provided in the `k8s/` directory.

### Quick Deploy

```bash
# Create namespace
kubectl create namespace phone-manager

# Create secret (copy and edit first)
cp k8s/secret.yaml.example k8s/secret.yaml
# Edit k8s/secret.yaml with your base64-encoded values
kubectl apply -f k8s/secret.yaml -n phone-manager

# Deploy all resources
kubectl apply -f k8s/configmap.yaml -n phone-manager
kubectl apply -f k8s/deployment.yaml -n phone-manager
kubectl apply -f k8s/service.yaml -n phone-manager
kubectl apply -f k8s/ingress.yaml -n phone-manager
kubectl apply -f k8s/hpa.yaml -n phone-manager
```

### Resources

| File | Description |
|------|-------------|
| `deployment.yaml` | Main deployment with 3 replicas, health checks, resource limits |
| `service.yaml` | ClusterIP service exposing port 80 → 8080 |
| `configmap.yaml` | Non-sensitive configuration |
| `secret.yaml.example` | Template for secrets (DATABASE_URL, API keys) |
| `ingress.yaml` | Ingress with TLS termination |
| `hpa.yaml` | Horizontal Pod Autoscaler (3-10 replicas, 70% CPU target) |

### Health Checks

The deployment includes:
- **Liveness Probe**: `/api/health/live` - Restarts unhealthy pods
- **Readiness Probe**: `/api/health/ready` - Removes from load balancer if not ready
- **Startup Probe**: `/api/health/ready` - Allows 30s for initial startup

## Admin Frontend Serving

The server can serve a Next.js static export admin frontend with hostname-based environment selection.

### Setup

1. **Build your Next.js app** with static export:
   ```bash
   cd admin-frontend
   npm run build  # Produces 'out' directory
   ```

2. **Copy build output** to the appropriate directory:
   ```bash
   # For staging
   cp -r out/* /path/to/frontend/staging/

   # For production
   cp -r out/* /path/to/frontend/production/
   ```

3. **Configure hostnames** in your environment:
   ```bash
   PM__FRONTEND__ENABLED=true
   PM__FRONTEND__STAGING_HOSTNAME=admin-staging.example.com
   PM__FRONTEND__PRODUCTION_HOSTNAME=admin.example.com
   ```

### Docker Compose

With Docker Compose, mount your frontend directories:

```yaml
# In docker-compose.yml
services:
  api:
    volumes:
      - ./frontend/staging:/app/frontend/staging:ro
      - ./frontend/production:/app/frontend/production:ro
    environment:
      FRONTEND_ENABLED: "true"
      FRONTEND_STAGING_HOSTNAME: admin-staging.example.com
      FRONTEND_PRODUCTION_HOSTNAME: admin.example.com
```

### How It Works

| Request Host | Served From |
|--------------|-------------|
| `admin-staging.example.com` | `/app/frontend/staging/` |
| `admin.example.com` | `/app/frontend/production/` |
| Any other hostname | Default environment (production) |

**Features:**
- **SPA Support**: Routes without file extensions serve `index.html`
- **Cache Headers**: Hashed assets (`/_next/static/*`) cached for 1 year; `index.html` cached for 60 seconds
- **API Priority**: All `/api/*` routes take precedence over frontend
- **Security**: Path traversal protection, read-only volume mounts

## Load Testing

Load tests use [k6](https://k6.io/) and are located in `tests/load/`.

### Run Load Tests

```bash
# Install k6
brew install k6  # macOS
# or download from https://k6.io/

# Set environment variables
export BASE_URL=http://localhost:8080
export API_KEY=your-api-key

# Run load test
k6 run tests/load/k6-load-test.js

# Run with custom parameters
k6 run --vus 100 --duration 5m tests/load/k6-load-test.js
```

### Test Scenarios

| Scenario | Description | Target |
|----------|-------------|--------|
| Sustained Load | 1000 req/s for 5 minutes | p95 < 200ms |
| Spike Test | Ramp to 2000 req/s | No errors |
| Soak Test | 500 req/s for 30 minutes | Memory stable |

See `docs/load-test-results.md` for test result templates.

## Monitoring

### Prometheus Metrics

Available at `/metrics`:

| Metric | Type | Description |
|--------|------|-------------|
| `http_requests_total` | Counter | Total HTTP requests by method, path, status |
| `http_request_duration_seconds` | Histogram | Request latency distribution |

### Grafana Dashboard

Import the dashboard from `docs/grafana-dashboard.json` (if available) or create panels for:

- Request rate by endpoint
- Error rate (4xx, 5xx)
- p50, p95, p99 latency
- Active connections

### Logging

Structured JSON logs include:
- `request_id` - Unique request identifier
- `method`, `path`, `status` - HTTP details
- `latency_ms` - Request duration
- `api_key_prefix` - Authenticated key prefix (for debugging)

## Project Structure

```
phone-manager-backend/
├── crates/
│   ├── api/              # HTTP handlers, middleware, app configuration
│   │   └── src/
│   │       ├── main.rs           # Entry point
│   │       ├── app.rs            # Router and middleware setup
│   │       ├── config.rs         # Configuration loading
│   │       ├── error.rs          # Error types and responses
│   │       ├── middleware/       # Auth, rate limit, metrics, security
│   │       └── routes/           # HTTP route handlers (incl. frontend.rs)
│   ├── domain/           # Business logic and domain models
│   ├── persistence/      # Database layer, repositories, migrations
│   │   └── src/
│   │       ├── entities/         # Database entity structs
│   │       ├── repositories/     # Data access layer
│   │       └── migrations/       # SQL migration files
│   └── shared/           # Common utilities
├── config/               # TOML configuration files
├── k8s/                  # Kubernetes manifests
├── scripts/              # CLI tools and utilities
├── tests/
│   ├── api/              # API integration tests
│   └── load/             # k6 load test scripts
└── docs/                 # Documentation and specifications
```

## Security

### Authentication

The API supports three authentication methods:

| Method | Header | Routes | Description |
|--------|--------|--------|-------------|
| API Key | `X-API-Key` | Device, location, geofence, webhook routes | Device-facing endpoints |
| JWT | `Authorization: Bearer <token>` | User profile, group management routes | User-facing endpoints |
| Admin API Key | `X-API-Key` (admin key) | Admin routes | Administrative operations |

**API Key Details:**
- Hashed with SHA-256 before storage
- Key prefix (8 chars after `pm_`) used for identification
- Admin keys have elevated privileges for management operations

**JWT Authentication:**
- RS256 algorithm (RSA-SHA256 with 2048-bit keys)
- OAuth social login support (Google, Apple)
- Tokens include user ID, email, and role claims

### Security Headers

All responses include:
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security: max-age=31536000; includeSubDomains` (HTTPS only)

### Rate Limiting

- Per-API-key sliding window rate limiting
- Default: 100 requests/minute
- Configurable via `PM__SECURITY__RATE_LIMIT_PER_MINUTE`
- Returns `429 Too Many Requests` when exceeded

### Best Practices

- Never log full API keys (only prefix)
- Use TLS in production
- Set specific CORS origins in production
- Rotate API keys regularly
- Use read-only database replicas for queries where possible

## Performance

### Targets

| Metric | Target |
|--------|--------|
| API Response Time (p95) | < 200ms |
| Uptime | 99.9% |
| Concurrent Connections | 10,000+ |
| Batch Upload | 50 locations/request |

### Optimizations

- Connection pooling with configurable min/max connections
- Batch inserts for location uploads
- Database indexes on frequently queried columns
- Response compression (gzip)
- Efficient JSON serialization with serde

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests and lints (`cargo test && cargo clippy`)
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## Support

- **Issues**: Open an issue on GitHub
- **Security**: Report security vulnerabilities privately via email
