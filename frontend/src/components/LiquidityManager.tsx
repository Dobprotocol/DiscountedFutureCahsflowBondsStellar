import { useState } from 'react';
import { Plus, Minus, Droplet, Info } from 'lucide-react';
import { formatAmount, parseAmount } from '../utils/stellar';
import ContractService from '../utils/contracts';
import type { UserBalances, PoolReserves } from '../types';

interface LiquidityManagerProps {
  service: ContractService;
  poolId: string;
  userAddress: string | null;
  userBalances: UserBalances | null;
  poolReserves: PoolReserves | null;
  onSuccess: () => void;
}

export function LiquidityManager({
  service,
  poolId,
  userAddress,
  userBalances,
  poolReserves,
  onSuccess,
}: LiquidityManagerProps) {
  const [mode, setMode] = useState<'add' | 'remove'>('add');
  const [usdcAmount, setUsdcAmount] = useState('');
  const [dobAmount, setDobAmount] = useState('');
  const [lpAmount, setLpAmount] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAddLiquidity = async () => {
    if (!userAddress) {
      setError('Please connect your wallet');
      return;
    }

    if (!usdcAmount || !dobAmount || parseFloat(usdcAmount) <= 0 || parseFloat(dobAmount) <= 0) {
      setError('Please enter amounts for both tokens');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const usdc = parseAmount(usdcAmount);
      const dob = parseAmount(dobAmount);

      await service.addLiquidity(poolId, userAddress, usdc, dob);

      setUsdcAmount('');
      setDobAmount('');
      onSuccess();
    } catch (err: any) {
      setError(err.message || 'Transaction failed');
      console.error('Add liquidity error:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveLiquidity = async () => {
    if (!userAddress) {
      setError('Please connect your wallet');
      return;
    }

    if (!lpAmount || parseFloat(lpAmount) <= 0) {
      setError('Please enter LP shares amount');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const lp = parseAmount(lpAmount);
      await service.removeLiquidity(poolId, userAddress, lp);

      setLpAmount('');
      onSuccess();
    } catch (err: any) {
      setError(err.message || 'Transaction failed');
      console.error('Remove liquidity error:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold flex items-center gap-2">
          <Droplet className="w-6 h-6 text-purple-400" />
          Manage Liquidity
        </h2>
        <div className="flex gap-2">
          <button
            onClick={() => {
              setMode('add');
              setError(null);
            }}
            className={`px-4 py-2 rounded-lg font-semibold transition-all ${
              mode === 'add'
                ? 'bg-green-600 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
          >
            <Plus className="w-4 h-4 inline mr-1" />
            Add
          </button>
          <button
            onClick={() => {
              setMode('remove');
              setError(null);
            }}
            className={`px-4 py-2 rounded-lg font-semibold transition-all ${
              mode === 'remove'
                ? 'bg-red-600 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
          >
            <Minus className="w-4 h-4 inline mr-1" />
            Remove
          </button>
        </div>
      </div>

      {/* Pool Reserves */}
      {poolReserves && (
        <div className="mb-6 p-4 bg-slate-700/30 rounded-lg">
          <div className="text-sm text-slate-400 mb-3 flex items-center gap-2">
            <Info className="w-4 h-4" />
            Pool Reserves
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <div className="text-xs text-slate-500">USDC</div>
              <div className="text-lg font-bold text-green-400">
                {formatAmount(poolReserves.usdc)}
              </div>
            </div>
            <div>
              <div className="text-xs text-slate-500">DOB</div>
              <div className="text-lg font-bold text-blue-400">
                {formatAmount(poolReserves.dob)}
              </div>
            </div>
          </div>
        </div>
      )}

      {mode === 'add' ? (
        <div className="space-y-4">
          {/* USDC Input */}
          <div>
            <label className="block text-sm text-slate-400 mb-2">
              USDC Amount
              {userBalances && (
                <span className="float-right text-xs">
                  Balance: {formatAmount(userBalances.usdc)}
                </span>
              )}
            </label>
            <input
              type="number"
              value={usdcAmount}
              onChange={(e) => setUsdcAmount(e.target.value)}
              placeholder="0.00"
              className="input"
              step="0.01"
            />
          </div>

          {/* DOB Input */}
          <div>
            <label className="block text-sm text-slate-400 mb-2">
              DOB Amount
              {userBalances && (
                <span className="float-right text-xs">
                  Balance: {formatAmount(userBalances.dob)}
                </span>
              )}
            </label>
            <input
              type="number"
              value={dobAmount}
              onChange={(e) => setDobAmount(e.target.value)}
              placeholder="0.00"
              className="input"
              step="0.01"
            />
          </div>

          <div className="p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg text-sm text-blue-300">
            You'll receive LP shares proportional to your contribution. LP shares represent your pool ownership and earn trading fees.
          </div>

          <button
            onClick={handleAddLiquidity}
            disabled={loading || !userAddress}
            className="btn-primary w-full"
          >
            {loading ? 'Processing...' : 'Add Liquidity'}
          </button>
        </div>
      ) : (
        <div className="space-y-4">
          {/* LP Shares Input */}
          <div>
            <label className="block text-sm text-slate-400 mb-2">
              LP Shares to Remove
              {userBalances && (
                <span className="float-right text-xs">
                  Balance: {formatAmount(userBalances.lpShares)}
                </span>
              )}
            </label>
            <input
              type="number"
              value={lpAmount}
              onChange={(e) => setLpAmount(e.target.value)}
              placeholder="0.00"
              className="input"
              step="0.01"
            />
          </div>

          <div className="p-3 bg-orange-500/10 border border-orange-500/20 rounded-lg text-sm text-orange-300">
            You'll burn your LP shares and receive a proportional amount of USDC and DOB from the pool.
          </div>

          <button
            onClick={handleRemoveLiquidity}
            disabled={loading || !userAddress}
            className="btn-primary w-full"
          >
            {loading ? 'Processing...' : 'Remove Liquidity'}
          </button>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-300 text-sm">
          {error}
        </div>
      )}
    </div>
  );
}
