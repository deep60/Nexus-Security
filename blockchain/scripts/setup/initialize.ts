import { ethers } from "hardhat";
import { ThreatToken, ReputationSystem, BountyManager } from "../../typechain-types";

interface DeployedContracts {
    threatToken: string;
    reputationSystem: string;
    bountyManager: string;
}

/**
 * Initialize contracts with proper roles and configurations
 */
async function initializeContracts(addresses: DeployedContracts) {
    console.log("\n™  Initializing contracts...");

    const [deployer] = await ethers.getSigners();
    console.log(`   Initializer: ${deployer.address}\n`);

    // Get contract instances
    const threatToken = await ethers.getContractAt("ThreatToken", addresses.threatToken) as unknown as ThreatToken;
    const reputationSystem = await ethers.getContractAt("ReputationSystem", addresses.reputationSystem) as unknown as ReputationSystem;
    const bountyManager = await ethers.getContractAt("BountyManager", addresses.bountyManager) as unknown as BountyManager;

    console.log("=Ý Step 1: Grant roles to contracts");

    // Grant BountyManager role in ThreatToken
    const BOUNTY_MANAGER_ROLE = await threatToken.BOUNTY_MANAGER_ROLE();
    let tx = await threatToken.grantRole(BOUNTY_MANAGER_ROLE, addresses.bountyManager);
    await tx.wait();
    console.log("    Granted BOUNTY_MANAGER_ROLE to BountyManager in ThreatToken");

    const REPUTATION_MANAGER_ROLE = await threatToken.REPUTATION_MANAGER_ROLE();
    tx = await threatToken.grantRole(REPUTATION_MANAGER_ROLE, addresses.reputationSystem);
    await tx.wait();
    console.log("    Granted REPUTATION_MANAGER_ROLE to ReputationSystem in ThreatToken");

    // Grant BountyManager role in ReputationSystem
    const BOUNTY_MANAGER_ROLE_REP = await reputationSystem.BOUNTY_MANAGER_ROLE();
    tx = await reputationSystem.grantRole(BOUNTY_MANAGER_ROLE_REP, addresses.bountyManager);
    await tx.wait();
    console.log("    Granted BOUNTY_MANAGER_ROLE to BountyManager in ReputationSystem");

    console.log("\n=Ý Step 2: Authorize test engines (optional for testnet)");

    // On testnets, authorize the deployer as a test engine
    const network = await ethers.provider.getNetwork();
    if (network.chainId !== 1n && network.chainId !== 137n) { // Not mainnet or Polygon
        tx = await threatToken.setEngineAuthorization(deployer.address, true);
        await tx.wait();
        console.log("    Authorized deployer as test engine");

        // Register deployer in reputation system
        tx = await reputationSystem.registerEngine(deployer.address, 0); // 0 = Human
        await tx.wait();
        console.log("    Registered deployer in ReputationSystem");
    }

    console.log("\n=Ý Step 3: Verify configurations");

    // Verify ThreatToken
    const maxSupply = await threatToken.MAX_SUPPLY();
    const totalSupply = await threatToken.totalSupply();
    console.log(`   ThreatToken Max Supply: ${ethers.formatEther(maxSupply)} THREAT`);
    console.log(`   ThreatToken Total Supply: ${ethers.formatEther(totalSupply)} THREAT`);

    // Verify ReputationSystem
    const minReputation = await reputationSystem.getMinimumReputation();
    console.log(`   ReputationSystem Min Reputation: ${minReputation}`);

    // Verify BountyManager
    const totalBounties = await bountyManager.getTotalBounties();
    console.log(`   BountyManager Total Bounties: ${totalBounties}`);

    console.log("\n Initialization complete!");

    return {
        threatToken,
        reputationSystem,
        bountyManager
    };
}

// Allow script to be run directly
if (require.main === module) {
    const args = process.argv.slice(2);
    if (args.length < 3) {
        console.error("Usage: npx hardhat run scripts/setup/initialize.ts --network <network> -- <threatTokenAddress> <reputationSystemAddress> <bountyManagerAddress>");
        process.exit(1);
    }

    const [threatToken, reputationSystem, bountyManager] = args;

    initializeContracts({
        threatToken,
        reputationSystem,
        bountyManager
    })
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

export { initializeContracts };
