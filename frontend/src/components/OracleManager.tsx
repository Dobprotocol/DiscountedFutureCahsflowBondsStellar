import { useState } from 'react';
import { Settings, TrendingUp, AlertTriangle, Save } from 'lucide-react';
import { formatAmount, parseAmount } from '../utils/stellar';
import type { OracleData } from '../types';
import ContractService from '../utils/contracts';

interface OracleManagerProps {
  oracleId: string;
  data: OracleData | null;
  service: ContractService;
  userAddress: string | null;
  onSuccess?: () => void;
}

export function OracleManager({ oracleId, data, service, userAddress, onSuccess }: OracleManagerProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [fairPriceInput, setFairPriceInput] = useState('');
  const [riskInput, setRiskInput] = useState('');
  const [updating, setUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isUpdater, setIsUpdater] = useState<boolean | null>(null);

  // Check if user is the updater
  const checkUpdater = async () => {
    if (!userAddress) return;
    try {
      const updater = await service.getOracleUpdater(oracleId);
      setIsUpdater(updater === userAddress);
    } catch (err) {
      console.error('Failed to check updater:', err);
      setIsUpdater(false);
    }
  };

  const handleStartEdit = async () => {
    if (!data) return;

    await checkUpdater();

    setFairPriceInput(formatAmount(data.fairPrice));
    setRiskInput((parseInt(data.risk) / 100).toFixed(2));
    setIsEditing(true);
    setError(null);
  };

  const handleUpdate = async () => {
    if (!userAddress || !data) return;

    setUpdating(true);
    setError(null);

    try {
      // Validation
      if (parseFloat(fairPriceInput) <= 0) {
        throw new Error('Fair price must be greater than 0');
      }
      if (parseFloat(riskInput) < 0 || parseFloat(riskInput) > 100) {
        throw new Error('Risk must be between 0% and 100%');
      }

      const fairPriceValue = parseAmount(fairPriceInput); // Convert to stroops (7 decimals)
      const riskValue = Math.floor(parseFloat(riskInput) * 100).toString(); // Convert to basis points

      console.log('Updating oracle with:', { fairPriceValue, riskValue });

      await service.updateOracle(oracleId, userAddress, fairPriceValue, riskValue);

      setIsEditing(false);
      if (onSuccess) {
        setTimeout(() => onSuccess(), 2000);
      }
    } catch (err: any) {
      setError(err.message || 'Failed to update oracle');
    } finally {
      setUpdating(false);
    }
  };

  if (!data) {
    return null;
  }

  const currentFairPrice = formatAmount(data.fairPrice);
  const currentRisk = (parseInt(data.risk) / 100).toFixed(2);
  const calculatedFee = (3 + parseFloat(currentRisk) / 10).toFixed(2);

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold flex items-center gap-2">
          <Settings className="w-6 h-6 text-purple-400" />
          Oracle Settings
        </h2>
        {!isEditing && userAddress && (
          <button
            onClick={handleStartEdit}
            className="btn-secondary text-sm flex items-center gap-2"
          >
            <Settings className="w-4 h-4" />
            Update
          </button>
        )}
      </div>

      {isEditing && isUpdater === false && (
        <div className="mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
          <div className="flex items-start gap-2">
            <AlertTriangle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
            <div className="text-sm text-red-300">
              You are not authorized to update this oracle
            </div>
          </div>
        </div>
      )}

      {error && (
        <div className="mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
          <div className="text-sm text-red-300">{error}</div>
        </div>
      )}

      <div className="space-y-4">
        {/* Fair Price */}
        <div className="stat-card">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <TrendingUp className="w-5 h-5 text-blue-400" />
              <span className="text-slate-300">Fair Price</span>
            </div>
          </div>
          {isEditing ? (
            <div>
              <input
                type="number"
                value={fairPriceInput}
                onChange={(e) => setFairPriceInput(e.target.value)}
                step="0.01"
                min="0.01"
                placeholder="1.00"
                disabled={!isUpdater}
                className="input w-full"
              />
              <div className="text-xs text-slate-500 mt-1">
                USDC per DOB token
              </div>
            </div>
          ) : (
            <div className="text-2xl font-bold text-green-400">
              ${currentFairPrice} USDC
            </div>
          )}
        </div>

        {/* Default Risk */}
        <div className="stat-card">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-orange-400" />
              <span className="text-slate-300">Default Risk</span>
            </div>
          </div>
          {isEditing ? (
            <div>
              <input
                type="number"
                value={riskInput}
                onChange={(e) => setRiskInput(e.target.value)}
                step="0.1"
                min="0"
                max="100"
                placeholder="10.00"
                disabled={!isUpdater}
                className="input w-full"
              />
              <div className="text-xs text-slate-500 mt-1">
                Percentage (0-100%)
              </div>
            </div>
          ) : (
            <div className="text-2xl font-bold text-orange-400">
              {currentRisk}%
            </div>
          )}
        </div>

        {/* Calculated Penalty */}
        <div className="stat-card bg-slate-700/30">
          <div className="flex items-center gap-2 mb-2">
            <span className="text-slate-400 text-sm">Swap Sell Fee</span>
          </div>
          <div className="text-lg font-bold text-purple-400">
            {isEditing && riskInput ? (
              (3 + parseFloat(riskInput) / 10).toFixed(2)
            ) : (
              calculatedFee
            )}%
          </div>
          <div className="text-xs text-slate-500 mt-1">
            3% base + risk/10 (used when selling DOB)
          </div>
        </div>
      </div>

      {/* Action Buttons */}
      {isEditing && (
        <div className="flex gap-3 mt-6">
          <button
            onClick={handleUpdate}
            disabled={updating || !isUpdater}
            className="btn-primary flex-1 flex items-center justify-center gap-2"
          >
            <Save className="w-4 h-4" />
            {updating ? 'Updating...' : 'Save Changes'}
          </button>
          <button
            onClick={() => {
              setIsEditing(false);
              setError(null);
            }}
            disabled={updating}
            className="btn-secondary"
          >
            Cancel
          </button>
        </div>
      )}
    </div>
  );
}
