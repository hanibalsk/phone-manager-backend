// Single Location Upload Endpoint Load Test
// Run with: k6 run tests/load/k6-location-upload.js
//
// Focused test for POST /api/v1/locations endpoint
// Target: p95 < 50ms, error rate < 1%

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

// Configuration from environment
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_KEY = __ENV.API_KEY || 'pm_test_api_key_for_load_testing';

// Custom metrics
const uploadDuration = new Trend('location_upload_duration', true);
const uploadErrors = new Rate('location_upload_errors');
const uploadCount = new Counter('location_upload_count');

// Test scenarios
export const options = {
  scenarios: {
    // High-frequency location updates (many devices reporting)
    high_frequency: {
      executor: 'constant-arrival-rate',
      rate: 500, // 500 location updates per second
      timeUnit: '1s',
      duration: '3m',
      preAllocatedVUs: 100,
      maxVUs: 400,
    },
    // Simulated rush hour (commute time spike)
    rush_hour: {
      executor: 'ramping-arrival-rate',
      startRate: 200,
      timeUnit: '1s',
      stages: [
        { target: 200, duration: '30s' },   // normal
        { target: 1000, duration: '30s' },  // ramp up
        { target: 1000, duration: '1m' },   // peak
        { target: 200, duration: '30s' },   // ramp down
      ],
      preAllocatedVUs: 200,
      maxVUs: 800,
      startTime: '3m30s',
    },
  },
  thresholds: {
    'location_upload_duration': ['p(95)<50', 'p(99)<100'],
    'location_upload_errors': ['rate<0.01'],
    'http_req_duration{endpoint:location}': ['p(95)<50'],
  },
};

// Headers
const headers = {
  'Content-Type': 'application/json',
  'X-API-Key': API_KEY,
};

// Pre-registered device pool (simulates real-world scenario)
const devicePool = [];
const DEVICE_POOL_SIZE = 1000;

// Initialize device pool
function initDevicePool() {
  for (let i = 0; i < DEVICE_POOL_SIZE; i++) {
    devicePool.push(uuidv4());
  }
}

// Get random device from pool
function getRandomDevice() {
  if (devicePool.length === 0) {
    return uuidv4();
  }
  return devicePool[Math.floor(Math.random() * devicePool.length)];
}

// Generate realistic location data
function generateLocationPayload(deviceId) {
  // San Francisco area coordinates with variance
  const baseLat = 37.7749;
  const baseLon = -122.4194;

  return {
    deviceId: deviceId,
    latitude: baseLat + (Math.random() - 0.5) * 0.1,
    longitude: baseLon + (Math.random() - 0.5) * 0.1,
    accuracy: 5 + Math.random() * 45, // 5-50 meters
    altitude: Math.random() * 100,
    speed: Math.random() * 30, // 0-30 m/s
    bearing: Math.random() * 360,
    batteryLevel: Math.floor(Math.random() * 100),
    timestamp: Date.now(),
  };
}

// Main test function
export default function() {
  const deviceId = getRandomDevice();
  const payload = generateLocationPayload(deviceId);

  const res = http.post(
    `${BASE_URL}/api/v1/locations`,
    JSON.stringify(payload),
    {
      headers,
      tags: { endpoint: 'location' },
    }
  );

  uploadCount.add(1);
  uploadDuration.add(res.timings.duration);

  const success = check(res, {
    'status is 200': (r) => r.status === 200,
    'response indicates success': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.success === true;
      } catch (e) {
        return false;
      }
    },
    'processed count is 1': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.processedCount === 1;
      } catch (e) {
        return false;
      }
    },
    'response time < 50ms': (r) => r.timings.duration < 50,
  });

  uploadErrors.add(!success);

  // Minimal delay for high-frequency testing
  sleep(0.01);
}

// Setup
export function setup() {
  console.log(`Single Location Upload Load Test`);
  console.log(`Target: ${BASE_URL}/api/v1/locations`);

  // Initialize device pool
  initDevicePool();
  console.log(`Device pool initialized with ${DEVICE_POOL_SIZE} devices`);

  // Health check
  const res = http.get(`${BASE_URL}/api/health`);
  if (res.status !== 200) {
    throw new Error(`API health check failed: ${res.status}`);
  }

  return {
    startTime: new Date().toISOString(),
    devicePool: devicePool,
  };
}

// Teardown
export function teardown(data) {
  console.log(`Test completed. Started: ${data.startTime}`);
}
