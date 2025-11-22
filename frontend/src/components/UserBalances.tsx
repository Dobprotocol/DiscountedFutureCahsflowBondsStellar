import { useState } from 'react';
import { Wallet, DollarSign, Droplet, AlertCircle } from 'lucide-react';
import { formatAmount, createUSDCTrustline } from '../utils/stellar';
import type { UserBalances, Network } from '../types';

interface UserBalancesProps {
  balances: UserBalances | null;
  loading: boolean;
  userAddress: string | null;
  network: Network;
  onRefresh?: () => void;
}

// USDC Issuer (deployer address from deployed-contracts.env)
const USDC_ISSUER = 'GACZ5Q42IZTOIOWSJR4B5SIQ45C4C5VVXEKXRFQ4N2GUWTAN2DOLYLWZ';

export function UserBalances({ balances, loading, userAddress, network, onRefresh }: UserBalancesProps) {
  const [creatingTrustline, setCreatingTrustline] = useState(false);
  const [trustlineError, setTrustlineError] = useState<string | null>(null);

  const handleCreateTrustline = async () => {
    if (!userAddress) return;

    setCreatingTrustline(true);
    setTrustlineError(null);

    try {
      await createUSDCTrustline(userAddress, USDC_ISSUER, network);
      // Refresh balances after creating trustline
      if (onRefresh) {
        setTimeout(() => onRefresh(), 2000);
      }
    } catch (error: any) {
      setTrustlineError(error.message || 'Failed to create trustline');
    } finally {
      setCreatingTrustline(false);
    }
  };

  if (loading || !balances) {
    return (
      <div className="card">
        <h2 className="text-xl font-bold mb-4">Your Balances</h2>
        <div className="animate-pulse space-y-3">
          <div className="h-16 bg-slate-700/30 rounded"></div>
          <div className="h-16 bg-slate-700/30 rounded"></div>
          <div className="h-16 bg-slate-700/30 rounded"></div>
        </div>
      </div>
    );
  }

  const needsUSDCTrustline = balances.usdc === '0';

  const balanceItems = [
    {
      icon: DollarSign,
      label: 'USDC',
      amount: formatAmount(balances.usdc),
      color: 'text-green-400',
    },
    {
      icon: Wallet,
      label: 'DOB',
      amount: formatAmount(balances.dob),
      color: 'text-blue-400',
    },
    {
      icon: Droplet,
      label: 'LP Shares',
      amount: formatAmount(balances.lpShares),
      color: 'text-purple-400',
    },
  ];

  return (
    <div className="card">
      <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
        <Wallet className="w-6 h-6 text-blue-400" />
        Your Balances
      </h2>

      {/* USDC Trustline Warning */}
      {needsUSDCTrustline && userAddress && (
        <div className="mb-4 p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
          <div className="flex items-start gap-2">
            <AlertCircle className="w-5 h-5 text-yellow-400 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <div className="text-sm text-yellow-300 font-semibold mb-1">
                USDC Trustline Required
              </div>
              <div className="text-xs text-yellow-400/80 mb-2">
                Create a trustline to receive USDC tokens
              </div>
              <button
                onClick={handleCreateTrustline}
                disabled={creatingTrustline}
                className="btn-primary text-xs py-1 px-3"
              >
                {creatingTrustline ? 'Creating...' : 'Create Trustline'}
              </button>
            </div>
          </div>
          {trustlineError && (
            <div className="mt-2 text-xs text-red-400">{trustlineError}</div>
          )}
        </div>
      )}

      <div className="space-y-3">
        {balanceItems.map((item) => (
          <div key={item.label} className="stat-card flex items-center justify-between">
            <div className="flex items-center gap-3">
              <item.icon className={`w-5 h-5 ${item.color}`} />
              <span className="text-slate-300">{item.label}</span>
            </div>
            <div className="text-xl font-bold">{item.amount}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
