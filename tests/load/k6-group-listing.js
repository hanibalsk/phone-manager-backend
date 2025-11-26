// Group Device Listing Endpoint Load Test
// Run with: k6 run tests/load/k6-group-listing.js
//
// Focused test for GET /api/v1/devices?groupId=... endpoint
// Target: p95 < 100ms, error rate < 1%

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

// Configuration from environment
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_KEY = __ENV.API_KEY || 'pm_test_api_key_for_load_testing';

// Custom metrics
const listingDuration = new Trend('device_listing_duration', true);
const listingErrors = new Rate('device_listing_errors');
const listingCount = new Counter('device_listing_count');
const devicesReturned = new Counter('devices_returned_total');

// Test scenarios
export const options = {
  scenarios: {
    // Regular polling (apps checking for group updates)
    regular_polling: {
      executor: 'constant-arrival-rate',
      rate: 300, // 300 requests per second
      timeUnit: '1s',
      duration: '3m',
      preAllocatedVUs: 75,
      maxVUs: 300,
    },
    // App open burst (many users opening app simultaneously)
    app_open_burst: {
      executor: 'ramping-arrival-rate',
      startRate: 100,
      timeUnit: '1s',
      stages: [
        { target: 100, duration: '30s' },   // baseline
        { target: 800, duration: '15s' },   // morning burst
        { target: 800, duration: '30s' },   // sustained
        { target: 100, duration: '30s' },   // settle
      ],
      preAllocatedVUs: 150,
      maxVUs: 600,
      startTime: '3m30s',
    },
  },
  thresholds: {
    'device_listing_duration': ['p(95)<100', 'p(99)<200'],
    'device_listing_errors': ['rate<0.01'],
    'http_req_duration{endpoint:listing}': ['p(95)<100'],
  },
};

// Headers
const headers = {
  'Content-Type': 'application/json',
  'X-API-Key': API_KEY,
};

// Group pool (simulates real group distribution)
const groupPool = [];
const GROUP_POOL_SIZE = 100;

// Initialize group pool
function initGroupPool() {
  for (let i = 0; i < GROUP_POOL_SIZE; i++) {
    groupPool.push(`group-${i}`);
  }
}

// Get random group from pool (weighted toward popular groups)
function getRandomGroup() {
  if (groupPool.length === 0) {
    return `group-${Math.floor(Math.random() * 100)}`;
  }
  // Bias toward first 20 groups (simulate popular groups)
  if (Math.random() < 0.7) {
    return groupPool[Math.floor(Math.random() * 20)];
  }
  return groupPool[Math.floor(Math.random() * groupPool.length)];
}

// Main test function
export default function() {
  const groupId = getRandomGroup();

  const res = http.get(
    `${BASE_URL}/api/v1/devices?groupId=${encodeURIComponent(groupId)}`,
    {
      headers,
      tags: { endpoint: 'listing' },
    }
  );

  listingCount.add(1);
  listingDuration.add(res.timings.duration);

  const success = check(res, {
    'status is 200': (r) => r.status === 200,
    'response has devices array': (r) => {
      try {
        const body = JSON.parse(r.body);
        return Array.isArray(body.devices);
      } catch (e) {
        return false;
      }
    },
    'response time < 100ms': (r) => r.timings.duration < 100,
  });

  // Track devices returned for throughput analysis
  if (success) {
    try {
      const body = JSON.parse(res.body);
      if (body.devices) {
        devicesReturned.add(body.devices.length);
      }
    } catch (e) {
      // Ignore parsing errors for metric tracking
    }
  }

  listingErrors.add(!success);

  // Simulate realistic polling interval
  sleep(0.02);
}

// Setup - optionally seed some test data
export function setup() {
  console.log(`Group Device Listing Load Test`);
  console.log(`Target: ${BASE_URL}/api/v1/devices?groupId=...`);

  // Initialize group pool
  initGroupPool();
  console.log(`Group pool initialized with ${GROUP_POOL_SIZE} groups`);

  // Health check
  const res = http.get(`${BASE_URL}/api/health`);
  if (res.status !== 200) {
    throw new Error(`API health check failed: ${res.status}`);
  }

  // Optionally seed some devices for more realistic testing
  console.log('Seeding test devices...');
  const seededDevices = [];
  for (let i = 0; i < 50; i++) {
    const deviceId = uuidv4();
    const groupId = groupPool[i % 20]; // Spread across first 20 groups

    const payload = JSON.stringify({
      deviceId: deviceId,
      displayName: `Test Device ${i}`,
      groupId: groupId,
      platform: i % 2 === 0 ? 'android' : 'ios',
    });

    const regRes = http.post(
      `${BASE_URL}/api/v1/devices/register`,
      payload,
      { headers }
    );

    if (regRes.status === 200) {
      seededDevices.push({ deviceId, groupId });
    }
  }
  console.log(`Seeded ${seededDevices.length} test devices`);

  return {
    startTime: new Date().toISOString(),
    seededDevices: seededDevices,
  };
}

// Teardown
export function teardown(data) {
  console.log(`Test completed. Started: ${data.startTime}`);
  console.log(`Seeded devices: ${data.seededDevices.length}`);
}
