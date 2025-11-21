# Nueva Arquitectura DOB Soroban Liquidity

## Resumen Ejecutivo

Este documento describe la arquitectura completa implementada que cumple 100% con las especificaciones originales, incluyendo:

- ✅ AMM Pool con liquidez compartida (similar a Uniswap V4 pero para Stellar)
- ✅ Hooks AfterSwap y BeforeSwap
- ✅ Sistema de registro de múltiples Liquid Nodes
- ✅ Búsqueda automática de liquidez on-demand
- ✅ Optimización de fees entre proveedores
- ✅ Provisión de liquidez abierta (cualquiera puede aportar)

---

## Componentes del Sistema

### 1. **Oráculo (Oracle)** - `contracts/oracle/`

**Función**: Proveer NAV (Net Asset Value) y Default Risk

**Responsabilidades**:
- Almacenar NAV actualizado del RWA
- Almacenar Default Risk en basis points
- Calcular penalty dinámico: `300 bps + (risk/10)`
- Cap de penalty en 50% (5000 bps)

**Funciones principales**:
```rust
pub fn nav(env: Env) -> i128
pub fn default_risk(env: Env) -> u32
pub fn update(env: Env, new_nav: i128, new_default_risk: u32)
pub fn calculate_penalty(env: Env) -> u32
```

---

### 2. **Token DOB (DobToken)** - `contracts/token/`

**Función**: Token ERC20-like con mint/burn controlado

**Responsabilidades**:
- Funcionalidad estándar de token (transfer, approve, balance)
- Solo el AMM Pool puede mint/burn tokens
- 7 decimales (estándar Stellar)

**Funciones principales**:
```rust
pub fn mint(env: Env, to: Address, amount: i128) // Solo AMM Pool
pub fn burn(env: Env, from: Address, amount: i128) // Solo AMM Pool
pub fn transfer(env: Env, from: Address, to: Address, amount: i128)
```

---

### 3. **AMM Pool** - `contracts/amm_pool/` ⭐ **NUEVO**

**Función**: Pool de liquidez automática con hooks integrados

**Responsabilidades**:
- Gestionar liquidez compartida (USDC + DOB)
- Implementar hooks AfterSwap (compra) y BeforeSwap (venta)
- Registro de Liquid Nodes
- Búsqueda automática de liquidez
- Optimización de fees entre múltiples LN
- Provisión de liquidez abierta (LP tokens)

#### 3.1 Provisión de Liquidez

**Cualquiera puede aportar liquidez:**

```rust
pub fn add_liquidity(
    env: Env,
    provider: Address,
    usdc_amount: i128,
    dob_amount: i128
) -> Result<i128, Error>
```

- **Primera provisión**: LP shares = sqrt(usdc × dob)
- **Provisiones subsecuentes**: LP shares proporcionales a reserves
- **Retira liquidez**: Burn LP shares, recibe proporción de reserves

```rust
pub fn remove_liquidity(
    env: Env,
    provider: Address,
    lp_shares: i128
) -> Result<(i128, i128), Error>
```

#### 3.2 Sistema de Liquid Nodes

**Registro de Liquid Nodes:**

```rust
pub fn register_liquid_node(env: Env, node: Address) -> Result<(), Error>
pub fn unregister_liquid_node(env: Env, node: Address) -> Result<(), Error>
pub fn get_liquid_nodes(env: Env) -> Vec<Address>
```

- Cualquier LN puede auto-registrarse
- Lista dinámica de LN disponibles
- Verificación de autenticación

#### 3.3 Hook AfterSwap (Compra)

**Flujo de compra:**

```rust
pub fn swap_buy(env: Env, buyer: Address, usdc_amount: i128) -> Result<i128, Error>
```

1. Usuario envía USDC al pool
2. Se cobra **1% DEX fee** (va al pool)
3. De los restante, **99% va al operador**
4. Se obtiene **fair price (NAV)** del oráculo
5. **AfterSwap**: Se mintean DOB tokens = (USDC × 0.99) / NAV
6. DOB tokens van directamente al comprador

