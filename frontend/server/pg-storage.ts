/**
 * PostgreSQL Storage Implementation
 * Uses Drizzle ORM for database operations
 */

import { eq, desc } from "drizzle-orm";
import type { NodePgDatabase } from "drizzle-orm/node-postgres";
import * as schema from "@shared/schema";
import type {
  User,
  InsertUser,
  SecurityEngine,
  InsertSecurityEngine,
  Submission,
  InsertSubmission,
  Analysis,
  InsertAnalysis,
  ConsensusResult,
  Bounty,
  InsertBounty
} from "@shared/schema";
import type { IStorage } from "./storage";
import { randomUUID } from "crypto";

export class PostgresStorage implements IStorage {
  private db: NodePgDatabase<typeof schema>;

  constructor(database: NodePgDatabase<typeof schema>) {
    this.db = database;
  }

  /**
   * Initialize database with mock data (for development)
   */
  async initializeMockData() {
    try {
      // Check if engines already exist
      const existingEngines = await this.getSecurityEngines();
      if (existingEngines.length > 0) {
        console.log("✅ Mock data already exists");
        return;
      }

      console.log("📦 Initializing mock security engines...");

      const mockEngines: InsertSecurityEngine[] = [
        {
          name: "DeepScan AI",
          type: "automated",
          description: "Advanced AI-powered malware detection",
        },
        {
          name: "CyberExpert_007",
          type: "human",
          description: "Senior cybersecurity researcher",
        },
        {
          name: "NeuralGuard",
          type: "ml",
          description: "Machine learning threat classifier",
        },
        {
          name: "SigHunter",
          type: "signature",
          description: "Signature-based detection engine",
        },
      ];

      for (const engine of mockEngines) {
        await this.createSecurityEngine(engine);
      }

      console.log("✅ Mock data initialized");
    } catch (error) {
      console.error("❌ Failed to initialize mock data:", error);
    }
  }

  // User methods
  async getUser(id: string): Promise<User | undefined> {
    const [user] = await this.db
      .select()
      .from(schema.users)
      .where(eq(schema.users.id, id))
      .limit(1);
    return user;
  }

  async getUserByUsername(username: string): Promise<User | undefined> {
    const [user] = await this.db
      .select()
      .from(schema.users)
      .where(eq(schema.users.username, username))
      .limit(1);
    return user;
  }

  async getUserByEmail(email: string): Promise<User | undefined> {
    const [user] = await this.db
      .select()
      .from(schema.users)
      .where(eq(schema.users.email, email))
      .limit(1);
    return user;
  }

  async createUser(insertUser: InsertUser): Promise<User> {
    const [user] = await this.db
      .insert(schema.users)
      .values({
        id: randomUUID(),
        ...insertUser,
      })
      .returning();
    return user;
  }

  async updateUser(id: string, updates: Partial<User>): Promise<User | undefined> {
    const [user] = await this.db
      .update(schema.users)
      .set(updates)
      .where(eq(schema.users.id, id))
      .returning();
    return user;
  }

  // Security Engine methods
  async getSecurityEngine(id: string): Promise<SecurityEngine | undefined> {
    const [engine] = await this.db
      .select()
      .from(schema.securityEngines)
      .where(eq(schema.securityEngines.id, id))
      .limit(1);
    return engine;
  }

  async getSecurityEngines(): Promise<SecurityEngine[]> {
    return await this.db
      .select()
      .from(schema.securityEngines)
      .orderBy(desc(schema.securityEngines.createdAt));
  }

  async createSecurityEngine(engine: InsertSecurityEngine): Promise<SecurityEngine> {
    const [newEngine] = await this.db
      .insert(schema.securityEngines)
      .values({
        id: randomUUID(),
        ...engine,
      })
      .returning();
    return newEngine;
  }

