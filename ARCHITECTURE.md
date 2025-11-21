# Architecture Documentation

## Overview

The DOB Soroban Liquidity system consists of four main smart contracts that work together to enable tokenization of Real World Assets (RWA) on the Stellar blockchain.

## Contract Architecture

```
                    ┌─────────────────────────────────────┐
                    │         External Users              │
                    │    (Investors, Operators, LPs)      │
                    └─────────────┬───────────────────────┘
                                  │
                    ┌─────────────┴───────────────────────┐
                    │                                     │
            ┌───────▼────────┐                   ┌───────▼────────┐
            │  DobPrimaryMkt │                   │   Stabilizer   │
            │   (Buy/Sell)   │                   │  (LiquidNode)  │
            └───┬────────┬───┘                   └───┬────────┬───┘
                │        │                           │        │
        ┌───────▼──┐  ┌──▼────────┐         ┌───────▼──┐  ┌──▼────────┐
        │ DobToken │  │ DobOracle │         │ DobToken │  │ DobOracle │
        │  (Mint)  │  │  (Query)  │         │ (Store)  │  │  (Query)  │
        └──────────┘  └───────────┘         └──────────┘  └───────────┘
```

## Contract Details

### 1. DobToken

**Purpose**: ERC20-like token contract with controlled minting and burning.

**Key Features**:
- Standard token functions (transfer, approve, balance)
- Controlled minting (only hook/primary market can mint)
- Controlled burning (only hook/primary market can burn)
- Admin management for upgrading hook address

**Storage**:
```rust
Admin           // Contract administrator
Hook            // Authorized contract for mint/burn
Name            // Token name
Symbol          // Token symbol
Decimals        // Token decimals (7 for Stellar)
TotalSupply     // Total minted supply
Balance(addr)   // User balances
Allowance(o,s)  // Transfer allowances
```

**Auth Model**:
- `initialize`: Requires admin auth
- `mint/burn`: Requires hook auth
- `transfer`: Requires sender auth
- `set_hook`: Requires admin auth

### 2. DobOracle

**Purpose**: Simple push oracle for Net Asset Value (NAV) and default risk.

**Key Features**:
- Stores current NAV (with 7 decimals)
- Stores default risk (in basis points)
- Authorized updater can change values
- Automatic penalty calculation based on risk

**Storage**:
```rust
Nav          // Current NAV (7 decimals)
DefaultRisk  // Risk in basis points
Updater      // Authorized updater address
```

**NAV Format**:
- 7 decimals (Stellar standard)
- Example: 10_000_000 = $1.00

**Risk Format**:
- Basis points (10000 = 100%)
- Example: 1000 = 10%

**Auth Model**:
- `initialize`: Requires updater auth
- `update`: Requires updater auth
- `set_updater`: Requires updater auth

### 3. DobPrimaryMarket

**Purpose**: Main liquidity contract handling primary and secondary markets.

**Key Features**:
- Primary market: Buy DOB with USDC at NAV
- Secondary market: Sell DOB for USDC with penalty
- 99/1 split: 99% to operator, 1% fee
- Dynamic penalty based on oracle risk

**Storage**:
```rust
DobToken      // DOB token contract
UsdcToken     // USDC token contract
Oracle        // Oracle contract
Operator      // Revenue recipient
TotalBought   // Cumulative USDC spent
TotalSold     // Cumulative DOB sold
```

**Buy Flow**:
```
1. User transfers USDC to contract
2. Contract queries NAV from oracle
3. 99% of USDC sent to operator
4. DOB minted to user: (USDC × 0.99) / NAV
5. Event emitted
```

**Sell Flow**:
```
1. User requests to sell DOB
2. Contract queries NAV and risk from oracle
3. Penalty calculated: 300 bps + risk/10 (max 5000)
4. DOB burned from user
5. USDC sent: DOB × NAV × (1 - penalty)
6. Event emitted
```

**Auth Model**:
- `initialize`: No auth required (one-time)
- `buy`: Requires buyer auth
- `sell`: Requires seller auth
- `fund`: Requires funder auth

### 4. LiquidNodeStabilizer

**Purpose**: Permissionless liquidity provider offering instant redemptions.

**Key Features**:
- Pre-funded with USDC and DOB
- Tiered fee structure based on risk
- Instant liquidity without penalties
- Fee accumulation for operator

**Storage**:
```rust
Oracle            // Oracle contract
UsdcToken         // USDC token contract
DobToken          // DOB token contract
Operator          // Fee recipient
TotalFeesEarned   // Cumulative fees
```

**Fee Tiers**:
- Low risk (<15%): 5% fee
- Medium risk (<30%): 10% fee
- High risk (≥30%): 20% fee

**Liquidity Flow**:
```
1. User requests instant redemption
2. Contract queries oracle for NAV and risk
3. Fee tier determined based on risk
4. DOB transferred from user to contract
5. USDC sent: DOB × NAV × (1 - fee)
6. Fee tracked for later withdrawal
```

**Auth Model**:
- `initialize`: No auth required (one-time)
- `fund_usdc/fund_dob`: Requires funder auth
- `provide_liquidity`: Requires seller auth
- `withdraw_fees`: Requires operator auth

## Data Flow

### Buy Transaction

