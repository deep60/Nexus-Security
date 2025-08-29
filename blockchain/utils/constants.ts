import { ethers } from "ethers";

// ================================
// NETWORK CONFIGURATIONS
// ================================

export interface NetworkConfig {
  name: string;
  chainId: number;
  rpcUrl?: string;
  blockExplorer?: string;
  nativeCurrency: {
    name: string;
    symbol: string;
    decimals: number;
  };
  gasPrice?: {
    min: string;
    max: string;
  };
  multicallAddress?: string;
}

export const NETWORKS: Record<string, NetworkConfig> = {
  mainnet: {
    name: "Ethereum Mainnet",
    chainId: 1,
    blockExplorer: "https://etherscan.io",
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18
    },
    gasPrice: {
      min: "10",  // 10 gwei
      max: "100"  // 100 gwei
    },
    multicallAddress: "0xcA11bde05977b3631167028862bE2a173976CA11"
  },
  goerli: {
    name: "Goerli Testnet",
    chainId: 5,
    blockExplorer: "https://goerli.etherscan.io",
    nativeCurrency: {
      name: "Goerli Ether",
      symbol: "ETH",
      decimals: 18
    },
    multicallAddress: "0xcA11bde05977b3631167028862bE2a173976CA11"
  },
  sepolia: {
    name: "Sepolia Testnet",
    chainId: 11155111,
    blockExplorer: "https://sepolia.etherscan.io",
    nativeCurrency: {
      name: "Sepolia Ether",
      symbol: "ETH",
      decimals: 18
    },
    multicallAddress: "0xcA11bde05977b3631167028862bE2a173976CA11"
  },
  polygon: {
    name: "Polygon Mainnet",
    chainId: 137,
    blockExplorer: "https://polygonscan.com",
    nativeCurrency: {
      name: "Polygon",
      symbol: "MATIC",
      decimals: 18
    },
    gasPrice: {
      min: "30",   // 30 gwei
      max: "300"   // 300 gwei
    },
    multicallAddress: "0xcA11bde05977b3631167028862bE2a173976CA11"
  },
  mumbai: {
    name: "Polygon Mumbai",
    chainId: 80001,
    blockExplorer: "https://mumbai.polygonscan.com",
    nativeCurrency: {
      name: "Mumbai MATIC",
      symbol: "MATIC",
      decimals: 18
    },
    multicallAddress: "0xcA11bde05977b3631167028862bE2a173976CA11"
  },
  bsc: {
    name: "Binance Smart Chain",
    chainId: 56,
    blockExplorer: "https://bscscan.com",
    nativeCurrency: {
      name: "Binance Coin",
      symbol: "BNB",
      decimals: 18
    },
    gasPrice: {
      min: "3",    // 3 gwei
      max: "20"    // 20 gwei
    },
    multicallAddress: "0xcA11bde05977b3631167028862bE2a173976CA11"
  },
  hardhat: {
    name: "Hardhat Local",
    chainId: 31337,
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18
    }
  }
} as const;

// ================================
// TOKEN CONSTANTS
// ================================

export const TOKEN_CONFIG = {
  THREAT_TOKEN: {
    NAME: "ThreatToken",
    SYMBOL: "THREAT",
    DECIMALS: 18,
    INITIAL_SUPPLY: ethers.parseEther("1000000"), // 1M tokens
    MAX_SUPPLY: ethers.parseEther("10000000"),    // 10M tokens max
  },
  
  // Token distribution (percentages)
  DISTRIBUTION: {
    REWARD_POOL: 40,        // 40% for bounty rewards
    TEAM_TREASURY: 20,      // 20% for team
    ECOSYSTEM_FUND: 15,     // 15% for ecosystem development
    LIQUIDITY: 10,          // 10% for DEX liquidity
    COMMUNITY: 10,          // 10% for community incentives
    ADVISORS: 5             // 5% for advisors
  }
} as const;

// ================================
// BOUNTY SYSTEM CONSTANTS
// ================================

