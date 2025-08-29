//  Helper functions for:

// Gas estimation and cost calculation
// Network connectivity validation
// Deployment artifact management
// Configuration loading and validation



import { ethers } from "hardhat";
import { writeFileSync, readFileSync, existsSync, mkdirSync } from "fs";
import { join } from "path";

// ================================
// TYPES AND INTERFACES
// ================================

export interface NetworkConfig {
  name: string;
  chainId: number;
  rpcUrl?: string;
  blockExplorer?: string;
  gasPrice?: string;
  gasLimit?: number;
  nativeCurrency: {
    name: string;
    symbol: string;
    decimals: number;
  };
}

export interface DeploymentAddresses {
  threatToken: string;
  reputationSystem: string;
  bountyManager: string;
  deployer: string;
  network: string;
  chainId: number;
  blockNumber: number;
  timestamp: number;
}

export interface ContractConfig {
  name: string;
  symbol: string;
  initialSupply: string;
}

export interface BountyManagerConfig {
  minimumStake: string;
  analysisTimeout: number;
  consensusThreshold: number;
  rewardPercentage: number;
  slashPercentage: number;
}

export interface ReputationSystemConfig {
  initialReputation: number;
  maxBonus: number;
  decayRate: number;
}

export interface SetupConfig {
  adminAddresses: string[];
  moderatorAddresses: string[];
  engineAddresses: string[];
  rewardPoolAmount: string;
  engineRegistrationFee: string;
}

export interface DeploymentConfig {
  network: NetworkConfig;
  contracts: {
    threatToken: ContractConfig;
    bountyManager: BountyManagerConfig;
    reputationSystem: ReputationSystemConfig;
  };
  setup: SetupConfig;
}

// ================================
// NETWORK CONFIGURATIONS
// ================================

export const NETWORK_CONFIGS: Record<string, NetworkConfig> = {
  mainnet: {
    name: "mainnet",
    chainId: 1,
    blockExplorer: "https://etherscan.io",
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18
    }
  },
  goerli: {
    name: "goerli",
    chainId: 5,
    blockExplorer: "https://goerli.etherscan.io",
    nativeCurrency: {
      name: "Goerli Ether",
      symbol: "ETH",
      decimals: 18
    }
  },
  sepolia: {
    name: "sepolia",
    chainId: 11155111,
    blockExplorer: "https://sepolia.etherscan.io",
    nativeCurrency: {
      name: "Sepolia Ether",
      symbol: "ETH",
      decimals: 18
    }
  },
  polygon: {
    name: "polygon",
    chainId: 137,
    blockExplorer: "https://polygonscan.com",
    nativeCurrency: {
      name: "Polygon",
      symbol: "MATIC",
      decimals: 18
    }
  },
  mumbai: {
    name: "mumbai",
    chainId: 80001,
    blockExplorer: "https://mumbai.polygonscan.com",
    nativeCurrency: {
      name: "Mumbai MATIC",
      symbol: "MATIC",
      decimals: 18
    }
  },
  bsc: {
    name: "bsc",
    chainId: 56,
    blockExplorer: "https://bscscan.com",
    nativeCurrency: {
      name: "Binance Coin",
      symbol: "BNB",
      decimals: 18
    }
  },
  bscTestnet: {
    name: "bscTestnet",
    chainId: 97,
    blockExplorer: "https://testnet.bscscan.com",
    nativeCurrency: {
      name: "Test BNB",
      symbol: "tBNB",
      decimals: 18
    }
  },
  hardhat: {
    name: "hardhat",
    chainId: 31337,
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18
    }
  },
  localhost: {
    name: "localhost",
    chainId: 31337,
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18
    }
  }
} as const;

// ================================
// DEFAULT CONFIGURATIONS
// ================================