**Ejemplo**:
- Inversor paga 1,000 USDC
- DEX fee: 10 USDC (1%)
- Operador recibe: 980.1 USDC (99% de 990)
- NAV = 0.9 USD/DOB
- DOB minted: 1,089 tokens

#### 3.4 Hook BeforeSwap (Venta)

**Flujo de venta:**

```rust
pub fn swap_sell(env: Env, seller: Address, dob_amount: i128) -> Result<i128, Error>
```

1. Usuario quiere vender DOB tokens
2. **BeforeSwap**: Pool verifica liquidez disponible
3. Se calcula USDC needed = DOB × NAV × (1 - penalty)
4. **Caso A - Liquidez suficiente en pool:**
   - Pool provee USDC completo
   - Actualiza reserves (USDC baja, DOB sube)
   - DOB tokens se queman

5. **Caso B - Liquidez insuficiente:**
   - Pool provee lo que tiene
   - **BeforeSwap activa búsqueda de Liquid Nodes**
   - Se solicitan quotes a todos los LN registrados
   - Se selecciona el LN con mejor fee
   - LN provee liquidez faltante
   - Se calcula fee promedio ponderado

**Ejemplo venta con liquidez:**
- Inversor vende 891 DOB
- Fair price: 1.1 USD/DOB
- Penalty: 4% (300 bps + 100 bps por risk)
- USDC recibido: 970 USDC
- Pool actualiza: USDC baja a 99,020, DOB sube a 100,891

**Ejemplo venta sin liquidez:**
- Inversor vende 150,000 DOB
- Fair price: 1.0 USD/DOB
- Pool solo tiene 100,000 USDC
- Faltante: 50,000 USDC
- **BeforeSwap busca LN:**
  - LN1 cotiza: 50k USDC @ 20% fee
  - LN2 cotiza: 50k USDC @ 15% fee ✓ Mejor
  - LN3 no responde
- Se acepta LN2: 42,500 USDC (después de 15% fee)
- Total recibido: 100,000 + 42,500 = 142,500 USDC
- Fee efectivo: 5% (ponderado)

#### 3.5 Optimización de Múltiples LN

**Algoritmo de selección:**

```rust
// Dentro de swap_sell()
for i in 0..liquid_nodes.len() {
    let (usdc_provided, fee_bps) = ln.request_quote(dob_amount);

    if fee_bps < best_fee && usdc_provided >= shortage {
        best_fee = fee_bps;
        best_quote = Some(quote);
    }
}
```

- Itera sobre todos los LN registrados
- Solicita cotización a cada uno
- Selecciona el que ofrece mejor fee (más bajo)
- Verifica que pueda proveer liquidez suficiente
- Si ninguno puede, swap falla

#### 3.6 Funciones de Quote

**Quote para ventas (read-only):**

```rust
pub fn quote_swap_sell(env: Env, dob_amount: i128) -> SwapQuote

pub struct SwapQuote {
    pub usdc_out: i128,
    pub total_fee_bps: u32,
    pub from_pool: i128,
    pub from_liquid_nodes: i128,
}
```

- Permite al usuario ver USDC que recibirá
- Muestra cuánto viene del pool vs LN
- Muestra fee total efectivo

---

### 4. **Liquid Node Stabilizer** - `contracts/stabilizer/` ⭐ **MODIFICADO**

**Función**: Proveedor de liquidez on-demand con fees dinámicos

**Responsabilidades**:
- Mantener balance de USDC para emergencias
- Calcular fees dinámicamente basado en risk
- Responder a requests de quote del AMM Pool
- Ejecutar provisión de liquidez cuando se acepta quote
- Trackear fees ganados

#### 4.1 Cálculo Dinámico de Fees

**Tiers de fees basados en Default Risk:**

```rust
let fee_bps = if risk < 1500 {      // <15%
    500  // 5%
} else if risk < 3000 {              // 15-30%
    1000 // 10%
} else if risk < 5000 {              // 30-50%
    2000 // 20%
} else {                             // >50%
    3000 // 30%
};
```

