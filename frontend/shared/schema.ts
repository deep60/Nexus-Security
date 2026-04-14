import { pgTable, text, varchar, decimal, integer, timestamp, boolean, jsonb, uuid, bigint } from "drizzle-orm/pg-core";
import { createInsertSchema } from "drizzle-zod";
import { z } from "zod";

// ─── Users (matches database/postgres/migrations/001_user_engine.sql) ───
export const users = pgTable("users", {
  id: uuid("id").primaryKey().defaultRandom(),
  username: varchar("username", { length: 50 }).notNull().unique(),
  email: varchar("email", { length: 255 }).notNull().unique(),
  passwordHash: varchar("password_hash", { length: 255 }).notNull(),
  walletAddress: varchar("wallet_address", { length: 42 }).unique(),
  reputationScore: integer("reputation_score").default(0),
  totalSubmissions: integer("total_submissions").default(0),
  successfulSubmissions: integer("successful_submissions").default(0),
  isVerified: boolean("is_verified").default(false),
  isActive: boolean("is_active").default(true),
  totalStakes: bigint("total_stakes", { mode: "number" }).default(0),
  successfulAnalyses: integer("successful_analyses").default(0),
  failedAnalyses: integer("failed_analyses").default(0),
  isEngine: boolean("is_engine").default(false),
  apiKey: varchar("api_key", { length: 255 }),
  lastLogin: timestamp("last_login", { withTimezone: true }),
  createdAt: timestamp("created_at", { withTimezone: true }).defaultNow(),
  updatedAt: timestamp("updated_at", { withTimezone: true }).defaultNow(),
});

// ─── Engines (matches database/postgres/migrations/001_user_engine.sql) ───
export const engines = pgTable("engines", {
  id: uuid("id").primaryKey().defaultRandom(),
  name: varchar("name", { length: 100 }).notNull(),
  engineType: varchar("engine_type", { length: 20 }).notNull(), // 'automated', 'human', 'hybrid'
  description: text("description"),
  ownerId: uuid("owner_id").references(() => users.id),
  apiEndpoint: varchar("api_endpoint", { length: 255 }),
  isActive: boolean("is_active").default(true),
  accuracyRate: decimal("accuracy_rate", { precision: 5, scale: 4 }).default("0.0000"),
  totalAnalyses: integer("total_analyses").default(0),
  correctAnalyses: integer("correct_analyses").default(0),
  stakeAmount: decimal("stake_amount", { precision: 20, scale: 8 }).default("0"),
  createdAt: timestamp("created_at", { withTimezone: true }).defaultNow(),
  updatedAt: timestamp("updated_at", { withTimezone: true }).defaultNow(),
});

// Keep legacy alias for backward compat in frontend code
export const securityEngines = engines;

// ─── Submissions (matches database/postgres/migrations/001_user_engine.sql) ───
export const submissions = pgTable("submissions", {
  id: uuid("id").primaryKey().defaultRandom(),
  submitterId: uuid("submitter_id").notNull().references(() => users.id),
  fileHash: varchar("file_hash", { length: 64 }).unique(),
  url: text("url"),
  originalFilename: varchar("original_filename", { length: 255 }),
  fileSize: bigint("file_size", { mode: "number" }),
  mimeType: varchar("mime_type", { length: 100 }),
  categoryId: integer("category_id"),
  filePath: text("file_path"),
  submissionType: varchar("submission_type", { length: 10 }).notNull(), // 'file', 'url'
  isMalicious: boolean("is_malicious"),
  confidenceScore: decimal("confidence_score", { precision: 5, scale: 4 }),
  analysisStatus: varchar("analysis_status", { length: 20 }).default("pending"),
  metadata: jsonb("metadata"),
  createdAt: timestamp("created_at", { withTimezone: true }).defaultNow(),
  updatedAt: timestamp("updated_at", { withTimezone: true }).defaultNow(),
});

// ─── Bounties (matches database/postgres/migrations/002_bounty_system.sql) ───
export const bounties = pgTable("bounties", {
  id: uuid("id").primaryKey().defaultRandom(),
  creatorId: uuid("creator_id").notNull().references(() => users.id),
  submissionId: uuid("submission_id").notNull().references(() => submissions.id),
  title: varchar("title", { length: 200 }).notNull(),
  description: text("description"),
  rewardAmount: decimal("reward_amount", { precision: 20, scale: 8 }).notNull(),
  minStakeAmount: decimal("min_stake_amount", { precision: 20, scale: 8 }).default("0"),
  maxParticipants: integer("max_participants"),
  deadline: timestamp("deadline", { withTimezone: true }),
  bountyStatus: varchar("bounty_status", { length: 20 }).default("active"),
  requiresVerification: boolean("requires_verification").default(false),
  priorityLevel: integer("priority_level").default(1),
  blockchainTxHash: varchar("blockchain_tx_hash", { length: 66 }),
  smartContractAddress: varchar("smart_contract_address", { length: 42 }),
  totalStaked: decimal("total_staked", { precision: 20, scale: 8 }).default("0"),
  participantCount: integer("participant_count").default(0),
  consensusThreshold: decimal("consensus_threshold", { precision: 3, scale: 2 }).default("0.60"),
  createdAt: timestamp("created_at", { withTimezone: true }).defaultNow(),
  updatedAt: timestamp("updated_at", { withTimezone: true }).defaultNow(),
  completedAt: timestamp("completed_at", { withTimezone: true }),
});

