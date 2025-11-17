import { ethers } from "hardhat";
import { ThreatToken, ReputationSystem, BountyManager } from "../../typechain-types";

export interface TestFixture {
    threatToken: ThreatToken;
    reputationSystem: ReputationSystem;
    bountyManager: BountyManager;
    deployer: any;
    feeCollector: any;
    analyst1: any;
    analyst2: any;
    analyst3: any;
    submitter: any;
    users: any[];
}

/**
 * Deploy all contracts for testing
 */
export async function deployFixture(): Promise<TestFixture> {
    const [deployer, feeCollector, analyst1, analyst2, analyst3, submitter, ...users] = await ethers.getSigners();

    // Deploy ThreatToken
    const ThreatToken = await ethers.getContractFactory("ThreatToken");
    const threatToken = await ThreatToken.deploy(deployer.address);
    await threatToken.waitForDeployment();

    // Deploy ReputationSystem
    const ReputationSystem = await ethers.getContractFactory("ReputationSystem");
    const reputationSystem = await ReputationSystem.deploy();
    await reputationSystem.waitForDeployment();

    // Deploy BountyManager
    const BountyManager = await ethers.getContractFactory("BountyManager");
    const bountyManager = await BountyManager.deploy(
        await threatToken.getAddress(),
        await reputationSystem.getAddress(),
        feeCollector.address
    );
    await bountyManager.waitForDeployment();

    // Setup roles
    const BOUNTY_MANAGER_ROLE = await threatToken.BOUNTY_MANAGER_ROLE();
    await threatToken.grantRole(BOUNTY_MANAGER_ROLE, await bountyManager.getAddress());

    const REPUTATION_MANAGER_ROLE = await threatToken.REPUTATION_MANAGER_ROLE();
    await threatToken.grantRole(REPUTATION_MANAGER_ROLE, await reputationSystem.getAddress());

    const BOUNTY_MANAGER_ROLE_REP = await reputationSystem.BOUNTY_MANAGER_ROLE();
    await reputationSystem.grantRole(BOUNTY_MANAGER_ROLE_REP, await bountyManager.getAddress());

    // Authorize analysts
    await threatToken.setEngineAuthorization(analyst1.address, true);
    await threatToken.setEngineAuthorization(analyst2.address, true);
    await threatToken.setEngineAuthorization(analyst3.address, true);

    // Register analysts
    await reputationSystem.registerEngine(analyst1.address, 0);
    await reputationSystem.registerEngine(analyst2.address, 0);
    await reputationSystem.registerEngine(analyst3.address, 1);

    // Fund analysts and submitter
    const fundAmount = ethers.parseEther("10000");
    await threatToken.transfer(analyst1.address, fundAmount);
    await threatToken.transfer(analyst2.address, fundAmount);
    await threatToken.transfer(analyst3.address, fundAmount);
    await threatToken.transfer(submitter.address, fundAmount);

    return {
        threatToken: threatToken as unknown as ThreatToken,
        reputationSystem: reputationSystem as unknown as ReputationSystem,
        bountyManager: bountyManager as unknown as BountyManager,
        deployer,
        feeCollector,
        analyst1,
        analyst2,
        analyst3,
        submitter,
        users
    };
}

/**
 * Create a sample bounty for testing
 */
export async function createTestBounty(
    bountyManager: BountyManager,
    threatToken: ThreatToken,
    creator: any,
    rewardAmount: bigint = ethers.parseEther("100")
) {
    // Approve tokens
    await threatToken.connect(creator).approve(await bountyManager.getAddress(), rewardAmount);

    // Create bounty
    const deadline = Math.floor(Date.now() / 1000) + 86400; // 24 hours from now
    const tx = await bountyManager.connect(creator).createBounty(
        "QmTest123456789",
        0, // File type
        rewardAmount,
        deadline,
        "Test malware sample"
    );

    const receipt = await tx.wait();
    const event = receipt?.logs.find((log: any) => {
        try {
            return bountyManager.interface.parseLog(log)?.name === "BountyCreated";
        } catch {
            return false;
        }
    });

    const bountyId = event ? bountyManager.interface.parseLog(event)?.args[0] : 1n;

    return { bountyId, receipt };
}

/**
 * Submit analysis for testing
 */
export async function submitTestAnalysis(
    bountyManager: BountyManager,
    threatToken: ThreatToken,
    analyst: any,
    bountyId: bigint,
    verdict: number = 1, // 1 = Malicious
    confidence: number = 90,
    stakeAmount: bigint = ethers.parseEther("10")
) {
    // Approve stake
    await threatToken.connect(analyst).approve(await bountyManager.getAddress(), stakeAmount);

    // Submit analysis
    const tx = await bountyManager.connect(analyst).submitAnalysis(
        bountyId,
        verdict,
        confidence,
        stakeAmount,
        "QmAnalysis123"
    );

    return await tx.wait();
}

/**
 * Advance time in the blockchain
 */
export async function advanceTime(seconds: number) {
    await ethers.provider.send("evm_increaseTime", [seconds]);
    await ethers.provider.send("evm_mine", []);
}

/**
 * Advance to a specific timestamp
 */
export async function advanceToTimestamp(timestamp: number) {
    const currentBlock = await ethers.provider.getBlock("latest");
    const currentTime = currentBlock?.timestamp || 0;
    const diff = timestamp - currentTime;

    if (diff > 0) {
        await advanceTime(diff);
    }
}
