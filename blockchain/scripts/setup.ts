// Post-deployment configuration script that:

// Configures contract parameters (stakes, timeouts, thresholds)
// Sets up admin and moderator roles
// Prepares initial analysis engines for registration
// Validates system configuration

import { ethers } from "hardhat";
import { readFileSync } from "fs";
import { join } from "path";
import { DeploymentAddresses } from "./deploy";

interface SetupConfig {
    // Bounty Manager Settings
    minimumStake: string;
    analysisTimeout: number;
    consensusThreshold: number;
    rewardPercentage: number;
    slashPercentage: number;

    // Reputation System Settings
    initialReputation: number;
    maxReputationBonus: number;
    reputationDecayRate: number;

    // Token Settings
    engineRegistrationFee: string;

    // System Roles
    adminAddresses: string[];
    moderatorAddresses: string[];
    engineAddresses: string[];
}

const DEFAULT_CONFIG: SetupConfig = {
    minimumStake: "10",    // 10 THREAT tokens
    analysisTimeout: 3600, // 1 hours
    consensusThreshold: 70,  // 70% agreeement needed
    rewardPercentage: 80,   // 80% of bounty goes to winners
    slashPercentage: 20,    // 20% penalty for wrong answers

    initialReputation: 100,
    maxReputationBonus: 50,
    reputationDecayRate: 1, // 1% per day

    engineRegistrationFee: "100",  // 100 THREAT tokens

    adminAddresses: [],
    moderatorAddresses: [],
    engineAddresses: []
};

async function loadDeploymentAddresses(networkName: string, chainId: number): Promise<DeploymentAddresses> {
    const deploymentFile = join(__dirname, "..", "deployments", `${networkName}-${chainId}.json`);

    try {
        const data = readFileSync(deploymentFile, "utf8");
        return JSON.parse(data);
    } catch (error) {
        throw new Error('❌ Could not load deployment addresses from ${deploymentFile}. Run deploy.ts first.');
    }
}

async function setupBountyManager(bountyManagerAddress: string, config: SetupConfig, signer: any) {
    console.log("⚙️  Setting up BountyManager...");
    const bountyManager = await ethers.getContractAt("BountyManager", bountyManagerAddress, signer);

    // Set minimum stake
    const currentMinStake = await bountyManager.minimumStake();
    const newMinStake = ethers.parseEther(config.minimumStake);

    if (currentMinStake !== newMinStake) {
        const tx = await bountyManager.setMinimumStake(newMinStake);
        await tx.await();
        console.log(`✅ Set minimum stake to ${config.minimumStake} THREAT`);
    }

    // Set analysis timeout
    const currentTimeout = await bountyManager.analysisTimeout();
    if (currentTimeout !== config.analysisTimeout) {
        const tx = await bountyManager.setAnalysisTimeout(config.analysisTimeout);
        await tx.await();
        console.log(`✅ Set analysis timeout to ${config.analysisTimeout} seconds`);
    }

    // Set consensus threshold
    const currentThreshold = await bountyManager.consensusThreshold();
    if (currentThreshold !== config.consensusThreshold) {
        const tx = await bountyManager.setConsensusThreshold(config.consensusThreshold);
        await tx.await();
        console.log(`✅ Set consensus threshold to ${config.consensusThreshold}%`);
    }

    // Set reward percentage
    const currentRewardPct = await bountyManager.rewardPercentage();
    if (currentRewardPct !== config.rewardPercentage) {
        const tx = await bountyManager.setRewardPercentage(config.rewardPercentage);
        await tx.await();
        console.log(`✅ Set reward percentage to ${config.rewardPercentage}%`);
    }
}

