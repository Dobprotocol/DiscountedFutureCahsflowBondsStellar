import { Server, Activity, DollarSign, Wallet } from 'lucide-react';
import { formatAmount, shortenAddress } from '../utils/stellar';
import type { LiquidNodeInfo } from '../types';

interface LiquidNodesProps {
  nodes: LiquidNodeInfo[];
  loading: boolean;
}

export function LiquidNodes({ nodes, loading }: LiquidNodesProps) {
  if (loading) {
    return (
      <div className="card">
        <h2 className="text-xl font-bold mb-4">Liquid Nodes</h2>
        <div className="animate-pulse space-y-3">
          <div className="h-24 bg-slate-700/30 rounded"></div>
          <div className="h-24 bg-slate-700/30 rounded"></div>
        </div>
      </div>
    );
  }

  if (nodes.length === 0) {
    return (
      <div className="card">
        <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
          <Server className="w-6 h-6 text-purple-400" />
          Liquid Nodes
        </h2>
        <div className="text-center py-8 text-slate-400">
          <Server className="w-12 h-12 mx-auto mb-2 opacity-50" />
          <p>No Liquid Nodes registered</p>
        </div>
      </div>
    );
  }

  return (
    <div className="card">
      <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
        <Server className="w-6 h-6 text-purple-400" />
        Liquid Nodes
        <span className="ml-2 px-2 py-1 bg-purple-500/20 text-purple-300 text-xs rounded-full">
          {nodes.length} Active
        </span>
      </h2>

      <div className="space-y-3">
        {nodes.map((node, index) => {
          const usdcBalance = parseFloat(formatAmount(node.usdcBalance));
          const dobBalance = parseFloat(formatAmount(node.dobBalance));

          return (
            <div
              key={node.address}
              className="stat-card hover:bg-slate-700/40 transition-all"
            >
              <div className="flex items-start justify-between mb-3">
                <div className="flex items-center gap-2">
                  <div className="w-8 h-8 bg-purple-500/20 rounded-full flex items-center justify-center">
                    <Activity className="w-4 h-4 text-purple-400" />
                  </div>
                  <div>
                    <div className="font-semibold">Node #{index + 1}</div>
                    <div className="text-xs text-slate-400 font-mono">
                      {shortenAddress(node.address)}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-1 text-xs text-green-400">
                  <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                  Online
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="bg-slate-900/50 p-2 rounded">
                  <div className="flex items-center gap-1 text-xs text-slate-400 mb-1">
                    <DollarSign className="w-3 h-3" />
                    USDC Liquidity
                  </div>
                  <div className="font-bold text-green-400">
                    {usdcBalance.toLocaleString()} USDC
                  </div>
                </div>

                <div className="bg-slate-900/50 p-2 rounded">
                  <div className="flex items-center gap-1 text-xs text-slate-400 mb-1">
                    <Wallet className="w-3 h-3" />
                    DOB Holdings
                  </div>
                  <div className="font-bold text-blue-400">
                    {dobBalance.toLocaleString()} DOB
                  </div>
                </div>
              </div>

              {node.quote && (
                <div className="mt-2 p-2 bg-yellow-500/10 border border-yellow-500/20 rounded text-xs">
                  <div className="text-yellow-300">
                    <strong>Last Quote:</strong> {formatAmount(node.quote.usdcProvided)} USDC @ {(node.quote.feeBps / 100).toFixed(2)}% fee
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      <div className="mt-4 p-3 bg-slate-900/50 rounded-lg">
        <div className="text-xs text-slate-400">
          Los Liquid Nodes proveen liquidez instantánea cuando el pool no tiene suficiente USDC.
          Los fees que cobran se calculan dinámicamente según el riesgo del Oracle.
        </div>
      </div>
    </div>
  );
}
