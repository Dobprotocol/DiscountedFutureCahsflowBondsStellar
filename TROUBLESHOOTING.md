# Deployment Troubleshooting Guide

Common errors and how to fix them.

## Quick Diagnosis

Run this first to check your setup:

```bash
./scripts/diagnose.sh
```

## Common Errors & Solutions

### Error 1: "command not found: stellar"

**Cause:** Stellar CLI not installed

**Solution:**
```bash
cargo install --locked stellar-cli --features opt

# Or if you have it but PATH issue:
export PATH="$HOME/.cargo/bin:$PATH"
```

### Error 2: "No such file or directory: *.wasm"

**Cause:** Contracts not built

**Solution:**
```bash
# Build contracts first
cargo build --target wasm32-unknown-unknown --release

# Verify files exist
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

### Error 3: "error: toolchain 'stable-aarch64-apple-darwin' does not contain component 'rust-std' for target 'wasm32-unknown-unknown'"

**Cause:** WASM target not installed

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
```

### Error 4: "account not found" or "sourceAccount not found"

**Cause:** Account not funded or doesn't exist

**Solution:**
```bash
# Get your address
export DEPLOYER=$(stellar keys address deployer)

# Fund it
curl "https://friendbot.stellar.org?addr=$DEPLOYER"

# Wait 5 seconds then retry
sleep 5
```

### Error 5: "error: Could not parse HostFunction"

**Cause:** Contract invocation with wrong parameters

**Solution:**
```bash
# Check contract function signature
stellar contract inspect --wasm target/wasm32-unknown-unknown/release/dob_token.wasm

# Make sure parameter types match
# addresses should be: $DEPLOYER (not quoted)
# numbers should be: 10000000 (not quoted)
# strings should be: "My String" (quoted)
```

### Error 6: "transaction submission failed"

**Cause:** Multiple possible causes

**Solutions:**

**A. Insufficient XLM:**
```bash
# Check balance
stellar account info --id $DEPLOYER --network testnet

# Refund if needed
curl "https://friendbot.stellar.org?addr=$DEPLOYER"
```

**B. Contract already exists:**
```bash
# Use --wasm-hash to check if deployed
stellar contract info --id YOUR_CONTRACT_ID --network testnet
```

**C. Network congestion:**
```bash
# Add higher fee
stellar contract deploy --fee 1000000 ...
```

### Error 7: "panic: HostError: Error(Auth, InvalidAction)"

**Cause:** Authorization failed

**Solution:**
```bash
# Make sure you're using the correct identity
stellar keys show deployer

# And correct address format
export DEPLOYER=$(stellar keys address deployer)
echo $DEPLOYER  # Should start with G
```

### Error 8: "Contract has been initialized already"

**Cause:** Trying to initialize twice

**Solution:**
```bash
# Check if already initialized
stellar contract invoke --id $ORACLE_ID --network testnet -- nav

# If it returns a value, it's already initialized
# Deploy a new contract or use existing one
```

### Error 9: "./scripts/deploy-and-test.sh: Permission denied"

**Cause:** Script not executable

**Solution:**
```bash
chmod +x ./scripts/deploy-and-test.sh
chmod +x ./scripts/*.sh
```

### Error 10: "error: unexpected argument '--updater' found"

**Cause:** Wrong parameter format in invoke

**Solution:**
```bash
# Correct format (note the -- before function name):
stellar contract invoke \
  --id $CONTRACT \
  --source deployer \
  --network testnet \
  -- initialize \
  --updater $DEPLOYER \
  --initial_nav 10000000

# Wrong format (missing -- ):
stellar contract invoke \
  --id $CONTRACT \
  initialize \  # ❌ WRONG
  --updater $DEPLOYER
```

## Step-by-Step Debugging

### 1. Test Stellar CLI
```bash
stellar --version
```
Should output version number.

### 2. Test Identity
```bash
stellar keys generate test-key --network testnet
stellar keys address test-key
```
Should output an address starting with G.

### 3. Test Funding
```bash
TEST_ADDR=$(stellar keys address test-key)
curl "https://friendbot.stellar.org?addr=$TEST_ADDR"
stellar account info --id $TEST_ADDR --network testnet
```
Should show 10,000 XLM balance.

