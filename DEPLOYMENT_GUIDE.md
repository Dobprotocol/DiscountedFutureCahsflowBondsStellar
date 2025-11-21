# Guía de Deployment - Nueva Arquitectura

## ✅ Script Actualizado

El script `scripts/deploy-and-test.sh` ha sido actualizado para usar la **nueva arquitectura con AMM Pool y Liquid Nodes**.

### 🔧 Cambios Realizados

#### 1. **Fix Crítico**
- ✅ **Corregido**: Error de instalación de `stellar-cli`
  - **Antes**: `cargo install --locked stellar-cli --features opt` ❌
  - **Ahora**: `cargo install --locked stellar-cli` ✅

#### 2. **Nueva Arquitectura**
- ✅ Deploy de **AMM Pool** (en lugar de Primary Market)
- ✅ Deploy de **2 Liquid Nodes** (en lugar de 1 Stabilizer)
- ✅ Registro automático de LN con el Pool
- ✅ Funding de 50k USDC a cada LN
- ✅ Provisión de liquidez inicial al Pool (10k USDC + 10k DOB)

#### 3. **Nuevos Tests**
- ✅ Test de `swap_buy()` (AfterSwap hook)
- ✅ Test de `swap_sell()` (BeforeSwap hook)
- ✅ Test de `get_reserves()` - Ver liquidez del pool
- ✅ Test de `get_liquid_nodes()` - Ver LN registrados
- ✅ Test de `quote_swap_sell()` - Cotizaciones

---

## 🚀 Cómo Usar

### Opción 1: Deploy Automático Completo

```bash
./scripts/deploy-and-test.sh
```

Este script hará:
1. ✅ Instalar `stellar-cli` si no existe
2. ✅ Compilar todos los contratos
3. ✅ Crear identidad de testnet
4. ✅ Fondear cuenta con Friendbot
5. ✅ Desplegar 6 contratos:
   - DOB Token
   - Oracle
   - AMM Pool
   - Liquid Node #1
   - Liquid Node #2
   - USDC (testnet)
6. ✅ Inicializar todos los contratos
7. ✅ Fondear Liquid Nodes con USDC
8. ✅ Registrar LN con el Pool
9. ✅ Agregar liquidez inicial al Pool
10. ✅ Ejecutar 8 tests de integración

**Duración estimada**: 3-5 minutos

---

### Opción 2: Deploy Manual Paso a Paso

#### Paso 1: Compilar
```bash
cargo build --release --target wasm32-unknown-unknown
```

#### Paso 2: Deploy Contratos
```bash
# Deploy Token
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_token.wasm \
  --source deployer \
  --network testnet

# Deploy Oracle
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle.wasm \
  --source deployer \
  --network testnet

# Deploy AMM Pool
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_amm_pool.wasm \
  --source deployer \
  --network testnet

# Deploy Liquid Nodes (repetir 2 veces)
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/liquid_node_stabilizer.wasm \
  --source deployer \
  --network testnet
```

#### Paso 3: Inicializar (ver script para detalles)

#### Paso 4: Fondear y Registrar

---

## 📊 Estructura del Deployment

```
┌─────────────────────────────────────────────────────┐
│                   TESTNET DEPLOYMENT                │
├─────────────────────────────────────────────────────┤
│                                                     │
│  1. DOB Token (7 decimals)                          │
│     - Hook: AMM Pool                                │
│                                                     │
│  2. Oracle                                          │
│     - NAV: $1.00                                    │
│     - Risk: 10%                                     │
│                                                     │
│  3. AMM Pool ⭐ MAIN CONTRACT                       │
│     - Reserves: 10k USDC + 10k DOB                  │
│     - Registered LN: [LN1, LN2]                     │
│                                                     │
│  4. Liquid Node #1                                  │
│     - Balance: 50k USDC                             │
│     - Registered with Pool                          │
│                                                     │
│  5. Liquid Node #2                                  │
│     - Balance: 50k USDC                             │
│     - Registered with Pool                          │
│                                                     │
│  6. USDC Token (testnet)                            │
│     - Minted to deployer: 200k                      │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

## 🧪 Tests Incluidos

El script ejecuta 8 tests automáticamente:

### Test 1: Compra con AfterSwap Hook
```
Comprar 1000 USDC de DOB
→ AfterSwap mint directo
→ Verificar DOB recibido
```

### Test 2: Verificar Balance
```
Verificar que el comprador recibió DOB tokens
```

### Test 3: Ver Reserves del Pool
```
Verificar liquidez disponible en el pool
→ USDC reserve
→ DOB reserve
```

### Test 4: Cotizar Venta
```
Obtener quote para vender 500 DOB
→ Muestra USDC que recibiría
→ Muestra fee aplicado
→ Muestra si usará LN
```

### Test 5: Venta con BeforeSwap Hook
```
Vender 500 DOB
→ BeforeSwap verifica liquidez
→ Pool o LN proveen USDC
→ Verificar USDC recibido
```

### Test 6: Ver Liquid Nodes Registrados
```
Listar todos los LN registrados en el pool
→ Debería mostrar LN1 y LN2
```

### Test 7: Actualizar Oracle
```
Cambiar NAV a $1.20 y Risk a 5%
→ Afecta próximas transacciones
```

### Test 8: Compra con Nuevo NAV
```
Comprar 500 USDC de DOB al nuevo precio
→ Recibe menos DOB por el nuevo NAV
```

---

## 📁 Archivos Generados

Después del deployment, se crea:

### `deployed-contracts.env`
```bash
export TOKEN_ID=CA...
export ORACLE_ID=CB...
export POOL_ID=CC...
export LN1_ID=CD...
export LN2_ID=CE...
export USDC_ID=CF...
export DEPLOYER=GA...
export NETWORK=testnet
```

**Uso:**
```bash
source deployed-contracts.env

