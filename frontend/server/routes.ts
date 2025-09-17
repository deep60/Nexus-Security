import type { Express } from "express";
import { createServer, type Server } from "http";
import { WebSocketServer, WebSocket } from "ws";
import { storage } from "./storage";
import { insertSubmissionSchema, insertAnalysisSchema, insertSecurityEngineSchema } from "@shared/schema";
import { randomUUID } from "crypto";

export async function registerRoutes(app: Express): Promise<Server> {
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

  // Security Engines endpoints
  app.get("/api/engines", async (req, res) => {
    try {
      const engines = await storage.getSecurityEngines();
      res.json(engines);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch security engines" });
    }
  });

  app.post("/api/engines", async (req, res) => {
    try {
      const engineData = insertSecurityEngineSchema.parse(req.body);
      const engine = await storage.createSecurityEngine(engineData);
      res.status(201).json(engine);
    } catch (error) {
      res.status(400).json({ error: "Invalid engine data" });
    }
  });

  // Submissions endpoints
  app.get("/api/submissions", async (req, res) => {
    try {
      const submissions = await storage.getSubmissions();
      res.json(submissions);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch submissions" });
    }
  });

  app.get("/api/submissions/:id", async (req, res) => {
    try {
      const submission = await storage.getSubmission(req.params.id);
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
      const submission = await storage.createSubmission({
        ...submissionData,
        fileHash: `sha256_${randomUUID()}`, // Mock file hash
        submitterId: null, // In real app, get from authenticated user
      });

      // Create associated bounty
      await storage.createBounty({
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
      const analyses = await storage.getAnalysesBySubmission(req.params.id);
      res.json(analyses);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch analyses" });
    }
  });

  app.post("/api/submissions/:submissionId/analyses", async (req, res) => {
    try {
      const analysisData = insertAnalysisSchema.parse(req.body);
      const analysis = await storage.createAnalysis({
        ...analysisData,
        submissionId: req.params.submissionId,
      });

      // Simulate analysis completion after random delay
      setTimeout(async () => {
        const verdicts = ['malicious', 'clean', 'suspicious'];
        const verdict = verdicts[Math.floor(Math.random() * verdicts.length)];
        const confidence = (Math.random() * 40 + 60).toFixed(1); // 60-100%
        
        await storage.updateAnalysis(analysis.id, {
          verdict,
          confidence,
          status: "completed",
          completedAt: new Date(),
        });

        // Check if all engines have completed analysis
        const allAnalyses = await storage.getAnalysesBySubmission(req.params.submissionId);
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
          
          const consensusResult = await storage.createConsensusResult({
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
          await storage.updateSubmission(req.params.submissionId, {
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

  // Bounties endpoints
  app.get("/api/bounties", async (req, res) => {
    try {
      const bounties = await storage.getActiveBounties();
      res.json(bounties);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch bounties" });
    }
  });

  // Statistics endpoints
  app.get("/api/stats", async (req, res) => {
    try {
      const submissions = await storage.getSubmissions();
      const engines = await storage.getSecurityEngines();
      const bounties = await storage.getActiveBounties();
      
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
      const submission = await storage.getSubmission(req.params.id);
      if (!submission) {
        return res.status(404).json({ error: "Submission not found" });
      }

      await storage.updateSubmission(req.params.id, { status: "analyzing" });

      // Create analyses for all engines
      const engines = await storage.getSecurityEngines();
      for (const engine of engines.slice(0, 4)) { // Use first 4 engines
        await storage.createAnalysis({
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
