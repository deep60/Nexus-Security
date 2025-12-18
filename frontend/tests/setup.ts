/**
 * Test Setup File
 * Global configuration and setup for all tests
 */

import { beforeAll, afterAll, afterEach } from 'vitest';
import dotenv from 'dotenv';

// Load test environment variables
dotenv.config({ path: '../.env.test' });

// Mock environment variables for testing
process.env.NODE_ENV = 'test';
process.env.JWT_SECRET = 'test-secret-key-for-testing-only';
process.env.BCRYPT_SALT_ROUNDS = '4'; // Lower for faster tests
process.env.SESSION_EXPIRY = '3600000'; // 1 hour for tests

// Global test setup
beforeAll(() => {
  console.log('🧪 Starting test suite...');
});

// Clean up after each test
afterEach(() => {
  // Clear any mocks
});

// Global test teardown
afterAll(() => {
  console.log('✅ Test suite completed');
});