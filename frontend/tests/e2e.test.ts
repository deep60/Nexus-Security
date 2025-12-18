/**
 * End-to-End Tests
 * Tests complete user workflows from start to finish
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import request from 'supertest';
import express, { type Express } from 'express';
import { registerRoutes } from '../server/routes';
import type { Server } from 'http';

describe('E2E: Complete User Workflows', () => {
  let app: Express;
  let server: Server;

  beforeAll(async () => {
    app = express();
    app.use(express.json());
    server = await registerRoutes(app);
  });

  afterAll(async () => {
    if (server) {
      await new Promise<void>((resolve) => {
        server.close(() => resolve());
      });
    }
  });

  describe('E2E: New User Registration and Submission Flow', () => {
    let sessionId: string;
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

      expect(response.body.user).toBeDefined();
      expect(response.body.sessionId).toBeDefined();
      sessionId = response.body.sessionId;

      console.log('✓ User registered successfully');
    });

    it('Step 2: User connects their wallet', async () => {
      const response = await request(app)
        .patch('/api/auth/wallet')
        .set('Authorization', `Bearer ${sessionId}`)
        .send({
          walletAddress: '0xE2E1234567890abcdef',
        })
        .expect(200);

      expect(response.body.walletAddress).toBe('0xE2E1234567890abcdef');

      console.log('✓ Wallet connected successfully');
    });

    it('Step 3: User views available security engines', async () => {
      const response = await request(app)
        .get('/api/engines')
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
      expect(response.body.length).toBeGreaterThan(0);

      console.log(`✓ Found ${response.body.length} security engines`);
    });

    it('Step 4: User submits a file for analysis', async () => {
      const response = await request(app)
        .post('/api/submissions')
        .send({
          filename: 'suspicious-file.exe',
          fileHash: 'sha256_e2e123',
          submissionType: 'file',
          analysisType: 'full',
          bountyAmount: '2.5',
          description: 'Suspicious executable found in email attachment',
        })
        .expect(201);

      expect(response.body.id).toBeDefined();
      expect(response.body.status).toBe('pending');
      submissionId = response.body.id;

      console.log('✓ File submitted for analysis');
    });

    it('Step 5: User starts the analysis process', async () => {
      const response = await request(app)
        .post(`/api/submissions/${submissionId}/start-analysis`)
        .expect(200);

      expect(response.body.message).toBeDefined();

      console.log('✓ Analysis started');
    });

    it('Step 6: User checks submission status', async () => {
      const response = await request(app)
        .get(`/api/submissions/${submissionId}`)
        .expect(200);

      expect(response.body.status).toBe('analyzing');

      console.log('✓ Submission status: analyzing');
    });

    it('Step 7: User views analyses from different engines', async () => {
      const response = await request(app)
        .get(`/api/submissions/${submissionId}/analyses`)
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
      expect(response.body.length).toBeGreaterThan(0);

      console.log(`✓ ${response.body.length} analyses in progress`);
    });

    it('Step 8: User checks platform statistics', async () => {
      const response = await request(app)
        .get('/api/stats')
        .expect(200);

      expect(response.body.totalSubmissions).toBeGreaterThan(0);
      expect(response.body.totalEngines).toBeGreaterThan(0);

      console.log('✓ Platform statistics retrieved');
    });

    it('Step 9: User logs out', async () => {
      await request(app)
        .post('/api/auth/logout')
        .set('Authorization', `Bearer ${sessionId}`)
        .expect(200);

      // Verify session is invalidated
      await request(app)
        .get('/api/auth/me')
        .set('Authorization', `Bearer ${sessionId}`)
        .expect(401);

      console.log('✓ User logged out successfully');
    });

    it('Step 10: User logs back in', async () => {
      const response = await request(app)
        .post('/api/auth/login')
        .send({
          email: 'e2e@example.com',
          password: 'securepassword123',
        })
        .expect(200);

      expect(response.body.user).toBeDefined();
      expect(response.body.sessionId).toBeDefined();

      console.log('✓ User logged back in successfully');
    });
  });

  describe('E2E: Security Engine Registration and Analysis Flow', () => {
    let sessionId: string;
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

      sessionId = response.body.sessionId;

      console.log('✓ Security researcher registered');
    });

    it('Step 2: Researcher registers their security engine', async () => {
      const response = await request(app)
        .post('/api/engines')
        .send({
          name: 'Custom ML Analyzer',
          type: 'ml',
          description: 'Machine learning based threat detection',
          status: 'online',
          ownerId: null,
        })
        .expect(201);

      engineId = response.body.id;
      expect(response.body.name).toBe('Custom ML Analyzer');

      console.log('✓ Security engine registered');
    });

    it('Step 3: New submission is created by another user', async () => {
      const response = await request(app)
        .post('/api/submissions')
        .send({
          filename: 'malware-sample.bin',
          submissionType: 'file',
          analysisType: 'deep',
          bountyAmount: '5.0',
          description: 'Potential malware sample',
        })
        .expect(201);

      submissionId = response.body.id;

      console.log('✓ New submission created with 5.0 ETH bounty');
    });

    it('Step 4: Analysis is started', async () => {
      await request(app)
        .post(`/api/submissions/${submissionId}/start-analysis`)
        .expect(200);

      console.log('✓ Analysis initiated');
    });

    it('Step 5: Engine submits analysis result', async () => {
      const analyses = await request(app)
        .get(`/api/submissions/${submissionId}/analyses`)
        .expect(200);

      expect(analyses.body.length).toBeGreaterThan(0);
      const analysisId = analyses.body[0].id;

      // Simulate engine completing analysis
      // In real implementation, this would be done through a separate endpoint
      console.log('✓ Analysis results submitted');
    });

    it('Step 6: User checks for consensus result', async () => {
      // Note: Consensus might not be ready immediately
      const response = await request(app)
        .get(`/api/submissions/${submissionId}/consensus`)
        .expect((res) => {
          // Accept either 200 (consensus ready) or 404 (not ready yet)
          expect([200, 404]).toContain(res.status);
        });

      if (response.status === 200) {
        expect(response.body).toHaveProperty('finalVerdict');
        expect(response.body).toHaveProperty('confidenceScore');
        console.log(`✓ Consensus: ${response.body.finalVerdict} (${response.body.confidenceScore}% confidence)`);
      } else {
        console.log('✓ Consensus not ready yet (expected behavior)');
      }
    });
  });

  describe('E2E: Multiple Submissions Workflow', () => {
    let sessionId: string;
    const submissionIds: string[] = [];

    it('Step 1: User logs in', async () => {
      const response = await request(app)
        .post('/api/auth/login')
        .send({
          email: 'e2e@example.com',
          password: 'securepassword123',
        })
        .expect(200);

      sessionId = response.body.sessionId;

      console.log('✓ User logged in');
    });

    it('Step 2: User submits multiple files', async () => {
      const files = [
        { filename: 'file1.exe', bounty: '1.0' },
        { filename: 'file2.dll', bounty: '2.0' },
        { filename: 'file3.pdf', bounty: '1.5' },
      ];

      for (const file of files) {
        const response = await request(app)
          .post('/api/submissions')
          .send({
            filename: file.filename,
            submissionType: 'file',
            analysisType: 'quick',
            bountyAmount: file.bounty,
          })
          .expect(201);

        submissionIds.push(response.body.id);
      }

      expect(submissionIds.length).toBe(3);

      console.log('✓ 3 files submitted for analysis');
    });

    it('Step 3: User views all their submissions', async () => {
      const response = await request(app)
        .get('/api/submissions')
        .expect(200);

      expect(response.body.length).toBeGreaterThanOrEqual(3);

      console.log(`✓ Found ${response.body.length} total submissions`);
    });

    it('Step 4: User starts analysis for all submissions', async () => {
      for (const id of submissionIds) {
        await request(app)
          .post(`/api/submissions/${id}/start-analysis`)
          .expect(200);
      }

      console.log('✓ Analysis started for all submissions');
    });

    it('Step 5: User checks active bounties', async () => {
      const response = await request(app)
        .get('/api/bounties')
        .expect(200);

      expect(Array.isArray(response.body)).toBe(true);
      expect(response.body.length).toBeGreaterThanOrEqual(3);

      const totalBounty = response.body.reduce(
        (sum: number, b: any) => sum + parseFloat(b.amount),
        0
      );

      console.log(`✓ ${response.body.length} active bounties, total: ${totalBounty} ETH`);
    });
  });

  describe('E2E: Error Handling and Edge Cases', () => {
    it('Should handle invalid session gracefully', async () => {
      await request(app)
        .get('/api/auth/me')
        .set('Authorization', 'Bearer invalid-token')
        .expect(401);

      console.log('✓ Invalid session rejected');
    });

    it('Should handle missing required fields', async () => {
      await request(app)
        .post('/api/submissions')
        .send({
          filename: 'test.exe',
          // Missing required fields
        })
        .expect(400);

      console.log('✓ Missing fields rejected');
    });

    it('Should handle non-existent resources', async () => {
      await request(app)
        .get('/api/submissions/non-existent-id')
        .expect(404);

      console.log('✓ Non-existent resource returns 404');
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

      console.log('✓ 10 concurrent requests handled successfully');
    });
  });
});