/**
 * End-to-End Tests
 * Tests complete user workflows from start to finish.
 *
 * Uses local Express handlers (test-routes.ts) rather than the production
 * proxy so the Rust api-gateway does not need to be running.
 *
 * Auth responses use the standard ApiResponse envelope:
 *   { success: true, data: { user, accessToken, refreshToken, expiresIn } }
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import request from 'supertest';
import type { Express } from 'express';
import type { Server } from 'http';
import { createTestApp } from './test-routes';

describe('E2E: Complete User Workflows', () => {
  let app: Express;
  let server: Server;

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

  describe('E2E: New User Registration and Submission Flow', () => {
    let accessToken: string;
    let submissionId: string;

    it('Step 1: User registers an account', async () => {
      const response = await request(app)
        .post('/api/auth/register')
        .send({
          username: 'e2euser',
          email: 'e2e@example.com',
          password: 'securepassword123',
        })
        .expect(201);

      expect(response.body.success).toBe(true);
      expect(response.body.data.user).toBeDefined();
      expect(response.body.data.accessToken).toBeDefined();
      accessToken = response.body.data.accessToken;
    });

    it('Step 2: User connects their wallet', async () => {
      const response = await request(app)
        .post('/api/auth/wallet/connect')
        .set('Authorization', `Bearer ${accessToken}`)
        .send({
          wallet_address: '0xE2E1234567890abcdef',
          signature: 'test-sig',
          message: 'connect wallet',
        })
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data.walletAddress).toBe('0xE2E1234567890abcdef');
    });

    it('Step 3: User views available security engines', async () => {
      const response = await request(app)
        .get('/api/engines')
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
      expect(response.body.length).toBeGreaterThan(0);
    });

    it('Step 4: User submits a file for analysis', async () => {
      const response = await request(app)
        .post('/api/submissions')
        .send({
          filename: 'suspicious-file.exe',
          fileHash: 'sha256_e2e123',
          submissionType: 'file',
          description: 'Suspicious executable found in email attachment',
        })
        .expect(201);

      expect(response.body.id).toBeDefined();
      expect(response.body.analysisStatus).toBe('pending');
      submissionId = response.body.id;
    });

    it('Step 5: User starts the analysis process', async () => {
      const response = await request(app)
        .post(`/api/submissions/${submissionId}/start-analysis`)
        .expect(200);

      expect(response.body.message).toBeDefined();
    });

    it('Step 6: User checks submission status', async () => {
      const response = await request(app)
        .get(`/api/submissions/${submissionId}`)
        .expect(200);

      expect(response.body.analysisStatus).toBe('analyzing');
    });

    it('Step 7: User views analyses from different engines', async () => {
      const response = await request(app)
        .get(`/api/submissions/${submissionId}/analyses`)
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
      expect(response.body.length).toBeGreaterThan(0);
    });

    it('Step 8: User checks platform statistics', async () => {
      const response = await request(app)
        .get('/api/stats')
        .expect(200);

      expect(response.body.totalSubmissions).toBeGreaterThan(0);
      expect(response.body.totalEngines).toBeGreaterThan(0);
    });

    it('Step 9: User logs out', async () => {
      const response = await request(app)
        .post('/api/auth/logout')
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(200);

      expect(response.body.success).toBe(true);

      // Verify session is invalidated
      await request(app)
        .get('/api/auth/me')
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(401);
    });

    it('Step 10: User logs back in', async () => {
      const response = await request(app)
        .post('/api/auth/login')
        .send({
          identifier: 'e2e@example.com',
          password: 'securepassword123',
        })
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.data.user).toBeDefined();
      expect(response.body.data.accessToken).toBeDefined();
    });
  });

  describe('E2E: Security Engine Registration and Analysis Flow', () => {
    let accessToken: string;
    let engineId: string;
    let submissionId: string;

    it('Step 1: Security researcher registers', async () => {
      const response = await request(app)
        .post('/api/auth/register')
        .send({
          username: 'securityresearcher',
          email: 'researcher@security.com',
          password: 'researcherpass123',
        })
        .expect(201);

      expect(response.body.success).toBe(true);
      accessToken = response.body.data.accessToken;
    });

    it('Step 2: Researcher registers their security engine', async () => {
      const response = await request(app)
        .post('/api/engines')
        .send({
          name: 'Custom ML Analyzer',
          type: 'ml',
          description: 'Machine learning based threat detection',
          ownerId: null,
        })
        .expect(201);

      engineId = response.body.id;
      expect(response.body.name).toBe('Custom ML Analyzer');
    });

    it('Step 3: New submission is created by another user', async () => {
      const response = await request(app)
        .post('/api/submissions')
        .send({
          filename: 'malware-sample.bin',
          submissionType: 'file',
          description: 'Potential malware sample',
        })
        .expect(201);

      submissionId = response.body.id;
    });

    it('Step 4: Analysis is started', async () => {
      await request(app)
        .post(`/api/submissions/${submissionId}/start-analysis`)
        .expect(200);
    });

    it('Step 5: Engine submits analysis result', async () => {
      const analyses = await request(app)
        .get(`/api/submissions/${submissionId}/analyses`)
        .expect(200);

      expect(analyses.body.length).toBeGreaterThan(0);
    });

    it('Step 6: User checks for consensus result', async () => {
      const response = await request(app)
        .get(`/api/submissions/${submissionId}/consensus`)
        .expect((res) => {
          // Accept either 200 (consensus ready) or 404 (not ready yet)
          expect([200, 404]).toContain(res.status);
        });

      if (response.status === 200) {
        expect(response.body).toHaveProperty('finalVerdict');
        expect(response.body).toHaveProperty('confidenceScore');
      }
    });
  });

  describe('E2E: Multiple Submissions Workflow', () => {
    let accessToken: string;
    const submissionIds: string[] = [];

    it('Step 1: User logs in', async () => {
      const response = await request(app)
        .post('/api/auth/login')
        .send({
          identifier: 'e2e@example.com',
          password: 'securepassword123',
        })
        .expect(200);

      expect(response.body.success).toBe(true);
      accessToken = response.body.data.accessToken;
    });

    it('Step 2: User submits multiple files', async () => {
      const files = [
        { filename: 'file1.exe' },
        { filename: 'file2.dll' },
        { filename: 'file3.pdf' },
      ];

      for (const file of files) {
        const response = await request(app)
          .post('/api/submissions')
          .send({
            filename: file.filename,
            submissionType: 'file',
          })
          .expect(201);

        submissionIds.push(response.body.id);
      }

      expect(submissionIds.length).toBe(3);
    });

    it('Step 3: User views all their submissions', async () => {
      const response = await request(app)
        .get('/api/submissions')
        .expect(200);

      expect(response.body.length).toBeGreaterThanOrEqual(3);
    });

    it('Step 4: User starts analysis for all submissions', async () => {
      for (const id of submissionIds) {
        await request(app)
          .post(`/api/submissions/${id}/start-analysis`)
          .expect(200);
      }
    });

    it('Step 5: User checks stats', async () => {
      const response = await request(app)
        .get('/api/stats')
        .expect(200);

      // We submitted at least 3 files and started analysis on them
      expect(response.body.totalSubmissions).toBeGreaterThanOrEqual(3);
    });
  });

  describe('E2E: Error Handling and Edge Cases', () => {
    it('Should handle invalid session gracefully', async () => {
      await request(app)
        .get('/api/auth/me')
        .set('Authorization', 'Bearer invalid-token')
        .expect(401);
    });

    it('Should handle missing required fields', async () => {
      await request(app)
        .post('/api/submissions')
        .send({
          // Missing filename and submissionType
        })
        .expect(400);
    });

    it('Should handle non-existent resources', async () => {
      await request(app)
        .get('/api/submissions/non-existent-id')
        .expect(404);
    });

    it('Should handle concurrent requests', async () => {
      const promises = Array(10)
        .fill(null)
        .map(() =>
          request(app)
            .get('/api/engines')
            .expect(200)
        );

      const responses = await Promise.all(promises);

      expect(responses.length).toBe(10);
      responses.forEach((response) => {
        expect(Array.isArray(response.body)).toBe(true);
      });
    });
  });
});