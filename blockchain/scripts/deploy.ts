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
}