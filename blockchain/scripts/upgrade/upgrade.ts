import { ethers, upgrades } from "hardhat";

/**
 * Upgrade BountyManager to BountyManagerV2 using UUPS pattern
 */
async function upgradeToV2(proxyAddress: string) {
    console.log("\n= Upgrading BountyManager to V2...");

    const [deployer] = await ethers.getSigners();
    console.log(`   Upgrader: ${deployer.address}`);
    console.log(`   Proxy Address: ${proxyAddress}`);

    // Get the V2 factory
    const BountyManagerV2 = await ethers.getContractFactory("BountyManagerV2");

    console.log("   Validating upgrade...");

    try {
        // Upgrade the proxy to V2
        const upgraded = await upgrades.upgradeProxy(proxyAddress, BountyManagerV2);
        await upgraded.waitForDeployment();

        const newImplAddress = await upgrades.erc1967.getImplementationAddress(proxyAddress);

        console.log(`    Upgraded successfully!`);
        console.log(`   =Í New Implementation: ${newImplAddress}`);
        console.log(`   =Í Proxy: ${proxyAddress}`);

        // Verify the version
        const version = await upgraded.version();
        console.log(`   =Ê New Version: ${version}`);

        return {
            proxy: proxyAddress,
            implementation: newImplAddress,
            version
        };
    } catch (error) {
        console.error("   L Upgrade failed:");
        throw error;
    }
}

// Allow script to be run directly
if (require.main === module) {
    const args = process.argv.slice(2);
    if (args.length < 1) {
        console.error("Usage: npx hardhat run scripts/upgrade/upgrade.ts --network <network> -- <proxyAddress>");
        process.exit(1);
    }

    const [proxyAddress] = args;

    upgradeToV2(proxyAddress)
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

export { upgradeToV2 };
