import { 
  type User, 
  type InsertUser, 
  type SecurityEngine, 
  type InsertSecurityEngine,
  type Submission,
  type InsertSubmission,
  type Analysis,
  type InsertAnalysis,
  type ConsensusResult,
  type Bounty,
  type InsertBounty
} from "@shared/schema";
import { randomUUID } from "crypto";

export interface IStorage {
  // User methods
  getUser(id: string): Promise<User | undefined>;
  getUserByUsername(username: string): Promise<User | undefined>;
  createUser(user: InsertUser): Promise<User>;

  // Security Engine methods
  getSecurityEngine(id: string): Promise<SecurityEngine | undefined>;
  getSecurityEngines(): Promise<SecurityEngine[]>;
  createSecurityEngine(engine: InsertSecurityEngine): Promise<SecurityEngine>;
  updateSecurityEngine(id: string, updates: Partial<SecurityEngine>): Promise<SecurityEngine | undefined>;

  // Submission methods
  getSubmission(id: string): Promise<Submission | undefined>;
  getSubmissions(): Promise<Submission[]>;
  createSubmission(submission: InsertSubmission): Promise<Submission>;
  updateSubmission(id: string, updates: Partial<Submission>): Promise<Submission | undefined>;

  // Analysis methods
  getAnalysis(id: string): Promise<Analysis | undefined>;
  getAnalysesBySubmission(submissionId: string): Promise<Analysis[]>;
  createAnalysis(analysis: InsertAnalysis): Promise<Analysis>;
  updateAnalysis(id: string, updates: Partial<Analysis>): Promise<Analysis | undefined>;

  // Consensus methods
  getConsensusResult(submissionId: string): Promise<ConsensusResult | undefined>;
  createConsensusResult(result: Omit<ConsensusResult, 'id' | 'createdAt'>): Promise<ConsensusResult>;

  // Bounty methods
  getBounty(id: string): Promise<Bounty | undefined>;
  getActiveBounties(): Promise<Bounty[]>;
  createBounty(bounty: InsertBounty): Promise<Bounty>;
  updateBounty(id: string, updates: Partial<Bounty>): Promise<Bounty | undefined>;
}

export class MemStorage implements IStorage {
  private users: Map<string, User>;
  private securityEngines: Map<string, SecurityEngine>;
  private submissions: Map<string, Submission>;
  private analyses: Map<string, Analysis>;
  private consensusResults: Map<string, ConsensusResult>;
  private bounties: Map<string, Bounty>;

  constructor() {
    this.users = new Map();
    this.securityEngines = new Map();
    this.submissions = new Map();
    this.analyses = new Map();
    this.consensusResults = new Map();
    this.bounties = new Map();
    
    // Initialize with some mock engines
    this.initializeMockData();
  }

  private initializeMockData() {
    // Create mock security engines
    const engines: SecurityEngine[] = [
      {
        id: randomUUID(),
        name: "DeepScan AI",
        type: "automated",
        description: "Advanced AI-powered malware detection",
        accuracy: "94.2",
        totalAnalyses: 15742,
        totalStaked: "245.0",
        status: "online",
        ownerId: null,
        createdAt: new Date(),
      },
      {
        id: randomUUID(),
        name: "CyberExpert_007",
        type: "human",
        description: "Senior cybersecurity researcher",
        accuracy: "98.7",
        totalAnalyses: 892,
        totalStaked: "89.0",
        status: "online",
        ownerId: null,
        createdAt: new Date(),
      },
      {
        id: randomUUID(),
        name: "NeuralGuard",
        type: "ml",
        description: "Machine learning threat classifier",
        accuracy: "91.8",
        totalAnalyses: 8234,
        totalStaked: "156.0",
        status: "online",
        ownerId: null,
        createdAt: new Date(),
      },
      {
        id: randomUUID(),
        name: "SigHunter",
        type: "signature",
        description: "Signature-based detection engine",
        accuracy: "89.4",
        totalAnalyses: 22156,
        totalStaked: "178.0",
        status: "online",
        ownerId: null,
        createdAt: new Date(),
      },
    ];

    engines.forEach(engine => this.securityEngines.set(engine.id, engine));
  }

  // User methods
  async getUser(id: string): Promise<User | undefined> {
    return this.users.get(id);
  }

  async getUserByUsername(username: string): Promise<User | undefined> {
    return Array.from(this.users.values()).find(
      (user) => user.username === username,
    );
  }

  async createUser(insertUser: InsertUser): Promise<User> {
    const id = randomUUID();
    const user: User = { 
      ...insertUser, 
      id, 
      reputation: "0",
      totalStaked: "0",
      totalEarned: "0",
      walletAddress: insertUser.walletAddress || null
    };
    this.users.set(id, user);
    return user;
  }

