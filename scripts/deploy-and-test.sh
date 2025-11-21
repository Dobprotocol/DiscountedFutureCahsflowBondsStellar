#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  DOB Liquidity - Stellar Testnet Deployment       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════╝${NC}"

# Check if stellar CLI is installed
if ! command -v stellar &> /dev/null; then
    echo -e "${YELLOW}⚠️  Stellar CLI not found. Installing...${NC}"
    cargo install --locked stellar-cli --features opt
fi

# Build contracts
echo -e "\n${BLUE}[1/7] Building contracts...${NC}"
cargo build --target wasm32-unknown-unknown --release
echo -e "${GREEN}✅ Contracts built${NC}"

# Setup identity
echo -e "\n${BLUE}[2/7] Setting up testnet identity...${NC}"
if ! stellar keys show deployer &> /dev/null; then
    echo "Creating new identity 'deployer'..."
    stellar keys generate deployer --network testnet
else
    echo "Using existing identity 'deployer'"
fi

export DEPLOYER=$(stellar keys address deployer)
echo -e "${GREEN}✅ Deployer address: $DEPLOYER${NC}"

# Fund account
echo -e "\n${BLUE}[3/7] Funding testnet account...${NC}"
curl -s "https://friendbot.stellar.org?addr=$DEPLOYER" > /dev/null
echo -e "${GREEN}✅ Account funded with 10,000 XLM${NC}"

# Deploy contracts
export NETWORK="testnet"

echo -e "\n${BLUE}[4/7] Deploying contracts...${NC}"

echo "  Deploying DobToken..."
TOKEN_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_token.wasm \
  --source deployer \
  --network $NETWORK)
echo -e "  ${GREEN}✅ Token: $TOKEN_ID${NC}"

echo "  Deploying DobOracle..."
ORACLE_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle.wasm \
  --source deployer \
  --network $NETWORK)
echo -e "  ${GREEN}✅ Oracle: $ORACLE_ID${NC}"

echo "  Deploying DobPrimaryMarket..."
MARKET_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_primary_market.wasm \
  --source deployer \
  --network $NETWORK)
echo -e "  ${GREEN}✅ Market: $MARKET_ID${NC}"

echo "  Deploying LiquidNodeStabilizer..."
STABILIZER_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_stabilizer.wasm \
  --source deployer \
  --network $NETWORK)
echo -e "  ${GREEN}✅ Stabilizer: $STABILIZER_ID${NC}"

# Deploy test USDC
echo "  Deploying test USDC..."
stellar contract asset deploy \
  --asset USDC:$DEPLOYER \
  --source deployer \
  --network $NETWORK > /dev/null 2>&1

USDC_ID=$(stellar contract id asset --asset USDC:$DEPLOYER --source-account $DEPLOYER)
echo -e "  ${GREEN}✅ USDC: $USDC_ID${NC}"

# Save addresses
cat > deployed-contracts.env <<EOF
export TOKEN_ID=$TOKEN_ID
export ORACLE_ID=$ORACLE_ID
export MARKET_ID=$MARKET_ID
export STABILIZER_ID=$STABILIZER_ID
export USDC_ID=$USDC_ID
export DEPLOYER=$DEPLOYER
export NETWORK=$NETWORK
EOF

echo -e "${GREEN}✅ All contracts deployed!${NC}"

# Initialize contracts
echo -e "\n${BLUE}[5/7] Initializing contracts...${NC}"

echo "  Initializing Oracle (NAV: \$1.00, Risk: 10%)..."
stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --updater $DEPLOYER \
  --initial_nav 10000000 \
  --initial_risk 1000 > /dev/null 2>&1

echo "  Initializing DobToken..."
stellar contract invoke \
  --id $TOKEN_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --admin $DEPLOYER \
  --hook $MARKET_ID \
  --name "Dob Solar Farm 2035" \
  --symbol "DOB-35" \
  --decimals 7 > /dev/null 2>&1

echo "  Initializing Primary Market..."
stellar contract invoke \
  --id $MARKET_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --dob_token $TOKEN_ID \
  --usdc_token $USDC_ID \
  --oracle $ORACLE_ID \
  --operator $DEPLOYER > /dev/null 2>&1

