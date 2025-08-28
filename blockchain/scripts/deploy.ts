import { ethers } from "hardhat";
import { join } from "path";
import { writeFileSync, mkdirSync } from "fs";

interface DeploymentAddresses {
    threatToken: string;
    reputationSystem: string;
    bountyManager: string;
    deployer: string;
    network: string;
    blocknumber: number;
    timestamp: number;
}

async function main() {
    console.log("🚀 Starting Nexus-Security smart contract deployment...\n");

    // Get network information
    const network = await ethers.provider.getNetwork();
    const [deployer] = await ethers.getSigners();

    console.log(`📡 Network: ${network.name} (Chain ID: ${network.chainId})`);
    console.log(`👤 Deployer: ${deployer.address}`);

    // check deployer balance
    const balance = await deployer.provider.getBalance(deployer.address);
    console.log(`💰 Deployer balance: ${ethers.formatEther(balance)} ETH\n`);

    if (balance < ethers.parseEther("0.1")) {
        throw new Error("❌ Insufficient balance for deployment. Need at least 0.1 ETH");
    }

    // Deploy ThreatToken first
    console.log("🗒️ Deploying ThreatToken...");
    const ThreatTokenFactory = await ethers.getContractFactory("ThreatToken");
    const threatToken = await ThreatTokenFactory.deploy(
        "ThreatToken",      // name
        "THREAT",          // symbol
        ethers.parseEther("1000000")      // initial supply: 1M tokens
    );
    await threatToken.waitForDeployment();
    const threatTokenAddress = await threatToken.getAddress();
    console.log('✅ ThreatToken deployed at: ${threatTokenAddress}');

    // Deploy ReputationSystem next
    console.log("\n📋 Deploying ReputationSystem...");
  const ReputationSystemFactory = await ethers.getContractFactory("ReputationSystem");
  const reputationSystem = await ReputationSystemFactory.deploy();
  await reputationSystem.waitForDeployment();
  const reputationSystemAddress = await reputationSystem.getAddress();
  console.log(`✅ ReputationSystem deployed at: ${reputationSystemAddress}`);

  // Deploy BountyManager 
  console.log("\n 📋 Deploying BountyManager...");
  const BountyManagerFactory = await ethers.getContractFactory("BountyManager");
  const bountyManager = await BountyManagerFactory.deploy(
    threatTokenAddress,
    reputationSystemAddress
  );
  await bountyManager.waitForDeployment();
  const bountyManagerAddress = await bountyManager.getAddress();
  console.log(`✅ BountyManager deployed at: ${bountyManagerAddress}`);

  // Setup initial configuration
  console.log("\n⚙️  Setting up initial configuration...");
  
  // Grant BountyManager role in ReputationSystem
  const MANAGER_ROLE = await reputationSystem.MANAGER_ROLE();
  await reputationSystem.grantRole(MANAGER_ROLE, bountyManagerAddress);
  console.log("✅ Granted MANAGER_ROLE to BountyManager in ReputationSystem");

  // Setup BountyManager with initial parameters
  await bountyManager.setMinimumStake(ethers.parseEther("10")); // 10 THREAT tokens minimum stake
  await bountyManager.setAnalysisTimeout(3600); // 1 hour analysis timeout
  console.log("✅ Set initial BountyManager parameters");

  // Transfer some tokens to BountyManager for rewards
  const rewardPoolAmount = ethers.parseEther("100000"); // 100K tokens for reward pool
  await threatToken.transfer(bountyManagerAddress, rewardPoolAmount);
  console.log(`✅ Transferred ${ethers.formatEther(rewardPoolAmount)} THREAT tokens to BountyManager`);

  // Get current block information
  const currentBlock = await ethers.provider.getBlockNumber();
  const timestamp = Math.floor(Date.now() / 1000);

  // Prepare deployment data
  const deploymentData: DeploymentAddresses = {
    threatToken: threatTokenAddress,
    reputationSystem: reputationSystemAddress,
    bountyManager: bountyManagerAddress,
    deployer: deployer.address,
    network: network.name,
    blockNumber: currentBlock,
    timestamp
  };

  // Save deployment addresses
  const deploymentsDir = join(__dirname, "..", "deployments");
  try {
    mkdirSync(deploymentsDir, { recursive: true });
  } catch (err) {
    // Directory already exists
  }

  const deploymentFile = join(deploymentsDir, `${network.name}-${network.chainId}.json`);
  writeFileSync(deploymentFile, JSON.stringify(deploymentData, null, 2));

  // Also save to a general deployments file
  const allDeploymentsFile = join(deploymentsDir, "deployments.json");
  let allDeployments: Record<string, DeploymentAddresses> = {};
  
  try {
    const existingData = require(allDeploymentsFile);
    allDeployments = existingData;
  } catch (err) {
    // File doesn't exist yet
  }
  
  allDeployments[`${network.name}-${network.chainId}`] = deploymentData;
  writeFileSync(allDeploymentsFile, JSON.stringify(allDeployments, null, 2));

  console.log("\n🎉 Deployment completed successfully!");
  console.log("📄 Deployment summary:");
  console.log("═".repeat(50));
  console.log(`Network: ${network.name} (${network.chainId})`);
  console.log(`Block Number: ${currentBlock}`);
  console.log(`Deployer: ${deployer.address}`);
  console.log(`ThreatToken: ${threatTokenAddress}`);
  console.log(`ReputationSystem: ${reputationSystemAddress}`);
  console.log(`BountyManager: ${bountyManagerAddress}`);
  console.log("═".repeat(50));
  console.log(`📁 Deployment data saved to: ${deploymentFile}`);

  // Print verification commands
  console.log("\n🔍 To verify contracts, run:");
  console.log(`npx hardhat verify --network ${network.name} ${threatTokenAddress} "ThreatToken" "THREAT" "${ethers.parseEther("1000000")}"`);
  console.log(`npx hardhat verify --network ${network.name} ${reputationSystemAddress}`);
  console.log(`npx hardhat verify --network ${network.name} ${bountyManagerAddress} ${threatTokenAddress} ${reputationSystemAddress}`);

  // Return deployment data for other scripts
  return deploymentData;
}

// Allow this script to be run directly
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("❌ Deployment failed:");
      console.error(error);
      process.exit(1);
    });
}

export { main as deployContracts };
export type { DeploymentAddresses };
