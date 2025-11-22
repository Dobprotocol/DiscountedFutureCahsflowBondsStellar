import { TrendingUp, AlertTriangle, DollarSign } from 'lucide-react';
import { formatAmount, formatBps } from '../utils/stellar';
import type { OracleData } from '../types';

interface OracleInfoProps {
  data: OracleData | null;
  loading: boolean;
}

export function OracleInfo({ data, loading }: OracleInfoProps) {
  if (loading || !data) {
    return (
      <div className="card">
        <h2 className="text-xl font-bold mb-4">Oracle Information</h2>
        <div className="animate-pulse space-y-4">
          <div className="h-20 bg-slate-700/30 rounded"></div>
          <div className="h-20 bg-slate-700/30 rounded"></div>
        </div>
      </div>
    );
  }

  const fairPrice = parseFloat(formatAmount(data.fairPrice));
  const risk = parseFloat(formatBps(parseInt(data.risk)));

  const getRiskColor = (risk: number) => {
    if (risk < 15) return 'text-green-400';
    if (risk < 30) return 'text-yellow-400';
    if (risk < 50) return 'text-orange-400';
    return 'text-red-400';
  };

  const getRiskLabel = (risk: number) => {
    if (risk < 15) return 'Low Risk';
    if (risk < 30) return 'Medium Risk';
    if (risk < 50) return 'High Risk';
    return 'Very High Risk';
  };

  return (
    <div className="card">
      <h2 className="text-xl font-bold mb-6 flex items-center gap-2">
        <TrendingUp className="w-6 h-6 text-blue-400" />
        Oracle Information
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Fair Price */}
        <div className="stat-card">
          <div className="flex items-start justify-between">
            <div>
              <div className="text-slate-400 text-sm mb-1">Fair Price</div>
              <div className="text-3xl font-bold text-green-400 flex items-center gap-1">
                <DollarSign className="w-6 h-6" />
                {fairPrice.toFixed(2)}
              </div>
              <div className="text-xs text-slate-500 mt-1">per DOB token</div>
            </div>
          </div>
        </div>

        {/* Risk */}
        <div className="stat-card">
          <div className="flex items-start justify-between">
            <div>
              <div className="text-slate-400 text-sm mb-1">Default Risk</div>
              <div className={`text-3xl font-bold ${getRiskColor(risk)} flex items-center gap-1`}>
                <AlertTriangle className="w-6 h-6" />
                {risk.toFixed(2)}%
              </div>
              <div className="text-xs text-slate-500 mt-1">{getRiskLabel(risk)}</div>
            </div>
          </div>
        </div>
      </div>

      <div className="mt-4 p-3 bg-slate-900/50 rounded-lg">
        <div className="text-xs text-slate-400">
          Fair Price determines the minting price for new DOB tokens. Risk affects the fees charged by Liquid Nodes when providing liquidity.
        </div>
      </div>
    </div>
  );
}
