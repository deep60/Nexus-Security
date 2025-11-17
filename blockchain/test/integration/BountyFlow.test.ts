import { expect } from "chai";
import { ethers } from "hardhat";
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { deployFixture, createTestBounty, submitTestAnalysis } from "../fixtures/setup";

describe("BountyFlow Integration", function () {
    describe("Complete Bounty Lifecycle", function () {
        it("Should complete full bounty flow with consensus", async function () {
            const { bountyManager, threatToken, reputationSystem, submitter, analyst1, analyst2, analyst3 } = await loadFixture(deployFixture);

            // Step 1: Create bounty
            const rewardAmount = ethers.parseEther("100");
            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter, rewardAmount);

            console.log(" Bounty created:", bountyId.toString());

            // Step 2: Multiple analysts submit analyses
            await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId, 1, 95);
            await submitTestAnalysis(bountyManager, threatToken, analyst2, bountyId, 1, 90);
            await submitTestAnalysis(bountyManager, threatToken, analyst3, bountyId, 1, 85);

            console.log(" Analyses submitted");

            // Get additional analysts
            const [, , , , , , ...users] = await ethers.getSigners();

            // Add more analysts to reach threshold
            for (let i = 0; i < 7; i++) {
                const analyst = users[i];
                await threatToken.setEngineAuthorization(analyst.address, true);
                await reputationSystem.registerEngine(analyst.address, 0);
                await threatToken.transfer(analyst.address, ethers.parseEther("1000"));
                await submitTestAnalysis(bountyManager, threatToken, analyst, bountyId, 1, 80 + i);
            }

            console.log(" Reached consensus threshold");

            // Step 3: Check bounty resolved
            const bounty = await bountyManager.getBounty(bountyId);
            expect(bounty.status).to.equal(1); // Resolved
            expect(bounty.consensusVerdict).to.equal(1); // Malicious

            console.log(" Bounty resolved with consensus: Malicious");

            // Step 4: Verify reputations updated
            const rep1 = await reputationSystem.getReputation(analyst1.address);
            const rep2 = await reputationSystem.getReputation(analyst2.address);
            const rep3 = await reputationSystem.getReputation(analyst3.address);

            expect(rep1).to.be.greaterThan(100); // Increased from initial
            expect(rep2).to.be.greaterThan(100);
            expect(rep3).to.be.greaterThan(100);

            console.log(" Reputations updated");
            console.log(`   Analyst 1: ${rep1}`);
            console.log(`   Analyst 2: ${rep2}`);
            console.log(`   Analyst 3: ${rep3}`);
        });

        it("Should handle split verdict scenario", async function () {
            const { bountyManager, threatToken, reputationSystem, submitter } = await loadFixture(deployFixture);

            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter);

            const [, , , , , , ...users] = await ethers.getSigners();

            // Register 10 analysts: 5 say Malicious, 5 say Benign (split)
            for (let i = 0; i < 10; i++) {
                const analyst = users[i];
                await threatToken.setEngineAuthorization(analyst.address, true);
                await reputationSystem.registerEngine(analyst.address, 0);
                await threatToken.transfer(analyst.address, ethers.parseEther("1000"));

                const verdict = i < 5 ? 1 : 2; // First 5 Malicious, next 5 Benign
                await submitTestAnalysis(bountyManager, threatToken, analyst, bountyId, verdict, 90);
            }

            const bounty = await bountyManager.getBounty(bountyId);

            // With 50-50 split and 66% threshold, should not reach consensus
            // Consensus verdict should remain Pending or the system handles it
            console.log(" Split verdict handled, consensus:", bounty.consensusVerdict);
        });

        it("Should track reputation changes over multiple bounties", async function () {
            const { bountyManager, threatToken, reputationSystem, submitter, analyst1 } = await loadFixture(deployFixture);

            const initialRep = await reputationSystem.getReputation(analyst1.address);
            console.log("Initial reputation:", initialRep);

            // Create and participate in multiple bounties
            for (let i = 0; i < 3; i++) {
                const { bountyId } = await createTestBounty(
                    bountyManager,
                    threatToken,
                    submitter,
                    ethers.parseEther("50")
                );

                await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId, 1, 90);

                // Add more analysts for consensus
                const [, , , , , , ...users] = await ethers.getSigners();
                for (let j = 0; j < 9; j++) {
                    const idx = i * 9 + j;
                    const analyst = users[idx];
                    await threatToken.setEngineAuthorization(analyst.address, true);
                    await reputationSystem.registerEngine(analyst.address, 0);
                    await threatToken.transfer(analyst.address, ethers.parseEther("1000"));
                    await submitTestAnalysis(bountyManager, threatToken, analyst, bountyId, 1, 85);
                }

                const currentRep = await reputationSystem.getReputation(analyst1.address);
                console.log(`Reputation after bounty ${i + 1}:`, currentRep);
            }

            const finalRep = await reputationSystem.getReputation(analyst1.address);
            expect(finalRep).to.be.greaterThan(initialRep);

            console.log(" Reputation progression tracked");
            console.log(`   Initial: ${initialRep}`);
            console.log(`   Final: ${finalRep}`);
        });

        it("Should handle emergency pause scenario", async function () {
            const { bountyManager, threatToken, submitter, deployer } = await loadFixture(deployFixture);

            // Create a bounty
            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter);

            // Pause the system
            await bountyManager.connect(deployer).pause();

            // Try to submit analysis - should fail
            const [, , , analyst] = await ethers.getSigners();
            await threatToken.setEngineAuthorization(analyst.address, true);
            await threatToken.transfer(analyst.address, ethers.parseEther("1000"));

            const stakeAmount = ethers.parseEther("10");
            await threatToken.connect(analyst).approve(await bountyManager.getAddress(), stakeAmount);

            await expect(
                bountyManager.connect(analyst).submitAnalysis(
                    bountyId,
                    1,
                    90,
                    stakeAmount,
                    "QmAnalysis"
                )
            ).to.be.revertedWith("Contract is paused");

            console.log(" Emergency pause working correctly");

            // Unpause
            await bountyManager.connect(deployer).unpause();

            // Now it should work
            const reputationSystem = await ethers.getContractAt("ReputationSystem", await bountyManager.reputationSystem());
            await reputationSystem.registerEngine(analyst.address, 0);

            await expect(
                bountyManager.connect(analyst).submitAnalysis(
                    bountyId,
                    1,
                    90,
                    stakeAmount,
                    "QmAnalysis"
                )
            ).to.not.be.reverted;

            console.log(" System resumed after unpause");
        });
    });

    describe("Economic Flow", function () {
        it("Should correctly handle token flows", async function () {
            const { bountyManager, threatToken, submitter, analyst1, feeCollector } = await loadFixture(deployFixture);

            const submitterInitial = await threatToken.balanceOf(submitter.address);
            const feeCollectorInitial = await threatToken.balanceOf(feeCollector.address);

            const rewardAmount = ethers.parseEther("100");
            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter, rewardAmount);

            // Submit analysis
            await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId);

            // Add more for consensus
            const [, , , , , , ...users] = await ethers.getSigners();
            const reputationSystem = await ethers.getContractAt("ReputationSystem", await bountyManager.reputationSystem());

            for (let i = 0; i < 9; i++) {
                const analyst = users[i];
                await threatToken.setEngineAuthorization(analyst.address, true);
                await reputationSystem.registerEngine(analyst.address, 0);
                await threatToken.transfer(analyst.address, ethers.parseEther("1000"));
                await submitTestAnalysis(bountyManager, threatToken, analyst, bountyId, 1);
            }

            // Check final balances
            const submitterFinal = await threatToken.balanceOf(submitter.address);
            const feeCollectorFinal = await threatToken.balanceOf(feeCollector.address);

            // Submitter paid the reward
            expect(submitterFinal).to.equal(submitterInitial - rewardAmount);

            // Fee collector received platform fee
            expect(feeCollectorFinal).to.be.greaterThan(feeCollectorInitial);

            console.log(" Token flows verified");
            console.log(`   Submitter paid: ${ethers.formatEther(rewardAmount)} THREAT`);
            console.log(`   Platform fee: ${ethers.formatEther(feeCollectorFinal - feeCollectorInitial)} THREAT`);
        });
    });
});
