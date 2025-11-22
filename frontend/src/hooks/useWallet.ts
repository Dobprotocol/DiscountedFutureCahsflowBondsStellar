import { useState, useEffect } from 'react';
import {
  isFreighterInstalled,
  getFreighterPublicKey,
  getFreighterNetwork,
} from '../utils/stellar';
import type { WalletState, Network } from '../types';

export function useWallet() {
  const [wallet, setWallet] = useState<WalletState>({
    connected: false,
    publicKey: null,
    network: 'testnet',
  });
  const [isInstalled, setIsInstalled] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;

    async function checkWithRetry() {
      // Try multiple times with delays since Freighter might load after page
      for (let i = 0; i < 5; i++) {
        if (!mounted) return;

        try {
          const installed = await isFreighterInstalled();
          if (installed && mounted) {
            console.log('✅ Freighter detected');
            setIsInstalled(true);
            return;
          }
        } catch (err) {
          console.log(`Attempt ${i + 1}: Freighter not found yet...`);
        }

        await new Promise(resolve => setTimeout(resolve, 500));
      }

      if (mounted) {
        console.log('❌ Freighter not detected after retries');
        setIsInstalled(false);
      }
    }

    checkWithRetry();

    return () => {
      mounted = false;
    };
  }, []);

  async function connect() {
    setLoading(true);
    setError(null);

    try {
      console.log('🔗 Attempting to connect to Freighter...');

      if (!isInstalled) {
        throw new Error('Freighter wallet not installed. Please install from freighter.app');
      }

      console.log('📝 Requesting public key...');
      const publicKey = await getFreighterPublicKey();
      console.log('✅ Public key received:', publicKey.slice(0, 8) + '...');

      console.log('🌐 Getting network...');
      const networkRaw = await getFreighterNetwork();
      console.log('✅ Network:', networkRaw);

      // Normalize network (Freighter returns 'TESTNET', we need 'testnet')
      const network = networkRaw.toLowerCase() as Network;

      setWallet({
        connected: true,
        publicKey,
        network,
      });

      console.log('✅ Wallet connected successfully!');
    } catch (err: any) {
      const errorMsg = err.message || 'Failed to connect wallet';
      setError(errorMsg);
      console.error('❌ Wallet connection error:', errorMsg);
    } finally {
      setLoading(false);
    }
  }

  function disconnect() {
    setWallet({
      connected: false,
      publicKey: null,
      network: 'testnet',
    });
  }

  return {
    wallet,
    isInstalled,
    loading,
    error,
    connect,
    disconnect,
  };
}
