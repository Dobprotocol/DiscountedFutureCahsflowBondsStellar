# Stellar Testnet Deployment Guide

Complete guide to deploy and test DOB Liquidity contracts on Stellar Testnet.

## Prerequisites

### 1. Install Stellar CLI

```bash
# Install latest stellar CLI
cargo install --locked stellar-cli --features opt

# Verify installation
stellar --version
```

### 2. Build Contracts

```bash
# Build optimized WASM files
cargo build --target wasm32-unknown-unknown --release

# Verify WASM files exist
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

## Step 1: Setup Testnet Identity

### Create or Import Identity

```bash
# Generate new identity
stellar keys generate deployer --network testnet

# OR import existing secret key
stellar keys add deployer --secret-key SXXX...YOUR_SECRET_KEY

# View your address
stellar keys address deployer
```

Save your address - you'll need it:
```bash
export DEPLOYER=$(stellar keys address deployer)
echo "Deployer address: $DEPLOYER"
```

### Fund Your Account

Get testnet XLM from Friendbot:

```bash
# Fund with Friendbot
curl "https://friendbot.stellar.org?addr=$DEPLOYER"

# Check balance
stellar account info --id $DEPLOYER --network testnet
```

You should see 10,000 XLM in your account.

## Step 2: Deploy Contracts

### Deploy All Contracts

```bash
# Set network for all commands
export NETWORK="testnet"

# Deploy Token Contract
echo "Deploying DobToken..."
TOKEN_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_token.wasm \
  --source deployer \
  --network $NETWORK)
echo "✅ Token deployed: $TOKEN_ID"

# Deploy Oracle Contract
echo "Deploying DobOracle..."
ORACLE_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle.wasm \
  --source deployer \
  --network $NETWORK)
echo "✅ Oracle deployed: $ORACLE_ID"

# Deploy Primary Market Contract
echo "Deploying DobPrimaryMarket..."
MARKET_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_primary_market.wasm \
  --source deployer \
  --network $NETWORK)
echo "✅ Primary Market deployed: $MARKET_ID"

# Deploy Stabilizer Contract
echo "Deploying LiquidNodeStabilizer..."
STABILIZER_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_stabilizer.wasm \
  --source deployer \
  --network $NETWORK)
echo "✅ Stabilizer deployed: $STABILIZER_ID"
```

### Save Contract Addresses

```bash
# Save to file for later use
cat > deployed-contracts.env <<EOF
export TOKEN_ID=$TOKEN_ID
export ORACLE_ID=$ORACLE_ID
export MARKET_ID=$MARKET_ID
export STABILIZER_ID=$STABILIZER_ID
export DEPLOYER=$DEPLOYER
export NETWORK=$NETWORK
EOF

echo "✅ Contract addresses saved to deployed-contracts.env"
```

## Step 3: Setup USDC (Test Token)

For testnet, we'll use the Stellar testnet USDC or create our own:

### Option A: Use Testnet USDC

```bash
# Testnet USDC contract (if available)
export USDC_ID="CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"
```

### Option B: Deploy Your Own Test Token

```bash
# Deploy a test USDC contract
stellar contract asset deploy \
  --asset USDC:$DEPLOYER \
  --source deployer \
  --network $NETWORK

# Get the contract ID
export USDC_ID=$(stellar contract id asset --asset USDC:$DEPLOYER --source deployer)
echo "Test USDC deployed: $USDC_ID"
```

## Step 4: Initialize Contracts

### 1. Initialize Oracle

```bash
echo "Initializing Oracle..."
stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  -- initialize \
  --updater $DEPLOYER \
  --initial_nav 10000000 \
  --initial_risk 1000

echo "✅ Oracle initialized (NAV: $1.00, Risk: 10%)"
```

### 2. Initialize Token

```bash
echo "Initializing DobToken..."
stellar contract invoke \
  --id $TOKEN_ID \
  --source deployer \
  --network $NETWORK \
  -- initialize \
  --admin $DEPLOYER \
  --hook $MARKET_ID \
  --name "Dob Solar Farm 2035" \
  --symbol "DOB-35" \
  --decimals 7

echo "✅ Token initialized"
```

### 3. Initialize Primary Market

```bash
echo "Initializing Primary Market..."
stellar contract invoke \
  --id $MARKET_ID \
  --source deployer \
  --network $NETWORK \
  -- initialize \
  --dob_token $TOKEN_ID \
  --usdc_token $USDC_ID \
  --oracle $ORACLE_ID \
  --operator $DEPLOYER

