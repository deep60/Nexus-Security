import { ethers } from "hardhat";

/**
 * Deploy ReputationSystem contract
 */
async function deployReputationSystem() {
    console.log("\n=Ý Deploying ReputationSystem...");

    const [deployer] = await ethers.getSigners();
    console.log(`   Deployer: ${deployer.address}`);

    // Get contract factory
    const ReputationSystem = await ethers.getContractFactory("ReputationSystem");

    // Deploy
    const reputationSystem = await ReputationSystem.deploy();
    await reputationSystem.waitForDeployment();

    const address = await reputationSystem.getAddress();
    console.log(`    ReputationSystem deployed to: ${address}`);

    // Get initial stats
    const minRep = await reputationSystem.getMinimumReputation();
    const totalAnalysts = await reputationSystem.getTotalAnalysts();

    console.log(`   =Ê Minimum Reputation: ${minRep}`);
    console.log(`   =Ê Total Analysts: ${totalAnalysts}`);

    return {
        reputationSystem,
        address
    };
}

// Allow script to be run directly
if (require.main === module) {
    deployReputationSystem()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

export { deployReputationSystem };
