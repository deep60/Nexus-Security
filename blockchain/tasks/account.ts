import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";

/**
 * Task to display account information
 */
task("accounts", "Prints the list of accounts with balances")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const accounts = await hre.ethers.getSigners();

        console.log("\n=Ë Available Accounts:");
        console.log("P".repeat(80));

        for (let i = 0; i < accounts.length; i++) {
            const account = accounts[i];
            const balance = await hre.ethers.provider.getBalance(account.address);

            console.log(`[${i}] ${account.address}`);
            console.log(`    Balance: ${hre.ethers.formatEther(balance)} ETH`);
        }

        console.log("P".repeat(80));
    });

/**
 * Task to check account balance
 */
task("balance", "Prints an account's balance")
    .addParam("account", "The account's address")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const balance = await hre.ethers.provider.getBalance(taskArgs.account);

        console.log(`\n=° Balance of ${taskArgs.account}:`);
        console.log(`   ${hre.ethers.formatEther(balance)} ETH`);
    });

/**
 * Task to check ThreatToken balance
 */
task("token-balance", "Prints an account's ThreatToken balance")
    .addParam("account", "The account's address")
    .addParam("token", "The ThreatToken contract address")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const token = await hre.ethers.getContractAt("ThreatToken", taskArgs.token);
        const balance = await token.balanceOf(taskArgs.account);
        const symbol = await token.symbol();

        console.log(`\n>™ ${symbol} Balance of ${taskArgs.account}:`);
        console.log(`   ${hre.ethers.formatEther(balance)} ${symbol}`);
    });

/**
 * Task to fund test accounts
 */
task("fund-accounts", "Fund multiple accounts with ETH")
    .addParam("amount", "Amount of ETH to send to each account")
    .addVariadicPositionalParam("recipients", "Recipient addresses")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const [sender] = await hre.ethers.getSigners();
        const amount = hre.ethers.parseEther(taskArgs.amount);

        console.log(`\n=¸ Funding accounts from ${sender.address}...`);
        console.log(`   Amount per account: ${taskArgs.amount} ETH\n`);

        for (const recipient of taskArgs.recipients) {
            const tx = await sender.sendTransaction({
                to: recipient,
                value: amount
            });
            await tx.wait();

            console.log(` Funded ${recipient}`);
        }

        console.log("\n All accounts funded!");
    });

/**
 * Task to get network info
 */
task("network-info", "Display current network information")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const network = await hre.ethers.provider.getNetwork();
        const blockNumber = await hre.ethers.provider.getBlockNumber();
        const gasPrice = await hre.ethers.provider.getFeeData();

        console.log("\n< Network Information:");
        console.log("P".repeat(60));
        console.log(`Network Name: ${network.name}`);
        console.log(`Chain ID: ${network.chainId}`);
        console.log(`Current Block: ${blockNumber}`);
        console.log(`Gas Price: ${hre.ethers.formatUnits(gasPrice.gasPrice || 0n, "gwei")} gwei`);
        console.log("P".repeat(60));
    });

/**
 * Task to register an engine
 */
task("register-engine", "Register an analysis engine")
    .addParam("reputation", "ReputationSystem contract address")
    .addParam("engine", "Engine address to register")
    .addParam("type", "Engine type (0=Human, 1=Automated, 2=Hybrid)")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const reputation = await hre.ethers.getContractAt("ReputationSystem", taskArgs.reputation);
        const engineType = parseInt(taskArgs.type);

        console.log(`\n> Registering engine: ${taskArgs.engine}`);
        console.log(`   Type: ${engineType === 0 ? "Human" : engineType === 1 ? "Automated" : "Hybrid"}`);

        const tx = await reputation.registerEngine(taskArgs.engine, engineType);
        await tx.wait();

        console.log(" Engine registered successfully!");

        const engineInfo = await reputation.getEngineInfo(taskArgs.engine);
        console.log(`\n=Ê Engine Info:`);
        console.log(`   Reputation: ${engineInfo.reputation}`);
        console.log(`   Active: ${engineInfo.isActive}`);
    });

/**
 * Task to check reputation
 */
task("check-reputation", "Check engine reputation")
    .addParam("reputation", "ReputationSystem contract address")
    .addParam("engine", "Engine address")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const reputation = await hre.ethers.getContractAt("ReputationSystem", taskArgs.reputation);

        const engineInfo = await reputation.getEngineInfo(taskArgs.engine);
        const accuracyRate = await reputation.getAccuracyRate(taskArgs.engine);

        console.log(`\n=Ê Reputation Info for ${taskArgs.engine}:`);
        console.log("P".repeat(60));
        console.log(`Reputation Score: ${engineInfo.reputation}`);
        console.log(`Total Analyses: ${engineInfo.totalAnalyses}`);
        console.log(`Correct Analyses: ${engineInfo.correctAnalyses}`);
        console.log(`Accuracy Rate: ${accuracyRate}%`);
        console.log(`Active: ${engineInfo.isActive}`);
        console.log(`Registered: ${engineInfo.isRegistered}`);
        console.log("P".repeat(60));
    });

/**
 * Task to list bounties
 */
task("list-bounties", "List all bounties")
    .addParam("manager", "BountyManager contract address")
    .addOptionalParam("limit", "Maximum number of bounties to display", "10")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const bountyManager = await hre.ethers.getContractAt("BountyManager", taskArgs.manager);
        const totalBounties = await bountyManager.getTotalBounties();
        const limit = Math.min(parseInt(taskArgs.limit), Number(totalBounties));

        console.log(`\n=Ë Bounties (showing ${limit} of ${totalBounties}):`);
        console.log("P".repeat(80));

        for (let i = 1; i <= limit; i++) {
            try {
                const bounty = await bountyManager.getBounty(i);

                const statusNames = ["Active", "Resolved", "Disputed", "Cancelled"];
                const verdictNames = ["Pending", "Malicious", "Benign"];

                console.log(`\nBounty #${i}:`);
                console.log(`  Creator: ${bounty.creator}`);
                console.log(`  Reward: ${hre.ethers.formatEther(bounty.rewardAmount)} THREAT`);
                console.log(`  Status: ${statusNames[bounty.status] || "Unknown"}`);
                console.log(`  Verdict: ${verdictNames[bounty.consensusVerdict] || "Unknown"}`);
                console.log(`  Analyses: ${bounty.analysisCount}`);
            } catch (error) {
                console.log(`\nBounty #${i}: Error fetching data`);
            }
        }

        console.log("\n" + "P".repeat(80));
    });

export {};