echo "✅ Primary Market initialized"
```

### 4. Initialize Stabilizer

```bash
echo "Initializing Stabilizer..."
stellar contract invoke \
  --id $STABILIZER_ID \
  --source deployer \
  --network $NETWORK \
  -- initialize \
  --oracle $ORACLE_ID \
  --usdc_token $USDC_ID \
  --dob_token $TOKEN_ID \
  --operator $DEPLOYER

echo "✅ Stabilizer initialized"
```

## Step 5: Fund Contracts with Test USDC

### Mint Test USDC to Your Account

```bash
# Mint 100,000 test USDC to deployer
stellar contract invoke \
  --id $USDC_ID \
  --source deployer \
  --network $NETWORK \
  -- mint \
  --to $DEPLOYER \
  --amount 1000000000000

echo "✅ Minted 100,000 USDC to your account"
```

### Fund Primary Market for Redemptions

```bash
# Send 10,000 USDC to primary market for redemptions
stellar contract invoke \
  --id $USDC_ID \
  --source deployer \
  --network $NETWORK \
  -- transfer \
  --from $DEPLOYER \
  --to $MARKET_ID \
  --amount 100000000000

echo "✅ Funded Primary Market with 10,000 USDC"
```

## Step 6: Test the System

### Test 1: Buy DOB Tokens

```bash
echo "=== TEST 1: Buying DOB Tokens ==="

# Buy $1,000 worth of DOB
stellar contract invoke \
  --id $MARKET_ID \
  --source deployer \
  --network $NETWORK \
  -- buy \
  --buyer $DEPLOYER \
  --usdc_amount 10000000000

echo "✅ Purchased DOB tokens"
```

### Test 2: Check Your Balance

```bash
echo "=== TEST 2: Checking Balances ==="

# Check DOB balance
DOB_BALANCE=$(stellar contract invoke \
  --id $TOKEN_ID \
  --network $NETWORK \
  -- balance \
  --account $DEPLOYER)

echo "Your DOB balance: $DOB_BALANCE"
echo "Expected: ~9,900,000,000 (990 DOB with 7 decimals)"
```

### Test 3: Get Redemption Quote

```bash
echo "=== TEST 3: Getting Redemption Quote ==="

# Quote for selling 500 DOB
stellar contract invoke \
  --id $MARKET_ID \
  --network $NETWORK \
  -- quote_redemption \
  --dob_amount 5000000000

echo "✅ Quote retrieved (should show penalty ~4%)"
```

### Test 4: Sell DOB Tokens

```bash
echo "=== TEST 4: Selling DOB Tokens ==="

# Sell 500 DOB
stellar contract invoke \
  --id $MARKET_ID \
  --source deployer \
  --network $NETWORK \
  -- sell \
  --seller $DEPLOYER \
  --dob_amount 5000000000

echo "✅ Sold 500 DOB tokens"
```

### Test 5: Update Oracle

```bash
echo "=== TEST 5: Updating Oracle ==="

# Update NAV to $1.15 and risk to 7%
stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  -- update \
  --new_nav 11500000 \
  --new_default_risk 700

echo "✅ Oracle updated (NAV: $1.15, Risk: 7%)"
```

### Test 6: Verify New Prices

```bash
echo "=== TEST 6: Verifying New Prices ==="

# Check NAV
NAV=$(stellar contract invoke \
  --id $ORACLE_ID \
  --network $NETWORK \
  -- nav)

# Check risk
RISK=$(stellar contract invoke \
  --id $ORACLE_ID \
  --network $NETWORK \
  -- default_risk)

echo "Current NAV: $NAV (expected: 11500000)"
echo "Current Risk: $RISK (expected: 700)"
```

## Step 7: View Contract on Stellar Expert

Visit Stellar Expert to see your contracts:

```bash
echo "View your contracts:"
echo "Token: https://stellar.expert/explorer/testnet/contract/$TOKEN_ID"
echo "Oracle: https://stellar.expert/explorer/testnet/contract/$ORACLE_ID"
echo "Market: https://stellar.expert/explorer/testnet/contract/$MARKET_ID"
echo "Stabilizer: https://stellar.expert/explorer/testnet/contract/$STABILIZER_ID"
```

## Complete Test Script

Create a file `test-deployment.sh`:

```bash
#!/bin/bash

