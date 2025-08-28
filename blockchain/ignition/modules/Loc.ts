// Main Features:

// Core Contract Deployment:

// ThreatToken (ERC20) for bounty payments and rewards
// ReputationSystem for tracking analyst credibility
// BountyManager as the main marketplace contract


// Configurable Parameters:

// Token supply and naming
// Bounty minimums and timing
// Reputation system settings
// Platform fee configuration


// Post-Deployment Setup:

// Role assignments for contracts to interact
// Initial token distribution
// Test account registration


// Additional Modules:

// TestDataModule: Creates sample bounties and test accounts
// DevConfigModule: Development-specific optimizations



// Key Benefits:

// Modular Design: Each component can be deployed and configured independently
// Environment Flexibility: Parameters can be adjusted for different deployment environments
// Testing Ready: Includes test data and accounts for immediate development use
// Verification Built-in: Includes verification calls to ensure proper deployment

import { parseEther } from "ethers";
import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

/**
 * Nexus-Security Local Deployment Configuration
 * This module deploys all core contracts for the threat intelligence marketplace
 */

const LocModule = buildModule("NexusSecurityLoc", (m) => {
  // ===== DEPLOYMENT PARAMETERS =====
  
  // Token parameters
  const tokenName = m.getParameter("tokenName", "Nexus Threat Token");
  const tokenSymbol = m.getParameter("tokenSymbol", "NTT");
  const initialSupply = m.getParameter("initialSupply", parseEther("1000000")); // 1M tokens
  
  // Bounty parameters
  const minBountyAmount = m.getParameter("minBountyAmount", parseEther("0.01")); // 0.01 ETH minimum
  const maxAnalysisTime = m.getParameter("maxAnalysisTime", 86400); // 24 hours in seconds
  const platformFeePercent = m.getParameter("platformFeePercent", 250); // 2.5% (basis points)
  
  // Reputation parameters
  const initialReputation = m.getParameter("initialReputation", 100);
  const maxReputation = m.getParameter("maxReputation", 10000);
  const reputationDecayRate = m.getParameter("reputationDecayRate", 5); // 0.05% per day
  
  // ===== CONTRACT DEPLOYMENTS =====
  
  // 1. Deploy Threat Token (ERC20)
  const threatToken = m.contract("ThreatToken", [
    tokenName,
    tokenSymbol,
    initialSupply
  ]);
  
  // 2. Deploy Reputation System
  const reputationSystem = m.contract("ReputationSystem", [
    initialReputation,
    maxReputation,
    reputationDecayRate
  ]);
  
  // 3. Deploy Bounty Manager (main contract)
  const bountyManager = m.contract("BountyManager", [
    threatToken,
    reputationSystem,
    minBountyAmount,
    maxAnalysisTime,
    platformFeePercent
  ]);
  
  // ===== POST-DEPLOYMENT SETUP =====
  
  // Grant necessary roles and permissions
  
  // Allow BountyManager to mint tokens for rewards
  m.call(threatToken, "grantRole", [
    m.staticCall(threatToken, "MINTER_ROLE"),
    bountyManager
  ]);
  
  // Allow BountyManager to update reputation scores
  m.call(reputationSystem, "grantRole", [
    m.staticCall(reputationSystem, "UPDATER_ROLE"),
    bountyManager
  ]);
  
  // Set up initial configuration for local development
  const deployer = m.getAccount(0);
  
  // Transfer some tokens to deployer for testing
  m.call(threatToken, "transfer", [
    deployer,
    parseEther("10000") // 10K tokens for testing
  ]);
  
  // Register deployer as an initial analyst for testing
  m.call(bountyManager, "registerAnalyst", [], {
    from: deployer
  });
  
  // ===== VERIFICATION CALLS =====
  
  // Verify token deployment
  m.call(threatToken, "totalSupply", [], {
    id: "verify_token_supply"
  });
  
  // Verify bounty manager configuration
  m.call(bountyManager, "getMinBountyAmount", [], {
    id: "verify_min_bounty"
  });
  
  // Verify reputation system initialization
  m.call(reputationSystem, "getReputation", [deployer], {
    id: "verify_initial_reputation"
  });
  
  // ===== RETURN DEPLOYED CONTRACTS =====
  
  return {
    threatToken,
    reputationSystem,
    bountyManager,
    // Configuration for easy access
    config: {
      tokenName,
      tokenSymbol,
      initialSupply,
      minBountyAmount,
      maxAnalysisTime,
      platformFeePercent,
      initialReputation,
      maxReputation,
      reputationDecayRate
    }
  };
});

