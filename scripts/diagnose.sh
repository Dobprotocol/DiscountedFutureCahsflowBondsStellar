#!/bin/bash

echo "=== DOB Liquidity Deployment Diagnostics ==="
echo ""

# Check stellar CLI
echo "1. Checking Stellar CLI..."
if command -v stellar &> /dev/null; then
    echo "   ✅ Stellar CLI installed"
    stellar --version
else
    echo "   ❌ Stellar CLI NOT found"
    echo "   Install with: cargo install --locked stellar-cli --features opt"
fi
echo ""

# Check Rust
echo "2. Checking Rust..."
if command -v cargo &> /dev/null; then
    echo "   ✅ Rust installed"
    rustc --version
else
    echo "   ❌ Rust NOT found"
fi
echo ""

# Check wasm target
echo "3. Checking wasm32 target..."
if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "   ✅ wasm32-unknown-unknown target installed"
else
    echo "   ❌ wasm32 target NOT installed"
    echo "   Install with: rustup target add wasm32-unknown-unknown"
fi
echo ""

# Check WASM files
echo "4. Checking WASM files..."
if [ -d "target/wasm32-unknown-unknown/release" ]; then
    WASM_COUNT=$(ls -1 target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null | wc -l)
    if [ $WASM_COUNT -gt 0 ]; then
        echo "   ✅ Found $WASM_COUNT WASM files"
        ls -lh target/wasm32-unknown-unknown/release/*.wasm | awk '{print "      "$9, "("$5")"}'
    else
        echo "   ❌ No WASM files found"
        echo "   Build with: cargo build --target wasm32-unknown-unknown --release"
    fi
else
    echo "   ❌ Build directory not found"
    echo "   Build with: cargo build --target wasm32-unknown-unknown --release"
fi
echo ""

# Check network connectivity
echo "5. Checking network connectivity..."
if curl -s --max-time 5 https://horizon-testnet.stellar.org > /dev/null; then
    echo "   ✅ Can reach Stellar testnet"
else
    echo "   ❌ Cannot reach Stellar testnet"
    echo "   Check your internet connection"
fi
echo ""

# Check if identity exists
echo "6. Checking Stellar identity..."
if stellar keys show deployer 2>/dev/null; then
    echo "   ✅ Identity 'deployer' exists"
    ADDR=$(stellar keys address deployer 2>/dev/null)
    echo "   Address: $ADDR"
else
    echo "   ℹ️  Identity 'deployer' not found (will be created on first run)"
fi
echo ""

# Check current directory
echo "7. Checking current directory..."
if [ -f "Cargo.toml" ] && [ -d "contracts" ]; then
    echo "   ✅ In correct directory (dob-soroban-liquidity)"
else
    echo "   ❌ Not in project root"
    echo "   Run: cd dob-soroban-liquidity"
fi
echo ""

echo "=== Diagnostic Complete ==="
echo ""
echo "If all checks pass, try running:"
echo "  ./scripts/deploy-and-test.sh"
