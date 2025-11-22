import { useState } from 'react';
import { ArrowDownUp, TrendingUp, TrendingDown, Info } from 'lucide-react';
import { formatAmount, parseAmount } from '../utils/stellar';
import ContractService from '../utils/contracts';
import type { PoolReserves, OracleData } from '../types';

interface SwapInterfaceProps {
  service: ContractService;
  poolId: string;
  oracleId: string;
  userAddress: string | null;
  poolReserves: PoolReserves | null;
  oracleData: OracleData | null;
  onSuccess: () => void;
}

export function SwapInterface({
  service,
  poolId,
  oracleId,
  userAddress,
  poolReserves,
  oracleData,
  onSuccess,
}: SwapInterfaceProps) {
  const [mode, setMode] = useState<'buy' | 'sell'>('buy');
  const [inputAmount, setInputAmount] = useState('');
  const [estimatedOutput, setEstimatedOutput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleInputChange = async (value: string) => {
    setInputAmount(value);
    setError(null);

    if (!value || parseFloat(value) <= 0) {
      setEstimatedOutput('');
      return;
    }

    try {
      const amount = parseAmount(value);
      let quote: string;

      if (mode === 'buy') {
        quote = await service.getSwapBuyQuote(oracleId, amount);
      } else {
        quote = await service.getSwapSellQuote(poolId, amount);
      }

      setEstimatedOutput(formatAmount(quote));
    } catch (err: any) {
      console.error('Failed to get quote:', err);
      setEstimatedOutput('');
    }
  };

  const handleSwap = async () => {
    if (!userAddress) {
      setError('Please connect your wallet');
      return;
    }

    if (!inputAmount || parseFloat(inputAmount) <= 0) {
      setError('Please enter an amount');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const amount = parseAmount(inputAmount);

      if (mode === 'buy') {
        const result = await service.swapBuy(poolId, userAddress, amount);
        console.log('Buy successful, DOB received:', formatAmount(result));
      } else {
        const result = await service.swapSell(poolId, userAddress, amount);
        console.log('Sell successful, USDC received:', formatAmount(result));
      }

      setInputAmount('');
      setEstimatedOutput('');

      // Refresh data after successful swap
      setTimeout(() => {
        onSuccess();
      }, 1000);
    } catch (err: any) {
      // Better error messages
      let errorMessage = err.message || 'Transaction failed';

      if (errorMessage.includes('InsufficientBalance') || errorMessage.includes('#2')) {
        errorMessage = mode === 'buy'
          ? 'Insufficient USDC balance'
          : 'Insufficient DOB balance. You need to buy DOB tokens first.';
      } else if (errorMessage.includes('NoLiquidityAvailable') || errorMessage.includes('#4')) {
        errorMessage = 'No liquidity available in pool or Liquid Nodes';
      } else if (errorMessage.includes('burn')) {
        errorMessage = 'Failed to burn tokens. Make sure you have sufficient DOB balance.';
      }

      setError(errorMessage);
      console.error('Swap error:', err);
    } finally {
      setLoading(false);
    }
  };

  const toggleMode = () => {
    setMode(mode === 'buy' ? 'sell' : 'buy');
    setInputAmount('');
    setEstimatedOutput('');
    setError(null);
  };

  const inputLabel = mode === 'buy' ? 'USDC' : 'DOB';
  const outputLabel = mode === 'buy' ? 'DOB' : 'USDC';
  const nav = oracleData ? parseFloat(formatAmount(oracleData.fairPrice)) : 0;

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold flex items-center gap-2">
          <ArrowDownUp className="w-6 h-6 text-blue-400" />
          Swap Tokens
        </h2>
        <div className="flex gap-2">
          <button
            onClick={() => {
              setMode('buy');
              setInputAmount('');
              setEstimatedOutput('');
            }}
            className={`px-4 py-2 rounded-lg font-semibold transition-all ${
              mode === 'buy'
                ? 'bg-green-600 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
          >
            <TrendingUp className="w-4 h-4 inline mr-1" />
            Buy
          </button>
          <button
            onClick={() => {
              setMode('sell');
              setInputAmount('');
              setEstimatedOutput('');
            }}
            className={`px-4 py-2 rounded-lg font-semibold transition-all ${
              mode === 'sell'
                ? 'bg-red-600 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
          >
            <TrendingDown className="w-4 h-4 inline mr-1" />
            Sell
          </button>
        </div>
      </div>

      {/* Input */}
      <div className="space-y-4">
        <div>
          <label className="block text-sm text-slate-400 mb-2">
            You {mode === 'buy' ? 'Pay' : 'Sell'}
          </label>
          <div className="relative">
            <input
              type="number"
              value={inputAmount}
              onChange={(e) => handleInputChange(e.target.value)}
              placeholder="0.00"
              className="input pr-20"
              step="0.01"
            />
            <div className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-400 font-semibold">
              {inputLabel}
            </div>
          </div>
        </div>

        {/* Arrow */}
        <div className="flex justify-center">
          <button
            onClick={toggleMode}
            className="w-10 h-10 bg-slate-700 hover:bg-slate-600 rounded-full flex items-center justify-center transition-all"
          >
            <ArrowDownUp className="w-5 h-5" />
          </button>
        </div>

        {/* Output */}
        <div>
          <label className="block text-sm text-slate-400 mb-2">
            You {mode === 'buy' ? 'Receive' : 'Get'}
          </label>
          <div className="relative">
            <input
              type="text"
              value={estimatedOutput}
              readOnly
              placeholder="0.00"
              className="input pr-20 bg-slate-900/80"
            />
            <div className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-400 font-semibold">
              {outputLabel}
            </div>
          </div>
        </div>
      </div>

      {/* Info */}
      {mode === 'buy' && oracleData && (
        <div className="mt-4 p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
          <div className="flex items-start gap-2 text-sm text-blue-300">
            <Info className="w-4 h-4 mt-0.5 flex-shrink-0" />
            <div>
              <strong>AfterSwap Hook:</strong> Tokens DOB se mintean al NAV actual (${nav.toFixed(2)}).
              Fee: 1% para el DEX, 99% va al operador de infraestructura.
            </div>
          </div>
        </div>
      )}

      {mode === 'sell' && (
        <div className="mt-4 p-3 bg-orange-500/10 border border-orange-500/20 rounded-lg">
          <div className="flex items-start gap-2 text-sm text-orange-300">
            <Info className="w-4 h-4 mt-0.5 flex-shrink-0" />
            <div>
              <strong>BeforeSwap Hook:</strong> Si el pool no tiene suficiente liquidez, el sistema
              consultará automáticamente a los Liquid Nodes. Los tokens se queman después del swap.
            </div>
          </div>
        </div>
      )}

      {/* Pool Stats */}
      {poolReserves && (
        <div className="mt-4 grid grid-cols-2 gap-3">
          <div className="stat-card">
            <div className="text-xs text-slate-400 mb-1">Pool USDC</div>
            <div className="text-lg font-bold">{formatAmount(poolReserves.usdc)}</div>
          </div>
          <div className="stat-card">
            <div className="text-xs text-slate-400 mb-1">Pool DOB</div>
            <div className="text-lg font-bold">{formatAmount(poolReserves.dob)}</div>
          </div>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-300 text-sm">
          {error}
        </div>
      )}

      {/* Button */}
      <button
        onClick={handleSwap}
        disabled={loading || !userAddress || !inputAmount || parseFloat(inputAmount) <= 0}
        className="btn-primary w-full mt-6"
      >
        {loading
          ? 'Processing...'
          : !userAddress
          ? 'Connect Wallet'
          : mode === 'buy'
          ? 'Buy DOB'
          : 'Sell DOB'}
      </button>
    </div>
  );
}