async function setupReputationSystem(reputationSystemAddress: string, config: SetupConfig, signer: any) {
    console.log("\n⚙️  Setting up ReputationSystem...");

    const reputationSystem = await ethers.getContractAt("ReputationSystem", reputationSystemAddress, signer);
  
  // Set initial reputation for new engines
  const currentInitialRep = await reputationSystem.initialReputation();
  if (currentInitialRep !== config.initialReputation) {
    const tx = await reputationSystem.setInitialReputation(config.initialReputation);
    await tx.wait();
    console.log(`✅ Set initial reputation to ${config.initialReputation}`);
  }
  
  // Set maximum reputation bonus
  const currentMaxBonus = await reputationSystem.maxReputationBonus();
  if (currentMaxBonus !== config.maxReputationBonus) {
    const tx = await reputationSystem.setMaxReputationBonus(config.maxReputationBonus);
    await tx.wait();
    console.log(`✅ Set max reputation bonus to ${config.maxReputationBonus}`);
  }
}

async function setupRoles(deploymentAddresses: DeploymentAddresses, config: SetupConfig, signer: any) {
     console.log("\n👥 Setting up roles...");
  
  const reputationSystem = await ethers.getContractAt("ReputationSystem", deploymentAddresses.reputationSystem, signer);
  const bountyManager = await ethers.getContractAt("BountyManager", deploymentAddresses.bountyManager, signer);
  
  // Get role identifiers
  const ADMIN_ROLE = await reputationSystem.DEFAULT_ADMIN_ROLE();
  const MANAGER_ROLE = await reputationSystem.MANAGER_ROLE();
  const MODERATOR_ROLE = await bountyManager.MODERATOR_ROLE();
  
  // Setup admin roles
  for (const adminAddress of config.adminAddresses) {
    if (ethers.isAddress(adminAddress)) {
      // Grant admin role in ReputationSystem
      if (!(await reputationSystem.hasRole(ADMIN_ROLE, adminAddress))) {
        const tx = await reputationSystem.grantRole(ADMIN_ROLE, adminAddress);
        await tx.wait();
        console.log(`✅ Granted ADMIN_ROLE to ${adminAddress} in ReputationSystem`);
      }
      
      // Grant admin role in BountyManager
      if (!(await bountyManager.hasRole(ADMIN_ROLE, adminAddress))) {
        const tx = await bountyManager.grantRole(ADMIN_ROLE, adminAddress);
        await tx.wait();
        console.log(`✅ Granted ADMIN_ROLE to ${adminAddress} in BountyManager`);
      }
    }
  }
  
  // Setup moderator roles
  for (const moderatorAddress of config.moderatorAddresses) {
    if (ethers.isAddress(moderatorAddress)) {
      if (!(await bountyManager.hasRole(MODERATOR_ROLE, moderatorAddress))) {
        const tx = await bountyManager.grantRole(MODERATOR_ROLE, moderatorAddress);
        await tx.wait();
        console.log(`✅ Granted MODERATOR_ROLE to ${moderatorAddress} in BountyManager`);
      }
    }
  }
}

async function registerInitialEngines(
  deploymentAddresses: DeploymentAddresses,
  config: SetupConfig,
  signer: any
) {
  console.log("\n🤖 Registering initial analysis engines...");
  
  const bountyManager = await ethers.getContractAt("BountyManager", deploymentAddresses.bountyManager, signer);
  const threatToken = await ethers.getContractAt("ThreatToken", deploymentAddresses.threatToken, signer);
  
  const registrationFee = ethers.parseEther(config.engineRegistrationFee);
  
  for (let i = 0; i < config.engineAddresses.length; i++) {
    const engineAddress = config.engineAddresses[i];
    
    if (ethers.isAddress(engineAddress)) {
      // Check if engine is already registered
      const isRegistered = await bountyManager.isRegisteredEngine(engineAddress);
      
      if (!isRegistered) {
        // Transfer registration fee to engine address (for testing purposes)
        const engineBalance = await threatToken.balanceOf(engineAddress);
        if (engineBalance < registrationFee) {
          const tx = await threatToken.transfer(engineAddress, registrationFee);
          await tx.wait();
          console.log(`✅ Transferred ${config.engineRegistrationFee} THREAT to ${engineAddress}`);
        }
        
        // Note: The actual registration would need to be done by the engine address
        // This is just preparation. In production, engines register themselves.
        console.log(`📝 Engine ${engineAddress} prepared for registration`);
      } else {
        console.log(`ℹ️  Engine ${engineAddress} already registered`);
      }
    }
  }
}

