import { ethers } from "hardhat";

/**
 * Deploy BountyManager contract
 */
async function deployBountyManager(
    threatTokenAddress: string,
    reputationSystemAddress: string,
    feeCollectorAddress?: string
) {
    console.log("\n=Ý Deploying BountyManager...");

    const [deployer] = await ethers.getSigners();
    console.log(`   Deployer: ${deployer.address}`);

    // Use deployer as fee collector if not provided
    const feeCollector = feeCollectorAddress || deployer.address;
    console.log(`   Fee Collector: ${feeCollector}`);

    // Validate inputs
    if (!threatTokenAddress || threatTokenAddress === ethers.ZeroAddress) {
        throw new Error("Invalid ThreatToken address");
    }
    if (!reputationSystemAddress || reputationSystemAddress === ethers.ZeroAddress) {
        throw new Error("Invalid ReputationSystem address");
    }

    // Get contract factory
    const BountyManager = await ethers.getContractFactory("BountyManager");

    // Deploy
    const bountyManager = await BountyManager.deploy(
        threatTokenAddress,
        reputationSystemAddress,
        feeCollector
    );
    await bountyManager.waitForDeployment();

    const address = await bountyManager.getAddress();
    console.log(`    BountyManager deployed to: ${address}`);

    // Get initial stats
    const totalBounties = await bountyManager.getTotalBounties();
    console.log(`   =Ê Total Bounties: ${totalBounties}`);

    return {
        bountyManager,
        address
    };
}

// Allow script to be run directly
if (require.main === module) {
    const args = process.argv.slice(2);
    if (args.length < 2) {
        console.error("Usage: npx hardhat run scripts/deploy/03-deploy-bounty.ts --network <network> -- <threatTokenAddress> <reputationSystemAddress> [feeCollectorAddress]");
        process.exit(1);
    }

    const [threatTokenAddress, reputationSystemAddress, feeCollectorAddress] = args;

    deployBountyManager(threatTokenAddress, reputationSystemAddress, feeCollectorAddress)
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

export { deployBountyManager };