// ─── Analysis Results (matches database/postgres/migrations/002_bounty_system.sql) ───
export const analysisResults = pgTable("analysis_results", {
  id: uuid("id").primaryKey().defaultRandom(),
  participationId: uuid("participation_id"),
  engineId: uuid("engine_id").notNull().references(() => engines.id),
  submissionId: uuid("submission_id").notNull().references(() => submissions.id),
  bountyId: uuid("bounty_id").references(() => bounties.id),
  analyzerId: uuid("analyzer_id").references(() => engines.id),
  verdict: varchar("verdict", { length: 20 }).notNull(),
  confidenceScore: decimal("confidence_score", { precision: 5, scale: 4 }).notNull(),
  threatTypes: text("threat_types").array(),
  analysisDuration: integer("analysis_duration"),
  detailedReport: jsonb("detailed_report"),
  analysisStatus: varchar("analysis_status", { length: 20 }).default("completed"),
  createdAt: timestamp("created_at", { withTimezone: true }).defaultNow(),
  completedAt: timestamp("completed_at", { withTimezone: true }),
});

// Keep legacy alias
export const analyses = analysisResults;

// ─── Consensus Results (matches database/postgres/migrations/002_bounty_system.sql) ───
export const consensusResults = pgTable("consensus_results", {
  id: uuid("id").primaryKey().defaultRandom(),
  bountyId: uuid("bounty_id").notNull().references(() => bounties.id),
  submissionId: uuid("submission_id").notNull().references(() => submissions.id),
  finalVerdict: varchar("final_verdict", { length: 20 }).notNull(),
  confidenceScore: decimal("confidence_score", { precision: 5, scale: 4 }).notNull(),
  maliciousVotes: integer("malicious_votes").default(0),
  benignVotes: integer("benign_votes").default(0),
  suspiciousVotes: integer("suspicious_votes").default(0),
  unknownVotes: integer("unknown_votes").default(0),
  totalParticipants: integer("total_participants").notNull(),
  weightedScore: decimal("weighted_score", { precision: 10, scale: 8 }),
  consensusAlgorithm: varchar("consensus_algorithm", { length: 50 }).default("majority_vote"),
  calculationMetadata: jsonb("calculation_metadata"),
  createdAt: timestamp("created_at", { withTimezone: true }).defaultNow(),
});

// ─── Insert Schemas ───
export const insertUserSchema = createInsertSchema(users).omit({
  id: true,
  reputationScore: true,
  totalSubmissions: true,
  successfulSubmissions: true,
  isVerified: true,
  isActive: true,
  totalStakes: true,
  successfulAnalyses: true,
  failedAnalyses: true,
  isEngine: true,
  apiKey: true,
  lastLogin: true,
  createdAt: true,
  updatedAt: true,
});

export const insertEngineSchema = createInsertSchema(engines).omit({
  id: true,
  accuracyRate: true,
  totalAnalyses: true,
  correctAnalyses: true,
  stakeAmount: true,
  createdAt: true,
  updatedAt: true,
});

// Legacy alias
export const insertSecurityEngineSchema = insertEngineSchema;

export const insertSubmissionSchema = createInsertSchema(submissions).omit({
  id: true,
  isMalicious: true,
  confidenceScore: true,
  analysisStatus: true,
  createdAt: true,
  updatedAt: true,
});

export const insertAnalysisSchema = createInsertSchema(analysisResults).omit({
  id: true,
  analysisStatus: true,
  createdAt: true,
  completedAt: true,
});

export const insertBountySchema = createInsertSchema(bounties).omit({
  id: true,
  bountyStatus: true,
  totalStaked: true,
  participantCount: true,
  createdAt: true,
  updatedAt: true,
  completedAt: true,
});

// ─── Types ───
export type User = typeof users.$inferSelect;
export type InsertUser = z.infer<typeof insertUserSchema>;

export type Engine = typeof engines.$inferSelect;
export type InsertEngine = z.infer<typeof insertEngineSchema>;

// Legacy aliases
export type SecurityEngine = Engine;
export type InsertSecurityEngine = InsertEngine;

export type Submission = typeof submissions.$inferSelect;
export type InsertSubmission = z.infer<typeof insertSubmissionSchema>;

export type Analysis = typeof analysisResults.$inferSelect;
export type InsertAnalysis = z.infer<typeof insertAnalysisSchema>;

export type ConsensusResult = typeof consensusResults.$inferSelect;

export type Bounty = typeof bounties.$inferSelect;
export type InsertBounty = z.infer<typeof insertBountySchema>;
