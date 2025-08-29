// Contract verification script that:

// Verifies all contracts on block explorers
// Handles different networks automatically
// Provides detailed verification status and links
// Supports individual contract verification


import { run, network } from "hardhat"; 
import { readFileSync } from "fs";
import { join } from "path";
import type { DeploymentAddresses } from "./deploy";

interface VerificationTask {
    name: string;
    address: string;
    constructorArgs: any[];
}

async function loadDeploymentAddresses(networkName: string, chainId: number): Promise<DeploymentAddresses> {
    const deploymentFile = join(__dirname, "..", "deployments", `${networkName}-${chainId}.json`);

    try {
        const data = readFileSync(deploymentFile, 'utf-8');
        return JSON.parse(data);
    } catch (error) {
        throw new Error(`❌ Could not load deployment addresses from ${deploymentFile}. Run deploy.ts first.`);
    }
}

async function verifyContract(contractName: string, address: string, constructorArgs: any[] = []): Promise<boolean> {
    console.log(`🔍 Verifying ${contractName} at ${address}...`);

    try {
        await run("verify:verify", {
            address: address,
            constructorArguments: constructorArgs,
        });
        console.log(`✅ ${contractName} verified successfully.`);
        return true;
        } catch (error: any) {
            if (error.message.includes("already verified")) {
        console.log(`ℹ️  ${contractName} is already verified`);
        return true;
        } else if (error.message.includes("does not have bytecode")) {
        console.error(`❌ ${contractName} - Contract not found at address ${address}`);
        return false;
        } else if (error.message.includes("constructor parameters")) {
        console.error(`❌ ${contractName} - Constructor parameters mismatch`);
        console.error(`   Expected args: ${JSON.stringify(constructorArgs)}`);
        return false;
        } else {
        console.error(`❌ ${contractName} verification failed:`);
        console.error(`   ${error.message}`);
        return false;
        }
    }
}

async function verifyAllContracts(deploymentAddresses: DeploymentAddresses): Promise<void> {
    console.log("🔍 Starting contract verification process...\n");
    const verificationTasks: VerificationTask[] = [
    {
      name: "ThreatToken",
      address: deploymentAddresses.threatToken,
      constructorArgs: [
        "ThreatToken",              // name
        "THREAT",                   // symbol
        "1000000000000000000000000"  // initial supply (1M tokens in wei)
      ]
    },
    {
      name: "ReputationSystem",
      address: deploymentAddresses.reputationSystem,
      constructorArgs: []
    },
    {
      name: "BountyManager",
      address: deploymentAddresses.bountyManager,
      constructorArgs: [
        deploymentAddresses.threatToken,
        deploymentAddresses.reputationSystem
      ]
    }
  ];
    
    const results: { name: string; success: boolean }[] = [];

    for (const task of verificationTasks) {
        const success = await verifyContract(task.name, task.address, task.constructorArgs);
        results.push({ name: task.name, success });
        
        // Add a small delay between verifications to avoid rate limiting
        if (verificationTasks.indexOf(task) < verificationTasks.length - 1) {
            console.log("⏳ Waiting 5 seconds before next verification...\n");
            await new Promise(resolve => setTimeout(resolve, 5000));
        }   
    }

    // Print verification summary
  console.log("\n📋 Verification Summary:");
  console.log("═".repeat(50));
  
  let allSuccessful = true;
  for (const result of results) {
    const status = result.success ? "✅ VERIFIED" : "❌ FAILED";
    console.log(`${result.name.padEnd(20)}: ${status}`);
    if (!result.success) allSuccessful = false;
  }
  
  console.log("═".repeat(50));
  
  if (allSuccessful) {
    console.log("🎉 All contracts verified successfully!");
  } else {
    console.log("⚠️  Some contracts failed verification. Check the logs above for details.");
  }
  
  // Print block explorer links
  console.log("\n🔗 Block Explorer Links:");
  console.log("═".repeat(50));
  
  const explorerUrls = getExplorerUrls(network.name);
  if (explorerUrls.length > 0) {
    for (const task of verificationTasks) {
      console.log(`${task.name}:`);
      for (const baseUrl of explorerUrls) {
        console.log(`  ${baseUrl}/address/${task.address}`);
      }
    }
  } else {
    console.log("No known block explorers for this network");
  }
  
  console.log("═".repeat(50));
}