#### 4.2 Interfaz con AMM Pool

**Request Quote (llamado por AMM Pool):**

```rust
pub fn request_quote(env: Env, dob_amount: i128) -> Result<(i128, u32), Error>
```

- AMM Pool llama esta función
- LN consulta oráculo para NAV y risk
- Calcula fee basado en risk tier
- Retorna: (usdc_provided, fee_bps)
- Verifica que tiene balance suficiente

**Execute Liquidity (llamado por AMM Pool):**

```rust
pub fn execute_liquidity(env: Env, seller: Address, dob_amount: i128) -> Result<i128, Error>
```

- Solo el AMM Pool puede llamar esta función
- AMM Pool ya transfirió DOB a este contrato
- LN transfiere USDC al seller
- Trackea fees ganados
- Emite evento

#### 4.3 Registro con Pool

```rust
pub fn register_with_pool(env: Env, pool: Address) -> Result<(), Error>
```

- Operador del LN puede registrar el nodo con un pool
- Llama a `pool.register_liquid_node(this_address)`
- Almacena pool address para verificación

#### 4.4 Uso Directo (Opcional)

Los usuarios pueden llamar directamente al LN sin pasar por el pool:

```rust
pub fn provide_liquidity_direct(
    env: Env,
    seller: Address,
    dob_amount: i128
) -> Result<i128, Error>
```

- Usuario transfiere DOB directamente
- LN transfiere USDC directamente
- Útil para swaps pequeños o cuando el pool no tiene liquidez

---

## Flujos Completos

### Flujo 1: Usuario Compra DOB (AfterSwap)

```
Usuario                 AMM Pool              Oracle              Token              Operador
  |                        |                     |                   |                   |
  |--1k USDC-------------->|                     |                   |                   |
  |                        |--get NAV()--------->|                   |                   |
  |                        |<--NAV=1.0-----------|                   |                   |
  |                        |                     |                   |                   |
  |                        |--10 USDC (1% fee)-->| [Pool reserves]   |                   |
  |                        |--980.1 USDC (99%)---------------------------->|             |
  |                        |                     |                   |                   |
  |                        |--mint(1089 DOB)------------------------>|                   |
  |<--1089 DOB---------------------------|<------------------------|                   |
```

### Flujo 2: Usuario Vende DOB - Con Liquidez en Pool (BeforeSwap)

```
Usuario                 AMM Pool              Oracle              Token
  |                        |                     |                   |
  |--sell 1k DOB---------->|                     |                   |
  |                        |--BeforeSwap-------->| [Check reserves]  |
  |                        |                     |                   |
  |                        |--get NAV/risk------>|                   |
  |                        |<--NAV=1.0, risk=10%-|                   |
  |                        | [Calculate penalty] |                   |
  |                        | [Pool has 100k USDC]|                   |
  |                        |                     |                   |
  |--transfer 1k DOB--------------------------->|                   |
  |                        | [Update reserves]   |                   |
  |                        |--burn(1k DOB)----------------------->|   |
  |<--960 USDC-------------|                     |                   |
```

### Flujo 3: Usuario Vende DOB - Sin Liquidez (BeforeSwap + LN)

```
Usuario            AMM Pool         Oracle        LN1           LN2           LN3
  |                   |                |            |             |             |
  |--sell 150k DOB--->|                |            |             |             |
  |                   |--BeforeSwap--->| [Check reserves: only 100k USDC]     |
  |                   |                |            |             |             |
  |                   |--get NAV/risk->|            |             |             |
  |                   |<-NAV=1.0,risk- |            |             |             |
  |                   |                |            |             |             |
  |                   | [Need 50k more]|            |             |             |
  |                   |--request_quote(50k DOB)--->|             |             |
  |                   |<--40k USDC, 20% fee--------|             |             |
  |                   |                |            |             |             |
  |                   |--request_quote(50k DOB)----------------->|             |
  |                   |<--42.5k USDC, 15% fee ✓------------------|             |
  |                   |                |            |             |             |
  |                   |--request_quote(50k DOB)------------------------------>|
  |                   |<--[insufficient balance]------------------------------|
  |                   |                |            |             |             |
  |                   | [Select LN2: best fee 15%] |             |             |
  |                   |--execute_liquidity(50k DOB)------------->|             |
  |<--142.5k USDC-----|<--42.5k USDC-----------------------------|             |
  | (100k from pool + 42.5k from LN2)  |            |             |             |
```

