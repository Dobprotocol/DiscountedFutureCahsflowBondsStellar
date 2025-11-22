import { useState, useEffect } from 'react';
import { Header } from './components/Header';
import { OracleInfo } from './components/OracleInfo';
import { OracleManager } from './components/OracleManager';
import { UserBalances } from './components/UserBalances';
import { SwapInterface } from './components/SwapInterface';
import { LiquidNodes } from './components/LiquidNodes';
import { LiquidityManager } from './components/LiquidityManager';
import { Tabs } from './components/Tabs';
import { useWallet } from './hooks/useWallet';
import { useContracts } from './hooks/useContracts';
import type { ContractAddresses } from './types';
import { RefreshCw, AlertCircle, ArrowLeftRight, Settings } from 'lucide-react';

// Load contract addresses from environment or use deployed ones
const DEFAULT_ADDRESSES: ContractAddresses = {
  token: import.meta.env.VITE_TOKEN_ID || '',
  oracle: import.meta.env.VITE_ORACLE_ID || '',
  pool: import.meta.env.VITE_POOL_ID || '',
  usdc: import.meta.env.VITE_USDC_ID || '',
  liquidNodes: [
    import.meta.env.VITE_LN1_ID || '',
    import.meta.env.VITE_LN2_ID || '',
  ].filter(Boolean),
};