  async updateSecurityEngine(id: string, updates: Partial<SecurityEngine>): Promise<SecurityEngine | undefined> {
    const [engine] = await this.db
      .update(schema.securityEngines)
      .set(updates)
      .where(eq(schema.securityEngines.id, id))
      .returning();
    return engine;
  }

  // Submission methods
  async getSubmission(id: string): Promise<Submission | undefined> {
    const [submission] = await this.db
      .select()
      .from(schema.submissions)
      .where(eq(schema.submissions.id, id))
      .limit(1);
    return submission;
  }

  async getSubmissions(): Promise<Submission[]> {
    return await this.db
      .select()
      .from(schema.submissions)
      .orderBy(desc(schema.submissions.createdAt));
  }

  async createSubmission(submission: InsertSubmission): Promise<Submission> {
    const [newSubmission] = await this.db
      .insert(schema.submissions)
      .values({
        id: randomUUID(),
        ...submission,
      })
      .returning();
    return newSubmission;
  }

  async updateSubmission(id: string, updates: Partial<Submission>): Promise<Submission | undefined> {
    const [submission] = await this.db
      .update(schema.submissions)
      .set(updates)
      .where(eq(schema.submissions.id, id))
      .returning();
    return submission;
  }

  // Analysis methods
  async getAnalysis(id: string): Promise<Analysis | undefined> {
    const [analysis] = await this.db
      .select()
      .from(schema.analyses)
      .where(eq(schema.analyses.id, id))
      .limit(1);
    return analysis;
  }

  async getAnalysesBySubmission(submissionId: string): Promise<Analysis[]> {
    return await this.db
      .select()
      .from(schema.analyses)
      .where(eq(schema.analyses.submissionId, submissionId))
      .orderBy(desc(schema.analyses.createdAt));
  }

  async createAnalysis(analysis: InsertAnalysis): Promise<Analysis> {
    const [newAnalysis] = await this.db
      .insert(schema.analyses)
      .values({
        id: randomUUID(),
        ...analysis,
      })
      .returning();
    return newAnalysis;
  }

  async updateAnalysis(id: string, updates: Partial<Analysis>): Promise<Analysis | undefined> {
    const [analysis] = await this.db
      .update(schema.analyses)
      .set(updates)
      .where(eq(schema.analyses.id, id))
      .returning();
    return analysis;
  }

  // Consensus methods
  async getConsensusResult(submissionId: string): Promise<ConsensusResult | undefined> {
    const [result] = await this.db
      .select()
      .from(schema.consensusResults)
      .where(eq(schema.consensusResults.submissionId, submissionId))
      .limit(1);
    return result;
  }

  async createConsensusResult(result: Omit<ConsensusResult, 'id' | 'createdAt'>): Promise<ConsensusResult> {
    const [consensusResult] = await this.db
      .insert(schema.consensusResults)
      .values({
        id: randomUUID(),
        ...result,
      })
      .returning();
    return consensusResult;
  }

  // Bounty methods
  async getBounty(id: string): Promise<Bounty | undefined> {
    const [bounty] = await this.db
      .select()
      .from(schema.bounties)
      .where(eq(schema.bounties.id, id))
      .limit(1);
    return bounty;
  }

  async getActiveBounties(): Promise<Bounty[]> {
    return await this.db
      .select()
      .from(schema.bounties)
      .where(eq(schema.bounties.status, "active"))
      .orderBy(desc(schema.bounties.createdAt));
  }

  async createBounty(bounty: InsertBounty): Promise<Bounty> {
    const [newBounty] = await this.db
      .insert(schema.bounties)
      .values({
        id: randomUUID(),
        ...bounty,
      })
      .returning();
    return newBounty;
  }

  async updateBounty(id: string, updates: Partial<Bounty>): Promise<Bounty | undefined> {
    const [bounty] = await this.db
      .update(schema.bounties)
      .set(updates)
      .where(eq(schema.bounties.id, id))
      .returning();
    return bounty;
  }
}
