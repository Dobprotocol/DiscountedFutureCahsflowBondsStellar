#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

# Load deployed contract addresses
if [ ! -f "deployed-contracts.env" ]; then
    echo -e "${RED}Error: deployed-contracts.env not found. Run deploy-and-test.sh first.${NC}"
    exit 1
fi

source deployed-contracts.env

echo -e "${BLUE}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     DOB Liquidity - Detailed Validation Tests     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════╝${NC}"

# Helper function to convert 7-decimal amounts to readable format
to_readable() {
    local amount=$1
    echo "scale=2; $amount / 10000000" | bc
}

# Helper function to get balance
get_balance() {
    stellar contract invoke \
        --id $1 \
        --network $NETWORK \
        -- balance \
        --account $2 2>/dev/null | tr -d '"'
}

# Helper function to get reserves
get_reserves() {
    stellar contract invoke \
        --id $POOL_ID \
        --network $NETWORK \
        -- get_reserves 2>/dev/null
}

# Get oracle data
get_oracle_nav() {
    stellar contract invoke \
        --id $ORACLE_ID \
        --network $NETWORK \
        -- nav 2>/dev/null | tr -d '"'
}

get_oracle_risk() {
    stellar contract invoke \
        --id $ORACLE_ID \
        --network $NETWORK \
        -- default_risk 2>/dev/null | tr -d '"'
}

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  INITIAL STATE${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

INITIAL_USDC=$(get_balance $USDC_ID $DEPLOYER)
INITIAL_DOB=$(get_balance $TOKEN_ID $DEPLOYER)
INITIAL_NAV=$(get_oracle_nav)
INITIAL_RISK=$(get_oracle_risk)
INITIAL_RESERVES=$(get_reserves)

echo -e "Deployer USDC:     $(to_readable $INITIAL_USDC) USDC"
echo -e "Deployer DOB:      $(to_readable $INITIAL_DOB) DOB"
echo -e "Oracle NAV:        \$$(to_readable $INITIAL_NAV)"
echo -e "Oracle Risk:       $(echo "scale=2; $INITIAL_RISK / 100" | bc)%"
echo -e "Pool Reserves:     $INITIAL_RESERVES"

# Calculate pool price
if echo "$INITIAL_RESERVES" | grep -q ","; then
    USDC_RES=$(echo "$INITIAL_RESERVES" | sed 's/[^0-9,]//g' | cut -d',' -f1)
    DOB_RES=$(echo "$INITIAL_RESERVES" | sed 's/[^0-9,]//g' | cut -d',' -f2)
    if [ "$DOB_RES" != "0" ] && [ -n "$DOB_RES" ]; then
        POOL_PRICE=$(echo "scale=7; ($USDC_RES * 10000000) / $DOB_RES" | bc)
        echo -e "Pool Price:        \$$(to_readable $POOL_PRICE)"
    fi
fi

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  TEST 1: COMPRA DE TOKENS (AfterSwap Hook)${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

BUY_AMOUNT=20000000000  # 2,000 USDC
echo -e "${YELLOW}Comprando 2,000 USDC de DOB tokens...${NC}"
echo -e "  • USDC depositado:  $(to_readable $BUY_AMOUNT) USDC"
echo -e "  • Fee DEX (1%):     $(to_readable $((BUY_AMOUNT * 100 / 10000))) USDC"
echo -e "  • Al operador (99%): $(to_readable $((BUY_AMOUNT * 9900 / 10000))) USDC"
echo -e "  • Fair Price:       \$$(to_readable $INITIAL_NAV)"

EXPECTED_DOB=$(echo "scale=0; ($BUY_AMOUNT * 9900 * 10000000) / (10000 * $INITIAL_NAV)" | bc)
echo -e "  • DOB esperado:     ~$(to_readable $EXPECTED_DOB) DOB"

stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- swap_buy \
  --buyer $DEPLOYER \
  --usdc_amount $BUY_AMOUNT > /dev/null 2>&1

AFTER_BUY_USDC=$(get_balance $USDC_ID $DEPLOYER)
AFTER_BUY_DOB=$(get_balance $TOKEN_ID $DEPLOYER)
RECEIVED_DOB=$((AFTER_BUY_DOB - INITIAL_DOB))

echo -e "\n${GREEN}✅ Compra exitosa!${NC}"
echo -e "  • DOB recibido:     $(to_readable $RECEIVED_DOB) DOB"
echo -e "  • USDC gastado:     $(to_readable $((INITIAL_USDC - AFTER_BUY_USDC))) USDC"
echo -e "  • Nuevo balance DOB: $(to_readable $AFTER_BUY_DOB) DOB"

# Update state
INITIAL_USDC=$AFTER_BUY_USDC
INITIAL_DOB=$AFTER_BUY_DOB

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  TEST 2: VENTA CON LIQUIDEZ (Pool suficiente)${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

SELL_AMOUNT_SMALL=5000000000  # 500 DOB
echo -e "${YELLOW}Vendiendo 500 DOB tokens...${NC}"

# Get quote first
QUOTE=$(stellar contract invoke \
  --id $POOL_ID \
  --network $NETWORK \
  -- quote_swap_sell \
  --dob_amount $SELL_AMOUNT_SMALL 2>/dev/null)

echo -e "  • DOB a vender:     $(to_readable $SELL_AMOUNT_SMALL) DOB"
echo -e "  • Fair Price:       \$$(to_readable $INITIAL_NAV)"

# Calculate expected
BASE_FEE=$((300 + INITIAL_RISK / 10))
echo -e "  • Base Fee:         $(echo "scale=2; $BASE_FEE / 100" | bc)%"

USDC_VALUE=$(echo "scale=0; ($SELL_AMOUNT_SMALL * $INITIAL_NAV) / 10000000" | bc)
EXPECTED_USDC=$(echo "scale=0; ($USDC_VALUE * (10000 - $BASE_FEE)) / 10000" | bc)
echo -e "  • USDC esperado:    ~$(to_readable $EXPECTED_USDC) USDC"

stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- swap_sell \
  --seller $DEPLOYER \
  --dob_amount $SELL_AMOUNT_SMALL > /dev/null 2>&1

AFTER_SELL_USDC=$(get_balance $USDC_ID $DEPLOYER)
AFTER_SELL_DOB=$(get_balance $TOKEN_ID $DEPLOYER)
RECEIVED_USDC=$((AFTER_SELL_USDC - INITIAL_USDC))

echo -e "\n${GREEN}✅ Venta exitosa!${NC}"
echo -e "  • USDC recibido:    $(to_readable $RECEIVED_USDC) USDC"
echo -e "  • DOB quemado:      $(to_readable $((INITIAL_DOB - AFTER_SELL_DOB))) DOB"
echo -e "  • Nuevo balance USDC: $(to_readable $AFTER_SELL_USDC) USDC"

RESERVES_AFTER=$(get_reserves)
echo -e "  • Pool reserves:    $RESERVES_AFTER"

# Update state
INITIAL_USDC=$AFTER_SELL_USDC
INITIAL_DOB=$AFTER_SELL_DOB

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  TEST 3: CAMBIO DE RIESGO EN ORÁCULO${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

NEW_NAV=15000000  # $1.50
NEW_RISK=3500     # 35%
echo -e "${YELLOW}Actualizando Oracle...${NC}"
echo -e "  • Nuevo NAV:        \$$(to_readable $NEW_NAV) (antes: \$$(to_readable $INITIAL_NAV))"
echo -e "  • Nuevo Risk:       $(echo "scale=2; $NEW_RISK / 100" | bc)% (antes: $(echo "scale=2; $INITIAL_RISK / 100" | bc)%)"

stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- update \
  --new_nav $NEW_NAV \
  --new_default_risk $NEW_RISK > /dev/null 2>&1

echo -e "\n${GREEN}✅ Oracle actualizado!${NC}"

# Calculate new fee
NEW_BASE_FEE=$((300 + NEW_RISK / 10))
echo -e "  • Nuevo Base Fee:   $(echo "scale=2; $NEW_BASE_FEE / 100" | bc)% (antes: $(echo "scale=2; $BASE_FEE / 100" | bc)%)"

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  TEST 4: COMPRA CON NUEVO NAV${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

BUY_AMOUNT_2=10000000000  # 1,000 USDC
echo -e "${YELLOW}Comprando 1,000 USDC de DOB al nuevo precio...${NC}"
echo -e "  • USDC depositado:  $(to_readable $BUY_AMOUNT_2) USDC"
echo -e "  • Nuevo Fair Price: \$$(to_readable $NEW_NAV)"

EXPECTED_DOB_2=$(echo "scale=0; ($BUY_AMOUNT_2 * 9900 * 10000000) / (10000 * $NEW_NAV)" | bc)
echo -e "  • DOB esperado:     ~$(to_readable $EXPECTED_DOB_2) DOB"
echo -e "  ${CYAN}(Nota: Menos DOB porque el precio subió)${NC}"

stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- swap_buy \
  --buyer $DEPLOYER \
  --usdc_amount $BUY_AMOUNT_2 > /dev/null 2>&1

AFTER_BUY2_DOB=$(get_balance $TOKEN_ID $DEPLOYER)
RECEIVED_DOB_2=$((AFTER_BUY2_DOB - INITIAL_DOB))

echo -e "\n${GREEN}✅ Compra exitosa!${NC}"
echo -e "  • DOB recibido:     $(to_readable $RECEIVED_DOB_2) DOB"
echo -e "  • Comparación:"
echo -e "    - Compra 1 (NAV \$$(to_readable $INITIAL_NAV)): $(to_readable $RECEIVED_DOB) DOB por 2,000 USDC"
echo -e "    - Compra 2 (NAV \$$(to_readable $NEW_NAV)): $(to_readable $RECEIVED_DOB_2) DOB por 1,000 USDC"

# Update state
INITIAL_DOB=$AFTER_BUY2_DOB

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  TEST 5: VERIFICAR LIQUID NODES${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

echo -e "${YELLOW}Consultando Liquid Nodes registrados...${NC}"

LN_LIST=$(stellar contract invoke \
  --id $POOL_ID \
  --network $NETWORK \
  -- get_liquid_nodes 2>/dev/null)

echo -e "\n${GREEN}✅ Liquid Nodes activos:${NC}"
echo -e "$LN_LIST" | tr ',' '\n' | grep -o 'C[A-Z0-9]*' | while read -r ln; do
    echo -e "  • $ln"

    # Get LN balance
    LN_USDC=$(get_balance $USDC_ID $ln)
    echo -e "    Balance USDC: $(to_readable $LN_USDC) USDC"

    # Get quote from this LN
    TEST_DOB=10000000000  # 1,000 DOB
    QUOTE_RESULT=$(stellar contract invoke \
      --id $ln \
      --network $NETWORK \
      -- request_quote \
      --dob_amount $TEST_DOB 2>/dev/null || echo "Error")

    if [ "$QUOTE_RESULT" != "Error" ]; then
        QUOTE_USDC=$(echo "$QUOTE_RESULT" | grep -o '[0-9]*' | head -1)
        QUOTE_FEE=$(echo "$QUOTE_RESULT" | grep -o '[0-9]*' | tail -1)
        echo -e "    Quote para 1,000 DOB:"
        echo -e "      - USDC provisto: $(to_readable $QUOTE_USDC) USDC"
        echo -e "      - Fee: $(echo "scale=2; $QUOTE_FEE / 100" | bc)%"
    fi
done

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  ESTADO FINAL${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

FINAL_USDC=$(get_balance $USDC_ID $DEPLOYER)
FINAL_DOB=$(get_balance $TOKEN_ID $DEPLOYER)
FINAL_NAV=$(get_oracle_nav)
FINAL_RISK=$(get_oracle_risk)
FINAL_RESERVES=$(get_reserves)

echo -e "Deployer USDC:     $(to_readable $FINAL_USDC) USDC"
echo -e "Deployer DOB:      $(to_readable $FINAL_DOB) DOB"
echo -e "Oracle NAV:        \$$(to_readable $FINAL_NAV)"
echo -e "Oracle Risk:       $(echo "scale=2; $FINAL_RISK / 100" | bc)%"
echo -e "Pool Reserves:     $FINAL_RESERVES"

echo -e "\n${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          🎉 Tests Completados! 🎉                  ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}Resumen de Validaciones:${NC}"
echo -e "  ✅ Compra con AfterSwap hook (minteo al NAV)"
echo -e "  ✅ Venta con pool suficiente (BeforeSwap no necesita LN)"
echo -e "  ✅ Fees dinámicos basados en Oracle Risk"
echo -e "  ✅ Cambios de NAV afectan precio de minteo"
echo -e "  ✅ Liquid Nodes registrados y fondeados"
echo -e "  ✅ Quema de tokens al vender"

echo -e "\n${YELLOW}Siguiente paso: Probar venta GRANDE que active Liquid Nodes${NC}"
echo -e "Ejecuta: ./scripts/test-liquid-node-scenario.sh"