export const BOUNTY_CONFIG = {
  // Minimum and maximum bounty amounts (in wei)
  MIN_BOUNTY_AMOUNT: ethers.parseEther("1"),      // 1 THREAT token
  MAX_BOUNTY_AMOUNT: ethers.parseEther("10000"),  // 10K THREAT tokens
  
  // Staking requirements
  MIN_STAKE_AMOUNT: ethers.parseEther("10"),      // 10 THREAT tokens
  MAX_STAKE_MULTIPLIER: 10,                       // Max 10x bounty amount stake
  
  // Timing constants (in seconds)
  MIN_ANALYSIS_TIME: 300,      // 5 minutes minimum
  MAX_ANALYSIS_TIME: 86400,    // 24 hours maximum
  DEFAULT_ANALYSIS_TIME: 3600, // 1 hour default
  
  RESOLUTION_TIME: 1800,       // 30 minutes for resolution
  APPEAL_TIME: 7200,          // 2 hours for appeals
  
  // Consensus requirements
  MIN_ENGINES_FOR_CONSENSUS: 3,    // Minimum 3 engines needed
  CONSENSUS_THRESHOLD: 70,         // 70% agreement needed
  SUPER_CONSENSUS_THRESHOLD: 85,   // 85% for high confidence
  
  // Reward distribution (percentages)
  WINNER_SHARE: 80,          // 80% to correct engines
  PLATFORM_FEE: 15,          // 15% platform fee
  INSURANCE_POOL: 5,         // 5% to insurance pool
  
  // Penalty settings
  SLASH_PERCENTAGE: 20,      // 20% of stake slashed for wrong answer
  MAX_SLASH_AMOUNT: ethers.parseEther("100"), // Max 100 tokens slashed
  
  // File size limits (in bytes)
  MAX_FILE_SIZE: 100 * 1024 * 1024,  // 100MB
  MAX_URL_LENGTH: 2048,               // 2KB URL length
} as const;

// ================================
// REPUTATION SYSTEM CONSTANTS
// ================================

export const REPUTATION_CONFIG = {
  // Initial values
  INITIAL_REPUTATION: 100,
  MIN_REPUTATION: 0,
  MAX_REPUTATION: 1000,
  
  // Reputation changes
  CORRECT_ANALYSIS_REWARD: 10,
  INCORRECT_ANALYSIS_PENALTY: 15,
  CONSENSUS_BONUS: 5,
  MINORITY_CORRECT_BONUS: 20,   // Bonus for being right when minority
  
  // Reputation thresholds for different actions
  MIN_REP_FOR_ANALYSIS: 50,     // Minimum to participate in analysis
  MIN_REP_FOR_APPEALS: 100,     // Minimum to file appeals
  MIN_REP_FOR_MODERATION: 200,  // Minimum for moderation actions
  
  // Decay settings
  REPUTATION_DECAY_RATE: 1,     // 1% decay per month of inactivity
  DECAY_INTERVAL: 2592000,      // 30 days in seconds
  
  // Reputation multipliers based on stake
  STAKE_MULTIPLIERS: {
    LOW: 1,      // 1x for minimum stake
    MEDIUM: 1.5, // 1.5x for 5x minimum stake
    HIGH: 2,     // 2x for 10x minimum stake
  },
  
  // Experience levels
  LEVELS: {
    NOVICE: 0,
    ANALYST: 100,
    EXPERT: 300,
    MASTER: 500,
    LEGEND: 750
  }
} as const;

// ================================
// ENGINE TYPES AND CATEGORIES
// ================================

export enum EngineType {
  HUMAN = 0,
  AUTOMATED = 1,
  HYBRID = 2
}

export enum ThreatCategory {
  MALWARE = 0,
  PHISHING = 1,
  SCAM = 2,
  SUSPICIOUS = 3,
  CLEAN = 4,
  UNKNOWN = 5
}

export enum AnalysisResult {
  MALICIOUS = 0,
  SUSPICIOUS = 1,
  BENIGN = 2,
  INCONCLUSIVE = 3
}

export enum BountyStatus {
  PENDING = 0,
  ACTIVE = 1,
  ANALYZING = 2,
  RESOLVING = 3,
  RESOLVED = 4,
  DISPUTED = 5,
  CANCELLED = 6,
  EXPIRED = 7
}

