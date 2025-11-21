#!/bin/bash

# Deploy to Stellar Testnet
# Requires:
# - stellar-cli installed
# - SOURCE_ACCOUNT environment variable set
# - Account funded with testnet XLM

set -e

echo "Deploying DOB Liquidity contracts to Stellar Testnet..."

# Set network
NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org"

# Check for source account
if [ -z "$SOURCE_ACCOUNT" ]; then
  echo "❌ Error: SOURCE_ACCOUNT environment variable not set"
  echo "Usage: SOURCE_ACCOUNT=GXXX... ./scripts/deploy-testnet.sh"
  exit 1
fi

echo "Using source account: $SOURCE_ACCOUNT"

# Build and optimize contracts
echo "Building contracts..."
make build optimize

# Deploy Token
echo "Deploying DobToken..."
TOKEN_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_token_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "Test SDF Network ; September 2015")

echo "Token deployed: $TOKEN_ID"
sleep 2

# Deploy Oracle
echo "Deploying DobOracle..."
ORACLE_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "Test SDF Network ; September 2015")

echo "Oracle deployed: $ORACLE_ID"
sleep 2

# Deploy Primary Market
echo "Deploying DobPrimaryMarket..."
PRIMARY_MARKET_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_primary_market_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "Test SDF Network ; September 2015")

echo "Primary Market deployed: $PRIMARY_MARKET_ID"
sleep 2

# Deploy Stabilizer
echo "Deploying LiquidNodeStabilizer..."
STABILIZER_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_stabilizer_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "Test SDF Network ; September 2015")

echo "Stabilizer deployed: $STABILIZER_ID"

# Save addresses
cat > deployed-addresses-testnet.json <<EOF
{
  "network": "testnet",
  "token": "$TOKEN_ID",
  "oracle": "$ORACLE_ID",
  "primaryMarket": "$PRIMARY_MARKET_ID",
  "stabilizer": "$STABILIZER_ID",
  "rpcUrl": "$RPC_URL"
}
EOF

echo ""
echo "✅ All contracts deployed successfully to Testnet!"
echo "Addresses saved to deployed-addresses-testnet.json"
echo ""
echo "Contract Addresses:"
echo "  Token:          $TOKEN_ID"
echo "  Oracle:         $ORACLE_ID"
echo "  Primary Market: $PRIMARY_MARKET_ID"
echo "  Stabilizer:     $STABILIZER_ID"
echo ""
echo "Next steps:"
echo "1. Initialize the Oracle with NAV and risk values"
echo "2. Initialize the Token with oracle address"
echo "3. Initialize the Primary Market with token, oracle, and operator"
echo "4. Initialize the Stabilizer with oracle, tokens, and operator"
