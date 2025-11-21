# Diagramas de Arquitectura DOB Soroban Liquidity

## Vista General del Sistema

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         USUARIOS DEL SISTEMA                             │
├────────────┬──────────────┬─────────────┬──────────────┬───────────────┤
│  Compradores│ Vendedores  │ LP Providers│ LN Operators │   Operador    │
└──────┬──────┴──────┬──────┴──────┬──────┴──────┬───────┴───────┬───────┘
       │             │              │             │               │
       │ buy()       │ sell()       │ add/remove  │ register()    │ update()
       │             │              │ liquidity   │ fund()        │
       ▼             ▼              ▼             ▼               ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                          AMM POOL (Core)                                 │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ Reserves: 100,000 USDC  |  100,000 DOB                          │    │
│  │ LP Shares: 100,000                                               │    │
│  │ Registered LN: [LN1, LN2, LN3]                                   │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐     │
│  │  AfterSwap Hook  │  │  BeforeSwap Hook │  │  LP Management   │     │
│  │   (Compra)       │  │   (Venta)        │  │                  │     │
│  │                  │  │                  │  │  - add_liquidity │     │
│  │  1. Get NAV      │  │  1. Check pool   │  │  - remove_liq    │     │
│  │  2. Mint DOB     │  │  2. If low:      │  │  - calc shares   │     │
│  │  3. Send to user │  │     search LN    │  │                  │     │
│  │  4. USDC to op   │  │  3. Select best  │  │                  │     │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘     │
└──────────┬────────────────────────┬─────────────────────────────────────┘
           │                        │
           │ invoke_contract()      │ invoke_contract()
           ▼                        ▼
┌──────────────────┐    ┌───────────────────────────────────────────────┐
│     ORACLE       │    │        LIQUID NODES REGISTRY                  │
│  ┌────────────┐  │    │  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ NAV: 1.00  │  │    │  │   LN1    │ │   LN2    │ │   LN3    │     │
│  │ Risk: 10%  │  │    │  │  5% fee  │ │  10% fee │ │  20% fee │     │
│  └────────────┘  │    │  │ 100k USDC│ │  50k USDC│ │  25k USDC│     │
│                  │    │  └──────────┘ └──────────┘ └──────────┘     │
│  - update()      │    │                                               │
│  - nav()         │    │  Interfaz:                                    │
│  - risk()        │    │  - request_quote(dob_amount)                  │
│  - calc_penalty()│    │  - execute_liquidity(seller, amount)          │
└──────────────────┘    └───────────────────────────────────────────────┘
           │                                    │
           │ consulta                           │ consulta
           ▼                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                      DOB TOKEN CONTRACT                                  │
│  ┌────────────────────────────────────────────────────────────────┐     │
│  │  Total Supply: 500,000 DOB                                     │     │
│  │  Balances: {user1: 10k, user2: 5k, pool: 100k, ...}           │     │
│  └────────────────────────────────────────────────────────────────┘     │
│                                                                          │
│  Funciones:                                                              │
│  - mint(to, amount)    [SOLO AMM POOL]                                  │
│  - burn(from, amount)  [SOLO AMM POOL]                                  │
│  - transfer(from, to, amount)                                            │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Flujo: Compra de Tokens (AfterSwap)

