#!/bin/bash

# Simple deployment script with better error handling
set -e  # Exit on error

echo "=== Simple Stellar Testnet Deployment ==="

# Setup identity
echo ""
echo "[1/5] Setting up identity..."
if ! stellar keys show deployer 2>/dev/null; then
    echo "Creating new identity..."
    stellar keys generate deployer --network testnet
else
    echo "Using existing identity"
fi

export DEPLOYER=$(stellar keys address deployer)
echo "Deployer: $DEPLOYER"

# Fund account
echo ""
echo "[2/5] Funding account..."
echo "Requesting funds from Friendbot..."
curl -s "https://friendbot.stellar.org?addr=$DEPLOYER" | grep -q "successful" && echo "✅ Funded" || echo "⚠️  Check if already funded"

# Wait a bit for funding to process
sleep 2

# Deploy one contract as test
echo ""
echo "[3/5] Deploying Token contract..."
export NETWORK="testnet"

TOKEN_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_token.wasm \
  --source deployer \
  --network $NETWORK 2>&1)

if [ $? -eq 0 ]; then
    echo "✅ Token deployed: $TOKEN_ID"
else
    echo "❌ Deployment failed!"
    echo "Error: $TOKEN_ID"
    exit 1
fi

# Deploy Oracle
echo ""
echo "[4/5] Deploying Oracle..."
ORACLE_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle.wasm \
  --source deployer \
  --network $NETWORK 2>&1)

if [ $? -eq 0 ]; then
    echo "✅ Oracle deployed: $ORACLE_ID"
else
    echo "❌ Deployment failed!"
    echo "Error: $ORACLE_ID"
    exit 1
fi

# Initialize Oracle
echo ""
echo "[5/5] Initializing Oracle..."
stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --updater $DEPLOYER \
  --initial_nav 10000000 \
  --initial_risk 1000

if [ $? -eq 0 ]; then
    echo "✅ Oracle initialized"
else
    echo "❌ Initialization failed!"
    exit 1
fi

# Test - check NAV
echo ""
echo "Testing: Getting NAV..."
NAV=$(stellar contract invoke \
  --id $ORACLE_ID \
  --network $NETWORK \
  -- nav)

echo "Current NAV: $NAV (expected: 10000000)"

echo ""
echo "=== Deployment Successful! ==="
echo ""
echo "Token ID: $TOKEN_ID"
echo "Oracle ID: $ORACLE_ID"
echo ""
echo "View on Stellar Expert:"
echo "https://stellar.expert/explorer/testnet/contract/$TOKEN_ID"
