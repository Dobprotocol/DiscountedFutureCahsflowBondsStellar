#!/bin/bash

# Deploy to local Stellar network
# Make sure stellar-cli is installed and a local network is running

set -e

echo "Deploying DOB Liquidity contracts to local network..."

# Set network
NETWORK="local"
RPC_URL="http://localhost:8000/soroban/rpc"

# Source account (replace with your local test account)
SOURCE_ACCOUNT="${SOURCE_ACCOUNT:-GDIY6AQQ75WMD4W46EYB7O6UYMHOCGQHLAQGQTKHDX4J2DYQCHVCR4W4}"

# Build and optimize contracts
echo "Building contracts..."
make build optimize

# Deploy Token
echo "Deploying DobToken..."
TOKEN_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_token_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "Standalone Network ; February 2017")

echo "Token deployed: $TOKEN_ID"

# Deploy Oracle
echo "Deploying DobOracle..."
ORACLE_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "Standalone Network ; February 2017")

echo "Oracle deployed: $ORACLE_ID"

# Deploy Primary Market
echo "Deploying DobPrimaryMarket..."
PRIMARY_MARKET_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_primary_market_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "Standalone Network ; February 2017")

echo "Primary Market deployed: $PRIMARY_MARKET_ID"

# Deploy Stabilizer
echo "Deploying LiquidNodeStabilizer..."
STABILIZER_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_stabilizer_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "Standalone Network ; February 2017")

echo "Stabilizer deployed: $STABILIZER_ID"

# Save addresses
cat > deployed-addresses-local.json <<EOF
{
  "network": "local",
  "token": "$TOKEN_ID",
  "oracle": "$ORACLE_ID",
  "primaryMarket": "$PRIMARY_MARKET_ID",
  "stabilizer": "$STABILIZER_ID"
}
EOF

echo ""
echo "✅ All contracts deployed successfully!"
echo "Addresses saved to deployed-addresses-local.json"
echo ""
echo "Next steps:"
echo "1. Initialize the Oracle with NAV and risk values"
echo "2. Initialize the Token with oracle address"
echo "3. Initialize the Primary Market with token, oracle, and operator"
echo "4. Initialize the Stabilizer with oracle, tokens, and operator"