export enum EngineStatus {
  UNREGISTERED = 0,
  REGISTERED = 1,
  VERIFIED = 2,
  SUSPENDED = 3,
  BANNED = 4
}

// ================================
// SECURITY AND ACCESS CONTROL
// ================================

export const ACCESS_CONTROL = {
  // Role identifiers (keccak256 hashes)
  ROLES: {
    ADMIN_ROLE: "0x0000000000000000000000000000000000000000000000000000000000000000",
    MANAGER_ROLE: ethers.id("MANAGER_ROLE"),
    MODERATOR_ROLE: ethers.id("MODERATOR_ROLE"),
    ENGINE_ROLE: ethers.id("ENGINE_ROLE"),
    VERIFIER_ROLE: ethers.id("VERIFIER_ROLE"),
    ORACLE_ROLE: ethers.id("ORACLE_ROLE"),
    PAUSER_ROLE: ethers.id("PAUSER_ROLE"),
  },
  
  // Multi-sig requirements
  MULTISIG_THRESHOLD: 2,     // Require 2 of 3 signatures for admin actions
  MIN_SIGNERS: 3,            // Minimum 3 signers for multi-sig
  
  // Emergency controls
  EMERGENCY_PAUSE_DURATION: 86400,  // 24 hours emergency pause
  UPGRADE_DELAY: 172800,            // 48 hours upgrade delay
} as const;

// ================================
// API AND INTEGRATION CONSTANTS
// ================================

export const API_CONFIG = {
  // Rate limiting
  REQUESTS_PER_MINUTE: 60,
  REQUESTS_PER_HOUR: 1000,
  REQUESTS_PER_DAY: 10000,
  
  // API versions
  CURRENT_API_VERSION: "v1",
  SUPPORTED_VERSIONS: ["v1"],
  
  // Endpoints
  BASE_PATH: "/api",
  HEALTH_CHECK: "/health",
  METRICS: "/metrics",
  
  // WebSocket events
  WS_EVENTS: {
    BOUNTY_CREATED: "bounty:created",
    ANALYSIS_SUBMITTED: "analysis:submitted",
    BOUNTY_RESOLVED: "bounty:resolved",
    REPUTATION_UPDATED: "reputation:updated",
    ENGINE_REGISTERED: "engine:registered",
  },
  
  // File upload limits
  MAX_FILE_UPLOADS_PER_HOUR: 100,
  MAX_CONCURRENT_ANALYSES: 50,
} as const;

// ================================
// GAS AND TRANSACTION LIMITS
// ================================

export const GAS_LIMITS = {
  // Contract deployment
  THREAT_TOKEN_DEPLOY: 2000000,
  REPUTATION_SYSTEM_DEPLOY: 3000000,
  BOUNTY_MANAGER_DEPLOY: 4000000,
  
  // Transaction gas limits
  CREATE_BOUNTY: 200000,
  SUBMIT_ANALYSIS: 150000,
  RESOLVE_BOUNTY: 300000,
  CLAIM_REWARDS: 100000,
  REGISTER_ENGINE: 150000,
  UPDATE_REPUTATION: 100000,
  
  // Batch operations
  BATCH_RESOLVE: 500000,
  BATCH_REWARD: 400000,
  
  // Emergency operations
  EMERGENCY_PAUSE: 50000,
  EMERGENCY_UNPAUSE: 50000,
} as const;

// ================================
// EVENTS AND LOGGING
// ================================

export const EVENTS = {
  // Bounty events
  BOUNTY_CREATED: "BountyCreated",
  BOUNTY_CANCELLED: "BountyCancelled",
  BOUNTY_RESOLVED: "BountyResolved",
  
  // Analysis events
  ANALYSIS_SUBMITTED: "AnalysisSubmitted",
  CONSENSUS_REACHED: "ConsensusReached",
  
  // Reward events
  REWARDS_DISTRIBUTED: "RewardsDistributed",
  STAKE_SLASHED: "StakeSlashed",
  
  // Engine events
  ENGINE_REGISTERED: "EngineRegistered",
  ENGINE_SUSPENDED: "EngineSuspended",
  
  // Reputation events
  REPUTATION_UPDATED: "ReputationUpdated",
  LEVEL_ACHIEVED: "LevelAchieved",
  
  // System events
  SYSTEM_PAUSED: "SystemPaused",
  SYSTEM_UNPAUSED: "SystemUnpaused",
  EMERGENCY_WITHDRAWAL: "EmergencyWithdrawal",
} as const;