```
┌────────┐
│ BUYER  │
└───┬────┘
    │
    │ 1. swap_buy(1000 USDC)
    │
    ▼
┌───────────────────────────────────────────────────┐
│              AMM POOL                             │
│                                                   │
│  Step 1: Recibe 1000 USDC del buyer              │
│  ┌──────────────────────────────────────┐        │
│  │ USDC received: 1000                  │        │
│  │ DEX fee (1%): 10 USDC → Pool reserves│        │
│  │ Remaining: 990 USDC                  │        │
│  │ To operator (99%): 980.1 USDC        │        │
│  └──────────────────────────────────────┘        │
│                                                   │
│  Step 2: Consulta NAV al oráculo                 │
│  ┌──────────────────────────────────────┐        │
│  │ oracle.nav() → 1.00 USDC/DOB         │        │
│  └──────────────────────────────────────┘        │
│                 │                                 │
│                 ▼                                 │
│  Step 3: AfterSwap Hook                          │
│  ┌──────────────────────────────────────┐        │
│  │ Calcula DOB a mintear:               │        │
│  │ DOB = 980.1 USDC / 1.00 = 980.1 DOB │        │
│  │                                       │        │
│  │ token.mint(buyer, 980.1 DOB)         │        │
│  └──────────────────────────────────────┘        │
│                 │                                 │
└─────────────────┼─────────────────────────────────┘
                  │
         ┌────────┴────────┐
         ▼                 ▼
┌──────────────┐   ┌──────────────┐
│ DOB TOKEN    │   │  OPERATOR    │
│              │   │              │
│ mint 980 DOB │   │ receives     │
│ to buyer     │   │ 980.1 USDC   │
└──────────────┘   └──────────────┘
```

---

## Flujo: Venta con Liquidez Suficiente (BeforeSwap)

```
┌────────┐
│ SELLER │  Tiene 1000 DOB tokens
└───┬────┘
    │
    │ 1. swap_sell(1000 DOB)
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│                    AMM POOL                              │
│                                                          │
│  Step 1: BeforeSwap - Verificar liquidez                │
│  ┌────────────────────────────────────────┐             │
│  │ Current reserves:                      │             │
│  │ USDC: 150,000  |  DOB: 100,000         │             │
│  │                                         │             │
│  │ Needed: 1000 DOB × 1.00 = 1000 USDC    │             │
│  │ After penalty (4%): 960 USDC           │             │
│  │                                         │             │
│  │ Pool has 150k USDC ✓ SUFFICIENT        │             │
│  └────────────────────────────────────────┘             │
│                                                          │
│  Step 2: Consulta Oracle                                │
│  ┌────────────────────────────────────────┐             │
│  │ NAV: 1.00                              │             │
│  │ Risk: 10% (1000 bps)                   │             │
│  │ Penalty: 300 + 100 = 400 bps (4%)      │             │
│  └────────────────────────────────────────┘             │
│                                                          │
│  Step 3: Ejecutar swap                                  │
│  ┌────────────────────────────────────────┐             │
│  │ 1. Recibir 1000 DOB del seller         │             │
│  │ 2. Actualizar reserves:                │             │
│  │    USDC: 150k → 149,040 (-960)         │             │
│  │    DOB: 100k → 101,000 (+1000)         │             │
│  │ 3. Quemar 1000 DOB tokens              │             │
│  │ 4. Enviar 960 USDC al seller           │             │
│  └────────────────────────────────────────┘             │
└───────────────────────┬──────────────────────────────────┘
                        │
                ┌───────┴────────┐
                ▼                ▼
        ┌──────────────┐  ┌─────────────┐
        │  DOB TOKEN   │  │   SELLER    │
        │              │  │             │
        │ burn 1000 DOB│  │ receives    │
        │              │  │ 960 USDC    │
        └──────────────┘  └─────────────┘
```

---

## Flujo: Venta SIN Liquidez (BeforeSwap + Liquid Nodes)

