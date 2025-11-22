import { useState, useEffect, useCallback } from 'react';
import ContractService from '../utils/contracts';
import type { Network, OracleData, PoolReserves, UserBalances, LiquidNodeInfo, ContractAddresses } from '../types';

export function useContracts(
  addresses: ContractAddresses,
  network: Network,
  userAddress: string | null
) {
  const [service] = useState(() => new ContractService(network));
  const [oracleData, setOracleData] = useState<OracleData | null>(null);
  const [poolReserves, setPoolReserves] = useState<PoolReserves | null>(null);
  const [userBalances, setUserBalances] = useState<UserBalances | null>(null);
  const [liquidNodes, setLiquidNodes] = useState<LiquidNodeInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchOracleData = useCallback(async () => {
    try {
      const data = await service.getOracleData(addresses.oracle);
      setOracleData(data);
    } catch (err: any) {
      console.error('Failed to fetch oracle data:', err);
    }
  }, [service, addresses.oracle]);

  const fetchPoolReserves = useCallback(async () => {
    try {
      const reserves = await service.getPoolReserves(addresses.pool);
      setPoolReserves(reserves);
    } catch (err: any) {
      console.error('Failed to fetch pool reserves:', err);
    }
  }, [service, addresses.pool]);

  const fetchUserBalances = useCallback(async () => {
    if (!userAddress) return;

    try {
      const balances = await service.getUserBalances(
        addresses.token,
        addresses.usdc,
        addresses.pool,
        userAddress
      );
      setUserBalances(balances);
    } catch (err: any) {
      console.error('Failed to fetch user balances:', err);
    }
  }, [service, addresses, userAddress]);

  const fetchLiquidNodes = useCallback(async () => {
    try {
      const nodes = await service.getLiquidNodes(addresses.pool);
      const nodesInfo: LiquidNodeInfo[] = await Promise.all(
        nodes.map(async (address) => {
          const [usdcBalance, dobBalance] = await Promise.all([
            service.getBalance(addresses.usdc, address),
            service.getBalance(addresses.token, address),
          ]);

          return {
            address,
            usdcBalance,
            dobBalance,
          };
        })
      );

      setLiquidNodes(nodesInfo);
    } catch (err: any) {
      console.error('Failed to fetch liquid nodes:', err);
    }
  }, [service, addresses]);

  const fetchAllData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      await Promise.all([
        fetchOracleData(),
        fetchPoolReserves(),
        fetchUserBalances(),
        fetchLiquidNodes(),
      ]);
    } catch (err: any) {
      setError(err.message || 'Failed to fetch data');
    } finally {
      setLoading(false);
    }
  }, [fetchOracleData, fetchPoolReserves, fetchUserBalances, fetchLiquidNodes]);

  useEffect(() => {
    fetchAllData();
    const interval = setInterval(fetchAllData, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  }, [fetchAllData]);

  return {
    service,
    oracleData,
    poolReserves,
    userBalances,
    liquidNodes,
    loading,
    error,
    refreshData: fetchAllData,
  };
}
