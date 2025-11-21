# DOB Soroban Liquidity

A Stellar Soroban smart contract system for tokenized Real World Assets (RWA) that creates an infinite primary market with protected secondary market liquidity.

> **Converted from Solidity/Uniswap V4 to Stellar Soroban**
>
> This is a Soroban implementation of the DobNodeLiquidity system, originally built on Ethereum using Uniswap V4 hooks. The core mechanics remain the same, adapted for the Stellar ecosystem.

## Overview

DobNodeLiquidity enables tokenization of revenue-generating assets (e.g., solar farms) with:

- **Primary Market**: Mint tokens at oracle NAV (99% to operator, 1% fee)
- **Secondary Market**: Redeem tokens at NAV minus dynamic penalty based on default risk
- **Liquid Nodes**: Permissionless liquidity providers offering instant redemption at competitive rates
- **Oracle-based Pricing**: Simple push oracle for NAV and default risk updates

## Architecture

```
┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│  DobOracle  │◄────│  PrimaryMarket  │────►│  DobToken   │
│  (NAV/Risk) │     │   (Buy/Sell)    │     │ (SAC Token) │
└─────────────┘     └─────────────────┘     └─────────────┘
                           ▲
                           │
                    ┌──────┴──────┐
                    │  Stabilizer │
                    │ (LiquidNode)│
                    └─────────────┘
```

## Smart Contracts

| Contract | Description | Location |
|----------|-------------|----------|
| `DobToken` | Soroban Asset Contract (SAC) compatible token with controlled mint/burn | `contracts/token/` |
| `DobOracle` | Push oracle storing NAV and default risk | `contracts/oracle/` |
| `DobPrimaryMarket` | Primary market for buying/selling DOB tokens | `contracts/primary_market/` |
| `LiquidNodeStabilizer` | Instant liquidity provider with tiered fees | `contracts/stabilizer/` |

## How It Works

### Buying DOB Tokens (Primary Market)

1. User sends USDC to the primary market contract
2. Contract reads current NAV from oracle
3. 99% of USDC goes to operator, 1% fee retained
4. DOB tokens minted to user at NAV rate

```
DOB received = (USDC × 0.99) / NAV
```

**Example:**
- User sends 1,000 USDC
- NAV = 1.00 (10_000_000 with 7 decimals)
- Operator receives: 990 USDC
- User receives: 990 DOB tokens

### Selling DOB Tokens (Secondary Market)

1. User sends DOB tokens to redeem
2. Contract calculates dynamic penalty based on default risk:
   - Base penalty: 3%
   - Risk adjustment: +risk/1000
   - Maximum: 50%
3. USDC returned minus penalty

```
Penalty BPS = min(300 + defaultRisk/10, 5000)
USDC received = DOB × NAV × (1 - penalty)
```

**Example:**
- User sells 500 DOB
- NAV = 1.00, Default Risk = 10% (1000 bps)
- Penalty = 300 + 100 = 400 bps (4%)
- User receives: 500 × 1.00 × 0.96 = 480 USDC

### Liquid Nodes (Instant Liquidity)

For users wanting instant liquidity without penalties, Liquid Nodes compete to provide redemptions:

1. LiquidNode queries oracle for current NAV and risk
2. Calculates fee based on risk tier:
   - Low risk (<15%): 5% fee
   - Medium risk (<30%): 10% fee
   - High risk (≥30%): 20% fee
3. User can choose the best offer