async function displaySystemStatus(deploymentAddresses: DeploymentAddresses, signer: any) {
  console.log("\n📊 System Status:");
  console.log("═".repeat(50));
  
  const threatToken = await ethers.getContractAt("ThreatToken", deploymentAddresses.threatToken, signer);
  const reputationSystem = await ethers.getContractAt("ReputationSystem", deploymentAddresses.reputationSystem, signer);
  const bountyManager = await ethers.getContractAt("BountyManager", deploymentAddresses.bountyManager, signer);
  
  // Token information
  const tokenName = await threatToken.name();
  const tokenSymbol = await threatToken.symbol();
  const totalSupply = await threatToken.totalSupply();
  const bountyManagerBalance = await threatToken.balanceOf(deploymentAddresses.bountyManager);
  
  console.log(`Token: ${tokenName} (${tokenSymbol})`);
  console.log(`Total Supply: ${ethers.formatEther(totalSupply)} ${tokenSymbol}`);
  console.log(`BountyManager Balance: ${ethers.formatEther(bountyManagerBalance)} ${tokenSymbol}`);
  
  // BountyManager configuration
  const minStake = await bountyManager.minimumStake();
  const analysisTimeout = await bountyManager.analysisTimeout();
  const consensusThreshold = await bountyManager.consensusThreshold();
  
  console.log(`Minimum Stake: ${ethers.formatEther(minStake)} ${tokenSymbol}`);
  console.log(`Analysis Timeout: ${analysisTimeout} seconds`);
  console.log(`Consensus Threshold: ${consensusThreshold}%`);
  
  // Reputation System configuration
  const initialRep = await reputationSystem.initialReputation();
  console.log(`Initial Reputation: ${initialRep}`);
  
  console.log("═".repeat(50));
}

async function main() {
  console.log("🔧 Starting Nexus-Security post-deployment setup...\n");
  
  const network = await ethers.provider.getNetwork();
  const [deployer] = await ethers.getSigners();
  
  console.log(`📡 Network: ${network.name} (Chain ID: ${network.chainId})`);
  console.log(`👤 Setup Account: ${deployer.address}\n`);
  
  // Load deployment addresses
  const deploymentAddresses = await loadDeploymentAddresses(network.name, Number(network.chainId));
  console.log("✅ Loaded deployment addresses");
  
  // Load configuration (in production, this might come from environment variables or config files)
  let config: SetupConfig = { ...DEFAULT_CONFIG };
  
  // Override with environment variables if available
  if (process.env.MIN_STAKE) {
    config.minimumStake = process.env.MIN_STAKE;
  }
  if (process.env.ANALYSIS_TIMEOUT) {
    config.analysisTimeout = parseInt(process.env.ANALYSIS_TIMEOUT);
  }
  if (process.env.ADMIN_ADDRESSES) {
    config.adminAddresses = process.env.ADMIN_ADDRESSES.split(',');
  }
  if (process.env.MODERATOR_ADDRESSES) {
    config.moderatorAddresses = process.env.MODERATOR_ADDRESSES.split(',');
  }
  if (process.env.ENGINE_ADDRESSES) {
    config.engineAddresses = process.env.ENGINE_ADDRESSES.split(',');
  }
  
  try {
    // Setup BountyManager
    await setupBountyManager(deploymentAddresses.bountyManager, config, deployer);
    
    // Setup ReputationSystem
    await setupReputationSystem(deploymentAddresses.reputationSystem, config, deployer);
    
    // Setup roles
    await setupRoles(deploymentAddresses, config, deployer);
    
    // Register initial engines
    if (config.engineAddresses.length > 0) {
      await registerInitialEngines(deploymentAddresses, config, deployer);
    }
    
    // Display final system status
    await displaySystemStatus(deploymentAddresses, deployer);
    
    console.log("\n🎉 Setup completed successfully!");
    console.log("The Nexus-Security platform is now configured and ready for use.");
    
  } catch (error) {
    console.error("❌ Setup failed:");
    console.error(error);
    throw error;
  }
}

// Allow this script to be run directly
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("❌ Setup failed:");
      console.error(error);
      process.exit(1);
    });
}

export { main as setupContracts };
export type { SetupConfig };