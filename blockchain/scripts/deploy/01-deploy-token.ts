import { ethers } from "hardhat";

/**
 * Deploy ThreatToken contract
 */
async function deployThreatToken() {
    console.log("\n=Ý Deploying ThreatToken...");

    const [deployer] = await ethers.getSigners();
    console.log(`   Deployer: ${deployer.address}`);

    // Get contract factory
    const ThreatToken = await ethers.getContractFactory("ThreatToken");

    // Deploy with deployer as admin
    const threatToken = await ThreatToken.deploy(deployer.address);
    await threatToken.waitForDeployment();

    const address = await threatToken.getAddress();
    console.log(`    ThreatToken deployed to: ${address}`);

    // Get initial stats
    const totalSupply = await threatToken.totalSupply();
    const maxSupply = await threatToken.MAX_SUPPLY();

    console.log(`   =Ê Initial Supply: ${ethers.formatEther(totalSupply)} THREAT`);
    console.log(`   =Ê Max Supply: ${ethers.formatEther(maxSupply)} THREAT`);

    return {
        threatToken,
        address
    };
}

// Allow script to be run directly
if (require.main === module) {
    deployThreatToken()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

export { deployThreatToken };