function getExplorerUrls(networkName: string): string[] {
  const explorerMap: Record<string, string[]> = {
    "mainnet": ["https://etherscan.io"],
    "goerli": ["https://goerli.etherscan.io"],
    "sepolia": ["https://sepolia.etherscan.io"],
    "polygon": ["https://polygonscan.com"],
    "polygonMumbai": ["https://mumbai.polygonscan.com"],
    "bsc": ["https://bscscan.com"],
    "bscTestnet": ["https://testnet.bscscan.com"],
    "avalanche": ["https://snowtrace.io"],
    "avalancheFuji": ["https://testnet.snowtrace.io"],
    "arbitrumOne": ["https://arbiscan.io"],
    "arbitrumGoerli": ["https://goerli.arbiscan.io"],
    "optimism": ["https://optimistic.etherscan.io"],
    "optimismGoerli": ["https://goerli-optimism.etherscan.io"]
  };
  
  return explorerMap[networkName] || [];
}

async function verifyIndividualContract(contractName: string, contractAddress?: string) {
  console.log(`🔍 Verifying individual contract: ${contractName}\n`);
  
  const networkInfo = await network.provider.send("eth_chainId", []);
  const chainId = parseInt(networkInfo, 16);
  const deploymentAddresses = await loadDeploymentAddresses(network.name, chainId);
  
  let address: string;
  let constructorArgs: any[] = [];
  
  if (contractAddress) {
    address = contractAddress;
    console.log(`Using provided address: ${address}`);
  } else {
    // Get address from deployment file
    switch (contractName.toLowerCase()) {
      case "threattoken":
        address = deploymentAddresses.threatToken;
        constructorArgs = ["ThreatToken", "THREAT", "1000000000000000000000000"];
        break;
      case "reputationsystem":
        address = deploymentAddresses.reputationSystem;
        constructorArgs = [];
        break;
      case "bountymanager":
        address = deploymentAddresses.bountyManager;
        constructorArgs = [deploymentAddresses.threatToken, deploymentAddresses.reputationSystem];
        break;
      default:
        throw new Error(`❌ Unknown contract name: ${contractName}`);
    }
    console.log(`Using address from deployment file: ${address}`);
  }
  
  const success = await verifyContract(contractName, address, constructorArgs);
  
  if (success) {
    const explorerUrls = getExplorerUrls(network.name);
    if (explorerUrls.length > 0) {
      console.log(`\n🔗 View on block explorer:`);
      for (const baseUrl of explorerUrls) {
        console.log(`   ${baseUrl}/address/${address}`);
      }
    }
  }
  
  return success;
}

async function main() {
  console.log("🔍 Starting Nexus-Security contract verification...\n");
  
  const networkInfo = await network.provider.send("eth_chainId", []);
  const chainId = parseInt(networkInfo, 16);
  
  console.log(`📡 Network: ${network.name} (Chain ID: ${chainId})\n`);
  
  // Check if specific contract verification is requested
  const args = process.argv.slice(2);
  
  if (args.length > 0) {
    // Verify specific contract
    const contractName = args[0];
    const contractAddress = args[1]; // optional
    
    try {
      await verifyIndividualContract(contractName, contractAddress);
    } catch (error) {
      console.error("❌ Verification failed:");
      console.error(error);
      process.exit(1);
    }
  } else {
    // Verify all contracts
    try {
      const deploymentAddresses = await loadDeploymentAddresses(network.name, chainId);
      await verifyAllContracts(deploymentAddresses);
    } catch (error) {
      console.error("❌ Verification failed:");
      console.error(error);
      process.exit(1);
    }
  }
}

// Allow this script to be run directly
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("❌ Verification failed:");
      console.error(error);
      process.exit(1);
    });
}

export { verifyAllContracts, verifyIndividualContract };