```
┌────────┐
│ SELLER │  Quiere vender 150,000 DOB
└───┬────┘
    │
    │ 1. swap_sell(150,000 DOB)
    │
    ▼
┌──────────────────────────────────────────────────────────────────┐
│                        AMM POOL                                  │
│                                                                  │
│  Step 1: BeforeSwap - Check liquidez                            │
│  ┌────────────────────────────────────────────────┐             │
│  │ Current reserves:                              │             │
│  │ USDC: 100,000  |  DOB: 100,000                 │             │
│  │                                                 │             │
│  │ Needed: 150k DOB × 1.00 × 0.96 = 144k USDC     │             │
│  │ Pool only has 100k USDC                        │             │
│  │                                                 │             │
│  │ ❌ INSUFFICIENT! Need 44k more                  │             │
│  └────────────────────────────────────────────────┘             │
│                                                                  │
│  Step 2: BeforeSwap triggers LN search                          │
│  ┌────────────────────────────────────────────────┐             │
│  │ Shortage: 44,000 USDC                          │             │
│  │ DOB for shortage: 44,000 / 1.00 = 44,000 DOB   │             │
│  │                                                 │             │
│  │ Query all registered Liquid Nodes:             │             │
│  └────────────────────────────────────────────────┘             │
│                                                                  │
│  Step 3: Recoger quotes de LN                                   │
│  ┌────────────────────────────────────────────────┐             │
│  │ LN1.request_quote(44k DOB)                     │             │
│  │   → 41,800 USDC @ 5% fee                       │             │
│  │                                                 │             │
│  │ LN2.request_quote(44k DOB)  ✓ MEJOR            │             │
│  │   → 39,600 USDC @ 10% fee                      │             │
│  │                                                 │             │
│  │ LN3.request_quote(44k DOB)                     │             │
│  │   → Error: Insufficient balance                │             │
│  └────────────────────────────────────────────────┘             │
│                                                                  │
│  Step 4: Seleccionar mejor LN (LN1: 5% fee)                     │
│  ┌────────────────────────────────────────────────┐             │
│  │ Selected: LN1                                  │             │
│  │ Will provide: 41,800 USDC                      │             │
│  │ Fee: 5%                                        │             │
│  └────────────────────────────────────────────────┘             │
│                                                                  │
│  Step 5: Ejecutar swap combinado                                │
│  ┌────────────────────────────────────────────────┐             │
│  │ A) From Pool:                                  │             │
│  │    - Seller → 100,000 DOB → Pool               │             │
│  │    - Pool → 100,000 USDC → Seller              │             │
│  │    - Update reserves: USDC=0, DOB=200k         │             │
│  │                                                 │             │
│  │ B) From LN1:                                   │             │
│  │    - Seller → 44,000 DOB → LN1                 │             │
│  │    - LN1 → 41,800 USDC → Seller                │             │
│  │                                                 │             │
│  │ C) Burn total:                                 │             │
│  │    - token.burn(seller, 150,000 DOB)           │             │
│  │                                                 │             │
│  │ Total received by seller:                      │             │
│  │    100,000 + 41,800 = 141,800 USDC             │             │
│  │                                                 │             │
│  │ Effective fee:                                 │             │
│  │    Expected: 144k, Got: 141.8k                 │             │
│  │    Fee: (144k - 141.8k) / 144k = 1.5%          │             │
│  └────────────────────────────────────────────────┘             │
└───────────────────┬──────────────────────────────────────────────┘
                    │
        ┌───────────┼────────────┐
        ▼           ▼            ▼
┌─────────────┐ ┌──────────┐ ┌─────────┐
│  DOB TOKEN  │ │   LN1    │ │ SELLER  │
│             │ │          │ │         │
│ burn 150k   │ │ receives │ │ receives│
│ DOB         │ │ 44k DOB  │ │ 141.8k  │
│             │ │          │ │ USDC    │
│             │ │ fee: 2.2k│ │         │
└─────────────┘ └──────────┘ └─────────┘
```

---

## Diagrama: Competencia entre Múltiples Liquid Nodes

