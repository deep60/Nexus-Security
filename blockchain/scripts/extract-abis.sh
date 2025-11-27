#!/bin/bash
# Extract ABIs from Hardhat artifacts and copy to abis directory

set -e

echo "🔍 Extracting ABIs from compiled contracts..."

# Directories
ARTIFACTS_DIR="./artifacts/contracts"
ABIS_DIR="./abis"
BACKEND_ABIS_DIR="../backend/api-gateway/abis"

# Create directories if they don't exist
mkdir -p "$ABIS_DIR"
mkdir -p "$BACKEND_ABIS_DIR"

# Extract core contract ABIs
echo "📦 Extracting BountyManager ABI..."
jq '.abi' "$ARTIFACTS_DIR/core/BountyManager.sol/BountyManager.json" > "$ABIS_DIR/BountyManager.abi.json"
cp "$ARTIFACTS_DIR/core/BountyManager.sol/BountyManager.json" "$ABIS_DIR/BountyManager.json"

echo "📦 Extracting ThreatToken ABI..."
jq '.abi' "$ARTIFACTS_DIR/core/ThreatToken.sol/ThreatToken.json" > "$ABIS_DIR/ThreatToken.abi.json"
cp "$ARTIFACTS_DIR/core/ThreatToken.sol/ThreatToken.json" "$ABIS_DIR/ThreatToken.json"

echo "📦 Extracting ReputationSystem ABI..."
jq '.abi' "$ARTIFACTS_DIR/core/ReputationSystem.sol/ReputationSystem.json" > "$ABIS_DIR/ReputationSystem.abi.json"
cp "$ARTIFACTS_DIR/core/ReputationSystem.sol/ReputationSystem.json" "$ABIS_DIR/ReputationSystem.json"

echo "📦 Extracting Governance ABI..."
jq '.abi' "$ARTIFACTS_DIR/core/Governance.sol/Governance.json" > "$ABIS_DIR/Governance.abi.json"
cp "$ARTIFACTS_DIR/core/Governance.sol/Governance.json" "$ABIS_DIR/Governance.json"

# Copy to backend
echo "📋 Copying ABIs to backend..."
cp "$ABIS_DIR"/*.json "$BACKEND_ABIS_DIR/"

# Show summary
echo ""
echo "✅ ABI extraction complete!"
echo ""
echo "📁 ABIs saved to:"
echo "   - $ABIS_DIR/"
echo "   - $BACKEND_ABIS_DIR/"
echo ""
echo "📋 Extracted contracts:"
ls -lh "$ABIS_DIR"/*.json
echo ""
echo "🎉 Backend services can now load these ABIs!"
