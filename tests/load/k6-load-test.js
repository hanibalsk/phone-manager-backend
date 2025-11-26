// Phone Manager API Load Test Script
// Run with: k6 run tests/load/k6-load-test.js
//
// Requirements:
// - k6 installed: https://k6.io/docs/getting-started/installation/
// - API running at BASE_URL
// - Valid API key in API_KEY environment variable

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

// Configuration from environment
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_KEY = __ENV.API_KEY || 'pm_test_api_key_for_load_testing';

// Custom metrics
const deviceRegistrationDuration = new Trend('device_registration_duration');
const locationUploadDuration = new Trend('location_upload_duration');
const batchLocationUploadDuration = new Trend('batch_location_upload_duration');
const deviceListingDuration = new Trend('device_listing_duration');
const errorRate = new Rate('errors');
const requestsCounter = new Counter('requests');

// Test configuration
export const options = {
  scenarios: {
    // Scenario 1: Sustained load
    sustained_load: {
      executor: 'constant-arrival-rate',
      rate: 1000, // 1000 requests per second
      timeUnit: '1s',
      duration: '5m',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
    // Scenario 2: Spike test
    spike_test: {
      executor: 'ramping-arrival-rate',
      startRate: 100,
      timeUnit: '1s',
      stages: [
        { target: 100, duration: '1m' },   // warm up
        { target: 2000, duration: '30s' }, // spike to 2x
        { target: 100, duration: '1m' },   // recover
      ],
      preAllocatedVUs: 200,
      maxVUs: 1000,
      startTime: '5m30s', // Start after sustained load
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<200', 'p(99)<500'], // 95% < 200ms, 99% < 500ms
    'device_registration_duration': ['p(95)<50'],
    'location_upload_duration': ['p(95)<50'],
    'batch_location_upload_duration': ['p(95)<150'],
    'device_listing_duration': ['p(95)<100'],
    'errors': ['rate<0.01'], // Error rate < 1%
  },
};

// Headers
const headers = {
  'Content-Type': 'application/json',
  'X-API-Key': API_KEY,
};

// Test data generators
function generateDeviceId() {
  return uuidv4();
}

function generateGroupId() {
  return `group-${Math.floor(Math.random() * 1000)}`;
}

function generateLocation() {
  return {
    latitude: 37.7749 + (Math.random() - 0.5) * 0.1,
    longitude: -122.4194 + (Math.random() - 0.5) * 0.1,
    accuracy: Math.random() * 50 + 5,
    altitude: Math.random() * 100,
    speed: Math.random() * 30,
    bearing: Math.random() * 360,
    batteryLevel: Math.floor(Math.random() * 100),
    timestamp: Date.now(),
  };
}

function generateBatchLocations(count) {
  const locations = [];
  const baseTime = Date.now() - count * 60000; // Go back in time
  for (let i = 0; i < count; i++) {
    const loc = generateLocation();
    loc.timestamp = baseTime + i * 60000; // 1 minute apart
    locations.push(loc);
  }
  return locations;
}

// Main test function
export default function() {
  const deviceId = generateDeviceId();
  const groupId = generateGroupId();

  group('Device Registration', function() {
    const payload = JSON.stringify({
      deviceId: deviceId,
      displayName: `Test Device ${deviceId.substring(0, 8)}`,
      groupId: groupId,
      platform: 'android',
    });

    const res = http.post(`${BASE_URL}/api/v1/devices/register`, payload, { headers });

    requestsCounter.add(1);
    deviceRegistrationDuration.add(res.timings.duration);

    const success = check(res, {
      'device registration status is 200': (r) => r.status === 200,
      'device registration response has deviceId': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.deviceId === deviceId;
        } catch (e) {
          return false;
        }
      },
    });

    if (!success) {
      errorRate.add(1);
    } else {
      errorRate.add(0);
    }
  });

  sleep(0.1);

  group('Single Location Upload', function() {
    const location = generateLocation();
    const payload = JSON.stringify({
      deviceId: deviceId,
      ...location,
    });

    const res = http.post(`${BASE_URL}/api/v1/locations`, payload, { headers });

    requestsCounter.add(1);
    locationUploadDuration.add(res.timings.duration);

    const success = check(res, {
      'location upload status is 200': (r) => r.status === 200,
      'location upload returns success': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.success === true && body.processedCount === 1;
        } catch (e) {
          return false;
        }
      },
    });

    if (!success) {
      errorRate.add(1);
    } else {
      errorRate.add(0);
    }
  });

  sleep(0.1);

  group('Batch Location Upload', function() {
    const locations = generateBatchLocations(25); // 25 locations per batch
    const payload = JSON.stringify({
      deviceId: deviceId,
      locations: locations,
    });

    const res = http.post(`${BASE_URL}/api/v1/locations/batch`, payload, { headers });

    requestsCounter.add(1);
    batchLocationUploadDuration.add(res.timings.duration);

    const success = check(res, {
      'batch upload status is 200': (r) => r.status === 200,
      'batch upload returns correct count': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.success === true && body.processedCount === 25;
        } catch (e) {
          return false;
        }
      },
    });

    if (!success) {
      errorRate.add(1);
    } else {
      errorRate.add(0);
    }
  });

  sleep(0.1);

  group('Device Listing', function() {
    const res = http.get(`${BASE_URL}/api/v1/devices?groupId=${groupId}`, { headers });

    requestsCounter.add(1);
    deviceListingDuration.add(res.timings.duration);

    const success = check(res, {
      'device listing status is 200': (r) => r.status === 200,
      'device listing returns array': (r) => {
        try {
          const body = JSON.parse(r.body);
          return Array.isArray(body.devices);
        } catch (e) {
          return false;
        }
      },
    });

    if (!success) {
      errorRate.add(1);
    } else {
      errorRate.add(0);
    }
  });

  sleep(0.1);
}

// Setup - run once before tests
export function setup() {
  console.log(`Running load test against: ${BASE_URL}`);
  console.log(`API Key: ${API_KEY.substring(0, 10)}...`);

  // Verify API is accessible
  const res = http.get(`${BASE_URL}/api/health`);
  if (res.status !== 200) {
    throw new Error(`API health check failed: ${res.status}`);
  }

  console.log('API health check passed');
  return { startTime: new Date().toISOString() };
}

// Teardown - run once after tests
export function teardown(data) {
  console.log(`Load test completed. Started at: ${data.startTime}`);
}