# Load contract addresses
source deployed-contracts.env

echo "╔════════════════════════════════════════╗"
echo "║  DOB Liquidity Testnet Test Suite     ║"
echo "╚════════════════════════════════════════╝"

# Test 1: Buy DOB
echo -e "\n[1/6] Buying 1,000 USDC worth of DOB..."
stellar contract invoke --id $MARKET_ID --source deployer --network $NETWORK \
  -- buy --buyer $DEPLOYER --usdc_amount 10000000000

# Test 2: Check balance
echo -e "\n[2/6] Checking DOB balance..."
DOB_BAL=$(stellar contract invoke --id $TOKEN_ID --network $NETWORK \
  -- balance --account $DEPLOYER)
echo "DOB Balance: $DOB_BAL"

# Test 3: Quote redemption
echo -e "\n[3/6] Getting quote for 500 DOB..."
stellar contract invoke --id $MARKET_ID --network $NETWORK \
  -- quote_redemption --dob_amount 5000000000

# Test 4: Sell DOB
echo -e "\n[4/6] Selling 500 DOB..."
stellar contract invoke --id $MARKET_ID --source deployer --network $NETWORK \
  -- sell --seller $DEPLOYER --dob_amount 5000000000

# Test 5: Update oracle
echo -e "\n[5/6] Updating oracle to NAV $1.20..."
stellar contract invoke --id $ORACLE_ID --source deployer --network $NETWORK \
  -- update --new_nav 12000000 --new_default_risk 500

# Test 6: Buy at new price
echo -e "\n[6/6] Buying 500 USDC at new NAV..."
stellar contract invoke --id $MARKET_ID --source deployer --network $NETWORK \
  -- buy --buyer $DEPLOYER --usdc_amount 5000000000

echo -e "\n✅ All tests completed!"
echo "View transactions: https://stellar.expert/explorer/testnet/account/$DEPLOYER"
```

Make it executable:

```bash
chmod +x test-deployment.sh
./test-deployment.sh
```

## Troubleshooting

### Issue: "account not found"

```bash
# Fund account again
curl "https://friendbot.stellar.org?addr=$DEPLOYER"
```

### Issue: "contract not found"

```bash
# Verify contract is deployed
stellar contract info --id $TOKEN_ID --network testnet
```

### Issue: "insufficient balance"

```bash
# Check your XLM balance
stellar account info --id $DEPLOYER --network testnet

# Fund if needed
curl "https://friendbot.stellar.org?addr=$DEPLOYER"
```

### Issue: "transaction failed"

```bash
# Add --fee argument for higher fees
stellar contract invoke --fee 1000000 ...
```

### View Detailed Logs

```bash
# Add RUST_LOG for debugging
RUST_LOG=debug stellar contract invoke ...
```

## Monitoring Your Contracts

### Check Contract Storage

```bash
# View all storage entries
stellar contract fetch --id $TOKEN_ID --network testnet
```

### Check Transaction History

```bash
# View account transactions
stellar account history --id $DEPLOYER --network testnet
```

### Monitor Events

```bash
# Watch for contract events (requires RPC endpoint)
stellar events --start-ledger 12345 --contract-id $MARKET_ID --network testnet
```

## Cost Estimation

Typical costs on testnet (similar to mainnet):

- Deploy contract: ~0.01 XLM
- Initialize contract: ~0.001 XLM
- Buy transaction: ~0.0005 XLM
- Sell transaction: ~0.0005 XLM
- Oracle update: ~0.0003 XLM

**Total deployment cost: ~0.05 XLM**

## Next Steps

1. ✅ Deploy to testnet (this guide)
2. ⏳ Build frontend integration
3. ⏳ Security audit
4. ⏳ Mainnet deployment
5. ⏳ Production launch

## Useful Links

- [Stellar Expert (Testnet)](https://stellar.expert/explorer/testnet)
- [Stellar Laboratory](https://laboratory.stellar.org/)
- [Soroban Docs](https://soroban.stellar.org/)
- [Stellar CLI Docs](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)
- [Friendbot (Get Testnet XLM)](https://laboratory.stellar.org/#account-creator?network=test)

---

**You're now ready to test on Stellar Testnet!** 🚀
