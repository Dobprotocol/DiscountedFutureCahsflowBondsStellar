# Quick Deploy to Testnet

**TL;DR:** One command to deploy and test everything.

## Option 1: Automated Script (Recommended)

```bash
./scripts/deploy-and-test.sh
```

This script will:
- ✅ Build all contracts
- ✅ Setup testnet identity
- ✅ Deploy all 4 contracts + test USDC
- ✅ Initialize everything
- ✅ Run 6 integration tests
- ✅ Save contract addresses to `deployed-contracts.env`

**Time:** ~2-3 minutes

## Option 2: Manual Step-by-Step

### 1. Install & Build
```bash
cargo install --locked stellar-cli --features opt
cargo build --target wasm32-unknown-unknown --release
```

### 2. Setup Identity
```bash
stellar keys generate deployer --network testnet
export DEPLOYER=$(stellar keys address deployer)
curl "https://friendbot.stellar.org?addr=$DEPLOYER"
```

### 3. Deploy (4 contracts)
```bash
export NETWORK="testnet"

TOKEN_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_token.wasm \
  --source deployer --network $NETWORK)

ORACLE_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle.wasm \
  --source deployer --network $NETWORK)

MARKET_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_primary_market.wasm \
  --source deployer --network $NETWORK)

STABILIZER_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_stabilizer.wasm \
  --source deployer --network $NETWORK)
```

### 4. Initialize
```bash
# Oracle
stellar contract invoke --id $ORACLE_ID --source deployer --network $NETWORK \
  -- initialize --updater $DEPLOYER --initial_nav 10000000 --initial_risk 1000

# Token
stellar contract invoke --id $TOKEN_ID --source deployer --network $NETWORK \
  -- initialize --admin $DEPLOYER --hook $MARKET_ID \
  --name "Dob Token" --symbol "DOB" --decimals 7

# Market (need USDC_ID first)
stellar contract invoke --id $MARKET_ID --source deployer --network $NETWORK \
  -- initialize --dob_token $TOKEN_ID --usdc_token $USDC_ID \
  --oracle $ORACLE_ID --operator $DEPLOYER
```

### 5. Test
```bash
# Buy DOB
stellar contract invoke --id $MARKET_ID --source deployer --network $NETWORK \
  -- buy --buyer $DEPLOYER --usdc_amount 10000000000

# Check balance
stellar contract invoke --id $TOKEN_ID --network $NETWORK \
  -- balance --account $DEPLOYER
```

## Common Commands

### Check Contract Info
```bash
stellar contract info --id $TOKEN_ID --network testnet
```

### View Balance
```bash
stellar contract invoke --id $TOKEN_ID --network testnet \
  -- balance --account $DEPLOYER
```

### Get NAV
```bash
stellar contract invoke --id $ORACLE_ID --network testnet -- nav
```

### Update Oracle
```bash
stellar contract invoke --id $ORACLE_ID --source deployer --network testnet \
  -- update --new_nav 12000000 --new_default_risk 500
```

### Buy Tokens
```bash
stellar contract invoke --id $MARKET_ID --source deployer --network testnet \
  -- buy --buyer $DEPLOYER --usdc_amount 10000000000
```

### Sell Tokens
```bash
stellar contract invoke --id $MARKET_ID --source deployer --network testnet \
  -- sell --seller $DEPLOYER --dob_amount 5000000000
```

### Get Quote
```bash
stellar contract invoke --id $MARKET_ID --network testnet \
  -- quote_redemption --dob_amount 5000000000
```

## Troubleshooting

### Error: account not found
```bash
curl "https://friendbot.stellar.org?addr=$DEPLOYER"
```

### Error: contract not found
```bash
# Redeploy or check contract ID
stellar contract info --id $TOKEN_ID --network testnet
```

### Load Saved Contracts
```bash
source deployed-contracts.env
echo $TOKEN_ID
```

## View on Stellar Expert

After deployment:
```bash
echo "https://stellar.expert/explorer/testnet/contract/$TOKEN_ID"
echo "https://stellar.expert/explorer/testnet/contract/$MARKET_ID"
echo "https://stellar.expert/explorer/testnet/account/$DEPLOYER"
```

## Next Steps

1. ✅ Deploy to testnet (this guide)
2. Test with friends
3. Build frontend
4. Deploy to mainnet

---

**Full guide:** See `TESTNET_DEPLOYMENT.md`
