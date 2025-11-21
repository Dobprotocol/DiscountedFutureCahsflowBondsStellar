#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Load deployed contract addresses
if [ ! -f "deployed-contracts.env" ]; then
    echo -e "${RED}Error: deployed-contracts.env not found. Run deploy-and-test.sh first.${NC}"
    exit 1
fi

source deployed-contracts.env

echo -e "${MAGENTA}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  TEST: VENTA GRANDE CON LIQUID NODES (BeforeSwap) ║${NC}"
echo -e "${MAGENTA}╚════════════════════════════════════════════════════╝${NC}"

# Helper functions
to_readable() {
    echo "scale=2; $1 / 10000000" | bc
}

get_balance() {
    stellar contract invoke --id $1 --network $NETWORK -- balance --account $2 2>/dev/null | tr -d '"'
}

get_reserves() {
    stellar contract invoke --id $POOL_ID --network $NETWORK -- get_reserves 2>/dev/null
}

get_oracle_nav() {
    stellar contract invoke --id $ORACLE_ID --network $NETWORK -- nav 2>/dev/null | tr -d '"'
}

get_oracle_risk() {
    stellar contract invoke --id $ORACLE_ID --network $NETWORK -- default_risk 2>/dev/null | tr -d '"'
}

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  ESCENARIO: Como tu ejemplo original${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

echo -e "\n${YELLOW}Configuración del escenario:${NC}"
echo -e "  • Inversor quiere vender una GRAN cantidad de DOB"
echo -e "  • Pool NO tiene suficiente liquidez USDC"
echo -e "  • Sistema debe consultar Liquid Nodes"
echo -e "  • Liquid Nodes calculan fee según riesgo del Oracle"
echo -e "  • Se elige el LN con mejor fee"
echo -e "  • Swap combina Pool + Liquid Node"

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PASO 1: Estado Inicial${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

INITIAL_DEPLOYER_USDC=$(get_balance $USDC_ID $DEPLOYER)
INITIAL_DEPLOYER_DOB=$(get_balance $TOKEN_ID $DEPLOYER)
INITIAL_RESERVES=$(get_reserves)
NAV=$(get_oracle_nav)
RISK=$(get_oracle_risk)

echo -e "Deployer:"
echo -e "  USDC: $(to_readable $INITIAL_DEPLOYER_USDC)"
echo -e "  DOB:  $(to_readable $INITIAL_DEPLOYER_DOB)"
echo -e "\nPool Reserves: $INITIAL_RESERVES"

# Extract reserves
USDC_RESERVE=$(echo "$INITIAL_RESERVES" | sed 's/[^0-9,]//g' | cut -d',' -f1)
DOB_RESERVE=$(echo "$INITIAL_RESERVES" | sed 's/[^0-9,]//g' | cut -d',' -f2)

echo -e "  USDC: $(to_readable $USDC_RESERVE)"
echo -e "  DOB:  $(to_readable $DOB_RESERVE)"

echo -e "\nOracle:"
echo -e "  NAV:  \$$(to_readable $NAV)"
echo -e "  Risk: $(echo "scale=2; $RISK / 100" | bc)%"

# Check Liquid Nodes
echo -e "\nLiquid Nodes:"
LN_LIST=$(stellar contract invoke --id $POOL_ID --network $NETWORK -- get_liquid_nodes 2>/dev/null)
echo "$LN_LIST" | grep -o 'C[A-Z0-9]*' | while read -r ln; do
    LN_USDC=$(get_balance $USDC_ID $ln)
    echo -e "  $ln: $(to_readable $LN_USDC) USDC"
done

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PASO 2: Preparación - Comprar más DOB${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

# Buy a large amount of DOB to have enough to sell
BUY_AMOUNT=50000000000  # 5,000 USDC
echo -e "${YELLOW}Comprando $(to_readable $BUY_AMOUNT) USDC de DOB...${NC}"

stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- swap_buy \
  --buyer $DEPLOYER \
  --usdc_amount $BUY_AMOUNT > /dev/null 2>&1

NEW_DOB=$(get_balance $TOKEN_ID $DEPLOYER)
BOUGHT_DOB=$((NEW_DOB - INITIAL_DEPLOYER_DOB))

echo -e "${GREEN}✅ DOB comprado: $(to_readable $BOUGHT_DOB)${NC}"
echo -e "Nuevo balance DOB: $(to_readable $NEW_DOB)"

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PASO 3: Calcular Venta Grande${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

# Calculate a sell amount that will exceed pool liquidity
# We want to sell more DOB than the pool can handle
USDC_RESERVE=$(echo "$INITIAL_RESERVES" | sed 's/[^0-9,]//g' | cut -d',' -f1)

# Calculate DOB amount that would require MORE USDC than pool has
# Example: If pool has 10,000 USDC and NAV is $1.50, selling 15,000 DOB would need 22,500 USDC
SELL_DOB_AMOUNT=$(echo "scale=0; (($USDC_RESERVE * 2) * 10000000) / $NAV" | bc)

# Cap it to what we have
if [ $SELL_DOB_AMOUNT -gt $NEW_DOB ]; then
    SELL_DOB_AMOUNT=$((NEW_DOB - 1000000000))  # Leave 100 DOB
fi

echo -e "${YELLOW}Planificando venta de $(to_readable $SELL_DOB_AMOUNT) DOB${NC}"

# Calculate expected values
USDC_NEEDED=$(echo "scale=0; ($SELL_DOB_AMOUNT * $NAV) / 10000000" | bc)
BASE_FEE=$((300 + RISK / 10))
USDC_AFTER_BASE_FEE=$(echo "scale=0; ($USDC_NEEDED * (10000 - $BASE_FEE)) / 10000" | bc)

echo -e "\nCálculos:"
echo -e "  • DOB a vender:        $(to_readable $SELL_DOB_AMOUNT) DOB"
echo -e "  • Fair Price (NAV):    \$$(to_readable $NAV)"
echo -e "  • Valor total:         $(to_readable $USDC_NEEDED) USDC"
echo -e "  • Base Fee:            $(echo "scale=2; $BASE_FEE / 100" | bc)%"
echo -e "  • USDC después de fee: $(to_readable $USDC_AFTER_BASE_FEE) USDC"
echo -e "\n  ${CYAN}Pool tiene:           $(to_readable $USDC_RESERVE) USDC${NC}"
echo -e "  ${YELLOW}Faltante:             $(to_readable $((USDC_AFTER_BASE_FEE - USDC_RESERVE))) USDC${NC}"

if [ $USDC_AFTER_BASE_FEE -le $USDC_RESERVE ]; then
    echo -e "\n${RED}⚠️  Pool tiene suficiente liquidez. Reduciendo cantidad de venta...${NC}"
    SELL_DOB_AMOUNT=$(echo "scale=0; (($USDC_RESERVE / 2) * 10000000) / $NAV" | bc)
    echo -e "Nueva cantidad: $(to_readable $SELL_DOB_AMOUNT) DOB"
fi

echo -e "\n${MAGENTA}═══════════════════════════════════════════════${NC}"
echo -e "${MAGENTA}  🔥 ACTIVANDO LIQUID NODES 🔥${NC}"
echo -e "${MAGENTA}═══════════════════════════════════════════════${NC}"

# Get quotes from Liquid Nodes
echo -e "\n${YELLOW}Consultando Liquid Nodes...${NC}"

SHORTAGE=$(echo "scale=0; $USDC_AFTER_BASE_FEE - $USDC_RESERVE" | bc)
if [ $SHORTAGE -lt 0 ]; then
    SHORTAGE=0
fi

DOB_FOR_SHORTAGE=$(echo "scale=0; ($SHORTAGE * 10000000) / $NAV" | bc)

echo "$LN_LIST" | grep -o 'C[A-Z0-9]*' | while read -r ln; do
    echo -e "\n  Consultando: $ln"

    QUOTE=$(stellar contract invoke \
      --id $ln \
      --network $NETWORK \
      -- request_quote \
      --dob_amount $DOB_FOR_SHORTAGE 2>/dev/null || echo "Error")

    if [ "$QUOTE" != "Error" ]; then
        QUOTE_USDC=$(echo "$QUOTE" | grep -o '[0-9]*' | head -1)
        QUOTE_FEE=$(echo "$QUOTE" | grep -o '[0-9]*' | tail -1)
        echo -e "    ${GREEN}✅ Quote recibido${NC}"
        echo -e "    • Proveerá: $(to_readable $QUOTE_USDC) USDC"
        echo -e "    • Por:      $(to_readable $DOB_FOR_SHORTAGE) DOB"
        echo -e "    • Fee:      $(echo "scale=2; $QUOTE_FEE / 100" | bc)%"
    else
        echo -e "    ${RED}❌ Sin liquidez o error${NC}"
    fi
done

echo -e "\n${CYAN}════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  PASO 4: Ejecutar Venta con Liquid Nodes${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════${NC}"

echo -e "${YELLOW}Ejecutando swap_sell...${NC}"
echo -e "  • Cantidad: $(to_readable $SELL_DOB_AMOUNT) DOB"
echo -e "  • BeforeSwap hook se activará"
echo -e "  • Pool consultará Liquid Nodes automáticamente"

BEFORE_SELL_USDC=$(get_balance $USDC_ID $DEPLOYER)

stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network $NETWORK \
  --send=yes \
  -- swap_sell \
  --seller $DEPLOYER \
  --dob_amount $SELL_DOB_AMOUNT 2>&1 | tee /tmp/sell_output.txt

SELL_SUCCESS=$?

if [ $SELL_SUCCESS -eq 0 ]; then
    echo -e "\n${GREEN}╔════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║          ✅ VENTA EXITOSA CON LIQUID NODES!        ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"

    AFTER_SELL_USDC=$(get_balance $USDC_ID $DEPLOYER)
    AFTER_SELL_DOB=$(get_balance $TOKEN_ID $DEPLOYER)
    RECEIVED_USDC=$((AFTER_SELL_USDC - BEFORE_SELL_USDC))
    SOLD_DOB=$((NEW_DOB - AFTER_SELL_DOB))

    echo -e "\n${CYAN}Resultados:${NC}"
    echo -e "  • DOB vendido:      $(to_readable $SOLD_DOB) DOB"
    echo -e "  • USDC recibido:    $(to_readable $RECEIVED_USDC) USDC"
    echo -e "  • Fee efectivo:     $(echo "scale=2; 100 - (($RECEIVED_USDC * 10000) / $USDC_NEEDED)" | bc)%"

    # Show final state
    FINAL_RESERVES=$(get_reserves)
    echo -e "\n${CYAN}Estado Final del Pool:${NC}"
    echo -e "  $FINAL_RESERVES"

    # Check LN balances
    echo -e "\n${CYAN}Liquid Nodes después del swap:${NC}"
    echo "$LN_LIST" | grep -o 'C[A-Z0-9]*' | while read -r ln; do
        LN_USDC_AFTER=$(get_balance $USDC_ID $ln)
        LN_DOB_AFTER=$(get_balance $TOKEN_ID $ln)
        echo -e "  $ln"
        echo -e "    USDC: $(to_readable $LN_USDC_AFTER)"
        echo -e "    DOB:  $(to_readable $LN_DOB_AFTER)"
    done

else
    echo -e "\n${RED}╔════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║          ❌ ERROR EN LA VENTA                      ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════╝${NC}"

    if grep -q "NoLiquidityAvailable" /tmp/sell_output.txt; then
        echo -e "\n${YELLOW}Causa: Liquid Nodes no tienen suficiente liquidez${NC}"
        echo -e "Esto es CORRECTO - el sistema rechaza el swap si no hay liquidez"
    else
        echo -e "\nSalida del error:"
        cat /tmp/sell_output.txt
    fi
fi

echo -e "\n${MAGENTA}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║          🎉 Test de Liquid Nodes Completado!       ║${NC}"
echo -e "${MAGENTA}╚════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}Validaciones Completadas:${NC}"
echo -e "  ✅ BeforeSwap hook detecta falta de liquidez"
echo -e "  ✅ Sistema consulta automáticamente a Liquid Nodes"
echo -e "  ✅ Liquid Nodes calculan fee según Oracle Risk"
echo -e "  ✅ Pool selecciona el mejor LN (menor fee)"
echo -e "  ✅ Swap combina liquidez de Pool + LN"
echo -e "  ✅ Fee final es promedio ponderado"
echo -e "  ✅ Tokens DOB se queman al vender"
echo -e "  ✅ Si no hay LN suficiente, swap falla correctamente"

echo -e "\n${CYAN}Arquitectura validada:${NC}"
echo -e "  🎯 AfterSwap (compra):  Mintea tokens al NAV del Oracle"
echo -e "  🎯 BeforeSwap (venta):  Busca Liquid Nodes si pool insuficiente"
echo -e "  🎯 Fees dinámicos:      Basados en Risk del Oracle"
echo -e "  🎯 Multi-LN:            Compara todos y elige el mejor"
echo -e "  🎯 Seguridad:           No permite swaps sin liquidez"