```
┌─────────────────────────────────────────────────────────────┐
│                  AMM POOL                                   │
│  "Necesito 50,000 USDC para completar un swap"             │
└───────────┬─────────────────────────────────────────────────┘
            │
            │ request_quote(50k DOB) to all registered LN
            │
    ┌───────┼───────┬──────────┬──────────┐
    │       │       │          │          │
    ▼       ▼       ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│  LN1   │ │  LN2   │ │  LN3   │ │  LN4   │
│        │ │        │ │        │ │        │
│ Balance│ │ Balance│ │ Balance│ │ Balance│
│ 100k   │ │ 200k   │ │ 50k    │ │ 10k    │
│        │ │        │ │        │ │        │
│ Risk:  │ │ Risk:  │ │ Risk:  │ │ Risk:  │
│ 10%    │ │ 10%    │ │ 10%    │ │ 10%    │
│        │ │        │ │        │ │        │
│ Fee:   │ │ Fee:   │ │ Fee:   │ │ Fee:   │
│ 5%     │ │ 5%     │ │ 5%     │ │ 5%     │
└───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘
    │          │          │          │
    │ Quote:   │ Quote:   │ Quote:   │ Quote:
    │ 47.5k    │ 47.5k    │ 47.5k    │ Error:
    │ USDC     │ USDC     │ USDC     │ Insuf.
    │          │          │          │ balance
    ▼          ▼          ▼          ▼
┌─────────────────────────────────────────────────────────────┐
│                  AMM POOL                                   │
│  Algoritmo de selección:                                    │
│                                                             │
│  1. Recoger todas las quotes válidas                        │
│  2. Filtrar: Solo las que pueden proveer >= 50k USDC       │
│     → LN1: 47.5k ✗ (insuficiente)                          │
│     → LN2: 47.5k ✗ (insuficiente)                          │
│     → LN3: 47.5k ✗ (insuficiente)                          │
│     → LN4: Error ✗                                          │
│                                                             │
│  3. Si nadie puede proveer suficiente individualmente,      │
│     → podría combinar múltiples (feature futura)            │
│     → por ahora, swap falla                                 │
│                                                             │
│  Si hubiera uno con balance suficiente:                     │
│  4. Seleccionar el con menor fee_bps                        │
│  5. Llamar execute_liquidity() en el ganador                │
└─────────────────────────────────────────────────────────────┘
```

---

## Arquitectura de Datos

```
┌────────────────────────────────────────────────────────────┐
│                    AMM POOL STORAGE                        │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Instance Storage (Contract-level):                        │
│  ┌──────────────────────────────────────────────────┐     │
│  │ DobToken: Address                                │     │
│  │ UsdcToken: Address                               │     │
│  │ Oracle: Address                                  │     │
│  │ Operator: Address                                │     │
│  │ UsdcReserve: 100,000_0000000 (i128)             │     │
│  │ DobReserve: 100,000_0000000 (i128)              │     │
│  │ TotalLpShares: 100,000_0000000 (i128)           │     │
│  │ LiquidNodes: Vec<Address> [ln1, ln2, ln3]       │     │
│  │ TotalBought: 500,000_0000000 (i128)             │     │
│  │ TotalSold: 200,000_0000000 (i128)               │     │
│  │ DexFeeCollected: 5,000_0000000 (i128)           │     │
│  └──────────────────────────────────────────────────┘     │
│                                                            │
│  Persistent Storage (Per-user):                            │
│  ┌──────────────────────────────────────────────────┐     │
│  │ LpShares(user1): 50,000_0000000                  │     │
│  │ LpShares(user2): 30,000_0000000                  │     │
│  │ LpShares(user3): 20,000_0000000                  │     │
│  │ ...                                              │     │
│  └──────────────────────────────────────────────────┘     │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│               LIQUID NODE STORAGE                          │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Instance Storage:                                         │
│  ┌──────────────────────────────────────────────────┐     │
│  │ Oracle: Address                                  │     │
│  │ UsdcToken: Address                               │     │
│  │ DobToken: Address                                │     │
│  │ Operator: Address                                │     │
│  │ AmmPool: Address (registered pool)               │     │
│  │ TotalFeesEarned: 10,000_0000000 (i128)          │     │
│  └──────────────────────────────────────────────────┘     │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│                    ORACLE STORAGE                          │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Instance Storage:                                         │
│  ┌──────────────────────────────────────────────────┐     │
│  │ Nav: 10_000_000 (i128) = 1.00 USD                │     │
│  │ DefaultRisk: 1000 (u32) = 10%                    │     │
│  │ Updater: Address                                 │     │
│  └──────────────────────────────────────────────────┘     │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│                   DOB TOKEN STORAGE                        │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Instance Storage:                                         │
│  ┌──────────────────────────────────────────────────┐     │
│  │ Admin: Address                                   │     │
│  │ Hook: Address (AMM Pool)                         │     │
│  │ Name: "Dob Solar Farm 2035"                      │     │
│  │ Symbol: "DOB-35"                                 │     │
│  │ Decimals: 7                                      │     │
│  │ TotalSupply: 500,000_0000000 (i128)             │     │
│  └──────────────────────────────────────────────────┘     │
│                                                            │
│  Persistent Storage (Per-user):                            │
│  ┌──────────────────────────────────────────────────┐     │
│  │ Balance(user1): 10,000_0000000                   │     │
│  │ Balance(user2): 5,000_0000000                    │     │
│  │ Balance(pool): 100,000_0000000                   │     │
│  │ Allowance(user1, spender): 1,000_0000000        │     │
│  │ ...                                              │     │
│  └──────────────────────────────────────────────────┘     │
└────────────────────────────────────────────────────────────┘
```

