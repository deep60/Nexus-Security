import { expect } from "chai";
import { ethers } from "hardhat";
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { deployFixture, createTestBounty, submitTestAnalysis, advanceTime } from "../fixtures/setup";

describe("BountyManager", function () {
    describe("Bounty Creation", function () {
        it("Should create a bounty successfully", async function () {
            const { bountyManager, threatToken, submitter } = await loadFixture(deployFixture);

            const rewardAmount = ethers.parseEther("100");
            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter, rewardAmount);

            const bounty = await bountyManager.getBounty(bountyId);
            expect(bounty.creator).to.equal(submitter.address);
            expect(bounty.rewardAmount).to.equal(rewardAmount);
            expect(bounty.status).to.equal(0); // Active
        });

        it("Should fail with insufficient token approval", async function () {
            const { bountyManager, submitter } = await loadFixture(deployFixture);

            const deadline = Math.floor(Date.now() / 1000) + 86400;
            await expect(
                bountyManager.connect(submitter).createBounty(
                    "QmTest",
                    0,
                    ethers.parseEther("100"),
                    deadline,
                    "Test"
                )
            ).to.be.revertedWith("Token transfer failed");
        });

        it("Should reject deadline that is too soon", async function () {
            const { bountyManager, threatToken, submitter } = await loadFixture(deployFixture);

            const rewardAmount = ethers.parseEther("100");
            await threatToken.connect(submitter).approve(await bountyManager.getAddress(), rewardAmount);

            const deadline = Math.floor(Date.now() / 1000) + 3000; // Less than 1 hour
            await expect(
                bountyManager.connect(submitter).createBounty(
                    "QmTest",
                    0,
                    rewardAmount,
                    deadline,
                    "Test"
                )
            ).to.be.revertedWith("Deadline too soon");
        });
    });

    describe("Analysis Submission", function () {
        it("Should allow analyst to submit analysis", async function () {
            const { bountyManager, threatToken, submitter, analyst1 } = await loadFixture(deployFixture);

            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter);

            await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId);

            const analysis = await bountyManager.getAnalysis(bountyId, analyst1.address);
            expect(analysis.analyst).to.equal(analyst1.address);
            expect(analysis.verdict).to.equal(1); // Malicious
        });

        it("Should enforce minimum stake", async function () {
            const { bountyManager, threatToken, submitter, analyst1 } = await loadFixture(deployFixture);

            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter);

            const lowStake = ethers.parseEther("5"); // Below minimum
            await threatToken.connect(analyst1).approve(await bountyManager.getAddress(), lowStake);

            await expect(
                bountyManager.connect(analyst1).submitAnalysis(
                    bountyId,
                    1,
                    90,
                    lowStake,
                    "QmAnalysis"
                )
            ).to.be.revertedWith("Insufficient stake");
        });

        it("Should prevent duplicate submissions", async function () {
            const { bountyManager, threatToken, submitter, analyst1 } = await loadFixture(deployFixture);

            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter);

            await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId);

            await expect(
                submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId)
            ).to.be.revertedWith("Already submitted");
        });
    });

    describe("Consensus and Resolution", function () {
        it("Should reach consensus with majority agreement", async function () {
            const { bountyManager, threatToken, submitter, analyst1, analyst2, analyst3 } = await loadFixture(deployFixture);

            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter);

            // All analysts agree: Malicious
            await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId, 1); // Malicious
            await submitTestAnalysis(bountyManager, threatToken, analyst2, bountyId, 1); // Malicious
            await submitTestAnalysis(bountyManager, threatToken, analyst3, bountyId, 1); // Malicious

            // Advance time past deadline
            await advanceTime(86400 + 1);

            // Resolve bounty
            await bountyManager.resolveBounty(bountyId);

            const bounty = await bountyManager.getBounty(bountyId);
            expect(bounty.consensusVerdict).to.equal(1); // Malicious
            expect(bounty.status).to.equal(1); // Resolved
        });

        it("Should distribute rewards to correct analysts", async function () {
            const { bountyManager, threatToken, submitter, analyst1, analyst2 } = await loadFixture(deployFixture);

            const rewardAmount = ethers.parseEther("100");
            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter, rewardAmount);

            const initialBalance1 = await threatToken.balanceOf(analyst1.address);
            const initialBalance2 = await threatToken.balanceOf(analyst2.address);

            // Both agree: Malicious
            await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId, 1);
            await submitTestAnalysis(bountyManager, threatToken, analyst2, bountyId, 1);

            // Need more submissions for auto-resolve
            for (let i = 3; i <= 10; i++) {
                const [, , , , , , ...users] = await ethers.getSigners();
                const analyst = users[i];

                // Authorize and register
                await threatToken.setEngineAuthorization(analyst.address, true);
                await (await ethers.getContractAt("ReputationSystem", await bountyManager.reputationSystem())).registerEngine(analyst.address, 0);

                // Fund analyst
                await threatToken.transfer(analyst.address, ethers.parseEther("1000"));

                await submitTestAnalysis(bountyManager, threatToken, analyst, bountyId, 1);
            }

            // Check balances increased (stake returned + reward)
            const finalBalance1 = await threatToken.balanceOf(analyst1.address);
            const finalBalance2 = await threatToken.balanceOf(analyst2.address);

            expect(finalBalance1).to.be.greaterThan(initialBalance1);
            expect(finalBalance2).to.be.greaterThan(initialBalance2);
        });

        it("Should slash stakes for incorrect analysts", async function () {
            const { bountyManager, threatToken, submitter, analyst1, analyst2, analyst3 } = await loadFixture(deployFixture);

            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter);

            const initialBalance = await threatToken.balanceOf(analyst3.address);

            // Majority says Malicious, analyst3 says Benign
            await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId, 1); // Malicious
            await submitTestAnalysis(bountyManager, threatToken, analyst2, bountyId, 1); // Malicious
            await submitTestAnalysis(bountyManager, threatToken, analyst3, bountyId, 2, 90); // Benign - wrong!

            // Add more correct submissions
            for (let i = 3; i <= 10; i++) {
                const [, , , , , , ...users] = await ethers.getSigners();
                const analyst = users[i];

                await threatToken.setEngineAuthorization(analyst.address, true);
                await (await ethers.getContractAt("ReputationSystem", await bountyManager.reputationSystem())).registerEngine(analyst.address, 0);
                await threatToken.transfer(analyst.address, ethers.parseEther("1000"));

                await submitTestAnalysis(bountyManager, threatToken, analyst, bountyId, 1); // Malicious
            }

            // Analyst3's balance should be less (stake slashed)
            const finalBalance = await threatToken.balanceOf(analyst3.address);
            expect(finalBalance).to.be.lessThan(initialBalance);
        });
    });

    describe("View Functions", function () {
        it("Should return bounty analysts", async function () {
            const { bountyManager, threatToken, submitter, analyst1, analyst2 } = await loadFixture(deployFixture);

            const { bountyId } = await createTestBounty(bountyManager, threatToken, submitter);

            await submitTestAnalysis(bountyManager, threatToken, analyst1, bountyId);
            await submitTestAnalysis(bountyManager, threatToken, analyst2, bountyId);

            const analysts = await bountyManager.getBountyAnalysts(bountyId);
            expect(analysts.length).to.equal(2);
            expect(analysts).to.include(analyst1.address);
            expect(analysts).to.include(analyst2.address);
        });

        it("Should return user bounties", async function () {
            const { bountyManager, threatToken, submitter } = await loadFixture(deployFixture);

            await createTestBounty(bountyManager, threatToken, submitter);
            await createTestBounty(bountyManager, threatToken, submitter);

            const userBounties = await bountyManager.getUserBounties(submitter.address);
            expect(userBounties.length).to.equal(2);
        });

        it("Should return total bounties", async function () {
            const { bountyManager, threatToken, submitter } = await loadFixture(deployFixture);

            const initialCount = await bountyManager.getTotalBounties();

            await createTestBounty(bountyManager, threatToken, submitter);
            await createTestBounty(bountyManager, threatToken, submitter);

            const finalCount = await bountyManager.getTotalBounties();
            expect(finalCount).to.equal(initialCount + 2n);
        });
    });

    describe("Admin Functions", function () {
        it("Should allow owner to pause", async function () {
            const { bountyManager, deployer } = await loadFixture(deployFixture);

            await bountyManager.connect(deployer).pause();
            // Note: There's no public paused() getter in the contract, would need to try creating a bounty
        });

        it("Should allow owner to set fee collector", async function () {
            const { bountyManager, deployer, analyst1 } = await loadFixture(deployFixture);

            await expect(
                bountyManager.connect(deployer).setFeeCollector(analyst1.address)
            ).to.not.be.reverted;
        });
    });
});