export default LocModule;

// // ===== ADDITIONAL HELPER MODULES =====

// /**
//  * Test Data Setup Module
//  * Deploys sample bounties and test data for local development
//  */
// export const TestDataModule = buildModule("NexusSecurityTestData", (m) => {
//   // Import the main deployment
//   const { bountyManager, threatToken } = m.useModule(LocModule);
  
//   const deployer = m.getAccount(0);
//   const testAnalyst = m.getAccount(1);
//   const testOrganization = m.getAccount(2);
  
//   // Create test accounts with tokens
//   m.call(threatToken, "transfer", [
//     testAnalyst,
//     parseEther("1000")
//   ], {
//     id: "fund_test_analyst"
//   });
  
//   m.call(threatToken, "transfer", [
//     testOrganization,
//     parseEther("5000")
//   ], {
//     id: "fund_test_organization"
//   });
  
//   // Register test users
//   m.call(bountyManager, "registerAnalyst", [], {
//     from: testAnalyst,
//     id: "register_test_analyst"
//   });
  
//   m.call(bountyManager, "registerOrganization", [], {
//     from: testOrganization,
//     id: "register_test_organization"
//   });
  
//   // Create sample bounties
//   const sampleFileHash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
//   const sampleBountyAmount = parseEther("0.1");
  
//   // Approve and create a test bounty
//   m.call(threatToken, "approve", [
//     bountyManager,
//     sampleBountyAmount
//   ], {
//     from: testOrganization,
//     id: "approve_test_bounty"
//   });
  
//   m.call(bountyManager, "createBounty", [
//     sampleFileHash,
//     "Suspicious executable file",
//     sampleBountyAmount,
//     7200 // 2 hours deadline
//   ], {
//     from: testOrganization,
//     id: "create_test_bounty"
//   });
  
//   return {
//     testAccounts: {
//       deployer,
//       testAnalyst,
//       testOrganization
//     },
//     sampleData: {
//       fileHash: sampleFileHash,
//       bountyAmount: sampleBountyAmount
//     }
//   };
// });

// /**
//  * Development Configuration Module
//  * Sets up development-specific configurations
//  */
// export const DevConfigModule = buildModule("NexusSecurityDevConfig", (m) => {
//   const { bountyManager, reputationSystem } = m.useModule(LocModule);
  
//   // Development-specific settings
//   const devSettings = {
//     // Faster analysis times for testing
//     fastAnalysisTime: 300, // 5 minutes
//     // Lower minimum bounty for testing
//     testMinBounty: parseEther("0.001"), // 0.001 ETH
//     // Higher reputation rewards for testing
//     testReputationBonus: 50
//   };
  
//   // Configure for development environment
//   m.call(bountyManager, "updateMinBountyAmount", [
//     devSettings.testMinBounty
//   ], {
//     id: "set_dev_min_bounty"
//   });
  
//   m.call(bountyManager, "updateDefaultAnalysisTime", [
//     devSettings.fastAnalysisTime
//   ], {
//     id: "set_dev_analysis_time"
//   });
  
//   return {
//     devSettings,
//     isConfigured: true
//   };
// });

// // `# Deploy to local network
// // npx hardhat ignition deploy ignition/Loc.ts --network localhost

// // # Deploy with test data
// // npx hardhat ignition deploy ignition/Loc.ts --network localhost --parameters '{"TestDataModule": true}'

// // # Deploy to testnet with custom parameters
// // npx hardhat ignition deploy ignition/Loc.ts --network sepolia --parameters parameters.json``
