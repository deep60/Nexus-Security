import { run } from "hardhat";

interface _ContractToVerify {
    address: string;
    constructorArguments: unknown[];
    contract?: string;
}

/**
 * Verify a contract on Etherscan/Polygonscan
 */
async function verifyContract(
    address: string,
    constructorArguments: unknown[],
    contractPath?: string
) {
    console.log(`\n🔍 Verifying contract at ${address}...`);

    try {
        await run("verify:verify", {
            address,
            constructorArguments,
            contract: contractPath
        });
        console.log(`   ✅ Verified successfully!`);
    } catch (error: unknown) {
        const message = error instanceof Error ? error.message : String(error);
        if (message.toLowerCase().includes("already verified")) {
            console.log(`   ℹ️  Contract already verified`);
        } else {
            console.error(`   ❌ Verification failed:`, message);
            throw error;
        }
    }
}

/**
 * Verify all Nexus-Security contracts
 */
async function verifyAllContracts(
    threatTokenAddress: string,
    reputationSystemAddress: string,
    bountyManagerAddress: string,
    feeCollectorAddress: string
) {
    console.log("🔍 Verifying all Nexus-Security contracts...\n");

    // Verify ThreatToken
    await verifyContract(
        threatTokenAddress,
        [feeCollectorAddress], // admin address
        "contracts/core/ThreatToken.sol:ThreatToken"
    );

    // Verify ReputationSystem
    await verifyContract(
        reputationSystemAddress,
        [],
        "contracts/core/ReputationSystem.sol:ReputationSystem"
    );

    // Verify BountyManager
    await verifyContract(
        bountyManagerAddress,
        [threatTokenAddress, reputationSystemAddress, feeCollectorAddress],
        "contracts/core/BountyManager.sol:BountyManager"
    );

    console.log("\n✅ All contracts verified!");
}

// Allow script to be run directly
if (require.main === module) {
    const args = process.argv.slice(2);
    if (args.length < 4) {
        console.error("Usage: npx hardhat run scripts/verify/verify.ts --network <network> -- <threatToken> <reputationSystem> <bountyManager> <feeCollector>");
        process.exit(1);
    }

    const [threatToken, reputationSystem, bountyManager, feeCollector] = args;

    verifyAllContracts(threatToken, reputationSystem, bountyManager, feeCollector)
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

export { verifyContract, verifyAllContracts };
