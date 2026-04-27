#!/bin/bash
# Deploy Smart Contracts to Local Hardhat Network
# This script deploys all contracts and updates configuration files

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Verdyx Local Contract Deployment       ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
echo ""

# Check if we're in the blockchain directory
if [ ! -f "hardhat.config.ts" ]; then
    echo -e "${RED}❌ Error: Please run this script from the blockchain directory${NC}"
    exit 1
fi

# Check if Hardhat node is running
echo -e "${BLUE}🔍 Checking if Hardhat node is running...${NC}"
if curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://localhost:8545 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Hardhat node is running${NC}"
else
    echo -e "${RED}❌ Hardhat node is not running${NC}"
    echo ""
    echo "Please start Hardhat node in another terminal:"
    echo -e "${YELLOW}  cd blockchain && npx hardhat node${NC}"
    echo ""
    read -p "Press Enter after starting the node, or Ctrl+C to cancel..."
fi

# Deploy contracts
echo ""
echo -e "${BLUE}🚀 Deploying contracts...${NC}"
npx hardhat run scripts/deploy.ts --network localhost

# Check if deployment was successful
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ Contracts deployed successfully!${NC}"
else
    echo ""
    echo -e "${RED}❌ Deployment failed${NC}"
    exit 1
fi

# Show deployed addresses
echo ""
echo -e "${BLUE}📋 Deployed Contract Addresses:${NC}"
if [ -f "deployed-addresses.json" ]; then
    cat deployed-addresses.json | jq -C '.'
else
    echo -e "${YELLOW}⚠️  deployed-addresses.json not found${NC}"
fi

echo ""
echo -e "${BLUE}📝 Next Steps:${NC}"
echo "  1. Copy addresses to your .env file"
echo "  2. Update backend configuration"
echo "  3. Start backend services"
echo ""
echo -e "${GREEN}🎉 Deployment complete!${NC}"
