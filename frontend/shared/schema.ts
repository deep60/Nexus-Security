import { sql } from "drizzle-orm";
import { pgTable, text, varchar, decimal, integer, timestamp, boolean, jsonb } from "drizzle-orm/pg-core";
import { createInsertSchema } from "drizzle-zod";
import { z } from "zod";

export const users = pgTable("users", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  username: text("username").notNull().unique(),
  password: text("password").notNull(),
  walletAddress: text("wallet_address"),
  reputation: decimal("reputation", { precision: 10, scale: 2 }).default("0"),
  totalStaked: decimal("total_staked", { precision: 18, scale: 8 }).default("0"),
  totalEarned: decimal("total_earned", { precision: 18, scale: 8 }).default("0"),
});

export const securityEngines = pgTable("security_engines", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  name: text("name").notNull(),
  type: text("type").notNull(), // 'automated', 'human', 'ml', 'signature'
  description: text("description"),
  accuracy: decimal("accuracy", { precision: 5, scale: 2 }).default("0"),
  totalAnalyses: integer("total_analyses").default(0),
  totalStaked: decimal("total_staked", { precision: 18, scale: 8 }).default("0"),
  status: text("status").default("online"), // 'online', 'offline', 'busy'
  ownerId: varchar("owner_id").references(() => users.id),
  createdAt: timestamp("created_at").defaultNow(),
});

export const submissions = pgTable("submissions", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  filename: text("filename").notNull(),
  fileHash: text("file_hash").notNull(),
  fileSize: integer("file_size"),
  submissionType: text("submission_type").notNull(), // 'file', 'url'
  analysisType: text("analysis_type").notNull(), // 'quick', 'full', 'deep', 'behavioral'
  bountyAmount: decimal("bounty_amount", { precision: 18, scale: 8 }).notNull(),
  priority: boolean("priority").default(false),
  description: text("description"),
  status: text("status").default("pending"), // 'pending', 'analyzing', 'completed', 'failed'
  submitterId: varchar("submitter_id").references(() => users.id),
  createdAt: timestamp("created_at").defaultNow(),
  completedAt: timestamp("completed_at"),
});

export const analyses = pgTable("analyses", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  submissionId: varchar("submission_id").references(() => submissions.id).notNull(),
  engineId: varchar("engine_id").references(() => securityEngines.id).notNull(),
  verdict: text("verdict"), // 'malicious', 'clean', 'suspicious', 'unknown'
  confidence: decimal("confidence", { precision: 5, scale: 2 }),
  stakeAmount: decimal("stake_amount", { precision: 18, scale: 8 }).notNull(),
  details: jsonb("details"), // Additional analysis details
  status: text("status").default("pending"), // 'pending', 'analyzing', 'completed'
  createdAt: timestamp("created_at").defaultNow(),
  completedAt: timestamp("completed_at"),
});

export const consensusResults = pgTable("consensus_results", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  submissionId: varchar("submission_id").references(() => submissions.id).notNull(),
  finalVerdict: text("final_verdict").notNull(),
  confidenceScore: decimal("confidence_score", { precision: 5, scale: 2 }).notNull(),
  totalEngines: integer("total_engines").notNull(),
  maliciousVotes: integer("malicious_votes").default(0),
  cleanVotes: integer("clean_votes").default(0),
  suspiciousVotes: integer("suspicious_votes").default(0),
  rewardsDistributed: boolean("rewards_distributed").default(false),
  createdAt: timestamp("created_at").defaultNow(),
});

export const bounties = pgTable("bounties", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  submissionId: varchar("submission_id").references(() => submissions.id).notNull(),
  amount: decimal("amount", { precision: 18, scale: 8 }).notNull(),
  status: text("status").default("active"), // 'active', 'completed', 'expired'
  expiresAt: timestamp("expires_at"),
  createdAt: timestamp("created_at").defaultNow(),
});

// Insert schemas
export const insertUserSchema = createInsertSchema(users).omit({
  id: true,
  reputation: true,
  totalStaked: true,
  totalEarned: true,
});

export const insertSecurityEngineSchema = createInsertSchema(securityEngines).omit({
  id: true,
  accuracy: true,
  totalAnalyses: true,
  totalStaked: true,
  createdAt: true,
});

export const insertSubmissionSchema = createInsertSchema(submissions).omit({
  id: true,
  status: true,
  createdAt: true,
  completedAt: true,
});

export const insertAnalysisSchema = createInsertSchema(analyses).omit({
  id: true,
  status: true,
  createdAt: true,
  completedAt: true,
});

export const insertBountySchema = createInsertSchema(bounties).omit({
  id: true,
  status: true,
  createdAt: true,
});

// Types
export type User = typeof users.$inferSelect;
export type InsertUser = z.infer<typeof insertUserSchema>;

export type SecurityEngine = typeof securityEngines.$inferSelect;
export type InsertSecurityEngine = z.infer<typeof insertSecurityEngineSchema>;

export type Submission = typeof submissions.$inferSelect;
export type InsertSubmission = z.infer<typeof insertSubmissionSchema>;

export type Analysis = typeof analyses.$inferSelect;
export type InsertAnalysis = z.infer<typeof insertAnalysisSchema>;

export type ConsensusResult = typeof consensusResults.$inferSelect;

export type Bounty = typeof bounties.$inferSelect;
export type InsertBounty = z.infer<typeof insertBountySchema>;
