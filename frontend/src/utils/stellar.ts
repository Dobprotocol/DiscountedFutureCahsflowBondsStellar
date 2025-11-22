import * as StellarSdk from '@stellar/stellar-sdk';
import {
  isConnected,
  isAllowed,
  setAllowed,
  getAddress,
  getNetwork,
  signTransaction,
} from '@stellar/freighter-api';
import type { Network } from '../types';

const NETWORKS = {
  testnet: {
    networkPassphrase: StellarSdk.Networks.TESTNET,
    horizonUrl: 'https://horizon-testnet.stellar.org',
    sorobanRpcUrl: 'https://soroban-testnet.stellar.org',
  },
  mainnet: {
    networkPassphrase: StellarSdk.Networks.PUBLIC,
    horizonUrl: 'https://horizon.stellar.org',
    sorobanRpcUrl: 'https://soroban.stellar.org',
  },
};

function normalizeNetwork(network: Network | string): Network {
  const normalized = network.toLowerCase();
  return (normalized === 'testnet' || normalized === 'mainnet') ? normalized as Network : 'testnet';
}

export function getServer(network: Network = 'testnet') {
  const config = NETWORKS[normalizeNetwork(network)];
  return new StellarSdk.SorobanRpc.Server(config.sorobanRpcUrl);
}

export function getHorizonServer(network: Network = 'testnet') {
  const config = NETWORKS[normalizeNetwork(network)];
  return new StellarSdk.Horizon.Server(config.horizonUrl);
}

export function getNetworkPassphrase(network: Network = 'testnet') {
  return NETWORKS[normalizeNetwork(network)].networkPassphrase;
}

// Format amount from stroop (7 decimals)
export function formatAmount(amount: string | number): string {
  const num = typeof amount === 'string' ? parseInt(amount) : amount;
  return (num / 10_000_000).toFixed(2);
}

// Parse amount to stroop (7 decimals)
export function parseAmount(amount: string): string {
  const num = parseFloat(amount);
  return Math.floor(num * 10_000_000).toString();
}

// Format percentage (basis points to percentage)
export function formatBps(bps: number): string {
  return (bps / 100).toFixed(2);
}

// Shorten address for display
export function shortenAddress(address: string): string {
  if (!address) return '';
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

// Check if Freighter is installed
export async function isFreighterInstalled(): Promise<boolean> {
  try {
    const result = await isConnected();
    return result.isConnected;
  } catch (error) {
    console.error('Error checking Freighter installation:', error);
    return false;
  }
}

// Get network from Freighter
export async function getFreighterNetwork(): Promise<string> {
  try {
    const result = await getNetwork();
    if (result.error) {
      throw new Error(result.error);
    }
    return result.network;
  } catch (error: any) {
    throw new Error(error.message || 'Failed to get network from Freighter');
  }
}

// Get public key from Freighter
export async function getFreighterPublicKey(): Promise<string> {
  try {
    // First check if we have permission
    const allowedResult = await isAllowed();
    if (!allowedResult.isAllowed) {
      // Request permission
      const setAllowedResult = await setAllowed();
      if (!setAllowedResult.isAllowed) {
        throw new Error('User denied access to Freighter');
      }
    }

    // Get the public key
    const result = await getAddress();
    if (result.error) {
      throw new Error(result.error);
    }
    return result.address;
  } catch (error: any) {
    throw new Error(error.message || 'Failed to get public key from Freighter');
  }
}

// Sign transaction with Freighter
export async function signTransactionWithFreighter(
  xdr: string,
  network: Network
): Promise<string> {
  try {
    const networkPassphrase = getNetworkPassphrase(network);
    const publicKey = await getFreighterPublicKey();

    const result = await signTransaction(xdr, {
      networkPassphrase,
      accountToSign: publicKey,
    });

    if (result.error) {
      throw new Error(result.error);
    }

    return result.signedTxXdr;
  } catch (error: any) {
    throw new Error(error.message || 'Failed to sign transaction with Freighter');
  }
}

// Create trustline to USDC asset
export async function createUSDCTrustline(
  userAddress: string,
  issuer: string,
  network: Network = 'testnet'
): Promise<void> {
  try {
    const server = getHorizonServer(network);
    const networkPassphrase = getNetworkPassphrase(network);

    // Load account
    const account = await server.loadAccount(userAddress);

    // Create USDC asset
    const usdcAsset = new StellarSdk.Asset('USDC', issuer);

    // Build change trust transaction
    const transaction = new StellarSdk.TransactionBuilder(account, {
      fee: StellarSdk.BASE_FEE,
      networkPassphrase,
    })
      .addOperation(
        StellarSdk.Operation.changeTrust({
          asset: usdcAsset,
        })
      )
      .setTimeout(30)
      .build();

    // Sign with Freighter
    const signedXdr = await signTransactionWithFreighter(transaction.toXDR(), network);
    const signedTx = StellarSdk.TransactionBuilder.fromXDR(signedXdr, networkPassphrase);

    // Submit transaction
    const result = await server.submitTransaction(signedTx as StellarSdk.Transaction);

    if (!result.successful) {
      throw new Error('Trustline creation failed');
    }

    console.log('✅ USDC Trustline created successfully!');
  } catch (error: any) {
    console.error('Failed to create USDC trustline:', error);
    throw new Error(error.message || 'Failed to create USDC trustline');
  }
}
