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
    cargo install --locked stellar-cli
fi

# Build contracts
echo -e "\n${BLUE}[1/9] Building contracts...${NC}"
cargo build --target wasm32-unknown-unknown --release
echo -e "${GREEN}✅ Contracts built${NC}"

# Setup identity
echo -e "\n${BLUE}[2/9] Setting up testnet identity...${NC}"
if ! stellar keys show deployer &> /dev/null; then
    echo "Creating new identity 'deployer'..."
    stellar keys generate deployer --network testnet
else
    echo "Using existing identity 'deployer'"
fi

export DEPLOYER=$(stellar keys address deployer)
echo -e "${GREEN}✅ Deployer address: $DEPLOYER${NC}"

# Fund account
echo -e "\n${BLUE}[3/9] Funding testnet account...${NC}"
curl -s "https://friendbot.stellar.org?addr=$DEPLOYER" > /dev/null
echo -e "${GREEN}✅ Account funded with 10,000 XLM${NC}"

# Deploy contracts
export NETWORK="testnet"

echo -e "\n${BLUE}[4/9] Deploying contracts...${NC}"

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

echo "  Deploying AMM Pool..."
POOL_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_amm_pool.wasm \
  --source deployer \
  --network $NETWORK)
echo -e "  ${GREEN}✅ AMM Pool: $POOL_ID${NC}"

echo "  Deploying LiquidNode #1..."
LN1_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_stabilizer.wasm \
  --source deployer \
  --network $NETWORK)
echo -e "  ${GREEN}✅ Liquid Node 1: $LN1_ID${NC}"

echo "  Deploying LiquidNode #2..."
LN2_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_stabilizer.wasm \
  --source deployer \
  --network $NETWORK)
echo -e "  ${GREEN}✅ Liquid Node 2: $LN2_ID${NC}"

# Deploy test USDC
echo "  Deploying test USDC..."
stellar contract asset deploy \
  --asset USDC:$DEPLOYER \
  --source deployer \
  --network $NETWORK > /dev/null 2>&1

USDC_ID=$(stellar contract id asset --asset USDC:$DEPLOYER)
echo -e "  ${GREEN}✅ USDC: $USDC_ID${NC}"

# Save addresses
cat > deployed-contracts.env <<EOF
export TOKEN_ID=$TOKEN_ID
export ORACLE_ID=$ORACLE_ID
export POOL_ID=$POOL_ID
export LN1_ID=$LN1_ID
export LN2_ID=$LN2_ID
export USDC_ID=$USDC_ID
export DEPLOYER=$DEPLOYER
export NETWORK=$NETWORK
EOF

echo -e "${GREEN}✅ All contracts deployed!${NC}"

# Initialize contracts
echo -e "\n${BLUE}[5/9] Initializing contracts...${NC}"

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
  --hook $POOL_ID \
  --name "Dob Solar Farm 2035" \
  --symbol "DOB-35" \
  --decimals 7 > /dev/null 2>&1

echo "  Initializing AMM Pool..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --dob_token $TOKEN_ID \
  --usdc_token $USDC_ID \
  --oracle $ORACLE_ID \
  --operator $DEPLOYER > /dev/null 2>&1

echo "  Initializing Liquid Node #1..."
stellar contract invoke \
  --id $LN1_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --oracle $ORACLE_ID \
  --usdc_token $USDC_ID \
  --dob_token $TOKEN_ID \
  --operator $DEPLOYER \
  --amm_pool $POOL_ID > /dev/null 2>&1

echo "  Initializing Liquid Node #2..."
stellar contract invoke \
  --id $LN2_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --oracle $ORACLE_ID \
  --usdc_token $USDC_ID \
  --dob_token $TOKEN_ID \
  --operator $DEPLOYER \
  --amm_pool $POOL_ID > /dev/null 2>&1

echo -e "${GREEN}✅ All contracts initialized!${NC}"

# Fund with test USDC and setup liquidity
echo -e "\n${BLUE}[6/9] Setting up test USDC and liquidity...${NC}"

echo "  Minting 200,000 USDC to deployer..."
stellar contract invoke \
  --id $USDC_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- mint \
  --to $DEPLOYER \
  --amount 2000000000000 > /dev/null 2>&1

echo "  Funding Liquid Node #1 with 50,000 USDC..."
stellar contract invoke \
  --id $LN1_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- fund_usdc \
  --funder $DEPLOYER \
  --amount 500000000000 > /dev/null 2>&1

echo "  Funding Liquid Node #2 with 50,000 USDC..."
stellar contract invoke \
  --id $LN2_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- fund_usdc \
  --funder $DEPLOYER \
  --amount 500000000000 > /dev/null 2>&1

echo -e "${GREEN}✅ Test USDC and LN funding complete!${NC}"

# Register Liquid Nodes with Pool
echo -e "\n${BLUE}[7/9] Registering Liquid Nodes with AMM Pool...${NC}"

echo "  Registering LN #1..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- register_liquid_node \
  --node $LN1_ID > /dev/null 2>&1

echo "  Registering LN #2..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- register_liquid_node \
  --node $LN2_ID > /dev/null 2>&1

echo -e "${GREEN}✅ Liquid Nodes registered!${NC}"

# Add initial liquidity to pool
echo -e "\n${BLUE}[8/9] Adding initial liquidity to AMM Pool...${NC}"

