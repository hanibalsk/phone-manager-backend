# Load Test Results

## Test Configuration

### Environment
- **Target URL**: `http://localhost:8080` (or staging environment)
- **Test Tool**: k6 (https://k6.io)
- **Test Script**: `tests/load/k6-load-test.js`

### Test Scenarios

#### Scenario 1: Sustained Load
- **Type**: Constant arrival rate
- **Rate**: 1000 requests/second
- **Duration**: 5 minutes
- **Virtual Users**: 100-500

#### Scenario 2: Spike Test
- **Type**: Ramping arrival rate
- **Stages**:
  - Warm up: 100 req/s for 1 minute
  - Spike: Ramp to 2000 req/s over 30 seconds
  - Recovery: Return to 100 req/s over 1 minute
- **Virtual Users**: 200-1000

## Performance Thresholds

| Metric | Target | Result |
|--------|--------|--------|
| HTTP Request Duration (p95) | < 200ms | _TBD_ |
| HTTP Request Duration (p99) | < 500ms | _TBD_ |
| Device Registration (p95) | < 50ms | _TBD_ |
| Location Upload (p95) | < 50ms | _TBD_ |
| Batch Location Upload (p95) | < 150ms | _TBD_ |
| Device Listing (p95) | < 100ms | _TBD_ |
| Error Rate | < 1% | _TBD_ |

## Running the Tests

### Prerequisites

1. Install k6:
   ```bash
   # macOS
   brew install k6

   # Linux
   sudo gpg -k
   sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
   echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
   sudo apt-get update
   sudo apt-get install k6
   ```

2. Ensure the API server is running with a valid API key

### Execute Tests

```bash
# Run with default settings (localhost:8080)
k6 run tests/load/k6-load-test.js

# Run against specific environment
BASE_URL=https://staging.api.example.com API_KEY=your_api_key k6 run tests/load/k6-load-test.js

# Run with specific duration
k6 run --duration 10m tests/load/k6-load-test.js

# Run with output to JSON
k6 run --out json=results.json tests/load/k6-load-test.js
```

## Test Results Template

### Summary

| Metric | Value |
|--------|-------|
| Test Date | YYYY-MM-DD |
| Duration | X minutes |
| Total Requests | N |
| Successful Requests | N |
| Failed Requests | N |
| Request Rate | N req/s |

### Response Time Distribution

| Percentile | Value (ms) |
|------------|------------|
| p50 | _TBD_ |
| p90 | _TBD_ |
| p95 | _TBD_ |
| p99 | _TBD_ |
| max | _TBD_ |

### Endpoint Breakdown

| Endpoint | Count | p50 (ms) | p95 (ms) | Error Rate |
|----------|-------|----------|----------|------------|
| POST /api/v1/devices/register | _TBD_ | _TBD_ | _TBD_ | _TBD_ |
| POST /api/v1/locations | _TBD_ | _TBD_ | _TBD_ | _TBD_ |
| POST /api/v1/locations/batch | _TBD_ | _TBD_ | _TBD_ | _TBD_ |
| GET /api/v1/devices | _TBD_ | _TBD_ | _TBD_ | _TBD_ |

### Resource Utilization

| Metric | Peak | Average |
|--------|------|---------|
| CPU Usage | _TBD_ | _TBD_ |
| Memory Usage | _TBD_ | _TBD_ |
| Database Connections | _TBD_ | _TBD_ |
| Network I/O | _TBD_ | _TBD_ |

## Performance Optimization Recommendations

Based on test results, consider:

1. **Database Connection Pool**: Adjust `database.max_connections` based on load
2. **Rate Limiting**: Configure `security.rate_limit_per_minute` appropriately
3. **Horizontal Scaling**: Use HPA settings in `k8s/hpa.yaml`
4. **Caching**: Consider Redis for frequently accessed data
5. **Query Optimization**: Monitor slow queries and add indexes

## Historical Results

| Date | Test Type | p95 (ms) | Error Rate | Notes |
|------|-----------|----------|------------|-------|
| _TBD_ | Sustained | _TBD_ | _TBD_ | Initial baseline |

## Appendix

### Sample k6 Output

```
          /\      |‾‾| /‾‾/   /‾‾/
     /\  /  \     |  |/  /   /  /
    /  \/    \    |     (   /   ‾‾\
   /          \   |  |\  \ |  (‾)  |
  / __________ \  |__| \__\ \_____/ .io

  execution: local
     script: tests/load/k6-load-test.js
     output: -

  scenarios: (100.00%) 2 scenarios, 1500 max VUs, 8m0s max duration (incl. graceful stop):
           * sustained_load: 1000.00 iterations/s for 5m0s (maxVUs: 100-500, gracefulStop: 30s)
           * spike_test: Up to 2000.00 iterations/s for 2m30s over 3 stages (maxVUs: 200-1000, gracefulStop: 30s)

running (Xm XXs), 0000/XXXX VUs, XXXXX complete and 0 interrupted iterations
```

### Grafana Dashboard

For real-time monitoring, import the k6 dashboard:
- Dashboard ID: 2587
- Data source: Prometheus

Configure Prometheus to scrape the `/metrics` endpoint for application metrics.