// ================================
// ERROR CODES AND MESSAGES
// ================================

export const ERROR_CODES = {
  // General errors
  UNAUTHORIZED: "UNAUTHORIZED",
  INSUFFICIENT_BALANCE: "INSUFFICIENT_BALANCE",
  INVALID_PARAMETERS: "INVALID_PARAMETERS",
  CONTRACT_PAUSED: "CONTRACT_PAUSED",
  
  // Bounty errors
  BOUNTY_NOT_FOUND: "BOUNTY_NOT_FOUND",
  BOUNTY_EXPIRED: "BOUNTY_EXPIRED",
  BOUNTY_ALREADY_RESOLVED: "BOUNTY_ALREADY_RESOLVED",
  INSUFFICIENT_BOUNTY: "INSUFFICIENT_BOUNTY",
  
  // Engine errors
  ENGINE_NOT_REGISTERED: "ENGINE_NOT_REGISTERED",
  ENGINE_SUSPENDED: "ENGINE_SUSPENDED",
  INSUFFICIENT_REPUTATION: "INSUFFICIENT_REPUTATION",
  ALREADY_ANALYZED: "ALREADY_ANALYZED",
  
  // Stake errors
  INSUFFICIENT_STAKE: "INSUFFICIENT_STAKE",
  STAKE_LOCKED: "STAKE_LOCKED",
  EXCESSIVE_STAKE: "EXCESSIVE_STAKE",
  
  // Time errors
  ANALYSIS_PERIOD_ENDED: "ANALYSIS_PERIOD_ENDED",
  TOO_EARLY_TO_RESOLVE: "TOO_EARLY_TO_RESOLVE",
  RESOLUTION_PERIOD_EXPIRED: "RESOLUTION_PERIOD_EXPIRED",
} as const;

// ================================
// MATHEMATICAL CONSTANTS
// ================================

export const MATH_CONSTANTS = {
  // Percentage calculations (basis points for precision)
  BASIS_POINTS: 10000,     // 100% = 10000 basis points
  HALF_PERCENT: 50,        // 0.5% in basis points
  ONE_PERCENT: 100,        // 1% in basis points
  
  // Precision for calculations
  PRECISION: ethers.parseEther("1"),  // 1e18 for high precision
  HALF_PRECISION: ethers.parseEther("0.5"), // 0.5e18
  
  // Time constants
  SECONDS_PER_MINUTE: 60,
  SECONDS_PER_HOUR: 3600,
  SECONDS_PER_DAY: 86400,
  SECONDS_PER_WEEK: 604800,
  SECONDS_PER_MONTH: 2592000, // 30 days
  SECONDS_PER_YEAR: 31536000, // 365 days
} as const;

// ================================
// DEVELOPMENT AND TESTING
// ================================

export const DEVELOPMENT = {
  // Test accounts (for hardhat/localhost)
  TEST_PRIVATE_KEYS: [
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2b4b032c93e3a79be8",
    "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
  ],
  
  // Mock data for testing
  MOCK_FILE_HASH: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  MOCK_URL: "https://suspicious-site.com/malware.exe",
  
  // Test configuration
  FAST_ANALYSIS_TIME: 60,    // 1 minute for testing
  SMALL_STAKE: ethers.parseEther("1"), // 1 token for testing
  
  // Deployment settings
  VERIFY_CONTRACTS: true,
  SAVE_DEPLOYMENTS: true,
  GAS_REPORTER: true,
} as const;

// ================================
// UTILITY FUNCTIONS
// ================================

