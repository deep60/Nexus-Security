import type { Express } from "express";
import { createServer, type Server } from "http";
import { WebSocketServer, WebSocket } from "ws";
import { MemStorage } from "./storage";
import { insertSubmissionSchema, insertAnalysisSchema, insertSecurityEngineSchema, insertUserSchema } from "@shared/schema";
import { randomUUID } from "crypto";
import bcrypt from "bcrypt";
import helmet from "helmet";
import rateLimit from "express-rate-limit";
import cors from "cors";
import { config } from "./config";
import { initializeDatabase, getDatabase } from "./db";
import { initializeRedis, getSessionStore } from "./redis";
import { PostgresStorage } from "./pg-storage";

// Initialize database and Redis connections
initializeDatabase();
initializeRedis();

// Get the appropriate storage implementation
const db = getDatabase();
let storageInstance = db ? new PostgresStorage(db) : new MemStorage();

// Initialize mock data if using PostgreSQL
if (db && storageInstance instanceof PostgresStorage) {
  storageInstance.initializeMockData().catch(console.error);
}

// Get session store (Redis or in-memory)
const sessionStore = getSessionStore();

// Middleware to check authentication
const requireAuth = async (req: any, res: any, next: any) => {
  const sessionId = req.headers.authorization?.replace('Bearer ', '');
  if (!sessionId) {
    return res.status(401).json({ error: "Unauthorized" });
  }

  const session = await sessionStore.get(sessionId);
  if (!session || session.expiresAt < Date.now()) {
    await sessionStore.delete(sessionId);
    return res.status(401).json({ error: "Session expired" });
  }

  req.userId = session.userId;
  next();
};