# First, mint DOB tokens to deployer for liquidity provision
echo "  Minting 10,000 DOB for initial liquidity..."
stellar contract invoke \
  --id $TOKEN_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- mint \
  --to $DEPLOYER \
  --amount 100000000000 > /dev/null 2>&1

echo "  Adding 10,000 USDC + 10,000 DOB to pool..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- add_liquidity \
  --provider $DEPLOYER \
  --usdc_amount 100000000000 \
  --dob_amount 100000000000 > /dev/null 2>&1

echo -e "${GREEN}✅ Initial liquidity added!${NC}"

# Run tests
echo -e "\n${BLUE}[9/9] Running integration tests...${NC}"

echo "  Test 1: Buying 1,000 USDC worth of DOB via AMM Pool..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- swap_buy \
  --buyer $DEPLOYER \
  --usdc_amount 10000000000 > /dev/null 2>&1
echo -e "  ${GREEN}✅ Purchase successful (AfterSwap hook)${NC}"

echo "  Test 2: Checking DOB balance..."
DOB_BAL=$(stellar contract invoke \
  --id $TOKEN_ID \
  --network $NETWORK \
  -- balance \
  --account $DEPLOYER)
echo -e "  ${GREEN}✅ DOB Balance: $DOB_BAL${NC}"

echo "  Test 3: Checking pool reserves..."
RESERVES=$(stellar contract invoke \
  --id $POOL_ID \
  --network $NETWORK \
  -- get_reserves)
echo -e "  ${GREEN}✅ Pool Reserves: $RESERVES${NC}"

echo "  Test 4: Getting sell quote for 500 DOB..."
QUOTE=$(stellar contract invoke \
  --id $POOL_ID \
  --network $NETWORK \
  -- quote_swap_sell \
  --dob_amount 5000000000)
echo -e "  ${GREEN}✅ Quote: $QUOTE${NC}"

echo "  Test 5: Selling 500 DOB via AMM Pool..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- swap_sell \
  --seller $DEPLOYER \
  --dob_amount 5000000000 > /dev/null 2>&1
echo -e "  ${GREEN}✅ Sale successful (BeforeSwap hook)${NC}"

echo "  Test 6: Checking registered Liquid Nodes..."
LN_LIST=$(stellar contract invoke \
  --id $POOL_ID \
  --network $NETWORK \
  -- get_liquid_nodes)
echo -e "  ${GREEN}✅ Registered LN: $LN_LIST${NC}"

echo "  Test 7: Updating oracle to NAV \$1.20, Risk 5%..."
stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- update \
  --new_nav 12000000 \
  --new_default_risk 500 > /dev/null 2>&1
echo -e "  ${GREEN}✅ Oracle updated${NC}"

echo "  Test 8: Buying 500 USDC at new NAV..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- swap_buy \
  --buyer $DEPLOYER \
  --usdc_amount 5000000000 > /dev/null 2>&1
echo -e "  ${GREEN}✅ Purchase at new NAV successful${NC}"

# Summary
echo -e "\n${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          🎉 Deployment Complete! 🎉                ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}Contract Addresses:${NC}"
echo "  Token:         $TOKEN_ID"
echo "  Oracle:        $ORACLE_ID"
echo "  AMM Pool:      $POOL_ID"
echo "  Liquid Node 1: $LN1_ID"
echo "  Liquid Node 2: $LN2_ID"
echo "  USDC:          $USDC_ID"

echo -e "\n${BLUE}View on Stellar Expert:${NC}"
echo "  Token:     https://stellar.expert/explorer/testnet/contract/$TOKEN_ID"
echo "  AMM Pool:  https://stellar.expert/explorer/testnet/contract/$POOL_ID"
echo "  Oracle:    https://stellar.expert/explorer/testnet/contract/$ORACLE_ID"

echo -e "\n${BLUE}Your Account:${NC}"
echo "  Address: $DEPLOYER"
echo "  https://stellar.expert/explorer/testnet/account/$DEPLOYER"

echo -e "\n${BLUE}Saved Configuration:${NC}"
echo "  File: deployed-contracts.env"
echo "  Load with: source deployed-contracts.env"

echo -e "\n${BLUE}Architecture Summary:${NC}"
echo "  ✅ AMM Pool deployed with liquidity (10k USDC + 10k DOB)"
echo "  ✅ 2 Liquid Nodes registered and funded (50k USDC each)"
echo "  ✅ AfterSwap hook (buy) - mints DOB at NAV price"
echo "  ✅ BeforeSwap hook (sell) - auto-searches LN if needed"

echo -e "\n${YELLOW}Next Steps:${NC}"
echo "  1. View contracts on Stellar Expert"
echo "  2. Test trading: swap_buy and swap_sell"
echo "  3. Add more liquidity: add_liquidity"
echo "  4. Deploy more Liquid Nodes for deeper liquidity"
echo "  5. Deploy to mainnet when ready"

echo -e "\n${BLUE}Documentation:${NC}"
echo "  - NEW_ARCHITECTURE.md - Full system architecture"
echo "  - IMPLEMENTATION_SUMMARY.md - Quick reference"
echo "  - ARCHITECTURE_DIAGRAM.md - Visual diagrams"

echo -e "\n${GREEN}Happy testing! 🚀${NC}"