export const DEFAULT_DEPLOYMENT_CONFIG: DeploymentConfig = {
  network: {
    name: "localhost",
    chainId: 31337,
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18
    }
  },
  contracts: {
    threatToken: {
      name: "ThreatToken",
      symbol: "THREAT",
      initialSupply: "1000000" // 1M tokens
    },
    bountyManager: {
      minimumStake: "10", // 10 THREAT tokens
      analysisTimeout: 3600, // 1 hour
      consensusThreshold: 70, // 70%
      rewardPercentage: 80, // 80% to winners
      slashPercentage: 20 // 20% penalty
    },
    reputationSystem: {
      initialReputation: 100,
      maxBonus: 50,
      decayRate: 1 // 1% per month
    }
  },
  setup: {
    adminAddresses: [],
    moderatorAddresses: [],
    engineAddresses: [],
    rewardPoolAmount: "100000", // 100K tokens
    engineRegistrationFee: "100" // 100 tokens
  }
};

// ================================
// UTILITY FUNCTIONS
// ================================

export function loadConfig(configPath?: string): DeploymentConfig {
  const defaultConfigPath = join(__dirname, "..", "config", "deployment.json");
  const filePath = configPath || defaultConfigPath;
  
  // If no config file exists, return default config
  if (!existsSync(filePath)) {
    console.log(`⚠️  Config file not found: ${filePath}`);
    console.log("🔧 Using default configuration");
    return DEFAULT_DEPLOYMENT_CONFIG;
  }
  
  try {
    const configData = readFileSync(filePath, 'utf8');
    const config = JSON.parse(configData) as DeploymentConfig;
    
    // Merge with default config to ensure all fields are present
    return {
      ...DEFAULT_DEPLOYMENT_CONFIG,
      ...config,
      contracts: {
        ...DEFAULT_DEPLOYMENT_CONFIG.contracts,
        ...config.contracts,
        threatToken: {
          ...DEFAULT_DEPLOYMENT_CONFIG.contracts.threatToken,
          ...config.contracts?.threatToken
        },
        bountyManager: {
          ...DEFAULT_DEPLOYMENT_CONFIG.contracts.bountyManager,
          ...config.contracts?.bountyManager
        },
        reputationSystem: {
          ...DEFAULT_DEPLOYMENT_CONFIG.contracts.reputationSystem,
          ...config.contracts?.reputationSystem
        }
      },
      setup: {
        ...DEFAULT_DEPLOYMENT_CONFIG.setup,
        ...config.setup
      }
    };
  } catch (error) {
    console.error(`❌ Failed to load config from ${filePath}:`, error);
    console.log("🔧 Using default configuration");
    return DEFAULT_DEPLOYMENT_CONFIG;
  }
}

export function saveDeploymentArtifacts(
  networkName: string,
  chainId: number,
  deploymentData: DeploymentAddresses
): void {
  const deploymentsDir = join(__dirname, "..", "deployments");
  const artifactsDir = join(__dirname, "..", "artifacts", "deployed");
  
  // Ensure directories exist
  [deploymentsDir, artifactsDir].forEach(dir => {
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
    }
  });
  
  // Save deployment addresses
  const deploymentFile = join(deploymentsDir, `${networkName}-${chainId}.json`);
  writeFileSync(deploymentFile, JSON.stringify(deploymentData, null, 2));
  
  try {
    // Save deployment artifacts for frontend consumption
    const frontendArtifacts = {
      network: networkName,
      chainId: chainId,
      timestamp: deploymentData.timestamp,
      contracts: {
        ThreatToken: {
          address: deploymentData.threatToken,
          abi: getContractABI("ThreatToken")
        },
        ReputationSystem: {
          address: deploymentData.reputationSystem,
          abi: getContractABI("ReputationSystem")
        },
        BountyManager: {
          address: deploymentData.bountyManager,
          abi: getContractABI("BountyManager")
        }
      }
    };
    
    const artifactsFile = join(artifactsDir, `${networkName}-${chainId}.json`);
    writeFileSync(artifactsFile, JSON.stringify(frontendArtifacts, null, 2));
    
    console.log(`📁 Saved deployment artifacts to ${artifactsFile}`);
  } catch (error) {
    console.warn(`⚠️  Could not save frontend artifacts: ${error}`);
    console.log("📁 Basic deployment addresses saved successfully");
  }
}

function getContractABI(contractName: string): any[] {
  try {
    const artifactPath = join(__dirname, "..", "artifacts", "contracts", `${contractName}.sol`, `${contractName}.json`);
    if (existsSync(artifactPath)) {
      const artifact = JSON.parse(readFileSync(artifactPath, 'utf8'));
      return artifact.abi;
    }
  } catch (error) {
    console.warn(`⚠️  Could not load ABI for ${contractName}`);
  }
  return [];
}

