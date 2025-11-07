# Smart Contract ABIs

This directory contains the Application Binary Interfaces (ABIs) for the deployed smart contracts.

## Usage

After compiling and deploying your smart contracts, copy the ABI files here for use by the backend services.

### Example Structure:
```
abis/
├── BountyManager.json
├── ReputationToken.json
├── StakeEscrow.json
└── PaymentDistributor.json
```

### Generating ABIs:

```bash
# After compiling contracts with Hardhat
npx hardhat compile

# ABIs are generated in artifacts/contracts/
# Copy them here for backend use
cp artifacts/contracts/BountyManager.sol/BountyManager.json abis/
cp artifacts/contracts/ReputationToken.sol/ReputationToken.json abis/
# ... etc
```

## Backend Integration

The backend services (particularly payment-service) use these ABIs to interact with the deployed smart contracts.

Update the contract addresses in `deployed-addresses.json` after deployment.