```
User Wallet
    │ 1. Approve USDC
    │ 2. Call buy(amount)
    ▼
DobPrimaryMarket
    │ 3. Transfer USDC from user
    │ 4. Query NAV from oracle
    ▼
DobOracle
    │ 5. Return NAV value
    ▼
DobPrimaryMarket
    │ 6. Send 99% USDC to operator
    │ 7. Calculate DOB amount
    │ 8. Call mint()
    ▼
DobToken
    │ 9. Mint DOB to user
    │ 10. Update total supply
    ▼
User Wallet
    │ 11. Receive DOB tokens
```

### Sell Transaction

```
User Wallet
    │ 1. Approve DOB
    │ 2. Call sell(amount)
    ▼
DobPrimaryMarket
    │ 3. Query NAV and risk
    ▼
DobOracle
    │ 4. Return NAV and risk
    ▼
DobPrimaryMarket
    │ 5. Calculate penalty
    │ 6. Calculate USDC out
    │ 7. Call burn()
    ▼
DobToken
    │ 8. Burn DOB from user
    │ 9. Update total supply
    ▼
DobPrimaryMarket
    │ 10. Transfer USDC to user
    ▼
User Wallet
    │ 11. Receive USDC
```

## Security Model

### Access Control

1. **DobToken**
   - Only hook can mint/burn
   - Only admin can update hook
   - Anyone can transfer their own tokens

2. **DobOracle**
   - Only updater can change NAV/risk
   - Only updater can transfer updater role
   - Anyone can query values

3. **DobPrimaryMarket**
   - Anyone can buy/sell (with auth)
   - Operator receives revenue automatically
   - Only contract can mint/burn tokens

4. **LiquidNodeStabilizer**
   - Anyone can provide liquidity (with auth)
   - Only operator can withdraw fees
   - Anyone can fund the pool

### Failure Modes

1. **Oracle Manipulation**
   - Risk: Single updater can set arbitrary values
   - Mitigation: Use multi-sig or DAO for updater role
   - Future: Integrate decentralized price feeds

2. **Insufficient Liquidity**
   - Risk: Primary market runs out of USDC for sells
   - Mitigation: Liquid Nodes provide backup liquidity
   - Monitoring: Track contract USDC balance

3. **Smart Contract Bugs**
   - Risk: Code vulnerabilities could lock funds
   - Mitigation: Comprehensive testing, audits
   - Recovery: Admin upgrade mechanisms

## Performance Considerations

### Gas Optimization (Stroops)

Soroban uses stroops (1/10,000,000 XLM) for transaction costs.

**Estimated Costs** (testnet):
- Deploy contract: ~10-20 XLM
- Initialize: ~1-2 XLM
- Buy transaction: ~0.5-1 XLM
- Sell transaction: ~0.5-1 XLM
- Oracle update: ~0.1-0.3 XLM

### Storage Optimization

- Use `instance()` storage for contract config (cheaper)
- Use `persistent()` storage for balances
- Minimize storage keys where possible

### Contract Size

- Token: ~20 KB WASM
- Oracle: ~15 KB WASM
- Primary Market: ~25 KB WASM
- Stabilizer: ~25 KB WASM

Total: ~85 KB deployed

## Upgrade Strategy

### Contract Upgrades

Soroban contracts can be upgraded if deployer maintains admin rights.

**Upgrade Process**:
1. Deploy new contract version
2. Update references in dependent contracts
3. Migrate data if needed
4. Archive old contract

**Migration Considerations**:
- Token balances persist across upgrades
- Oracle data can be exported/imported
- Primary market stats can be reset or migrated

## Monitoring & Observability

### Key Metrics to Track

1. **Oracle Health**
   - NAV updates frequency
   - Risk level changes
   - Price deviation from market

2. **Primary Market Activity**
   - Buy volume (USDC in)
   - Sell volume (DOB out)
   - Total supply changes
   - Penalty rates applied

3. **Liquid Node Performance**
   - Available liquidity (USDC/DOB)
   - Fees earned
   - Intervention frequency

### Event Monitoring

Track these events for analytics:
- `buy` events - New investments
- `sell` events - Redemptions
- `oracle_updated` - NAV/risk changes
- `liquidity_provided` - LN interventions
- `fees_withdrawn` - Revenue distribution

## Integration Guide

### Frontend Integration

```typescript
// Example using Stellar SDK
import { SorobanRpc, Contract } from 'stellar-sdk';

const server = new SorobanRpc.Server('https://soroban-testnet.stellar.org');
const primaryMarket = new Contract(PRIMARY_MARKET_ID);

// Get quote
const quote = await primaryMarket.call('quote_redemption', dobAmount);

// Execute buy
const tx = await primaryMarket.call('buy', buyer, usdcAmount);
await tx.sign(userKeypair).submit();
```

### Oracle Integration

For production, integrate with:
- Chainlink (when available on Stellar)
- Band Protocol
- Custom oracle network
- DAO-controlled multi-sig

## Future Enhancements

1. **Governance Module**
   - DAO voting for parameter changes
   - Timelocks for critical updates
   - Community proposals

2. **Advanced Risk Models**
   - Time-based penalties
   - Volume-based discounts
   - Dynamic fee structures

3. **Cross-chain Support**
   - Bridge to Ethereum
   - Multi-chain oracle aggregation
   - Unified liquidity pools

4. **DeFi Integration**
   - Stellar DEX integration
   - Liquidity mining rewards
   - Yield farming strategies
