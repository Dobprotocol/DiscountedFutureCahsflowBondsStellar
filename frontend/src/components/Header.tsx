import { Wallet, Sun } from 'lucide-react';
import { shortenAddress } from '../utils/stellar';
import type { WalletState } from '../types';

interface HeaderProps {
  wallet: WalletState;
  onConnect: () => void;
  onDisconnect: () => void;
  isInstalled: boolean;
}

export function Header({ wallet, onConnect, onDisconnect, isInstalled }: HeaderProps) {
  return (
    <header className="border-b border-slate-700/50 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-10">
      <div className="container mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-br from-yellow-400 to-orange-500 rounded-lg flex items-center justify-center">
              <Sun className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-xl font-bold">DOB Liquidity</h1>
              <p className="text-xs text-slate-400">Solar Farm 2035</p>
            </div>
          </div>

          <div className="flex items-center gap-4">
            {wallet.connected ? (
              <div className="flex items-center gap-3">
                <div className="text-right">
                  <div className="text-sm font-medium">
                    {shortenAddress(wallet.publicKey || '')}
                  </div>
                  <div className="text-xs text-slate-400 capitalize">
                    {wallet.network}
                  </div>
                </div>
                <button
                  onClick={onDisconnect}
                  className="bg-slate-700 hover:bg-slate-600 px-4 py-2 rounded-lg transition-all"
                >
                  Disconnect
                </button>
              </div>
            ) : (
              <button
                onClick={onConnect}
                disabled={!isInstalled}
                className="btn-primary flex items-center gap-2"
              >
                <Wallet className="w-5 h-5" />
                {isInstalled ? 'Connect Freighter' : 'Install Freighter'}
              </button>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
