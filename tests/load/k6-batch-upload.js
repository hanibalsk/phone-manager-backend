// Batch Location Upload Endpoint Load Test
// Run with: k6 run tests/load/k6-batch-upload.js
//
// Focused test for POST /api/v1/locations/batch endpoint
// Target: p95 < 150ms, error rate < 1%

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

// Configuration from environment
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_KEY = __ENV.API_KEY || 'pm_test_api_key_for_load_testing';

// Custom metrics
const batchDuration = new Trend('batch_upload_duration', true);
const batchErrors = new Rate('batch_upload_errors');
const batchCount = new Counter('batch_upload_count');
const locationsProcessed = new Counter('locations_processed_total');

// Test scenarios
export const options = {
  scenarios: {
    // Standard batch uploads (devices syncing accumulated locations)
    standard_batch: {
      executor: 'constant-arrival-rate',
      rate: 100, // 100 batch uploads per second
      timeUnit: '1s',
      duration: '3m',
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
    // Sync storm (many devices coming online after offline period)
    sync_storm: {
      executor: 'ramping-arrival-rate',
      startRate: 50,
      timeUnit: '1s',
      stages: [
        { target: 50, duration: '30s' },    // baseline
        { target: 300, duration: '15s' },   // storm begins
        { target: 300, duration: '1m' },    // sustained storm
        { target: 50, duration: '30s' },    // calming
      ],
      preAllocatedVUs: 100,
      maxVUs: 400,
      startTime: '3m30s',
    },
  },
  thresholds: {
    'batch_upload_duration': ['p(95)<150', 'p(99)<300'],
    'batch_upload_errors': ['rate<0.01'],
    'http_req_duration{endpoint:batch}': ['p(95)<150'],
  },
};

// Headers
const headers = {
  'Content-Type': 'application/json',
  'X-API-Key': API_KEY,
};

// Batch sizes to test (simulates different offline durations)
const batchSizes = [10, 25, 50]; // Max is 50 per spec

// Pre-registered device pool
const devicePool = [];
const DEVICE_POOL_SIZE = 500;

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

// Generate batch of location data
function generateBatchPayload(deviceId, batchSize) {
  const locations = [];
  const baseTime = Date.now() - batchSize * 60000; // Go back in time
  const baseLat = 37.7749;
  const baseLon = -122.4194;

  for (let i = 0; i < batchSize; i++) {
    locations.push({
      latitude: baseLat + (Math.random() - 0.5) * 0.1,
      longitude: baseLon + (Math.random() - 0.5) * 0.1,
      accuracy: 5 + Math.random() * 45,
      altitude: Math.random() * 100,
      speed: Math.random() * 30,
      bearing: Math.random() * 360,
      batteryLevel: Math.floor(Math.random() * 100),
      timestamp: baseTime + i * 60000, // 1 minute intervals
    });
  }

  return {
    deviceId: deviceId,
    locations: locations,
  };
}

// Main test function
export default function() {
  const deviceId = getRandomDevice();
  const batchSize = batchSizes[Math.floor(Math.random() * batchSizes.length)];
  const payload = generateBatchPayload(deviceId, batchSize);

  const res = http.post(
    `${BASE_URL}/api/v1/locations/batch`,
    JSON.stringify(payload),
    {
      headers,
      tags: { endpoint: 'batch', batchSize: batchSize.toString() },
    }
  );

  batchCount.add(1);
  batchDuration.add(res.timings.duration);

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
    'processed count matches batch size': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.processedCount === batchSize;
      } catch (e) {
        return false;
      }
    },
    'response time < 150ms': (r) => r.timings.duration < 150,
  });

  if (success) {
    locationsProcessed.add(batchSize);
  }
  batchErrors.add(!success);

  // Batch operations need slightly more breathing room
  sleep(0.05);
}

// Setup
export function setup() {
  console.log(`Batch Location Upload Load Test`);
  console.log(`Target: ${BASE_URL}/api/v1/locations/batch`);
  console.log(`Batch sizes: ${batchSizes.join(', ')}`);

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
  };
}

// Teardown
export function teardown(data) {
  console.log(`Test completed. Started: ${data.startTime}`);
}