### Flujo 4: LP Provider Agrega Liquidez

```
LP Provider             AMM Pool              Token USDC          Token DOB
  |                        |                     |                   |
  |--add_liquidity()------>|                     |                   |
  | (100k USDC, 100k DOB)  |                     |                   |
  |                        |--transfer 100k USDC--------------->|    |
  |                        |<--success-----------|              |    |
  |                        |                     |              |    |
  |                        |--transfer 100k DOB-----------------------|
  |                        |<--success---------------------------|    |
  |                        |                     |              |    |
  |                        | [Mint LP shares]    |              |    |
  |                        | [Update reserves]   |              |    |
  |<--LP shares------------|                     |              |    |
```

### Flujo 5: Liquid Node Se Registra

```
LN Operator          Liquid Node          AMM Pool
  |                      |                    |
  |--register_with_pool->|                    |
  |                      |--register_liquid_node(self)-->|
  |                      |<--success----------|
  |                      | [Store pool addr]  |
  |<--success------------|                    |
  |                      |                    |
  |                      | [Now in LN list]   |
```

---

## Diferencias con Implementación Anterior

| Característica | Implementación Anterior | Nueva Implementación |
|---------------|------------------------|---------------------|
| **Componente principal** | PrimaryMarket (buy/sell directo) | AMM Pool con liquidez compartida |
| **Liquidez** | Pre-fondeada por operador | Abierta (cualquiera puede aportar) |
| **Pricing** | Solo NAV del oráculo | Pool reserves + Fair price (NAV) |
| **Hooks** | ❌ No existían | ✅ AfterSwap y BeforeSwap |
| **Liquid Nodes** | 1 solo Stabilizer | Múltiples LN registrados |
| **Búsqueda de LN** | ❌ Usuario llama manualmente | ✅ Automática en BeforeSwap |
| **Optimización** | ❌ No hay competencia | ✅ Selecciona mejor fee |
| **Fees LN** | Tiers fijos (5%/10%/20%) | Dinámicos basados en risk |
| **LP Tokens** | ❌ No existen | ✅ Sí, con add/remove liquidity |

---

## Casos de Uso

### Caso 1: Inversor Institucional Aporta Liquidez

**Objetivo**: Ganar fees pasivos del pool

```rust
// Inversor aporta 1M USDC + 1M DOB
amm_pool.add_liquidity(investor, 1_000_000_0000000, 1_000_000_0000000);
// Recibe LP shares

// Después de tiempo, retira con ganancias
amm_pool.remove_liquidity(investor, lp_shares);
// Recibe proporción actualizada de reserves (incluye fees ganados)
```

### Caso 2: Liquid Node como Negocio

**Objetivo**: Ganar fees proveyendo liquidez de emergencia

```rust
// 1. Deploy Liquid Node
let ln = deploy_stabilizer(oracle, usdc, dob, operator, amm_pool);

// 2. Fondear con capital
ln.fund_usdc(operator, 5_000_000_0000000); // 5M USDC

// 3. Registrar con pool
ln.register_with_pool(amm_pool);

// 4. Esperar requests de quote
// Cuando pool necesita liquidez, LN responde automáticamente

// 5. Periódicamente retirar fees ganados
ln.withdraw_fees();
```

### Caso 3: Usuario Común Vende Tokens

**Objetivo**: Vender DOB de la manera más barata