export async function estimateGasCosts(): Promise<void> {
  console.log("⛽ Estimating deployment gas costs...\n");
  
  try {
    const [deployer] = await ethers.getSigners();
    const feeData = await deployer.provider.getFeeData();
    
    const gasPrice = feeData.gasPrice || ethers.parseUnits("20", "gwei"); // Default 20 gwei
    console.log(`Current gas price: ${ethers.formatUnits(gasPrice, 'gwei')} gwei`);
    
    // Contract deployment estimates
    const contracts = [
      { 
        name: "ThreatToken", 
        factory: "ThreatToken", 
        args: ["ThreatToken", "THREAT", ethers.parseEther("1000000")],
        estimatedGas: 2000000n
      },
      { 
        name: "ReputationSystem", 
        factory: "ReputationSystem", 
        args: [],
        estimatedGas: 3000000n
      },
      { 
        name: "BountyManager", 
        factory: "BountyManager", 
        args: [ethers.ZeroAddress, ethers.ZeroAddress], // Placeholder addresses
        estimatedGas: 4000000n
      }
    ];
    
    let totalEstimatedGas = 0n;
    
    for (const contract of contracts) {
      try {
        // Try to get actual gas estimate
        const factory = await ethers.getContractFactory(contract.factory);
        const deployTransaction = await factory.getDeployTransaction(...contract.args);
        const estimatedGas = deployTransaction.gasLimit || contract.estimatedGas;
        
        const gasCost = estimatedGas * gasPrice;
        
        console.log(`${contract.name}:`);
        console.log(`  Estimated gas: ${estimatedGas.toString()}`);
        console.log(`  Estimated cost: ${ethers.formatEther(gasCost)} ETH`);
        
        totalEstimatedGas += estimatedGas;
      } catch (error) {
        // Use fallback estimate if contract compilation failed
        const gasCost = contract.estimatedGas * gasPrice;
        
        console.log(`${contract.name} (estimated):`);
        console.log(`  Estimated gas: ${contract.estimatedGas.toString()}`);
        console.log(`  Estimated cost: ${ethers.formatEther(gasCost)} ETH`);
        
        totalEstimatedGas += contract.estimatedGas;
      }
    }
    
    const totalCost = totalEstimatedGas * gasPrice;
    const recommendedBalance = totalCost * 120n / 100n; // 20% buffer
    
    console.log(`\nTotal estimated gas: ${totalEstimatedGas.toString()}`);
    console.log(`Total estimated cost: ${ethers.formatEther(totalCost)} ETH`);
    console.log(`Recommended balance: ${ethers.formatEther(recommendedBalance)} ETH (20% buffer)\n`);
  } catch (error) {
    console.error("❌ Failed to estimate gas costs:", error);
  }
}

export async function checkNetworkConnection(): Promise<boolean> {
  try {
    const network = await ethers.provider.getNetwork();
    const blockNumber = await ethers.provider.getBlockNumber();
    
    console.log(`✅ Connected to ${network.name} (Chain ID: ${network.chainId})`);
    console.log(`📦 Latest block: ${blockNumber}`);
    return true;
  } catch (error) {
    console.error("❌ Failed to connect to network:");
    console.error(error);
    return false;
  }
}

export async function validateDeployerAccount(): Promise<boolean> {
  try {
    const [deployer] = await ethers.getSigners();
    const balance = await deployer.provider.getBalance(deployer.address);
    
    console.log(`👤 Deployer: ${deployer.address}`);
    console.log(`💰 Balance: ${ethers.formatEther(balance)} ETH`);
    
    if (balance < ethers.parseEther("0.01")) {
      console.warn("⚠️  Low balance detected. May not be sufficient for deployment.");
      return false;
    }
    
    return true;
  } catch (error) {
    console.error("❌ Failed to validate deployer account:");
    console.error(error);
    return false;
  }
}

export function getBlockExplorerUrl(networkName: string, address: string): string | null {
  const networkConfig = NETWORK_CONFIGS[networkName];
  if (!networkConfig?.blockExplorer) {
    return null;
  }
  return `${networkConfig.blockExplorer}/address/${address}`;
}

