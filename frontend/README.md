# DOB Liquidity Frontend

Frontend web application for interacting with the DOB Liquidity Pool smart contracts on Stellar Soroban.

## Features

- 🔐 **Wallet Connection**: Connect with Freighter wallet
- 📊 **Oracle Dashboard**: View NAV and risk metrics in real-time
- 💱 **Token Swap**: Buy and sell DOB tokens
  - AfterSwap hook for buying (mints at NAV)
  - BeforeSwap hook for selling (uses Liquid Nodes if needed)
- 💧 **Liquidity Management**: Add/remove liquidity to earn fees
- 🖥️ **Liquid Nodes Monitor**: View all registered Liquid Nodes and their balances
- 📈 **User Balances**: Track your USDC, DOB, and LP tokens

## Prerequisites

- Node.js 18+ and npm/yarn/pnpm
- [Freighter Wallet](https://www.freighter.app/) browser extension
- Deployed contracts on Stellar testnet/mainnet

## Installation

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Or with yarn
yarn install

# Or with pnpm
pnpm install
```

## Configuration

### Option 1: Manual Configuration (UI)

1. Start the development server
2. Click "Configure Contracts" in the UI
3. Enter your deployed contract addresses

### Option 2: Environment File

```bash
# Copy the example env file
cp .env.example .env

# Edit .env and add your contract addresses
nano .env
```

Get the contract addresses from `../deployed-contracts.env` after running the deployment script.

## Development

```bash
# Start development server
npm run dev

# Application will be available at http://localhost:3000
```

## Build for Production

```bash
# Build the application
npm run build

# Preview the production build
npm run preview
```

## Project Structure

```
frontend/
├── src/
│   ├── components/       # React components
│   │   ├── Header.tsx
│   │   ├── OracleInfo.tsx
│   │   ├── SwapInterface.tsx
│   │   ├── LiquidityManager.tsx
│   │   ├── LiquidNodes.tsx
│   │   └── UserBalances.tsx
│   ├── hooks/           # Custom React hooks
│   │   ├── useWallet.ts
│   │   └── useContracts.ts
│   ├── utils/           # Utility functions
│   │   ├── stellar.ts
│   │   └── contracts.ts
│   ├── types/           # TypeScript types
│   │   └── index.ts
│   ├── App.tsx          # Main application
│   ├── main.tsx         # Entry point
│   └── index.css        # Global styles
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

## Usage Guide

### 1. Connect Your Wallet

Click "Connect Freighter" and approve the connection in your Freighter wallet extension.

### 2. Buy DOB Tokens

1. Go to the "Swap Tokens" section
2. Select "Buy" mode
3. Enter the amount of USDC you want to spend
4. Review the estimated DOB you'll receive (minted at NAV price)
5. Click "Buy DOB" and approve the transaction

**How it works:** AfterSwap hook mints new DOB tokens at the current NAV from the Oracle. 1% fee goes to the DEX, 99% goes to the infrastructure operator.

### 3. Sell DOB Tokens

1. Go to the "Swap Tokens" section
2. Select "Sell" mode
3. Enter the amount of DOB you want to sell
4. Review the estimated USDC you'll receive
5. Click "Sell DOB" and approve the transaction

**How it works:** BeforeSwap hook first tries to use pool liquidity. If insufficient, it automatically queries Liquid Nodes for the best quote and combines pool + LN liquidity. Tokens are burned after the swap.

### 4. Provide Liquidity

1. Go to "Manage Liquidity" section
2. Click "Add" mode
3. Enter amounts for both USDC and DOB
4. Click "Add Liquidity" to receive LP shares
5. Earn fees from all swaps proportional to your share

### 5. Remove Liquidity

1. Go to "Manage Liquidity" section
2. Click "Remove" mode
3. Enter LP shares to burn
4. Receive your proportional share of USDC and DOB

## Key Concepts

### Oracle (NAV & Risk)
- **NAV**: Net Asset Value - determines the fair price for minting DOB tokens
- **Risk**: Default risk percentage - affects Liquid Node fees dynamically

### Liquid Nodes
- Pre-funded contracts that provide instant liquidity when the pool is insufficient
- Calculate fees based on Oracle risk (5%-30%)
- Pool automatically selects the LN with the best (lowest) fee

### LP Shares
- Represent your ownership in the liquidity pool
- Earn proportional fees from all trading activity
- Can be burned to withdraw your liquidity

## Troubleshooting

### "Freighter wallet not installed"
Install the [Freighter browser extension](https://www.freighter.app/)

### "Contract addresses not configured"
Either configure them via the UI or add them to your `.env` file

### "Transaction failed"
- Ensure you have enough balance for the transaction
- Check that you're on the correct network (testnet/mainnet)
- Verify the contract addresses are correct

### "Insufficient liquidity"
If selling a large amount and Liquid Nodes don't have enough USDC, the transaction will fail. This is expected behavior to protect users.

## Network Support

- **Testnet**: For testing (default)
- **Mainnet**: For production use

The network is automatically detected from your Freighter wallet.

## Technologies

- **React 18** - UI library
- **TypeScript** - Type safety
- **Tailwind CSS** - Styling
- **Vite** - Build tool
- **Stellar SDK** - Blockchain interaction
- **Lucide React** - Icons

## License

MIT

## Support

For issues and questions:
- Check the [main project documentation](../README.md)
- Review contract details in [ARCHITECTURE.md](../ARCHITECTURE.md)
- Report bugs in the GitHub repository