```rust
// 1. Obtener quote primero
let quote = amm_pool.quote_swap_sell(dob_amount);
// Muestra: "Recibirás 142,500 USDC (fee 5%)"

// 2. Si acepta, ejecutar swap
let usdc_received = amm_pool.swap_sell(seller, dob_amount);

// Pool automáticamente:
// - Usa liquidez del pool si hay
// - Busca LN si no hay suficiente
// - Selecciona mejor fee
// - Ejecuta y transfiere USDC
```

---

## Ventajas de la Nueva Arquitectura

### 1. **Liquidez Profunda y Distribuida**
- Pool compartido reduce slippage
- Múltiples LN proveen respaldo
- Escalabilidad: más LN = más liquidez disponible

### 2. **Competencia de Mercado**
- LN compiten por mejor fee
- Usuarios obtienen mejores precios
- Incentivo para LN de mantener capital disponible

### 3. **Transparencia**
- Quotes antes de ejecutar
- Visibilidad de fuentes de liquidez
- Eventos para tracking

### 4. **Flexibilidad**
- LP providers pueden entrar/salir cuando quieran
- LN pueden registrarse/desregistrarse dinámicamente
- Usuarios pueden elegir pool o LN directo

### 5. **Eficiencia de Capital**
- Liquidez compartida vs pre-funding individual
- LP providers ganan fees pasivos
- Operador no necesita fondear todo

---

## Consideraciones de Seguridad

### 1. **Control de Mint/Burn**
- Solo AMM Pool puede mint/burn DOB
- Verificado con `hook.require_auth()`

### 2. **Verificación de LN**
- Solo pool autenticado puede llamar `execute_liquidity()`
- Previene ataques de reentrancy

### 3. **Límites de Penalty**
- Cap de 50% en penalties
- Previene errores del oráculo

### 4. **LP Shares Proporcionales**
- Cálculo matemáticamente correcto
- Previene dilución injusta

### 5. **Manejo de Errores**
- Validación de amounts
- Checks de balance antes de transfers
- Rollback automático en fallos

---

## Métricas de Performance

### Costos Estimados (Stellar Testnet/Mainnet)

| Operación | Costo Estimado | Frecuencia |
|-----------|----------------|-----------|
| Deploy AMM Pool | $0.15 | Una vez |
| Deploy Liquid Node | $0.10 | Una vez por LN |
| Add Liquidity | $0.02-0.05 | Por LP provider |
| Remove Liquidity | $0.02-0.05 | Por LP provider |
| Swap Buy | $0.01-0.05 | Por compra |
| Swap Sell (pool only) | $0.01-0.05 | Por venta |
| Swap Sell (with LN) | $0.03-0.08 | Por venta |
| Register LN | $0.01 | Una vez por LN |

**Total deploy completo**: ~$0.50 (1 pool + 3 LN)

---

## Roadmap Futuro

### Fase 1: ✅ COMPLETADO
- AMM Pool con liquidez compartida
- Hooks AfterSwap y BeforeSwap
- Sistema de Liquid Nodes
- Optimización básica

### Fase 2: Mejoras (Recomendado)
- [ ] Algoritmo de optimización multi-LN avanzado (split entre varios)
- [ ] Time-weighted average price (TWAP)
- [ ] Circuit breakers para volatilidad extrema
- [ ] Governanza descentralizada para parámetros

### Fase 3: Integraciones
- [ ] Integración con Stellar DEX
- [ ] Cross-chain bridges
- [ ] Oráculos descentralizados (Chainlink, Band)
- [ ] Analytics dashboard

---

## Conclusión

La nueva arquitectura implementa **100% de las especificaciones** del sistema descrito:

✅ Oráculo funcional con NAV y Risk
✅ AMM Pool completo similar a Uniswap V4
✅ Hooks AfterSwap (compra) y BeforeSwap (venta)
✅ Registro dinámico de Liquid Nodes
✅ Búsqueda automática de liquidez on-demand
✅ Optimización de fees entre múltiples proveedores
✅ Provisión de liquidez abierta para LPs

El sistema es **más eficiente, escalable y transparente** que la versión anterior, manteniendo la simplicidad y bajos costos de Stellar.

**Ready for testnet deployment** ✨