export function getBlockExplorerUrls(networkName: string): string[] {
  const networkConfig = NETWORK_CONFIGS[networkName];
  return networkConfig?.blockExplorer ? [networkConfig.blockExplorer] : [];
}

export function generateDeploymentReport(deploymentData: DeploymentAddresses): string {
  const networkConfig = NETWORK_CONFIGS[deploymentData.network];
  const blockExplorerUrl = networkConfig?.blockExplorer;
  
  const report = `
# Nexus-Security Deployment Report

**Network:** ${deploymentData.network} (Chain ID: ${deploymentData.chainId})
**Deployer:** ${deploymentData.deployer}
**Block Number:** ${deploymentData.blockNumber}
**Timestamp:** ${new Date(deploymentData.timestamp * 1000).toISOString()}

## Contract Addresses

| Contract | Address | Block Explorer |
|----------|---------|----------------|
| ThreatToken | \`${deploymentData.threatToken}\` | ${blockExplorerUrl ? `[View](${blockExplorerUrl}/address/${deploymentData.threatToken})` : 'N/A'} |
| ReputationSystem | \`${deploymentData.reputationSystem}\` | ${blockExplorerUrl ? `[View](${blockExplorerUrl}/address/${deploymentData.reputationSystem})` : 'N/A'} |
| BountyManager | \`${deploymentData.bountyManager}\` | ${blockExplorerUrl ? `[View](${blockExplorerUrl}/address/${deploymentData.bountyManager})` : 'N/A'} |

## Verification Commands

\`\`\`bash
# ThreatToken
npx hardhat verify --network ${deploymentData.network} ${deploymentData.threatToken} "ThreatToken" "THREAT" "1000000000000000000000000"

# ReputationSystem  
npx hardhat verify --network ${deploymentData.network} ${deploymentData.reputationSystem}

# BountyManager
npx hardhat verify --network ${deploymentData.network} ${deploymentData.bountyManager} ${deploymentData.threatToken} ${deploymentData.reputationSystem}
\`\`\`

## Next Steps

1. ✅ Verify contracts on block explorer
2. 🔧 Run setup script to configure initial parameters
3. 🤖 Register initial analysis engines
4. 🎨 Update frontend configuration with new addresses
5. 🧪 Test system functionality

---
*Generated on: ${new Date().toISOString()}*
`;

  return report;
}

export function saveDeploymentReport(deploymentData: DeploymentAddresses): void {
  const report = generateDeploymentReport(deploymentData);
  const reportsDir = join(__dirname, "..", "reports");
  
  if (!existsSync(reportsDir)) {
    mkdirSync(reportsDir, { recursive: true });
  }
  
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const reportFile = join(reportsDir, `deployment-report-${deploymentData.network}-${timestamp}.md`);
  
  writeFileSync(reportFile, report);
  console.log(`📄 Deployment report saved to: ${reportFile}`);
}

export function validateNetworkConfig(networkName: string): NetworkConfig | null {
  const config = NETWORK_CONFIGS[networkName];
  if (!config) {
    console.error(`❌ Unknown network: ${networkName}`);
    console.log(`Available networks: ${Object.keys(NETWORK_CONFIGS).join(', ')}`);
    return null;
  }
  return config;
}

export function isTestNetwork(networkName: string): boolean {
  const testNetworks = ['goerli', 'sepolia', 'mumbai', 'bscTestnet', 'hardhat', 'localhost'];
  return testNetworks.includes(networkName);
}

export function requireConfirmation(networkName: string): boolean {
  if (isTestNetwork(networkName)) {
    return false; // No confirmation needed for testnets
  }
  
  const mainNets = ['mainnet', 'polygon', 'bsc'];
  return mainNets.includes(networkName);
}

// Export all functions and types
export default {
  loadConfig,
  saveDeploymentArtifacts,
  estimateGasCosts,
  checkNetworkConnection,
  validateDeployerAccount,
  getBlockExplorerUrl,
  getBlockExplorerUrls,
  generateDeploymentReport,
  saveDeploymentReport,
  validateNetworkConfig,
  isTestNetwork,
  requireConfirmation,
  NETWORK_CONFIGS,
  DEFAULT_DEPLOYMENT_CONFIG,
};