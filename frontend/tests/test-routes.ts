/**
 * Test Routes  — lightweight Express handlers backed by MemStorage.
 *
 * In production, routes.ts proxies /api/* to the Rust api-gateway.
 * During tests the gateway is not running, so we mount these local
 * handlers that exercise the same MemStorage layer that storage.test.ts
 * tests directly.
 */

import express, { type Express, type Request, type Response, type NextFunction } from 'express';
import { createServer, type Server } from 'http';
import { MemStorage, setStorage, getStorage } from '../server/storage';
import { randomUUID } from 'crypto';

/**
 * Create an Express app with locally-handled API routes for testing.
 * Returns both the app and the http Server so callers can shut it down.
 * Each call creates isolated session, rate-limit, and storage state.
 */
export async function createTestApp(): Promise<{ app: Express; server: Server }> {
  const app = express();
  app.use(express.json());

  // ─── Per-app isolated state ───
  const sessions = new Map<string, string>(); // sessionId → userId
  const authAttempts = new Map<string, number>();

  function rateLimitMiddleware(limit: number) {
    return (req: Request, res: Response, next: NextFunction) => {
      const key = req.ip ?? req.socket.remoteAddress ?? 'test';
      const count = authAttempts.get(key) ?? 0;
      if (count >= limit) {
        return res.status(429).json({ error: 'Too many requests' });
      }
      authAttempts.set(key, count + 1);
      next();
    };
  }

  // Reset storage for every test app
  const storage = new MemStorage();
  setStorage(storage);

  // ── Security headers (matching what helmet would add) ──
  app.use((_req, res, next) => {
    res.set('x-content-type-options', 'nosniff');
    res.set('x-frame-options', 'DENY');
    res.set('x-xss-protection', '1; mode=block');
    next();
  });

  // ── CORS ──
  app.use((req, res, next) => {
    if (req.method === 'OPTIONS') {
      res.set('access-control-allow-origin', req.headers.origin ?? '*');
      res.set('access-control-allow-methods', 'GET,POST,PUT,PATCH,DELETE');
      res.set('access-control-allow-headers', 'content-type,authorization');
      return res.status(204).end();
    }
    next();
  });

  // ── AUTH ROUTES ──

  app.post('/api/auth/register', rateLimitMiddleware(5), async (req, res) => {
    try {
      const { username, email, password } = req.body;

      if (!username || !email || !password) {
        return res.status(400).json({ error: 'Missing required fields' });
      }
      if (password.length < 8) {
        return res.status(400).json({ error: 'Password too short' });
      }

      const existingEmail = await getStorage().getUserByEmail(email);
      if (existingEmail) {
        return res.status(400).json({ error: 'Email already registered' });
      }
      const existingUsername = await getStorage().getUserByUsername(username);
      if (existingUsername) {
        return res.status(400).json({ error: 'Username already taken' });
      }

      const user = await getStorage().createUser({
        username,
        email,
        passwordHash: password, // In real app: hashed
      });

      const sessionId = randomUUID();
      sessions.set(sessionId, user.id);

      const { passwordHash: _, ...safeUser } = user;
      return res.status(201).json({ user: safeUser, sessionId });
    } catch (e) {
      return res.status(500).json({ error: 'Internal error' });
    }
  });

  app.post('/api/auth/login', async (req, res) => {
    const { email, password } = req.body;
    const user = await getStorage().getUserByEmail(email);
    if (!user || user.passwordHash !== password) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    const sessionId = randomUUID();
    sessions.set(sessionId, user.id);
    const { passwordHash: _, ...safeUser } = user;
    return res.json({ user: safeUser, sessionId });
  });

  app.get('/api/auth/me', async (req, res) => {
    const auth = req.headers.authorization;
    if (!auth?.startsWith('Bearer ')) return res.status(401).json({ error: 'Unauthorized' });
    const userId = sessions.get(auth.slice(7));
    if (!userId) return res.status(401).json({ error: 'Invalid session' });
    const user = await getStorage().getUser(userId);
    if (!user) return res.status(401).json({ error: 'User not found' });
    const { passwordHash: _, ...safeUser } = user;
    return res.json(safeUser);
  });

  app.patch('/api/auth/wallet', async (req, res) => {
    const auth = req.headers.authorization;
    if (!auth?.startsWith('Bearer ')) return res.status(401).json({ error: 'Unauthorized' });
    const userId = sessions.get(auth.slice(7));
    if (!userId) return res.status(401).json({ error: 'Invalid session' });
    const updated = await getStorage().updateUser(userId, { walletAddress: req.body.walletAddress });
    if (!updated) return res.status(404).json({ error: 'User not found' });
    const { passwordHash: _, ...safeUser } = updated;
    return res.json(safeUser);
  });

  app.post('/api/auth/logout', async (req, res) => {
    const auth = req.headers.authorization;
    if (auth?.startsWith('Bearer ')) sessions.delete(auth.slice(7));
    return res.json({ message: 'Logged out' });
  });

  // ── ENGINE ROUTES ──

  app.get('/api/engines', async (_req, res) => {
    const engines = await getStorage().getSecurityEngines();
    return res.json(engines);
  });

  app.post('/api/engines', async (req, res) => {
    const engine = await getStorage().createSecurityEngine({
      name: req.body.name,
      engineType: req.body.type ?? req.body.engineType ?? 'automated',
      description: req.body.description ?? null,
      ownerId: req.body.ownerId ?? null,
      isActive: true,
    });
    return res.status(201).json(engine);
  });

  // ── SUBMISSION ROUTES ──

  app.post('/api/submissions', async (req, res) => {
    const { filename, originalFilename, submissionType, description, fileHash } = req.body;

    if (!filename && !originalFilename) {
      return res.status(400).json({ error: 'Missing filename' });
    }
    if (!submissionType && !req.body.submissionType) {
      return res.status(400).json({ error: 'Missing submissionType' });
    }

    const submission = await getStorage().createSubmission({
      submitterId: '00000000-0000-0000-0000-000000000001', // placeholder
      originalFilename: originalFilename ?? filename ?? null,
      fileHash: fileHash ?? null,
      submissionType: submissionType ?? 'file',
    });

    return res.status(201).json(submission);
  });

  app.get('/api/submissions', async (_req, res) => {
    const subs = await getStorage().getSubmissions();
    return res.json(subs);
  });

  app.get('/api/submissions/:id', async (req, res) => {
    const sub = await getStorage().getSubmission(req.params.id);
    if (!sub) return res.status(404).json({ error: 'Not found' });
    return res.json(sub);
  });

  app.post('/api/submissions/:id/start-analysis', async (req, res) => {
    const sub = await getStorage().getSubmission(req.params.id);
    if (!sub) return res.status(404).json({ error: 'Not found' });

    await getStorage().updateSubmission(req.params.id, { analysisStatus: 'analyzing' });

    // Create analyses for each engine
    const engines = await getStorage().getSecurityEngines();
    for (const engine of engines) {
      await getStorage().createAnalysis({
        submissionId: req.params.id,
        engineId: engine.id,
        verdict: 'benign',
        confidenceScore: '0.5000',
      });
    }

    return res.json({ message: 'Analysis started', engines: engines.length });
  });

  app.get('/api/submissions/:id/analyses', async (req, res) => {
    const analyses = await getStorage().getAnalysesBySubmission(req.params.id);
    return res.json(analyses);
  });

  app.get('/api/submissions/:id/consensus', async (req, res) => {
    const result = await getStorage().getConsensusResult(req.params.id);
    if (!result) return res.status(404).json({ error: 'Consensus not available' });
    return res.json(result);
  });

  // ── STATS / BOUNTIES ──

  app.get('/api/stats', async (_req, res) => {
    const subs = await getStorage().getSubmissions();
    const engines = await getStorage().getSecurityEngines();
    return res.json({
      totalSubmissions: subs.length,
      activeAnalyses: subs.filter(s => s.analysisStatus === 'analyzing').length,
      completedToday: subs.filter(s => s.analysisStatus === 'completed').length,
      threatsDetected: subs.filter(s => s.isMalicious === true).length,
      totalEngines: engines.length,
    });
  });

  app.get('/api/bounties', async (_req, res) => {
    const bounties = await getStorage().getActiveBounties();
    return res.json(bounties);
  });

  const server = createServer(app);
  return { app, server };
}
