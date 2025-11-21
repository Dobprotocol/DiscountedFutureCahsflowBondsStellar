# Quick Start Guide

Get up and running with DOB Soroban Liquidity in minutes.

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

## Build

```bash
# Clone and navigate
cd dob-soroban-liquidity

# Build all contracts
make build

# Run tests
make test
```

## Deploy to Testnet

### 1. Get Testnet XLM

Visit [Stellar Laboratory](https://laboratory.stellar.org/#account-creator?network=test) to create an account and fund it.

### 2. Deploy Contracts

```bash
export SOURCE_ACCOUNT=GXXX...YOUR_TESTNET_ACCOUNT

./scripts/deploy-testnet.sh
```

### 3. Initialize Contracts

```bash
# Save contract IDs from deployment output
TOKEN_ID="CXXX..."
ORACLE_ID="CXXX..."
PRIMARY_MARKET_ID="CXXX..."
STABILIZER_ID="CXXX..."
USDC_ID="CXXX..."  # Testnet USDC contract

# Initialize Oracle (NAV = $1.00, Risk = 10%)
stellar contract invoke \
  --id $ORACLE_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- initialize \
  --updater $SOURCE_ACCOUNT \
  --initial_nav 10000000 \
  --initial_risk 1000

# Initialize Token
stellar contract invoke \
  --id $TOKEN_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- initialize \
  --admin $SOURCE_ACCOUNT \
  --hook $PRIMARY_MARKET_ID \
  --name "Dob Solar Farm 2035" \
  --symbol "DOB-35" \
  --decimals 7

# Initialize Primary Market
stellar contract invoke \
  --id $PRIMARY_MARKET_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- initialize \
  --dob_token $TOKEN_ID \
  --usdc_token $USDC_ID \
  --oracle $ORACLE_ID \
  --operator $SOURCE_ACCOUNT

# Initialize Stabilizer
stellar contract invoke \
  --id $STABILIZER_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- initialize \
  --oracle $ORACLE_ID \
  --usdc_token $USDC_ID \
  --dob_token $TOKEN_ID \
  --operator $SOURCE_ACCOUNT
```

## Test Transactions

### Buy DOB Tokens

```bash
# Buy 100 USDC worth of DOB tokens
stellar contract invoke \
  --id $PRIMARY_MARKET_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- buy \
  --buyer $SOURCE_ACCOUNT \
  --usdc_amount 1000000000
```

### Check Balance

```bash
stellar contract invoke \
  --id $TOKEN_ID \
  --network testnet \
  -- balance \
  --account $SOURCE_ACCOUNT
```

### Get Quote for Selling

```bash
# Quote for selling 50 DOB tokens
stellar contract invoke \
  --id $PRIMARY_MARKET_ID \
  --network testnet \
  -- quote_redemption \
  --dob_amount 500000000
```

### Sell DOB Tokens

```bash
stellar contract invoke \
  --id $PRIMARY_MARKET_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- sell \
  --seller $SOURCE_ACCOUNT \
  --dob_amount 500000000
```

## Update Oracle (Simulate Market Changes)

```bash
# Increase NAV to $1.15 and reduce risk to 7%
stellar contract invoke \
  --id $ORACLE_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- update \
  --new_nav 11500000 \
  --new_default_risk 700
```

## Common Issues

### Issue: "Contract not found"
**Solution**: Make sure you're using the correct network and contract IDs from deployment.

### Issue: "Insufficient balance"
**Solution**: Fund your account with testnet XLM from Friendbot and USDC from testnet faucets.

### Issue: "Unauthorized"
**Solution**: Ensure you're using the correct source account that has permissions (admin/operator).

### Issue: Build fails
**Solution**:
```bash
# Clean and rebuild
cargo clean
make build
```

## Next Steps

- Read the [full README](README.md) for detailed documentation
- Explore contract source code in `contracts/`
- Join [Stellar Discord](https://discord.gg/stellar) for help

## Example Investment Lifecycle

```bash
# 1. Launch - Set initial conditions
stellar contract invoke --id $ORACLE_ID --source $SOURCE_ACCOUNT --network testnet \
  -- update --new_nav 10000000 --new_default_risk 1000

# 2. Initial Investment - Alice buys $10,000 of DOB
stellar contract invoke --id $PRIMARY_MARKET_ID --source $ALICE --network testnet \
  -- buy --buyer $ALICE --usdc_amount 100000000000

# 3. Revenue Period - NAV rises, risk drops
stellar contract invoke --id $ORACLE_ID --source $SOURCE_ACCOUNT --network testnet \
  -- update --new_nav 11500000 --new_default_risk 700

# 4. Profit Taking - Alice sells some DOB
stellar contract invoke --id $PRIMARY_MARKET_ID --source $ALICE --network testnet \
  -- sell --seller $ALICE --dob_amount 20000000000

# 5. Check earnings
stellar contract invoke --id $TOKEN_ID --network testnet \
  -- balance --account $ALICE
```

## Useful Commands

```bash
# Check oracle status
stellar contract invoke --id $ORACLE_ID --network testnet -- nav
stellar contract invoke --id $ORACLE_ID --network testnet -- default_risk

# Check primary market stats
stellar contract invoke --id $PRIMARY_MARKET_ID --network testnet -- get_stats

# Check stabilizer balances
stellar contract invoke --id $STABILIZER_ID --network testnet -- get_balances
```
