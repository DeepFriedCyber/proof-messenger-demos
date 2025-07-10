/**
 * Vitest setup file for WASM testing
 */

import { beforeAll } from 'vitest';

// Mock browser APIs that might be missing in test environment
global.TextEncoder = TextEncoder;
global.TextDecoder = TextDecoder;

// Setup WASM loading for tests
beforeAll(async () => {
  // Ensure WASM is built before running tests
  console.log('Setting up WASM test environment...');
  
  // Add any global test setup here
  global.testStartTime = Date.now();
});