// Ownership handover script.
//
// Transfers control of all Verdyx contracts from the deployer key to a
// multisig/timelock (e.g. a Gnosis Safe) as required before mainnet launch.
//
//   - BountyManager uses a custom two-step owner model: this script calls
//     transferOwnership(multisig). The multisig MUST then call acceptOwnership()
//     to complete the handover — until it does, the deployer stays the owner.
//   - ThreatToken / ReputationSystem / Governance use OpenZeppelin AccessControl:
//     this script grants DEFAULT_ADMIN_ROLE to the multisig. The multisig can
//     then manage every other role.
//
// The deployer's own roles are NOT renounced by default. Renouncing
// DEFAULT_ADMIN_ROLE is irreversible, so it is gated behind RENOUNCE_DEPLOYER=true
// and only runs after verifying the multisig already holds the role.
//
// Usage:
//   MULTISIG_ADDRESS=0xSafe... npx hardhat run scripts/setup/transfer-ownership.ts --network sepolia
//   # After the Safe has accepted BountyManager ownership and you've verified control:
//   MULTISIG_ADDRESS=0xSafe... RENOUNCE_DEPLOYER=true npx hardhat run scripts/setup/transfer-ownership.ts --network mainnet

import { ethers } from "hardhat";
import { readFileSync } from "fs";
import { join } from "path";
import type { DeploymentAddresses } from "../deploy";
import type { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";

async function loadDeploymentAddresses(
  networkName: string,
  chainId: number
): Promise<DeploymentAddresses> {
  const file = join(__dirname, "..", "..", "deployments", `${networkName}-${chainId}.json`);
  try {
    return JSON.parse(readFileSync(file, "utf8"));
  } catch (_error) {
    throw new Error(`❌ Could not load deployment addresses from ${file}. Run deploy.ts first.`);
  }
}

// Grant DEFAULT_ADMIN_ROLE to the multisig on an AccessControl contract, then
// optionally renounce the deployer's admin role.
async function handOverAccessControl(
  label: string,
  contractName: string,
  address: string,
  multisig: string,
  deployer: HardhatEthersSigner,
  renounce: boolean
) {
  const contract = await ethers.getContractAt(contractName, address, deployer);
  const adminRole = await contract.DEFAULT_ADMIN_ROLE();
  const deployerAddress = await deployer.getAddress();

  if (!(await contract.hasRole(adminRole, multisig))) {
    const tx = await contract.grantRole(adminRole, multisig);
    await tx.wait();
    console.log(`✅ ${label}: granted DEFAULT_ADMIN_ROLE to multisig`);
  } else {
    console.log(`ℹ️  ${label}: multisig already holds DEFAULT_ADMIN_ROLE`);
  }

  if (renounce) {
    if (!(await contract.hasRole(adminRole, multisig))) {
      throw new Error(`Refusing to renounce: multisig does not hold DEFAULT_ADMIN_ROLE on ${label}`);
    }
    if (await contract.hasRole(adminRole, deployerAddress)) {
      const tx = await contract.renounceRole(adminRole, deployerAddress);
      await tx.wait();
      console.log(`✅ ${label}: deployer renounced DEFAULT_ADMIN_ROLE`);
    } else {
      console.log(`ℹ️  ${label}: deployer no longer holds DEFAULT_ADMIN_ROLE`);
    }
  }
}

async function main() {
  const multisig = process.env.MULTISIG_ADDRESS;
  const renounce = process.env.RENOUNCE_DEPLOYER === "true";

  if (!multisig || !ethers.isAddress(multisig)) {
    throw new Error("❌ Set MULTISIG_ADDRESS to a valid multisig/timelock address");
  }

  const network = await ethers.provider.getNetwork();
  const [deployer] = await ethers.getSigners();
  const addrs = await loadDeploymentAddresses(network.name, Number(network.chainId));

  console.log(`📡 Network: ${network.name} (${network.chainId})`);
  console.log(`👤 Deployer: ${deployer.address}`);
  console.log(`🔐 Multisig/timelock: ${multisig}`);
  console.log(`♻️  Renounce deployer roles: ${renounce}\n`);

  // AccessControl contracts
  await handOverAccessControl("ThreatToken", "ThreatToken", addrs.threatToken, multisig, deployer, renounce);
  await handOverAccessControl("ReputationSystem", "ReputationSystem", addrs.reputationSystem, multisig, deployer, renounce);
  if (addrs.governance) {
    await handOverAccessControl("Governance", "Governance", addrs.governance, multisig, deployer, renounce);
  }

  // BountyManager two-step ownership
  const bountyManager = await ethers.getContractAt("BountyManager", addrs.bountyManager, deployer);
  const currentOwner = await bountyManager.owner();
  const pending = await bountyManager.pendingOwner();

  if (currentOwner.toLowerCase() === multisig.toLowerCase()) {
    console.log("ℹ️  BountyManager: multisig is already the owner");
  } else if (pending.toLowerCase() === multisig.toLowerCase()) {
    console.log("ℹ️  BountyManager: transfer already initiated — waiting for multisig acceptOwnership()");
  } else {
    const tx = await bountyManager.transferOwnership(multisig);
    await tx.wait();
    console.log("✅ BountyManager: ownership transfer started to multisig");
  }

  console.log("\n📋 Next steps:");
  console.log(`   1. From the multisig, call BountyManager.acceptOwnership() at ${addrs.bountyManager}`);
  console.log("   2. Verify: BountyManager.owner() == multisig");
  console.log("   3. Verify the multisig holds DEFAULT_ADMIN_ROLE on the token/reputation/governance contracts");
  console.log("   4. Once verified, re-run with RENOUNCE_DEPLOYER=true to drop the deployer's admin roles");
  console.log("\n🎉 Ownership handover step completed.");
}

if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("❌ Ownership transfer failed:");
      console.error(error);
      process.exit(1);
    });
}

export { main as transferOwnership };