  // Security Engine methods
  async getSecurityEngine(id: string): Promise<SecurityEngine | undefined> {
    return this.securityEngines.get(id);
  }

  async getSecurityEngines(): Promise<SecurityEngine[]> {
    return Array.from(this.securityEngines.values());
  }

  async createSecurityEngine(engine: InsertSecurityEngine): Promise<SecurityEngine> {
    const id = randomUUID();
    const newEngine: SecurityEngine = {
      ...engine,
      id,
      accuracy: "0",
      totalAnalyses: 0,
      totalStaked: "0",
      createdAt: new Date(),
      description: engine.description || null,
      status: engine.status || "online",
      ownerId: engine.ownerId || null
    };
    this.securityEngines.set(id, newEngine);
    return newEngine;
  }

  async updateSecurityEngine(id: string, updates: Partial<SecurityEngine>): Promise<SecurityEngine | undefined> {
    const engine = this.securityEngines.get(id);
    if (!engine) return undefined;
    
    const updated = { ...engine, ...updates };
    this.securityEngines.set(id, updated);
    return updated;
  }

  // Submission methods
  async getSubmission(id: string): Promise<Submission | undefined> {
    return this.submissions.get(id);
  }

  async getSubmissions(): Promise<Submission[]> {
    return Array.from(this.submissions.values()).sort((a, b) => 
      new Date(b.createdAt!).getTime() - new Date(a.createdAt!).getTime()
    );
  }

  async createSubmission(submission: InsertSubmission): Promise<Submission> {
    const id = randomUUID();
    const newSubmission: Submission = {
      ...submission,
      id,
      status: "pending",
      createdAt: new Date(),
      completedAt: null,
      description: submission.description || null,
      fileSize: submission.fileSize || null,
      priority: submission.priority || null,
      submitterId: submission.submitterId || null
    };
    this.submissions.set(id, newSubmission);
    return newSubmission;
  }

  async updateSubmission(id: string, updates: Partial<Submission>): Promise<Submission | undefined> {
    const submission = this.submissions.get(id);
    if (!submission) return undefined;
    
    const updated = { ...submission, ...updates };
    this.submissions.set(id, updated);
    return updated;
  }

  // Analysis methods
  async getAnalysis(id: string): Promise<Analysis | undefined> {
    return this.analyses.get(id);
  }

  async getAnalysesBySubmission(submissionId: string): Promise<Analysis[]> {
    return Array.from(this.analyses.values()).filter(
      analysis => analysis.submissionId === submissionId
    );
  }

  async createAnalysis(analysis: InsertAnalysis): Promise<Analysis> {
    const id = randomUUID();
    const newAnalysis: Analysis = {
      ...analysis,
      id,
      status: "pending",
      createdAt: new Date(),
      completedAt: null,
      details: analysis.details || null,
      verdict: analysis.verdict || null,
      confidence: analysis.confidence || null
    };
    this.analyses.set(id, newAnalysis);
    return newAnalysis;
  }

  async updateAnalysis(id: string, updates: Partial<Analysis>): Promise<Analysis | undefined> {
    const analysis = this.analyses.get(id);
    if (!analysis) return undefined;
    
    const updated = { ...analysis, ...updates };
    this.analyses.set(id, updated);
    return updated;
  }

  // Consensus methods
  async getConsensusResult(submissionId: string): Promise<ConsensusResult | undefined> {
    return Array.from(this.consensusResults.values()).find(
      result => result.submissionId === submissionId
    );
  }

  async createConsensusResult(result: Omit<ConsensusResult, 'id' | 'createdAt'>): Promise<ConsensusResult> {
    const id = randomUUID();
    const consensusResult: ConsensusResult = {
      ...result,
      id,
      createdAt: new Date(),
    };
    this.consensusResults.set(id, consensusResult);
    return consensusResult;
  }

  // Bounty methods
  async getBounty(id: string): Promise<Bounty | undefined> {
    return this.bounties.get(id);
  }

  async getActiveBounties(): Promise<Bounty[]> {
    return Array.from(this.bounties.values()).filter(
      bounty => bounty.status === "active"
    );
  }

  async createBounty(bounty: InsertBounty): Promise<Bounty> {
    const id = randomUUID();
    const newBounty: Bounty = {
      ...bounty,
      id,
      status: "active",
      createdAt: new Date(),
      expiresAt: bounty.expiresAt || null
    };
    this.bounties.set(id, newBounty);
    return newBounty;
  }

  async updateBounty(id: string, updates: Partial<Bounty>): Promise<Bounty | undefined> {
    const bounty = this.bounties.get(id);
    if (!bounty) return undefined;
    
    const updated = { ...bounty, ...updates };
    this.bounties.set(id, updated);
    return updated;
  }
}

export const storage = new MemStorage();