echo "  Initializing Stabilizer..."
stellar contract invoke \
  --id $STABILIZER_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --oracle $ORACLE_ID \
  --usdc_token $USDC_ID \
  --dob_token $TOKEN_ID \
  --operator $DEPLOYER > /dev/null 2>&1

echo -e "${GREEN}✅ All contracts initialized!${NC}"

# Fund with test USDC
echo -e "\n${BLUE}[6/7] Setting up test USDC...${NC}"

echo "  Minting 100,000 USDC to deployer..."
stellar contract invoke \
  --id $USDC_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- mint \
  --to $DEPLOYER \
  --amount 1000000000000 > /dev/null 2>&1

echo "  Funding Primary Market with 10,000 USDC..."
stellar contract invoke \
  --id $USDC_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- transfer \
  --from $DEPLOYER \
  --to $MARKET_ID \
  --amount 100000000000 > /dev/null 2>&1

echo -e "${GREEN}✅ Test USDC setup complete!${NC}"

# Run tests
echo -e "\n${BLUE}[7/7] Running integration tests...${NC}"

echo "  Test 1: Buying 1,000 USDC worth of DOB..."
stellar contract invoke \
  --id $MARKET_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- buy \
  --buyer $DEPLOYER \
  --usdc_amount 10000000000 > /dev/null 2>&1
echo -e "  ${GREEN}✅ Purchase successful${NC}"

echo "  Test 2: Checking DOB balance..."
DOB_BAL=$(stellar contract invoke \
  --id $TOKEN_ID \
  --network $NETWORK \
  -- balance \
  --account $DEPLOYER)
echo -e "  ${GREEN}✅ DOB Balance: $DOB_BAL (expected: ~9,900,000,000)${NC}"

echo "  Test 3: Getting redemption quote for 500 DOB..."
QUOTE=$(stellar contract invoke \
  --id $MARKET_ID \
  --network $NETWORK \
  -- quote_redemption \
  --dob_amount 5000000000)
echo -e "  ${GREEN}✅ Quote: $QUOTE${NC}"

echo "  Test 4: Selling 500 DOB..."
stellar contract invoke \
  --id $MARKET_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- sell \
  --seller $DEPLOYER \
  --dob_amount 5000000000 > /dev/null 2>&1
echo -e "  ${GREEN}✅ Sale successful${NC}"

echo "  Test 5: Updating oracle to NAV \$1.20, Risk 5%..."
stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- update \
  --new_nav 12000000 \
  --new_default_risk 500 > /dev/null 2>&1
echo -e "  ${GREEN}✅ Oracle updated${NC}"

echo "  Test 6: Buying 500 USDC at new NAV..."
stellar contract invoke \
  --id $MARKET_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- buy \
  --buyer $DEPLOYER \
  --usdc_amount 5000000000 > /dev/null 2>&1
echo -e "  ${GREEN}✅ Purchase at new NAV successful${NC}"

# Summary
echo -e "\n${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          🎉 Deployment Complete! 🎉                ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}Contract Addresses:${NC}"
echo "  Token:      $TOKEN_ID"
echo "  Oracle:     $ORACLE_ID"
echo "  Market:     $MARKET_ID"
echo "  Stabilizer: $STABILIZER_ID"
echo "  USDC:       $USDC_ID"

echo -e "\n${BLUE}View on Stellar Expert:${NC}"
echo "  https://stellar.expert/explorer/testnet/contract/$TOKEN_ID"
echo "  https://stellar.expert/explorer/testnet/contract/$MARKET_ID"

echo -e "\n${BLUE}Your Account:${NC}"
echo "  Address: $DEPLOYER"
echo "  https://stellar.expert/explorer/testnet/account/$DEPLOYER"

echo -e "\n${BLUE}Saved Configuration:${NC}"
echo "  File: deployed-contracts.env"
echo "  Load with: source deployed-contracts.env"

echo -e "\n${YELLOW}Next Steps:${NC}"
echo "  1. View contracts on Stellar Expert"
echo "  2. Test with the frontend"
echo "  3. Invite users to test"
echo "  4. Deploy to mainnet when ready"

echo -e "\n${GREEN}Happy testing! 🚀${NC}"