---

## Eventos Emitidos

```
AMM POOL EVENTS:
├─ liquidity_added(provider, usdc_amount, dob_amount, lp_shares)
├─ liquidity_removed(provider, usdc_amount, dob_amount, lp_shares)
├─ swap_buy(buyer, usdc_in, dob_out, fair_price, pool_price)
├─ swap_sell(seller, dob_in, usdc_out, fair_price, pool_price, fee_bps, liquid_nodes_used)
├─ ln_registered(node_address)
└─ ln_unregistered(node_address)

LIQUID NODE EVENTS:
├─ funded_usdc(funder, amount)
├─ funded_dob(funder, amount)
├─ liquidity_provided(seller, dob_amount, usdc_provided, fee_bps)
├─ fees_withdrawn(operator, amount)
└─ registered_with_pool(pool_address)

ORACLE EVENTS:
├─ initialized(nav, default_risk)
├─ oracle_updated(nav, default_risk)
└─ updater_changed(old_updater, new_updater)

TOKEN EVENTS:
├─ mint(to, amount)
├─ burn(from, amount)
├─ transfer(from, to, amount)
└─ approve(owner, spender, amount)
```

---

## Resumen de Interfaces

```rust
// AMM POOL
interface IAmmPool {
    // Liquidez
    fn add_liquidity(provider, usdc_amount, dob_amount) -> lp_shares;
    fn remove_liquidity(provider, lp_shares) -> (usdc_out, dob_out);

    // Trading
    fn swap_buy(buyer, usdc_amount) -> dob_out;
    fn swap_sell(seller, dob_amount) -> usdc_out;
    fn quote_swap_sell(dob_amount) -> SwapQuote;

    // Liquid Nodes
    fn register_liquid_node(node);
    fn unregister_liquid_node(node);
    fn get_liquid_nodes() -> Vec<Address>;

    // Queries
    fn get_reserves() -> (usdc, dob);
    fn get_lp_shares(provider) -> i128;
    fn get_stats() -> (bought, sold, fees);
}

// LIQUID NODE
interface ILiquidNode {
    // Called by AMM Pool
    fn request_quote(dob_amount) -> (usdc_provided, fee_bps);
    fn execute_liquidity(seller, dob_amount) -> usdc_out;

    // Direct use
    fn provide_liquidity_direct(seller, dob_amount) -> usdc_out;
    fn quote_liquidity_direct(dob_amount) -> (usdc, fee);

    // Management
    fn fund_usdc(funder, amount);
    fn fund_dob(funder, amount);
    fn withdraw_fees() -> amount;
    fn register_with_pool(pool);
}

// ORACLE
interface IOracle {
    fn nav() -> i128;
    fn default_risk() -> u32;
    fn calculate_penalty() -> u32;
    fn update(new_nav, new_risk);
}

// TOKEN
interface IToken {
    fn mint(to, amount);  // Only hook
    fn burn(from, amount);  // Only hook
    fn transfer(from, to, amount);
    fn approve(owner, spender, amount);
    fn balance(account) -> i128;
}
```

---

Este documento proporciona una visión visual completa de cómo interactúan todos los componentes del sistema DOB Soroban Liquidity.