function App() {
  const { wallet, isInstalled, loading: walletLoading, error: walletError, connect, disconnect } = useWallet();
  const [addresses, setAddresses] = useState<ContractAddresses>(DEFAULT_ADDRESSES);
  const [showConfig, setShowConfig] = useState(false);
  const [activeTab, setActiveTab] = useState('trading');

  const {
    service,
    oracleData,
    poolReserves,
    userBalances,
    liquidNodes,
    loading: contractsLoading,
    error: contractsError,
    refreshData,
  } = useContracts(addresses, wallet.network, wallet.publicKey);

  // Check if addresses are configured
  const isConfigured =
    addresses.token && addresses.oracle && addresses.pool && addresses.usdc;

  useEffect(() => {
    // Show config modal if not configured
    if (!isConfigured && !showConfig) {
      setShowConfig(true);
    }
  }, [isConfigured, showConfig]);

  return (
    <div className="min-h-screen">
      <Header
        wallet={wallet}
        onConnect={connect}
        onDisconnect={disconnect}
        isInstalled={isInstalled}
      />

      <main className="container mx-auto px-4 py-8">
        {/* Configuration Modal */}
        {showConfig && (
          <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
            <div className="card max-w-2xl w-full m-4">
              <h2 className="text-2xl font-bold mb-4">Contract Configuration</h2>
              <p className="text-slate-400 mb-6">
                Enter deployed contract addresses. Find them in{' '}
                <code className="bg-slate-700 px-2 py-1 rounded">deployed-contracts.env</code>
              </p>

              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-slate-400 mb-2">Token Contract</label>
                  <input
                    type="text"
                    value={addresses.token}
                    onChange={(e) =>
                      setAddresses({ ...addresses, token: e.target.value })
                    }
                    placeholder="CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
                    className="input font-mono text-sm"
                  />
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">Oracle Contract</label>
                  <input
                    type="text"
                    value={addresses.oracle}
                    onChange={(e) =>
                      setAddresses({ ...addresses, oracle: e.target.value })
                    }
                    placeholder="CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
                    className="input font-mono text-sm"
                  />
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">AMM Pool Contract</label>
                  <input
                    type="text"
                    value={addresses.pool}
                    onChange={(e) =>
                      setAddresses({ ...addresses, pool: e.target.value })
                    }
                    placeholder="CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
                    className="input font-mono text-sm"
                  />
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">USDC Token Contract</label>
                  <input
                    type="text"
                    value={addresses.usdc}
                    onChange={(e) =>
                      setAddresses({ ...addresses, usdc: e.target.value })
                    }
                    placeholder="CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
                    className="input font-mono text-sm"
                  />
                </div>
              </div>

              <div className="flex gap-3 mt-6">
                <button
                  onClick={() => {
                    if (isConfigured) {
                      setShowConfig(false);
                      refreshData();
                    }
                  }}
                  disabled={!isConfigured}
                  className="btn-primary flex-1"
                >
                  Save Configuration
                </button>
                {isConfigured && (
                  <button
                    onClick={() => setShowConfig(false)}
                    className="btn-secondary"
                  >
                    Cancel
                  </button>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Errors */}
        {(walletError || contractsError) && (
          <div className="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-lg flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
            <div>
              <div className="font-semibold text-red-300">Error</div>
              <div className="text-sm text-red-400">
                {walletError || contractsError}
              </div>
            </div>
          </div>
        )}

        {/* Not Configured */}
        {!isConfigured ? (
          <div className="card text-center py-12">
            <AlertCircle className="w-16 h-16 mx-auto mb-4 text-yellow-400" />
            <h2 className="text-2xl font-bold mb-2">Configuration Required</h2>
            <p className="text-slate-400 mb-6">
              Please configure the contract addresses to continue
            </p>
            <button onClick={() => setShowConfig(true)} className="btn-primary">
              Configure Contracts
            </button>
          </div>
        ) : (
          <>
            {/* Header with Refresh */}
            <div className="flex justify-between items-center mb-6">
              <h1 className="text-2xl font-bold">DOB Liquidity Pool</h1>
              <button
                onClick={refreshData}
                disabled={contractsLoading}
                className="btn-secondary flex items-center gap-2"
              >
                <RefreshCw className={`w-4 h-4 ${contractsLoading ? 'animate-spin' : ''}`} />
                Refresh
              </button>
            </div>

            {/* Tabs */}
            <Tabs
              tabs={[
                { id: 'trading', label: 'Trading', icon: <ArrowLeftRight className="w-4 h-4" /> },
                { id: 'settings', label: 'Settings', icon: <Settings className="w-4 h-4" /> },
              ]}
              activeTab={activeTab}
              onChange={setActiveTab}
            />

            {/* Main Grid */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              {/* Left Column - Tab Content */}
              <div className="lg:col-span-2 space-y-6">
                {activeTab === 'trading' && (
                  <>
                    <OracleInfo data={oracleData} loading={contractsLoading} />
                    <SwapInterface
                      service={service}
                      poolId={addresses.pool}
                      oracleId={addresses.oracle}
                      userAddress={wallet.publicKey}
                      poolReserves={poolReserves}
                      oracleData={oracleData}
                      onSuccess={refreshData}
                    />
                  </>
                )}

                {activeTab === 'settings' && (
                  <>
                    <OracleManager
                      oracleId={addresses.oracle}
                      data={oracleData}
                      service={service}
                      userAddress={wallet.publicKey}
                      onSuccess={refreshData}
                    />
                    <LiquidNodes nodes={liquidNodes} loading={contractsLoading} />
                    <LiquidityManager
                      service={service}
                      poolId={addresses.pool}
                      userAddress={wallet.publicKey}
                      userBalances={userBalances}
                      poolReserves={poolReserves}
                      onSuccess={refreshData}
                    />
                  </>
                )}
              </div>

              {/* Right Column - Always Visible */}
              <div className="space-y-6">
                <UserBalances
                  balances={userBalances}
                  loading={contractsLoading}
                  userAddress={wallet.publicKey}
                  network={wallet.network}
                  onRefresh={refreshData}
                />
              </div>
            </div>

            {/* Footer */}
            <footer className="mt-12 text-center text-sm text-slate-500">
              <p>DOB Liquidity Pool - Solar Farm 2035</p>
              <p className="mt-1">
                Built on Stellar Soroban •{' '}
                <button
                  onClick={() => setShowConfig(true)}
                  className="text-blue-400 hover:text-blue-300"
                >
                  Configure Addresses
                </button>
              </p>
            </footer>
          </>
        )}
      </main>
    </div>
  );
}

export default App;
