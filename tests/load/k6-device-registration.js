// Device Registration Endpoint Load Test
// Run with: k6 run tests/load/k6-device-registration.js
//
// Focused test for POST /api/v1/devices/register endpoint
// Target: p95 < 50ms, error rate < 1%

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

// Configuration from environment
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_KEY = __ENV.API_KEY || 'pm_test_api_key_for_load_testing';

// Custom metrics
const registrationDuration = new Trend('device_registration_duration', true);
const registrationErrors = new Rate('device_registration_errors');
const registrationCount = new Counter('device_registration_count');

// Test scenarios
export const options = {
  scenarios: {
    // Sustained registration load
    sustained_registrations: {
      executor: 'constant-arrival-rate',
      rate: 200, // 200 registrations per second
      timeUnit: '1s',
      duration: '3m',
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
    // Burst registration scenario (app launch spike)
    burst_registrations: {
      executor: 'ramping-arrival-rate',
      startRate: 50,
      timeUnit: '1s',
      stages: [
        { target: 50, duration: '30s' },   // baseline
        { target: 500, duration: '15s' },  // burst
        { target: 500, duration: '30s' },  // sustain burst
        { target: 50, duration: '30s' },   // recover
      ],
      preAllocatedVUs: 100,
      maxVUs: 500,
      startTime: '3m30s',
    },
  },
  thresholds: {
    'device_registration_duration': ['p(95)<50', 'p(99)<100'],
    'device_registration_errors': ['rate<0.01'],
    'http_req_duration{endpoint:register}': ['p(95)<50'],
  },
};

// Headers
const headers = {
  'Content-Type': 'application/json',
  'X-API-Key': API_KEY,
};

// Platforms to simulate
const platforms = ['android', 'ios'];

// Test data generator
function generateRegistrationPayload() {
  const deviceId = uuidv4();
  return {
    deviceId: deviceId,
    displayName: `Device ${deviceId.substring(0, 8)}`,
    groupId: `group-${Math.floor(Math.random() * 100)}`,
    platform: platforms[Math.floor(Math.random() * platforms.length)],
    fcmToken: `fcm_${uuidv4()}`,
  };
}

// Main test function
export default function() {
  const payload = generateRegistrationPayload();

  const res = http.post(
    `${BASE_URL}/api/v1/devices/register`,
    JSON.stringify(payload),
    {
      headers,
      tags: { endpoint: 'register' },
    }
  );

  registrationCount.add(1);
  registrationDuration.add(res.timings.duration);

  const success = check(res, {
    'status is 200': (r) => r.status === 200,
    'response has deviceId': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.deviceId === payload.deviceId;
      } catch (e) {
        return false;
      }
    },
    'response time < 50ms': (r) => r.timings.duration < 50,
  });

  registrationErrors.add(!success);

  // Small delay between iterations
  sleep(0.05);
}

// Setup
export function setup() {
  console.log(`Device Registration Load Test`);
  console.log(`Target: ${BASE_URL}/api/v1/devices/register`);

  // Health check
  const res = http.get(`${BASE_URL}/api/health`);
  if (res.status !== 200) {
    throw new Error(`API health check failed: ${res.status}`);
  }

  return { startTime: new Date().toISOString() };
}

// Teardown
export function teardown(data) {
  console.log(`Test completed. Started: ${data.startTime}`);
}
