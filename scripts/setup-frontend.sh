#!/bin/bash
set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         DOB Liquidity - Frontend Setup            ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════╝${NC}"

# Check if deployed-contracts.env exists
if [ ! -f "deployed-contracts.env" ]; then
    echo -e "${YELLOW}⚠️  deployed-contracts.env not found${NC}"
    echo -e "Please run ./scripts/deploy-and-test.sh first to deploy contracts"
    exit 1
fi

# Source the contract addresses
source deployed-contracts.env

echo -e "\n${BLUE}[1/3] Creating frontend .env file...${NC}"

# Create .env file in frontend directory
cat > frontend/.env <<EOF
# Auto-generated from deployed-contracts.env
VITE_TOKEN_ID=$TOKEN_ID
VITE_ORACLE_ID=$ORACLE_ID
VITE_POOL_ID=$POOL_ID
VITE_USDC_ID=$USDC_ID
VITE_LN1_ID=$LN1_ID
VITE_LN2_ID=$LN2_ID
VITE_NETWORK=$NETWORK
EOF

echo -e "${GREEN}✅ Frontend .env created${NC}"

echo -e "\n${BLUE}[2/3] Installing dependencies...${NC}"

cd frontend

if ! command -v npm &> /dev/null; then
    echo -e "${YELLOW}⚠️  npm not found. Please install Node.js${NC}"
    exit 1
fi

npm install

echo -e "${GREEN}✅ Dependencies installed${NC}"

echo -e "\n${BLUE}[3/3] Configuration Summary${NC}"
echo -e "Network:       ${NETWORK}"
echo -e "Token ID:      ${TOKEN_ID}"
echo -e "Oracle ID:     ${ORACLE_ID}"
echo -e "Pool ID:       ${POOL_ID}"
echo -e "USDC ID:       ${USDC_ID}"
echo -e "Liquid Node 1: ${LN1_ID}"
echo -e "Liquid Node 2: ${LN2_ID}"

echo -e "\n${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          ✅ Frontend Setup Complete!               ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}To start the development server:${NC}"
echo -e "  cd frontend"
echo -e "  npm run dev"

echo -e "\n${BLUE}The application will be available at:${NC}"
echo -e "  http://localhost:3000"

echo -e "\n${YELLOW}Next steps:${NC}"
echo -e "  1. Install Freighter wallet extension if you haven't"
echo -e "  2. Switch to ${NETWORK} in Freighter"
echo -e "  3. Import your deployer account or create a new one"
echo -e "  4. Start the dev server and connect your wallet"
echo -e "  5. Start swapping and providing liquidity!"