export const UTILS = {
  // Convert percentage to basis points
  percentToBasisPoints: (percent: number): number => Math.floor(percent * 100),
  
  // Convert basis points to percentage
  basisPointsToPercent: (basisPoints: number): number => basisPoints / 100,
  
  // Check if address is valid
  isValidAddress: (address: string): boolean => ethers.isAddress(address),
  
  // Generate unique bounty ID
  generateBountyId: (creator: string, nonce: number): string => 
    ethers.solidityPackedKeccak256(["address", "uint256"], [creator, nonce]),
  
  // Calculate reputation change
  calculateReputationChange: (
    correct: boolean, 
    consensusMatch: boolean, 
    stakeMultiplier: number
  ): number => {
    let change = correct ? REPUTATION_CONFIG.CORRECT_ANALYSIS_REWARD : -REPUTATION_CONFIG.INCORRECT_ANALYSIS_PENALTY;
    if (consensusMatch) change += REPUTATION_CONFIG.CONSENSUS_BONUS;
    return Math.floor(change * stakeMultiplier);
  },
  
  // Format token amount for display
  formatTokenAmount: (amount: bigint, decimals: number = 18): string => 
    ethers.formatUnits(amount, decimals),
  
  // Parse token amount from string
  parseTokenAmount: (amount: string, decimals: number = 18): bigint => 
    ethers.parseUnits(amount, decimals),
} as const;

// ================================
// VERSION INFORMATION
// ================================

export const VERSION_INFO = {
  CONTRACT_VERSION: "1.0.0",
  API_VERSION: "1.0.0",
  PROTOCOL_VERSION: "1.0.0",
  
  // Upgrade compatibility
  MIN_SUPPORTED_VERSION: "1.0.0",
  UPGRADE_REQUIRED_VERSION: "2.0.0",
  
  // Build information
  BUILD_DATE: new Date().toISOString(),
  SOLIDITY_VERSION: "0.8.19",
} as const;

// Export all constants as a single object for easy importing
export const NEXUS_CONSTANTS = {
  NETWORKS,
  TOKEN_CONFIG,
  BOUNTY_CONFIG,
  REPUTATION_CONFIG,
  ACCESS_CONTROL,
  API_CONFIG,
  GAS_LIMITS,
  EVENTS,
  ERROR_CODES,
  MATH_CONSTANTS,
  DEVELOPMENT,
  UTILS,
  VERSION_INFO,
} as const;

// Type exports for TypeScript support
export type NetworkName = keyof typeof NETWORKS;
export type EventName = keyof typeof EVENTS;
export type ErrorCode = keyof typeof ERROR_CODES;
export type RoleName = keyof typeof ACCESS_CONTROL.ROLES;

// Default export
export default NEXUS_CONSTANTS;






// Network Configuration

// Complete network definitions for Ethereum, Polygon, BSC, and testnets
// Gas price ranges, block explorers, and multicall addresses
// Native currency information for each network

// Token Economics

// ThreatToken configuration (name, symbol, supply limits)
// Distribution percentages for different stakeholders
// Decimal precision and formatting utilities

// Bounty System Parameters

// Minimum/maximum bounty and stake amounts
// Timing constraints (analysis periods, resolution times)
// Consensus thresholds and reward distribution
// File size limits and validation rules

// Reputation System

// Initial reputation values and level thresholds
// Reward/penalty amounts for correct/incorrect analyses
// Reputation decay mechanisms
// Stake multipliers based on reputation levels

// Enums and Types

// Engine types (Human, Automated, Hybrid)
// Threat categories (Malware, Phishing, etc.)
// Analysis results and bounty statuses
// Engine registration states

// Security & Access Control

// Role-based access control identifiers
// Multi-signature requirements
// Emergency pause mechanisms
// Upgrade delay configurations

// Gas Limits & Performance

// Deployment gas limits for each contract
// Transaction-specific gas limits
// Batch operation limits
// Emergency operation gas costs

// Events & Error Handling

// Standardized event names for logging
// Comprehensive error codes and messages
// WebSocket event identifiers

// Development & Testing

// Test account private keys for local development
// Mock data for testing scenarios
// Fast-mode configurations for development
// Build and version information