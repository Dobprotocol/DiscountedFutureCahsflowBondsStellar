.PHONY: build test clean install optimize deploy-testnet deploy-local

# Build all contracts
build:
	@echo "Building all contracts..."
	cargo build --target wasm32-unknown-unknown --release
	@echo "Build complete!"

# Optimize WASM files
optimize:
	@echo "Optimizing WASM files..."
	@for contract in token oracle primary_market stabilizer; do \
		soroban contract optimize \
			--wasm target/wasm32-unknown-unknown/release/dob_$$contract.wasm \
			--wasm-out target/wasm32-unknown-unknown/release/dob_$$contract_optimized.wasm; \
	done
	@echo "Optimization complete!"

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf target/
	@echo "Clean complete!"

# Install dependencies and tools
install:
	@echo "Installing Stellar CLI..."
	cargo install --locked stellar-cli --features opt
	@echo "Installing cargo-watch..."
	cargo install cargo-watch
	@echo "Installation complete!"

# Build and optimize
all: build optimize

# Deploy to testnet (requires environment variables)
deploy-testnet:
	@echo "Deploying to Stellar Testnet..."
	@./scripts/deploy-testnet.sh

# Deploy to local network
deploy-local:
	@echo "Deploying to local Stellar network..."
	@./scripts/deploy-local.sh

# Watch and rebuild on changes
watch:
	cargo watch -x "build --target wasm32-unknown-unknown --release"

# Format code
fmt:
	cargo fmt --all

# Run clippy lints
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
docs:
	cargo doc --no-deps --open
