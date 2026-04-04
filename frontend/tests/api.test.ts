/**
 * API Integration Tests
 * Tests API endpoints using supertest against local MemStorage-backed handlers.
 *
 * Auth responses use the standard ApiResponse envelope:
 *   { success: true, data: { user, accessToken, refreshToken, expiresIn } }
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import request from 'supertest';
import type { Express } from 'express';
import type { Server } from 'http';
import { createTestApp } from './test-routes';

describe('API Integration Tests', () => {
  let app: Express;
  let server: Server;
  let accessToken: string;

  beforeAll(async () => {
    ({ app, server } = await createTestApp());
  });

  afterAll(async () => {
    if (server) {
      await new Promise<void>((resolve) => {
        server.close(() => resolve());
      });
    }
  });

  describe('Authentication Endpoints', () => {
    it('POST /api/auth/register - should register a new user', async () => {
      const response = await request(app)
        .post('/api/auth/register')
        .send({
          username: 'testuser',
          email: 'test@example.com',
          password: 'password123',
        })
        .expect(201);

      expect(response.body.success).toBe(true);
      expect(response.body.data.user).toBeDefined();
      expect(response.body.data.user.username).toBe('testuser');
      expect(response.body.data.user.email).toBe('test@example.com');
      expect(response.body.data.user.passwordHash).toBeUndefined(); // Should not be returned
      expect(response.body.data.accessToken).toBeDefined();

      // Save access token for subsequent tests
      accessToken = response.body.data.accessToken;
    });

    it('POST /api/auth/register - should reject duplicate email', async () => {
      await request(app)
        .post('/api/auth/register')
        .send({
          username: 'testuser2',
          email: 'test@example.com', // Same email
          password: 'password123',
        })
        .expect(400);
    });

    it('POST /api/auth/register - should reject duplicate username', async () => {
      await request(app)
        .post('/api/auth/register')
        .send({
          username: 'testuser', // Same username
          email: 'test2@example.com',
          password: 'password123',
        })
        .expect(400);
    });

    it('POST /api/auth/register - should reject short password', async () => {
      await request(app)
        .post('/api/auth/register')
        .send({
          username: 'testuser3',
          email: 'test3@example.com',
          password: 'short', // Less than 8 characters
        })
        .expect(400);
    });

    it('POST /api/auth/login - should login with correct credentials (identifier)', async () => {
      const response = await request(app)
        .post('/api/auth/login')
        .send({
          identifier: 'test@example.com',
          password: 'password123',
        })
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data.user).toBeDefined();
      expect(response.body.data.user.email).toBe('test@example.com');
      expect(response.body.data.accessToken).toBeDefined();
    });

    it('POST /api/auth/login - should reject invalid credentials', async () => {
      await request(app)
        .post('/api/auth/login')
        .send({
          identifier: 'test@example.com',
          password: 'wrongpassword',
        })
        .expect(401);
    });

    it('POST /api/auth/login - should reject non-existent user', async () => {
      await request(app)
        .post('/api/auth/login')
        .send({
          identifier: 'nonexistent@example.com',
          password: 'password123',
        })
        .expect(401);
    });

    it('GET /api/auth/me - should get current user with valid token', async () => {
      const response = await request(app)
        .get('/api/auth/me')
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data.username).toBe('testuser');
      expect(response.body.data.email).toBe('test@example.com');
      expect(response.body.data.passwordHash).toBeUndefined();
    });

    it('GET /api/auth/me - should reject without token', async () => {
      await request(app)
        .get('/api/auth/me')
        .expect(401);
    });

    it('GET /api/auth/me - should reject with invalid token', async () => {
      await request(app)
        .get('/api/auth/me')
        .set('Authorization', 'Bearer invalid-session-id')
        .expect(401);
    });

    it('POST /api/auth/wallet/connect - should update wallet address', async () => {
      const response = await request(app)
        .post('/api/auth/wallet/connect')
        .set('Authorization', `Bearer ${accessToken}`)
        .send({
          wallet_address: '0x1234567890abcdef',
          signature: 'test-sig',
          message: 'connect wallet',
        })
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data.walletAddress).toBe('0x1234567890abcdef');
    });

    it('POST /api/auth/logout - should logout successfully', async () => {
      const response = await request(app)
        .post('/api/auth/logout')
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(200);

      expect(response.body.success).toBe(true);

      // Session should be invalidated
      await request(app)
        .get('/api/auth/me')
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(401);
    });
  });

  describe('Security Engines Endpoints', () => {
    it('GET /api/engines - should get all security engines', async () => {
      const response = await request(app)
        .get('/api/engines')
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
      expect(response.body.length).toBeGreaterThan(0);
      expect(response.body[0]).toHaveProperty('name');
      expect(response.body[0]).toHaveProperty('engineType');
      expect(response.body[0]).toHaveProperty('accuracyRate');
    });

    it('POST /api/engines - should create a new engine', async () => {
      const response = await request(app)
        .post('/api/engines')
        .send({
          name: 'Test Engine',
          type: 'automated',
          description: 'Test engine description',
          ownerId: null,
        })
        .expect(201);

      expect(response.body.name).toBe('Test Engine');
      expect(response.body.engineType).toBe('automated');
    });
  });

  describe('Submissions Endpoints', () => {
    let submissionId: string;

    it('POST /api/submissions - should create a submission', async () => {
      const response = await request(app)
        .post('/api/submissions')
        .send({
          filename: 'test.exe',
          submissionType: 'file',
          description: 'Test file for analysis',
        })
        .expect(201);

      expect(response.body.originalFilename).toBe('test.exe');
      expect(response.body.analysisStatus).toBe('pending');

      submissionId = response.body.id;
    });

    it('GET /api/submissions - should get all submissions', async () => {
      const response = await request(app)
        .get('/api/submissions')
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
      expect(response.body.length).toBeGreaterThan(0);
    });

    it('GET /api/submissions/:id - should get submission by id', async () => {
      const response = await request(app)
        .get(`/api/submissions/${submissionId}`)
        .expect(200);

      expect(response.body.id).toBe(submissionId);
      expect(response.body.originalFilename).toBe('test.exe');
    });

    it('GET /api/submissions/:id - should return 404 for non-existent submission', async () => {
      await request(app)
        .get('/api/submissions/non-existent-id')
        .expect(404);
    });

    it('POST /api/submissions/:id/start-analysis - should start analysis', async () => {
      await request(app)
        .post(`/api/submissions/${submissionId}/start-analysis`)
        .expect(200);

      // Check that submission status changed
      const response = await request(app)
        .get(`/api/submissions/${submissionId}`)
        .expect(200);

      expect(response.body.analysisStatus).toBe('analyzing');
    });

    it('GET /api/submissions/:id/analyses - should get analyses for submission', async () => {
      const response = await request(app)
        .get(`/api/submissions/${submissionId}/analyses`)
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
      // Should have analyses created by start-analysis
      expect(response.body.length).toBeGreaterThan(0);
    });
  });

  describe('Statistics Endpoints', () => {
    it('GET /api/stats - should get platform statistics', async () => {
      const response = await request(app)
        .get('/api/stats')
        .expect(200);

      expect(response.body).toHaveProperty('totalSubmissions');
      expect(response.body).toHaveProperty('activeAnalyses');
      expect(response.body).toHaveProperty('completedToday');
      expect(response.body).toHaveProperty('threatsDetected');
      expect(response.body).toHaveProperty('totalEngines');
    });

    it('GET /api/bounties - should get active bounties', async () => {
      const response = await request(app)
        .get('/api/bounties')
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
    });
  });

  describe('Rate Limiting', () => {
    it('should rate limit authentication endpoints', async () => {
      // Use a fresh app so previous auth tests don't pollute the counter
      const { app: freshApp, server: freshServer } = await createTestApp();

      try {
        // Make 6 registration attempts (limit is 5)
        for (let i = 0; i < 6; i++) {
          const response = await request(freshApp)
            .post('/api/auth/register')
            .send({
              username: `ratetest${i}`,
              email: `ratetest${i}@example.com`,
              password: 'password123',
            });

          if (i < 5) {
            // First 5 should succeed or fail normally
            expect([201, 400]).toContain(response.status);
          } else {
            // 6th request should be rate limited
            expect(response.status).toBe(429);
          }
        }
      } finally {
        await new Promise<void>((resolve) => freshServer.close(() => resolve()));
      }
    }, 10000);
  });

  describe('Security Headers', () => {
    it('should include security headers', async () => {
      const response = await request(app)
        .get('/api/engines')
        .expect(200);

      // Check for security headers
      expect(response.headers).toHaveProperty('x-content-type-options');
      expect(response.headers).toHaveProperty('x-frame-options');
      expect(response.headers).toHaveProperty('x-xss-protection');
    });

    it('should have CORS headers', async () => {
      const response = await request(app)
        .options('/api/engines')
        .set('Origin', 'http://localhost:5173')
        .expect(204);

      expect(response.headers).toHaveProperty('access-control-allow-origin');
      expect(response.headers).toHaveProperty('access-control-allow-methods');
    });
  });
});