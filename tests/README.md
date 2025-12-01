# Phone Manager Backend Tests

## Test Types

### Unit Tests
Unit tests are located alongside the code in each crate's `src` directory using `#[cfg(test)]` modules.

Run unit tests:
```bash
cargo test --lib
```

### Integration Tests
Integration tests require a running PostgreSQL database and test the full HTTP API flow.

Located in:
- `crates/api/tests/` - API integration tests

Run integration tests:
```bash
# 1. Start test database
docker-compose -f docker-compose.test.yml up -d

# 2. Wait for database to be ready
sleep 5

# 3. Run integration tests
TEST_DATABASE_URL="postgres://phone_manager:phone_manager_dev@localhost:5433/phone_manager_test" \
    cargo test --test auth_integration

# 4. Stop test database (optional)
docker-compose -f docker-compose.test.yml down
```

### Load Tests
Load tests using k6 are located in `tests/load/`.

Run load tests:
```bash
# Ensure API server is running
k6 run tests/load/k6-load-test.js
```

## Test Database Setup

The test database uses port 5433 to avoid conflicts with the development database (port 5432).

### Quick Setup
```bash
# Start only the test database
docker-compose -f docker-compose.test.yml up -d

# Verify it's running
docker ps | grep phone-manager-db-test
```

### Clean Database
To reset the test database to a clean state:
```bash
docker-compose -f docker-compose.test.yml down -v
docker-compose -f docker-compose.test.yml up -d
```

## Authentication Integration Tests

The `auth_integration` test suite covers:

- **Registration**
  - Successful registration
  - Duplicate email handling
  - Weak password rejection
  - Invalid email format rejection

- **Login**
  - Successful login
  - Invalid password rejection
  - Non-existent user handling
  - Case-insensitive email matching

- **Token Management**
  - Token refresh flow
  - Invalid token rejection
  - Logout functionality

- **Protected Routes**
  - Access with valid token
  - Access without token (401)
  - Access with invalid token (401)

- **Session Management**
  - Multiple concurrent sessions
  - Session isolation

- **OAuth**
  - Invalid provider rejection
  - Missing token validation

- **Password Reset**
  - Reset request for existing user
  - Reset request for non-existent user (security: no user enumeration)

- **Email Verification**
  - Invalid token handling

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `TEST_DATABASE_URL` | PostgreSQL connection URL for tests | `postgres://phone_manager:phone_manager_dev@localhost:5433/phone_manager_test` |

## CI/CD Integration

For CI/CD pipelines, use the test docker-compose:

```yaml
# GitHub Actions example
services:
  postgres:
    image: postgres:16-alpine
    env:
      POSTGRES_USER: phone_manager
      POSTGRES_PASSWORD: phone_manager_dev
      POSTGRES_DB: phone_manager_test
    ports:
      - 5433:5432
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5

steps:
  - name: Run integration tests
    env:
      TEST_DATABASE_URL: postgres://phone_manager:phone_manager_dev@localhost:5433/phone_manager_test
    run: cargo test --test auth_integration
```