**Example:**
- User wants to redeem 1,000 DOB instantly
- NAV = 1.15, Risk = 7% (low tier)
- LiquidNode offers: 1,000 × 1.15 × 0.95 = 1,092.50 USDC
- User receives instant liquidity at 5% fee vs 4% penalty + wait time

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)
- [Soroban SDK](https://soroban.stellar.org/docs/getting-started/setup)

## Installation

### 1. Install Rust and Stellar CLI

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

### 2. Clone and Build

```bash
cd dob-soroban-liquidity

# Build all contracts
make build

# Optimize WASM files
make optimize

# Run tests
make test
```

## Development

### Building Contracts

```bash
# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Or use make
make build
```

Compiled WASM files will be in `target/wasm32-unknown-unknown/release/`

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific contract
cargo test -p dob-token
cargo test -p dob-oracle
```

### Optimizing WASM

```bash
# Optimize all contracts
make optimize

# Or manually optimize a single contract
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/dob_token.wasm
```

## Deployment

### Local Development Network

1. Start a local Stellar network (using Docker):

```bash
docker run --rm -it \
  -p 8000:8000 \
  --name stellar \
  stellar/quickstart:soroban-dev@sha256:a057ec6f06c6702c005693f8265ed1261387b4f29ba0fe48399e24d047c09ead \
  --standalone \
  --enable-soroban-rpc
```

2. Deploy contracts:

```bash
# Set your source account
export SOURCE_ACCOUNT=GDIY6AQQ75WMD4W46EYB7O6UYMHOCGQHLAQGQTKHDX4J2DYQCHVCR4W4

# Deploy
./scripts/deploy-local.sh
```

### Stellar Testnet

1. Get testnet XLM from the [Friendbot](https://laboratory.stellar.org/#account-creator?network=test)

2. Deploy to testnet:

```bash
# Set your source account
export SOURCE_ACCOUNT=GXXX...YOUR_TESTNET_ACCOUNT

# Deploy
./scripts/deploy-testnet.sh
```

Contract addresses will be saved to `deployed-addresses-testnet.json`

### Mainnet Deployment

**⚠️ Important:** Thoroughly test on testnet before deploying to mainnet!

```bash
export SOURCE_ACCOUNT=GXXX...YOUR_MAINNET_ACCOUNT

# Build and optimize
make all

# Deploy contracts manually using stellar-cli
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_token_optimized.wasm \
  --source "$SOURCE_ACCOUNT" \
  --network mainnet
```

## Initialization

After deploying contracts, initialize them in this order:

### 1. Initialize Oracle

```bash
stellar contract invoke \
  --id <ORACLE_CONTRACT_ID> \
  --source <YOUR_ACCOUNT> \
  --network testnet \
  -- initialize \
  --updater <YOUR_ACCOUNT> \
  --initial_nav 10000000 \
  --initial_risk 1000
```

- `initial_nav`: 10000000 = 1.00 USDC (7 decimals)
- `initial_risk`: 1000 = 10% (basis points)

### 2. Initialize Token

```bash
stellar contract invoke \
  --id <TOKEN_CONTRACT_ID> \
  --source <YOUR_ACCOUNT> \
  --network testnet \
  -- initialize \
  --admin <YOUR_ACCOUNT> \
  --hook <PRIMARY_MARKET_CONTRACT_ID> \
  --name "Dob Solar Farm 2035" \
  --symbol "DOB-35" \
  --decimals 7
```

### 3. Initialize Primary Market

```bash
stellar contract invoke \
  --id <PRIMARY_MARKET_CONTRACT_ID> \
  --source <YOUR_ACCOUNT> \
  --network testnet \
  -- initialize \
  --dob_token <TOKEN_CONTRACT_ID> \
  --usdc_token <USDC_CONTRACT_ID> \
  --oracle <ORACLE_CONTRACT_ID> \
  --operator <OPERATOR_ACCOUNT>
```

### 4. Initialize Stabilizer

```bash
stellar contract invoke \
  --id <STABILIZER_CONTRACT_ID> \
  --source <YOUR_ACCOUNT> \
  --network testnet \
  -- initialize \
  --oracle <ORACLE_CONTRACT_ID> \
  --usdc_token <USDC_CONTRACT_ID> \
  --dob_token <TOKEN_CONTRACT_ID> \
  --operator <OPERATOR_ACCOUNT>
```

## Usage Examples

### Buying DOB Tokens

```bash
stellar contract invoke \
  --id <PRIMARY_MARKET_CONTRACT_ID> \
  --source <BUYER_ACCOUNT> \
  --network testnet \
  -- buy \
  --buyer <BUYER_ACCOUNT> \
  --usdc_amount 10000000
```

### Selling DOB Tokens

```bash
stellar contract invoke \
  --id <PRIMARY_MARKET_CONTRACT_ID> \
  --source <SELLER_ACCOUNT> \
  --network testnet \
  -- sell \
  --seller <SELLER_ACCOUNT> \
  --dob_amount 5000000
```

### Getting a Quote

```bash
stellar contract invoke \
  --id <PRIMARY_MARKET_CONTRACT_ID> \
  --network testnet \
  -- quote_redemption \
  --dob_amount 10000000
```

### Updating Oracle (Operator Only)

```bash
stellar contract invoke \
  --id <ORACLE_CONTRACT_ID> \
  --source <UPDATER_ACCOUNT> \
  --network testnet \
  -- update \
  --new_nav 11500000 \
  --new_default_risk 700
```

- `new_nav`: 11500000 = 1.15 USDC (7 decimals)
- `new_default_risk`: 700 = 7% (basis points)

## Key Differences from Solidity Version

### Architecture Changes

1. **No Uniswap V4 Hooks**: Direct contract calls replace hook mechanism
2. **SAC Integration**: Uses Soroban Asset Contract (SAC) standard for tokens
3. **Simplified AMM**: No pool manager - direct buy/sell from primary market
4. **Native Oracle**: Simple push oracle instead of external oracle integration

### Token Decimals

- Stellar uses **7 decimals** by default (Soroban standard)
- Ethereum version used **18 decimals**
- All calculations adjusted accordingly

### Authentication

- Uses Soroban's `require_auth()` instead of Solidity's `msg.sender` checks
- Contract addresses verified through Soroban's built-in auth system

### Events

- Soroban events use topic-based publishing
- Different format than Solidity events but same semantic meaning

## Contract Functions

### DobToken

```rust
initialize(admin, hook, name, symbol, decimals)
mint(to, amount)          // Hook only
burn(from, amount)        // Hook only
transfer(from, to, amount)
approve(owner, spender, amount)
balance(account)
```

### DobOracle

```rust
initialize(updater, initial_nav, initial_risk)
nav() -> i128
default_risk() -> u32
update(new_nav, new_default_risk)
calculate_penalty() -> u32
```

### DobPrimaryMarket

```rust
initialize(dob_token, usdc_token, oracle, operator)
buy(buyer, usdc_amount) -> i128
sell(seller, dob_amount) -> i128
quote_redemption(dob_amount) -> RedemptionQuote
get_nav() -> i128
fund(funder, amount)
```

### LiquidNodeStabilizer

```rust
initialize(oracle, usdc_token, dob_token, operator)
fund_usdc(funder, amount)
fund_dob(funder, amount)
provide_liquidity(seller, dob_amount) -> i128
quote_from_oracle(dob_amount) -> LiquidityQuote
withdraw_fees() -> i128
get_balances() -> (i128, i128)
```

## Testing

Run the full test suite:

```bash
# All tests
cargo test

# Specific contract tests
cargo test -p dob-token
cargo test -p dob-oracle
cargo test -p dob-primary-market
cargo test -p dob-stabilizer

# With output
cargo test -- --nocapture
```

## Security Considerations

- **Oracle Trust**: Oracle updater is a trusted role - single point of control
- **Hook Permissions**: Only the primary market contract can mint/burn tokens
- **Penalty Cap**: Maximum 50% penalty prevents total loss on redemption
- **Authorization**: All sensitive functions use Soroban's `require_auth()`
- **Overflow Protection**: Rust's built-in overflow checks prevent arithmetic errors

## Project Structure

```
dob-soroban-liquidity/
├── contracts/
│   ├── token/           # DobToken (SAC-compatible)
│   ├── oracle/          # DobOracle (NAV and risk)
│   ├── primary_market/  # Buy/sell logic
│   └── stabilizer/      # Liquid Node
├── scripts/
│   ├── deploy-local.sh
│   └── deploy-testnet.sh
├── Cargo.toml           # Workspace configuration
├── Makefile             # Build automation
└── README.md
```

## FAQ

### How do I get USDC on Stellar Testnet?

Use the [Stellar Laboratory](https://laboratory.stellar.org/) to create a trustline to the testnet USDC issuer, then request testnet USDC from faucets.

### Can I use this on Stellar Mainnet?

Yes, but ensure thorough testing on testnet first. Update the oracle mechanism for production use (consider using price feeds from oracles like Chainlink on Stellar).

### How do Soroban costs compare to Ethereum?

Soroban transactions are significantly cheaper than Ethereum L1, with costs measured in fractions of XLM rather than expensive gas fees.

### What's the maximum token supply?

No hard cap - tokens are minted on demand based on USDC deposits and burned on redemption.

## Roadmap

- [ ] Frontend integration (React + Freighter wallet)
- [ ] Integration with Stellar DEX
- [ ] Multi-oracle support
- [ ] Governance module for parameter updates
- [ ] Advanced risk models
- [ ] Cross-chain bridge support

## Resources

- [Stellar Documentation](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Soroban Examples](https://github.com/stellar/soroban-examples)
- [Original Solidity Version](../dob-node-liquidity/)

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.

## Support

For questions or issues:
- Open a GitHub issue
- Join [Stellar Discord](https://discord.gg/stellar)
- Check [Soroban Docs](https://soroban.stellar.org/)
