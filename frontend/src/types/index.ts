export interface ContractAddresses {
  token: string;
  oracle: string;
  pool: string;
  usdc: string;
  liquidNodes: string[];
}

export interface OracleData {
  fairPrice: string;
  risk: string;
  lastUpdate: string;
}

export interface PoolReserves {
  usdc: string;
  dob: string;
  totalLp: string;
}

export interface LiquidNodeInfo {
  address: string;
  usdcBalance: string;
  dobBalance: string;
  quote?: {
    usdcProvided: string;
    feeBps: number;
  };
}

export interface UserBalances {
  usdc: string;
  dob: string;
  lpShares: string;
}

export interface SwapQuote {
  inputAmount: string;
  outputAmount: string;
  priceImpact: number;
  fee: number;
}

export type Network = 'testnet' | 'mainnet';

export interface WalletState {
  connected: boolean;
  publicKey: string | null;
  network: Network;
}
