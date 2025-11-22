#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Initializing DOB Liquidity Contracts ===${NC}\n"

# Load contract addresses
if [ ! -f "deployed-contracts.env" ]; then
  echo -e "${RED}Error: deployed-contracts.env not found!${NC}"
  echo "Please run deploy-and-test.sh first."
  exit 1
fi

source deployed-contracts.env

echo "Using contracts:"
echo "  TOKEN_ID: $TOKEN_ID"
echo "  ORACLE_ID: $ORACLE_ID"
echo "  POOL_ID: $POOL_ID"
echo "  LN1_ID: $LN1_ID"
echo "  LN2_ID: $LN2_ID"
echo "  USDC_ID: $USDC_ID"
echo "  DEPLOYER: $DEPLOYER"
echo ""

# Check if deployer identity exists
if ! stellar keys show deployer > /dev/null 2>&1; then
  echo -e "${RED}Error: 'deployer' identity not found!${NC}"
  echo "Please create it with: stellar keys generate deployer --network testnet"
  exit 1
fi

# Initialize Oracle
echo -e "${BLUE}[1/8] Initializing Oracle...${NC}"
echo "  Setting NAV: \$1.00 (10000000 stroops)"
echo "  Setting Risk: 10% (1000 basis points)"
stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --updater $DEPLOYER \
  --initial_fair_price 10000000 \
  --initial_risk 1000

if [ $? -ne 0 ]; then
  echo -e "${RED}Failed to initialize Oracle${NC}"
  exit 1
fi
echo -e "${GREEN}✅ Oracle initialized${NC}\n"

# Initialize DobToken
echo -e "${BLUE}[2/8] Initializing DobToken...${NC}"
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
  --decimals 7

if [ $? -ne 0 ]; then
  echo -e "${RED}Failed to initialize DobToken${NC}"
  exit 1
fi
echo -e "${GREEN}✅ DobToken initialized${NC}\n"

# Initialize AMM Pool
echo -e "${BLUE}[3/8] Initializing AMM Pool...${NC}"
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- initialize \
  --dob_token $TOKEN_ID \
  --usdc_token $USDC_ID \
  --oracle $ORACLE_ID \
  --operator $DEPLOYER

if [ $? -ne 0 ]; then
  echo -e "${RED}Failed to initialize AMM Pool${NC}"
  exit 1
fi
echo -e "${GREEN}✅ AMM Pool initialized${NC}\n"

# Initialize Liquid Node #1
echo -e "${BLUE}[4/8] Initializing Liquid Node #1...${NC}"
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
  --amm_pool $POOL_ID

if [ $? -ne 0 ]; then
  echo -e "${RED}Failed to initialize Liquid Node #1${NC}"
  exit 1
fi
echo -e "${GREEN}✅ Liquid Node #1 initialized${NC}\n"

# Initialize Liquid Node #2
echo -e "${BLUE}[5/8] Initializing Liquid Node #2...${NC}"
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
  --amm_pool $POOL_ID

if [ $? -ne 0 ]; then
  echo -e "${RED}Failed to initialize Liquid Node #2${NC}"
  exit 1
fi
echo -e "${GREEN}✅ Liquid Node #2 initialized${NC}\n"

# Mint test USDC
echo -e "${BLUE}[6/8] Minting test USDC...${NC}"
echo "  Minting 200,000 USDC to deployer..."
stellar contract invoke \
  --id $USDC_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- mint \
  --to $DEPLOYER \
  --amount 2000000000000

if [ $? -ne 0 ]; then
  echo -e "${RED}Failed to mint USDC${NC}"
  exit 1
fi
echo -e "${GREEN}✅ USDC minted${NC}\n"

# Register and fund Liquid Nodes
echo -e "${BLUE}[7/8] Registering Liquid Nodes with AMM Pool...${NC}"

echo "  Registering Liquid Node #1..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- register_liquid_node \
  --node $LN1_ID

echo "  Registering Liquid Node #2..."
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- register_liquid_node \
  --node $LN2_ID

echo -e "${GREEN}✅ Liquid Nodes registered${NC}\n"

# Fund Liquid Nodes
echo -e "${BLUE}[8/8] Funding Liquid Nodes...${NC}"

echo "  Funding Liquid Node #1 with 50,000 USDC..."
stellar contract invoke \
  --id $LN1_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- deposit \
  --from $DEPLOYER \
  --amount 500000000000

echo "  Funding Liquid Node #2 with 50,000 USDC..."
stellar contract invoke \
  --id $LN2_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- deposit \
  --from $DEPLOYER \
  --amount 500000000000

echo -e "${GREEN}✅ Liquid Nodes funded${NC}\n"

echo -e "${GREEN}==================================${NC}"
echo -e "${GREEN}✅ All contracts initialized!${NC}"
echo -e "${GREEN}==================================${NC}"
echo ""
echo "Contract Status:"
echo "  • Oracle: NAV=\$1.00, Risk=10%"
echo "  • DobToken: Ready for minting via swaps"
echo "  • AMM Pool: Ready to accept liquidity"
echo "  • Liquid Node #1: 50,000 USDC available"
echo "  • Liquid Node #2: 50,000 USDC available"
echo "  • Deployer Balance: ~100,000 USDC"
echo ""
echo "Next steps:"
echo "  1. Refresh the frontend (http://localhost:3000)"
echo "  2. Connect your wallet"
echo "  3. Try swapping USDC for DOB tokens"
echo ""