### 4. Test Simple Deployment
```bash
# Try deploying just the oracle (smallest contract)
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle.wasm \
  --source test-key \
  --network testnet
```
Should output a contract ID starting with C.

### 5. Test Contract Invocation
```bash
ORACLE_ID="YOUR_DEPLOYED_CONTRACT_ID"

# Initialize
stellar contract invoke \
  --id $ORACLE_ID \
  --source test-key \
  --network testnet \
  -- initialize \
  --updater $(stellar keys address test-key) \
  --initial_nav 10000000 \
  --initial_risk 1000

# Query
stellar contract invoke \
  --id $ORACLE_ID \
  --network testnet \
  -- nav
```
Should output: 10000000

## Simplest Deployment

If the full script fails, try this minimal version:

```bash
./scripts/simple-deploy.sh
```

This deploys just 2 contracts (Token + Oracle) with better error reporting.

## Checking What's Deployed

### View Contract Info
```bash
stellar contract info --id CONTRACT_ID --network testnet
```

### View Account Info
```bash
stellar account info --id $DEPLOYER --network testnet
```

### View Recent Transactions
```bash
# On Stellar Expert
open "https://stellar.expert/explorer/testnet/account/$DEPLOYER"
```

## Network Issues

### Testnet Down?
Check status: https://status.stellar.org/

### Friendbot Not Working?
Try alternative:
```bash
# Use Stellar Laboratory
open "https://laboratory.stellar.org/#account-creator?network=test"
```

### RPC Issues?
Try different RPC:
```bash
export STELLAR_RPC_URL="https://soroban-testnet.stellar.org"
```

## Still Having Issues?

### Collect Debug Info

```bash
# Run diagnostics
./scripts/diagnose.sh > debug-info.txt

# Add stellar version
stellar --version >> debug-info.txt

# Add environment
env | grep DEPLOYER >> debug-info.txt

# Add last few commands from history
history | tail -20 >> debug-info.txt
```

### Share Error Details

When asking for help, include:
1. Full error message
2. Command that failed
3. Output of `./scripts/diagnose.sh`
4. Stellar CLI version
5. Operating system

### Test with Verbose Output

```bash
# Add RUST_LOG for detailed logs
RUST_LOG=debug stellar contract deploy ...

# Or use --verbose flag
stellar --verbose contract deploy ...
```

## Alternative: Manual Step-by-Step

If automation fails, try manual deployment:

```bash
# 1. Setup
stellar keys generate deployer --network testnet
export DEPLOYER=$(stellar keys address deployer)
export NETWORK="testnet"

# 2. Fund
curl "https://friendbot.stellar.org?addr=$DEPLOYER"
sleep 5

# 3. Deploy Oracle only
ORACLE_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/dob_oracle.wasm \
  --source deployer \
  --network $NETWORK)
echo "Oracle: $ORACLE_ID"

# 4. Initialize Oracle
stellar contract invoke \
  --id $ORACLE_ID \
  --source deployer \
  --network $NETWORK \
  -- initialize \
  --updater $DEPLOYER \
  --initial_nav 10000000 \
  --initial_risk 1000

# 5. Test it
stellar contract invoke --id $ORACLE_ID --network $NETWORK -- nav
```

If this works, continue with other contracts one by one.

## Known Limitations

1. **Friendbot rate limiting**: Can only fund an address once per day
2. **Testnet resets**: Testnet occasionally resets, losing all data
3. **Contract size limits**: Max 64KB per contract (we're well under)
4. **Resource limits**: Testnet has same limits as mainnet

## Success Indicators

You'll know it worked when:
- ✅ Contract IDs start with `C`
- ✅ Account address starts with `G`
- ✅ `stellar contract invoke ... -- nav` returns `10000000`
- ✅ No errors in output
- ✅ Can view contracts on Stellar Expert

## Need More Help?

1. Check Stellar Discord: https://discord.gg/stellar
2. Soroban Docs: https://soroban.stellar.org/
3. GitHub Issues: Open an issue with debug info