# Ahora puedes usar las variables
stellar contract invoke --id $POOL_ID --network $NETWORK -- get_reserves
```

---

## 🔍 Verificar Deployment

### En Stellar Expert

Después del deploy, visita:

**AMM Pool:**
```
https://stellar.expert/explorer/testnet/contract/[POOL_ID]
```

**Token:**
```
https://stellar.expert/explorer/testnet/contract/[TOKEN_ID]
```

**Tu Cuenta:**
```
https://stellar.expert/explorer/testnet/account/[DEPLOYER]
```

### Con CLI

**Ver reserves del pool:**
```bash
stellar contract invoke \
  --id $POOL_ID \
  --network testnet \
  -- get_reserves
```

**Ver Liquid Nodes registrados:**
```bash
stellar contract invoke \
  --id $POOL_ID \
  --network testnet \
  -- get_liquid_nodes
```

**Ver balance de DOB:**
```bash
stellar contract invoke \
  --id $TOKEN_ID \
  --network testnet \
  -- balance \
  --account $DEPLOYER
```

**Cotizar venta:**
```bash
stellar contract invoke \
  --id $POOL_ID \
  --network testnet \
  -- quote_swap_sell \
  --dob_amount 10000000000
```

---

## 🎯 Ejemplos de Uso Post-Deployment

### Comprar DOB Tokens
```bash
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network testnet \
  --send=yes \
  -- swap_buy \
  --buyer $DEPLOYER \
  --usdc_amount 10000000000
```

### Vender DOB Tokens
```bash
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network testnet \
  --send=yes \
  -- swap_sell \
  --seller $DEPLOYER \
  --dob_amount 10000000000
```

### Agregar Liquidez al Pool
```bash
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network testnet \
  --send=yes \
  -- add_liquidity \
  --provider $DEPLOYER \
  --usdc_amount 10000000000 \
  --dob_amount 10000000000
```

### Registrar Nuevo Liquid Node
```bash
# Primero deploy un nuevo LN
NEW_LN=$(stellar contract deploy --wasm target/.../liquid_node_stabilizer.wasm ...)

# Inicializar
stellar contract invoke --id $NEW_LN ... -- initialize ...

# Fondear
stellar contract invoke --id $NEW_LN ... -- fund_usdc ...

# Registrar con pool
stellar contract invoke \
  --id $POOL_ID \
  --source deployer \
  --network testnet \
  --send=yes \
  -- register_liquid_node \
  --node $NEW_LN
```

---

## ⚠️ Troubleshooting

### Error: "stellar: command not found"
```bash
cargo install --locked stellar-cli
```

### Error: "wasm file not found"
```bash
cargo build --release --target wasm32-unknown-unknown
```

### Error: "insufficient balance"
```bash
# Fondea tu cuenta con Friendbot
curl "https://friendbot.stellar.org?addr=$DEPLOYER"
```

### Error: "contract not found"
```bash
# Verifica que los contratos fueron deployed
stellar contract id wasm --wasm target/.../contract.wasm
```

---

## 📚 Documentación Adicional

- **`NEW_ARCHITECTURE.md`** - Arquitectura completa del sistema
- **`IMPLEMENTATION_SUMMARY.md`** - Resumen ejecutivo
- **`ARCHITECTURE_DIAGRAM.md`** - Diagramas visuales
- **`QUICK_DEPLOY.md`** - Guía rápida (legacy)

---

## 🎉 Próximos Pasos

1. ✅ **Deploy completado** - Contratos en testnet
2. 🧪 **Testing** - Probar todas las funciones
3. 👥 **Invitar usuarios** - Compartir links de Stellar Expert
4. 📊 **Monitorear** - Ver transacciones y eventos
5. 🚀 **Mainnet** - Cuando esté listo, deploy a producción

---

**¿Preguntas?** Ver documentación completa en `NEW_ARCHITECTURE.md`

**Ready to deploy!** ✨