export async function registerRoutes(app: Express): Promise<Server> {
  // Security middleware - Helmet for HTTP headers
  app.use(helmet({
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        styleSrc: ["'self'", "'unsafe-inline'"],
        scriptSrc: ["'self'", "'unsafe-inline'"],
        imgSrc: ["'self'", "data:", "https:"],
        connectSrc: ["'self'", "ws:", "wss:"],
      },
    },
    crossOriginEmbedderPolicy: false,
  }));

  // CORS configuration
  app.use(cors({
    origin: config.isProduction
      ? [config.frontendUrl]
      : [config.frontendUrl, "http://localhost:5173", "http://127.0.0.1:5173"],
    credentials: true,
    methods: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization'],
  }));

  // Rate limiting for API routes
  const apiLimiter = rateLimit({
    windowMs: config.rateLimitWindowMs, // 15 minutes
    max: config.rateLimitMaxRequests, // limit each IP to 100 requests per windowMs
    message: 'Too many requests from this IP, please try again later.',
    standardHeaders: true,
    legacyHeaders: false,
  });

  // Apply rate limiting to all API routes
  app.use('/api/', apiLimiter);

  // Stricter rate limiting for authentication endpoints
  const authLimiter = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 5, // limit each IP to 5 requests per windowMs
    message: 'Too many authentication attempts, please try again later.',
    standardHeaders: true,
    legacyHeaders: false,
  });

  const httpServer = createServer(app);

  // WebSocket server for real-time updates
  const wss = new WebSocketServer({ server: httpServer, path: '/ws' });

  // Store connected clients
  const clients = new Set<WebSocket>();

  wss.on('connection', (ws) => {
    clients.add(ws);
    console.log('Client connected to WebSocket');

    ws.on('close', () => {
      clients.delete(ws);
      console.log('Client disconnected from WebSocket');
    });
  });

  // Broadcast to all connected clients
  const broadcast = (data: any) => {
    const message = JSON.stringify(data);
    clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(message);
      }
    });
  };

  // Authentication endpoints with stricter rate limiting
  app.post("/api/auth/register", authLimiter, async (req, res) => {
    try {
      const { username, email, password } = req.body;

      // Validate input
      if (!username || !email || !password) {
        return res.status(400).json({ error: "Missing required fields" });
      }

      // Check password strength
      if (password.length < 8) {
        return res.status(400).json({ error: "Password must be at least 8 characters" });
      }

      // Check if user already exists
      const existingUser = await storageInstance.getUserByEmail(email);
      if (existingUser) {
        return res.status(400).json({ error: "Email already registered" });
      }

      const existingUsername = await storageInstance.getUserByUsername(username);
      if (existingUsername) {
        return res.status(400).json({ error: "Username already taken" });
      }

      // Hash password with bcrypt
      const hashedPassword = await bcrypt.hash(password, config.bcryptSaltRounds);

      // Create user with hashed password
      const userData = insertUserSchema.parse({ username, email, password: hashedPassword });
      const user = await storageInstance.createUser(userData);

      // Create session
      const sessionId = randomUUID();
      await sessionStore.set(sessionId, {
        userId: user.id,
        expiresAt: Date.now() + config.sessionExpiry,
      });

      // Remove password from response
      const { password: _, ...userWithoutPassword } = user;

      res.status(201).json({
        user: userWithoutPassword,
        sessionId,
      });
    } catch (error) {
      console.error("Registration error:", error);
      res.status(400).json({ error: "Registration failed" });
    }
  });

  app.post("/api/auth/login", authLimiter, async (req, res) => {
    try {
      const { email, password } = req.body;

      // Validate input
      if (!email || !password) {
        return res.status(400).json({ error: "Missing email or password" });
      }

      const user = await storageInstance.getUserByEmail(email);
      if (!user) {
        // Use generic error to prevent user enumeration
        return res.status(401).json({ error: "Invalid credentials" });
      }

      // Compare password with bcrypt
      const isValidPassword = await bcrypt.compare(password, user.password);
      if (!isValidPassword) {
        return res.status(401).json({ error: "Invalid credentials" });
      }

      // Create session
      const sessionId = randomUUID();
      await sessionStore.set(sessionId, {
        userId: user.id,
        expiresAt: Date.now() + config.sessionExpiry,
      });

      // Remove password from response
      const { password: _, ...userWithoutPassword } = user;

      res.json({
        user: userWithoutPassword,
        sessionId,
      });
    } catch (error) {
      console.error("Login error:", error);
      res.status(500).json({ error: "Login failed" });
    }
  });

  app.post("/api/auth/logout", async (req, res) => {
    const sessionId = req.headers.authorization?.replace('Bearer ', '');
    if (sessionId) {
      await sessionStore.delete(sessionId);
    }
    res.json({ message: "Logged out successfully" });
  });

  app.get("/api/auth/me", requireAuth, async (req: any, res) => {
    try {
      const user = await storageInstance.getUser(req.userId);
      if (!user) {
        return res.status(404).json({ error: "User not found" });
      }

      const { password: _, ...userWithoutPassword } = user;
      res.json(userWithoutPassword);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch user" });
    }
  });

  app.patch("/api/auth/wallet", requireAuth, async (req: any, res) => {
    try {
      const { walletAddress } = req.body;
      const updated = await storageInstance.updateUser(req.userId, { walletAddress });

      if (!updated) {
        return res.status(404).json({ error: "User not found" });
      }

      const { password: _, ...userWithoutPassword } = updated;
      res.json(userWithoutPassword);
    } catch (error) {
      res.status(500).json({ error: "Failed to update wallet" });
    }
  });

  // Security Engines endpoints
  app.get("/api/engines", async (req, res) => {
    try {
      const engines = await storageInstance.getSecurityEngines();
      res.json(engines);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch security engines" });
    }
  });

  app.post("/api/engines", async (req, res) => {
    try {
      const engineData = insertSecurityEngineSchema.parse(req.body);
      const engine = await storageInstance.createSecurityEngine(engineData);
      res.status(201).json(engine);
    } catch (error) {
      res.status(400).json({ error: "Invalid engine data" });
    }
  });

  // Submissions endpoints
  app.get("/api/submissions", async (req, res) => {
    try {
      const submissions = await storageInstance.getSubmissions();
      res.json(submissions);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch submissions" });
    }
  });

  app.get("/api/submissions/:id", async (req, res) => {
    try {
      const submission = await storageInstance.getSubmission(req.params.id);
      if (!submission) {
        return res.status(404).json({ error: "Submission not found" });
      }
      res.json(submission);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch submission" });
    }
  });

  app.post("/api/submissions", async (req, res) => {
    try {
      const submissionData = insertSubmissionSchema.parse(req.body);
      const submission = await storageInstance.createSubmission({
        ...submissionData,
        fileHash: `sha256_${randomUUID()}`, // Mock file hash
        submitterId: null, // In real app, get from authenticated user
      });

      // Create associated bounty
      await storageInstance.createBounty({
        submissionId: submission.id,
        amount: submissionData.bountyAmount,
        expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000), // 7 days
      });

      // Broadcast new submission
      broadcast({
        type: 'new_submission',
        data: submission
      });

      res.status(201).json(submission);
    } catch (error) {
      res.status(400).json({ error: "Invalid submission data" });
    }
  });

  // Analysis endpoints
  app.get("/api/submissions/:id/analyses", async (req, res) => {
    try {
      const analyses = await storageInstance.getAnalysesBySubmission(req.params.id);
      res.json(analyses);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch analyses" });
    }
  });

  app.post("/api/submissions/:submissionId/analyses", async (req, res) => {
    try {
      const analysisData = insertAnalysisSchema.parse(req.body);
      const analysis = await storageInstance.createAnalysis({
        ...analysisData,
        submissionId: req.params.submissionId,
      });

      // Simulate analysis completion after random delay
      setTimeout(async () => {
        const verdicts = ['malicious', 'clean', 'suspicious'];
        const verdict = verdicts[Math.floor(Math.random() * verdicts.length)];
        const confidence = (Math.random() * 40 + 60).toFixed(1); // 60-100%
        
        await storageInstance.updateAnalysis(analysis.id, {
          verdict,
          confidence,
          status: "completed",
          completedAt: new Date(),
        });

        // Check if all engines have completed analysis
        const allAnalyses = await storageInstance.getAnalysesBySubmission(req.params.submissionId);
        const completedAnalyses = allAnalyses.filter(a => a.status === "completed");
        
        if (completedAnalyses.length >= 4) { // Assume 4 engines minimum for consensus
          // Calculate consensus
          const maliciousVotes = completedAnalyses.filter(a => a.verdict === "malicious").length;
          const cleanVotes = completedAnalyses.filter(a => a.verdict === "clean").length;
          const suspiciousVotes = completedAnalyses.filter(a => a.verdict === "suspicious").length;
          
          const finalVerdict = maliciousVotes > cleanVotes ? 
            (maliciousVotes > suspiciousVotes ? "malicious" : "suspicious") :
            (cleanVotes > suspiciousVotes ? "clean" : "suspicious");
          
          const confidenceScore = ((Math.max(maliciousVotes, cleanVotes, suspiciousVotes) / completedAnalyses.length) * 100).toFixed(1);
          
          const consensusResult = await storageInstance.createConsensusResult({
            submissionId: req.params.submissionId,
            finalVerdict,
            confidenceScore,
            totalEngines: completedAnalyses.length,
            maliciousVotes,
            cleanVotes,
            suspiciousVotes,
            rewardsDistributed: false,
          });

          // Update submission status
          await storageInstance.updateSubmission(req.params.submissionId, {
            status: "completed",
            completedAt: new Date(),
          });

          // Broadcast analysis completion
          broadcast({
            type: 'analysis_completed',
            data: {
              submissionId: req.params.submissionId,
              consensus: consensusResult,
              analyses: completedAnalyses
            }
          });
        }

        // Broadcast analysis update
        broadcast({
          type: 'analysis_updated',
          data: analysis
        });
      }, Math.random() * 5000 + 2000); // 2-7 seconds

      res.status(201).json(analysis);
    } catch (error) {
      res.status(400).json({ error: "Invalid analysis data" });
    }
  });

  // Consensus endpoint
  app.get("/api/submissions/:id/consensus", async (req, res) => {
    try {
      const consensus = await storageInstance.getConsensusResult(req.params.id);
      if (!consensus) {
        return res.status(404).json({ error: "Consensus result not found" });
      }
      res.json(consensus);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch consensus" });
    }
  });

  // Bounties endpoints
  app.get("/api/bounties", async (req, res) => {
    try {
      const bounties = await storageInstance.getActiveBounties();
      res.json(bounties);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch bounties" });
    }
  });

  // Statistics endpoints
  app.get("/api/stats", async (req, res) => {
    try {
      const submissions = await storageInstance.getSubmissions();
      const engines = await storageInstance.getSecurityEngines();
      const bounties = await storageInstance.getActiveBounties();
      
      const stats = {
        totalSubmissions: submissions.length,
        activeAnalyses: submissions.filter(s => s.status === "analyzing").length,
        completedToday: submissions.filter(s => 
          s.completedAt && 
          new Date(s.completedAt).toDateString() === new Date().toDateString()
        ).length,
        threatsDetected: submissions.filter(s => s.status === "completed").length * 0.184, // ~18.4% threat rate
        totalActiveBounties: bounties.reduce((sum, b) => sum + parseFloat(b.amount), 0).toFixed(2),
        totalEngines: engines.length,
        avgResponseTime: "24.7s",
        totalRewardsPaid: "312.8",
      };
      
      res.json(stats);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch statistics" });
    }
  });

  // Mock endpoint to start analysis for demo purposes
  app.post("/api/submissions/:id/start-analysis", async (req, res) => {
    try {
      const submission = await storageInstance.getSubmission(req.params.id);
      if (!submission) {
        return res.status(404).json({ error: "Submission not found" });
      }

      await storageInstance.updateSubmission(req.params.id, { status: "analyzing" });

      // Create analyses for all engines
      const engines = await storageInstance.getSecurityEngines();
      for (const engine of engines.slice(0, 4)) { // Use first 4 engines
        await storageInstance.createAnalysis({
          submissionId: req.params.id,
          engineId: engine.id,
          stakeAmount: (Math.random() * 0.1 + 0.05).toFixed(3), // 0.05-0.15 ETH
          verdict: null,
          confidence: null,
          details: null,
        });
      }

      broadcast({
        type: 'analysis_started',
        data: { submissionId: req.params.id }
      });

      res.json({ message: "Analysis started" });
    } catch (error) {
      res.status(500).json({ error: "Failed to start analysis" });
    }
  });

  return httpServer;
}
