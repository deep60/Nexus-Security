/**
 * Unit Tests for Storage Layer
 * Tests the MemStorage implementation
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { MemStorage } from '../server/storage';
import type { InsertUser, InsertSubmission, InsertAnalysis, InsertSecurityEngine } from '@shared/schema';

describe('MemStorage - User Operations', () => {
  let storage: MemStorage;

  beforeEach(() => {
    storage = new MemStorage();
  });

  it('should create a user', async () => {
    const userData: InsertUser = {
      username: 'testuser',
      email: 'test@example.com',
      password: 'hashedpassword123',
    };

    const user = await storage.createUser(userData);

    expect(user).toBeDefined();
    expect(user.id).toBeDefined();
    expect(user.username).toBe('testuser');
    expect(user.email).toBe('test@example.com');
    expect(user.reputation).toBe('0');
    expect(user.totalStaked).toBe('0');
    expect(user.totalEarned).toBe('0');
  });

  it('should get user by email', async () => {
    const userData: InsertUser = {
      username: 'testuser',
      email: 'test@example.com',
      password: 'hashedpassword123',
    };

    const createdUser = await storage.createUser(userData);
    const foundUser = await storage.getUserByEmail('test@example.com');

    expect(foundUser).toBeDefined();
    expect(foundUser?.id).toBe(createdUser.id);
    expect(foundUser?.email).toBe('test@example.com');
  });

  it('should get user by username', async () => {
    const userData: InsertUser = {
      username: 'testuser',
      email: 'test@example.com',
      password: 'hashedpassword123',
    };

    const createdUser = await storage.createUser(userData);
    const foundUser = await storage.getUserByUsername('testuser');

    expect(foundUser).toBeDefined();
    expect(foundUser?.id).toBe(createdUser.id);
    expect(foundUser?.username).toBe('testuser');
  });

  it('should get user by id', async () => {
    const userData: InsertUser = {
      username: 'testuser',
      email: 'test@example.com',
      password: 'hashedpassword123',
    };

    const createdUser = await storage.createUser(userData);
    const foundUser = await storage.getUser(createdUser.id);

    expect(foundUser).toBeDefined();
    expect(foundUser?.id).toBe(createdUser.id);
  });

  it('should update user', async () => {
    const userData: InsertUser = {
      username: 'testuser',
      email: 'test@example.com',
      password: 'hashedpassword123',
    };

    const createdUser = await storage.createUser(userData);
    const updated = await storage.updateUser(createdUser.id, {
      walletAddress: '0x1234567890abcdef',
      reputation: '100',
    });

    expect(updated).toBeDefined();
    expect(updated?.walletAddress).toBe('0x1234567890abcdef');
    expect(updated?.reputation).toBe('100');
  });

  it('should return undefined for non-existent user', async () => {
    const user = await storage.getUser('non-existent-id');
    expect(user).toBeUndefined();
  });
});

describe('MemStorage - Security Engine Operations', () => {
  let storage: MemStorage;

  beforeEach(() => {
    storage = new MemStorage();
  });

  it('should initialize with mock security engines', async () => {
    const engines = await storage.getSecurityEngines();

    expect(engines.length).toBeGreaterThan(0);
    expect(engines[0]).toHaveProperty('name');
    expect(engines[0]).toHaveProperty('type');
    expect(engines[0]).toHaveProperty('accuracy');
  });

  it('should create a security engine', async () => {
    const engineData: InsertSecurityEngine = {
      name: 'Test Engine',
      type: 'automated',
      description: 'Test description',
      status: 'online',
      ownerId: null,
    };

    const engine = await storage.createSecurityEngine(engineData);

    expect(engine).toBeDefined();
    expect(engine.id).toBeDefined();
    expect(engine.name).toBe('Test Engine');
    expect(engine.type).toBe('automated');
    expect(engine.accuracy).toBe('0');
    expect(engine.totalAnalyses).toBe(0);
  });

  it('should get security engine by id', async () => {
    const engines = await storage.getSecurityEngines();
    const firstEngine = engines[0];

    const found = await storage.getSecurityEngine(firstEngine.id);

    expect(found).toBeDefined();
    expect(found?.id).toBe(firstEngine.id);
  });

  it('should update security engine', async () => {
    const engines = await storage.getSecurityEngines();
    const firstEngine = engines[0];

    const updated = await storage.updateSecurityEngine(firstEngine.id, {
      accuracy: '95.5',
      totalAnalyses: 1000,
    });

    expect(updated).toBeDefined();
    expect(updated?.accuracy).toBe('95.5');
    expect(updated?.totalAnalyses).toBe(1000);
  });
});

describe('MemStorage - Submission Operations', () => {
  let storage: MemStorage;

  beforeEach(() => {
    storage = new MemStorage();
  });

  it('should create a submission', async () => {
    const submissionData: InsertSubmission = {
      filename: 'test.exe',
      fileHash: 'sha256_abc123',
      submissionType: 'file',
      analysisType: 'full',
      bountyAmount: '1.5',
    };

    const submission = await storage.createSubmission(submissionData);

    expect(submission).toBeDefined();
    expect(submission.id).toBeDefined();
    expect(submission.filename).toBe('test.exe');
    expect(submission.status).toBe('pending');
    expect(submission.bountyAmount).toBe('1.5');
  });

  it('should get submission by id', async () => {
    const submissionData: InsertSubmission = {
      filename: 'test.exe',
      fileHash: 'sha256_abc123',
      submissionType: 'file',
      analysisType: 'full',
      bountyAmount: '1.5',
    };

    const created = await storage.createSubmission(submissionData);
    const found = await storage.getSubmission(created.id);

    expect(found).toBeDefined();
    expect(found?.id).toBe(created.id);
  });

  it('should get all submissions', async () => {
    const submissionData: InsertSubmission = {
      filename: 'test.exe',
      fileHash: 'sha256_abc123',
      submissionType: 'file',
      analysisType: 'full',
      bountyAmount: '1.5',
    };

    await storage.createSubmission(submissionData);
    await storage.createSubmission({ ...submissionData, filename: 'test2.exe' });

    const submissions = await storage.getSubmissions();

    expect(submissions.length).toBe(2);
  });

  it('should update submission status', async () => {
    const submissionData: InsertSubmission = {
      filename: 'test.exe',
      fileHash: 'sha256_abc123',
      submissionType: 'file',
      analysisType: 'full',
      bountyAmount: '1.5',
    };

    const created = await storage.createSubmission(submissionData);
    const updated = await storage.updateSubmission(created.id, {
      status: 'analyzing',
    });

    expect(updated).toBeDefined();
    expect(updated?.status).toBe('analyzing');
  });
});

describe('MemStorage - Analysis Operations', () => {
  let storage: MemStorage;
  let submissionId: string;
  let engineId: string;

  beforeEach(async () => {
    storage = new MemStorage();

    // Create a submission for testing
    const submissionData: InsertSubmission = {
      filename: 'test.exe',
      fileHash: 'sha256_abc123',
      submissionType: 'file',
      analysisType: 'full',
      bountyAmount: '1.5',
    };
    const submission = await storage.createSubmission(submissionData);
    submissionId = submission.id;

    // Get an engine
    const engines = await storage.getSecurityEngines();
    engineId = engines[0].id;
  });

  it('should create an analysis', async () => {
    const analysisData: InsertAnalysis = {
      submissionId,
      engineId,
      stakeAmount: '0.1',
      verdict: null,
      confidence: null,
      details: null,
    };

    const analysis = await storage.createAnalysis(analysisData);

    expect(analysis).toBeDefined();
    expect(analysis.id).toBeDefined();
    expect(analysis.submissionId).toBe(submissionId);
    expect(analysis.engineId).toBe(engineId);
    expect(analysis.status).toBe('pending');
  });

  it('should get analyses by submission', async () => {
    const analysisData: InsertAnalysis = {
      submissionId,
      engineId,
      stakeAmount: '0.1',
      verdict: null,
      confidence: null,
      details: null,
    };

    await storage.createAnalysis(analysisData);
    await storage.createAnalysis(analysisData);

    const analyses = await storage.getAnalysesBySubmission(submissionId);

    expect(analyses.length).toBe(2);
    expect(analyses[0].submissionId).toBe(submissionId);
  });

  it('should update analysis with verdict', async () => {
    const analysisData: InsertAnalysis = {
      submissionId,
      engineId,
      stakeAmount: '0.1',
      verdict: null,
      confidence: null,
      details: null,
    };

    const created = await storage.createAnalysis(analysisData);
    const updated = await storage.updateAnalysis(created.id, {
      verdict: 'malicious',
      confidence: '95.5',
      status: 'completed',
    });

    expect(updated).toBeDefined();
    expect(updated?.verdict).toBe('malicious');
    expect(updated?.confidence).toBe('95.5');
    expect(updated?.status).toBe('completed');
  });
});

describe('MemStorage - Consensus Operations', () => {
  let storage: MemStorage;
  let submissionId: string;

  beforeEach(async () => {
    storage = new MemStorage();

    // Create a submission
    const submissionData: InsertSubmission = {
      filename: 'test.exe',
      fileHash: 'sha256_abc123',
      submissionType: 'file',
      analysisType: 'full',
      bountyAmount: '1.5',
    };
    const submission = await storage.createSubmission(submissionData);
    submissionId = submission.id;
  });

  it('should create a consensus result', async () => {
    const consensusData = {
      submissionId,
      finalVerdict: 'malicious',
      confidenceScore: '92.5',
      totalEngines: 4,
      maliciousVotes: 3,
      cleanVotes: 1,
      suspiciousVotes: 0,
      rewardsDistributed: false,
    };

    const consensus = await storage.createConsensusResult(consensusData);

    expect(consensus).toBeDefined();
    expect(consensus.id).toBeDefined();
    expect(consensus.submissionId).toBe(submissionId);
    expect(consensus.finalVerdict).toBe('malicious');
    expect(consensus.confidenceScore).toBe('92.5');
    expect(consensus.totalEngines).toBe(4);
  });

  it('should get consensus result by submission', async () => {
    const consensusData = {
      submissionId,
      finalVerdict: 'clean',
      confidenceScore: '88.0',
      totalEngines: 4,
      maliciousVotes: 0,
      cleanVotes: 4,
      suspiciousVotes: 0,
      rewardsDistributed: false,
    };

    await storage.createConsensusResult(consensusData);
    const found = await storage.getConsensusResult(submissionId);

    expect(found).toBeDefined();
    expect(found?.submissionId).toBe(submissionId);
    expect(found?.finalVerdict).toBe('clean');
  });
});

describe('MemStorage - Bounty Operations', () => {
  let storage: MemStorage;
  let submissionId: string;

  beforeEach(async () => {
    storage = new MemStorage();

    // Create a submission
    const submissionData: InsertSubmission = {
      filename: 'test.exe',
      fileHash: 'sha256_abc123',
      submissionType: 'file',
      analysisType: 'full',
      bountyAmount: '1.5',
    };
    const submission = await storage.createSubmission(submissionData);
    submissionId = submission.id;
  });

  it('should create a bounty', async () => {
    const bountyData = {
      submissionId,
      amount: '1.5',
      expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
    };

    const bounty = await storage.createBounty(bountyData);

    expect(bounty).toBeDefined();
    expect(bounty.id).toBeDefined();
    expect(bounty.submissionId).toBe(submissionId);
    expect(bounty.amount).toBe('1.5');
    expect(bounty.status).toBe('active');
  });

  it('should get active bounties', async () => {
    const bountyData = {
      submissionId,
      amount: '1.5',
      expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
    };

    await storage.createBounty(bountyData);

    const activeBounties = await storage.getActiveBounties();

    expect(activeBounties.length).toBeGreaterThan(0);
    expect(activeBounties[0].status).toBe('active');
  });

  it('should update bounty status', async () => {
    const bountyData = {
      submissionId,
      amount: '1.5',
      expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
    };

    const created = await storage.createBounty(bountyData);
    const updated = await storage.updateBounty(created.id, {
      status: 'completed',
    });

    expect(updated).toBeDefined();
    expect(updated?.status).toBe('completed');
  });
});