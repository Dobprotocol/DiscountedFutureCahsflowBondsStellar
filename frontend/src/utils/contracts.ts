import * as StellarSdk from '@stellar/stellar-sdk';
import { getServer, getNetworkPassphrase, signTransactionWithFreighter } from './stellar';
import type { Network, OracleData, PoolReserves, UserBalances } from '../types';

const {Contract, Address, nativeToScVal, scValToNative } = StellarSdk;

export class ContractService {
  private server: StellarSdk.SorobanRpc.Server;
  private network: Network;
  private networkPassphrase: string;

  constructor(network: Network = 'testnet') {
    this.network = network;
    this.server = getServer(network);
    this.networkPassphrase = getNetworkPassphrase(network);
  }

  // Generic contract call
  private async callContract(
    contractId: string,
    method: string,
    params: StellarSdk.xdr.ScVal[] = [],
    sourceAccount?: string,
    shouldSign = false
  ): Promise<any> {
    const contract = new Contract(contractId);

    if (!sourceAccount) {
      // Read-only call
      const result = await this.server.simulateTransaction(
        new StellarSdk.TransactionBuilder(
          new StellarSdk.Account('GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF', '0'),
          {
            fee: '100',
            networkPassphrase: this.networkPassphrase,
          }
        )
          .addOperation(contract.call(method, ...params))
          .setTimeout(30)
          .build()
      );

      if (StellarSdk.SorobanRpc.Api.isSimulationSuccess(result)) {
        return scValToNative(result.result!.retval);
      }

      // Log detailed error information
      console.error('Contract simulation failed:', {
        contractId,
        method,
        error: result.error,
        result: result,
      });

      throw new Error(`Contract call failed: ${result.error || 'Unknown error'}`);
    }

    // Transaction that needs signing
    const account = await this.server.getAccount(sourceAccount);

    let tx = new StellarSdk.TransactionBuilder(account, {
      fee: '100000',
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(contract.call(method, ...params))
      .setTimeout(30)
      .build();

    // Simulate to get proper fees
    const simulated = await this.server.simulateTransaction(tx);

    if (!StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated)) {
      throw new Error('Transaction simulation failed');
    }

    tx = StellarSdk.SorobanRpc.assembleTransaction(tx, simulated).build();

    if (shouldSign) {
      const signedXdr = await signTransactionWithFreighter(tx.toXDR(), this.network);
      tx = StellarSdk.TransactionBuilder.fromXDR(signedXdr, this.networkPassphrase) as StellarSdk.Transaction;
    }

    const sent = await this.server.sendTransaction(tx);

    // Wait for confirmation
    let response = await this.server.getTransaction(sent.hash);
    while (response.status === 'NOT_FOUND') {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      response = await this.server.getTransaction(sent.hash);
    }

    if (response.status === 'SUCCESS' && response.resultMetaXdr) {
      const meta = response.resultMetaXdr;
      return scValToNative(meta.v3().sorobanMeta()?.returnValue()!);
    }

    throw new Error('Transaction failed');
  }

  // Oracle methods
  async getOracleFairPrice(oracleId: string): Promise<string> {
    return await this.callContract(oracleId, 'fair_price');
  }

  async getOracleRisk(oracleId: string): Promise<number> {
    return await this.callContract(oracleId, 'default_risk');
  }

  async getOracleData(oracleId: string): Promise<OracleData> {
    const [fairPrice, risk] = await Promise.all([
      this.getOracleFairPrice(oracleId),
      this.getOracleRisk(oracleId),
    ]);

    return {
      fairPrice: fairPrice.toString(),
      risk: risk.toString(),
      lastUpdate: new Date().toISOString(),
    };
  }

  async updateOracle(
    oracleId: string,
    updater: string,
    newFairPrice: string,
    newRisk: string
  ): Promise<void> {
    const contract = new Contract(oracleId);
    const account = await this.server.getAccount(updater);

    // Build transaction
    let tx = new StellarSdk.TransactionBuilder(account, {
      fee: '100000',
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(
        contract.call(
          'update',
          nativeToScVal(BigInt(newFairPrice), { type: 'i128' }),
          nativeToScVal(parseInt(newRisk), { type: 'u32' })
        )
      )
      .setTimeout(30)
      .build();

    // Simulate to get proper fees
    const simulated = await this.server.simulateTransaction(tx);

    if (!StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated)) {
      console.error('Simulation failed:', simulated);
      throw new Error(`Simulation failed: ${simulated.error || 'Unknown error'}`);
    }

    tx = StellarSdk.SorobanRpc.assembleTransaction(tx, simulated).build();

    // Sign with Freighter
    const signedXdr = await signTransactionWithFreighter(tx.toXDR(), this.network);
    tx = StellarSdk.TransactionBuilder.fromXDR(signedXdr, this.networkPassphrase) as StellarSdk.Transaction;

    const sent = await this.server.sendTransaction(tx);

    // Wait for confirmation
    let response = await this.server.getTransaction(sent.hash);
    while (response.status === 'NOT_FOUND') {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      response = await this.server.getTransaction(sent.hash);
    }

    if (response.status !== 'SUCCESS') {
      throw new Error('Transaction failed');
    }
  }

  async getOracleUpdater(oracleId: string): Promise<string> {
    return await this.callContract(oracleId, 'updater');
  }

  // Token methods
  async getBalance(tokenId: string, address: string): Promise<string> {
    try {
      const params = [new Address(address).toScVal()];
      const balance = await this.callContract(tokenId, 'balance', params);
      return balance.toString();
    } catch (error: any) {
      // If balance doesn't exist or trustline missing, return 0
      if (error.message?.includes('trustline') ||
          error.message?.includes('MissingValue') ||
          error.message?.includes('#13')) {
        console.warn(`Balance not found for ${tokenId}, returning 0:`, error.message);
        return '0';
      }
      throw error;
    }
  }

  // Check if trustline exists for a token
  async hasTrustline(tokenId: string, address: string): Promise<boolean> {
    try {
      const params = [new Address(address).toScVal()];
      await this.callContract(tokenId, 'balance', params);
      return true;
    } catch (error: any) {
      if (error.message?.includes('trustline') || error.message?.includes('#13')) {
        return false;
      }
      return true; // Other errors mean trustline exists but different issue
    }
  }

  // Pool methods
  async getPoolReserves(poolId: string): Promise<PoolReserves> {
    const reserves = await this.callContract(poolId, 'get_reserves');
    return {
      usdc: reserves[0].toString(),
      dob: reserves[1].toString(),
      totalLp: '0', // You can add a method to get this if needed
    };
  }

  async getLiquidNodes(poolId: string): Promise<string[]> {
    const nodes = await this.callContract(poolId, 'get_liquid_nodes');
    return nodes.map((node: any) => node);
  }

  async getSwapBuyQuote(oracleId: string, usdcAmount: string): Promise<string> {
    // swap_buy doesn't have a quote function, calculate locally
    // Logic: DEX fee 1%, then 99% to operator, DOB = (operator_amount * 10_000_000) / fair_price
    const fairPrice = await this.getOracleFairPrice(oracleId);

    const usdc = BigInt(usdcAmount);
    // DEX fee = 1%
    const dexFee = (usdc * 100n) / 10000n;
    const afterDexFee = usdc - dexFee;
    // 99% to operator
    const operatorAmount = (afterDexFee * 99n) / 100n;
    // Calculate DOB: (operator_amount * 10_000_000) / fair_price
    const dobAmount = (operatorAmount * 10000000n) / BigInt(fairPrice);

    return dobAmount.toString();
  }

  async getSwapSellQuote(poolId: string, dobAmount: string): Promise<string> {
    const params = [nativeToScVal(parseInt(dobAmount), { type: 'i128' })];
    const quote = await this.callContract(poolId, 'quote_swap_sell', params);
    return quote.toString();
  }

  async swapBuy(
    poolId: string,
    buyer: string,
    usdcAmount: string
  ): Promise<string> {
    const contract = new Contract(poolId);
    const account = await this.server.getAccount(buyer);

    let tx = new StellarSdk.TransactionBuilder(account, {
      fee: '100000',
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(
        contract.call(
          'swap_buy',
          new Address(buyer).toScVal(),
          nativeToScVal(BigInt(usdcAmount), { type: 'i128' })
        )
      )
      .setTimeout(30)
      .build();

    const simulated = await this.server.simulateTransaction(tx);
    if (!StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated)) {
      throw new Error(`Simulation failed: ${simulated.error || 'Unknown error'}`);
    }

    tx = StellarSdk.SorobanRpc.assembleTransaction(tx, simulated).build();
    const signedXdr = await signTransactionWithFreighter(tx.toXDR(), this.network);
    tx = StellarSdk.TransactionBuilder.fromXDR(signedXdr, this.networkPassphrase) as StellarSdk.Transaction;

    const sent = await this.server.sendTransaction(tx);
    let response = await this.server.getTransaction(sent.hash);
    while (response.status === 'NOT_FOUND') {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      response = await this.server.getTransaction(sent.hash);
    }

    if (response.status === 'SUCCESS' && response.resultMetaXdr) {
      try {
        const meta = response.resultMetaXdr;
        const returnValue = meta.v3().sorobanMeta()?.returnValue();
        if (!returnValue) {
          throw new Error('No return value from transaction');
        }
        const result = scValToNative(returnValue);
        return result.toString();
      } catch (parseError: any) {
        console.error('Error parsing swap_buy result:', parseError);
        // Transaction succeeded, parsing failed - still success
        return '0';
      }
    }

    throw new Error('Transaction failed');
  }

  async swapSell(
    poolId: string,
    seller: string,
    dobAmount: string
  ): Promise<string> {
    const contract = new Contract(poolId);
    const account = await this.server.getAccount(seller);

    let tx = new StellarSdk.TransactionBuilder(account, {
      fee: '100000',
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(
        contract.call(
          'swap_sell',
          new Address(seller).toScVal(),
          nativeToScVal(BigInt(dobAmount), { type: 'i128' })
        )
      )
      .setTimeout(30)
      .build();

    const simulated = await this.server.simulateTransaction(tx);
    if (!StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated)) {
      throw new Error(`Simulation failed: ${simulated.error || 'Unknown error'}`);
    }

    tx = StellarSdk.SorobanRpc.assembleTransaction(tx, simulated).build();
    const signedXdr = await signTransactionWithFreighter(tx.toXDR(), this.network);
    tx = StellarSdk.TransactionBuilder.fromXDR(signedXdr, this.networkPassphrase) as StellarSdk.Transaction;

    const sent = await this.server.sendTransaction(tx);
    let response = await this.server.getTransaction(sent.hash);
    while (response.status === 'NOT_FOUND') {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      response = await this.server.getTransaction(sent.hash);
    }

    if (response.status === 'SUCCESS' && response.resultMetaXdr) {
      try {
        const meta = response.resultMetaXdr;
        const returnValue = meta.v3().sorobanMeta()?.returnValue();
        if (!returnValue) {
          throw new Error('No return value from transaction');
        }
        const result = scValToNative(returnValue);
        return result.toString();
      } catch (parseError: any) {
        console.error('Error parsing swap_sell result:', parseError);
        // Transaction succeeded, parsing failed - still success
        return '0';
      }
    }

    throw new Error('Transaction failed');
  }

  async addLiquidity(
    poolId: string,
    provider: string,
    usdcAmount: string,
    dobAmount: string
  ): Promise<string> {
    const contract = new Contract(poolId);
    const account = await this.server.getAccount(provider);

    let tx = new StellarSdk.TransactionBuilder(account, {
      fee: '100000',
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(
        contract.call(
          'add_liquidity',
          new Address(provider).toScVal(),
          nativeToScVal(BigInt(usdcAmount), { type: 'i128' }),
          nativeToScVal(BigInt(dobAmount), { type: 'i128' })
        )
      )
      .setTimeout(30)
      .build();

    const simulated = await this.server.simulateTransaction(tx);
    if (!StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated)) {
      throw new Error(`Simulation failed: ${simulated.error || 'Unknown error'}`);
    }

    tx = StellarSdk.SorobanRpc.assembleTransaction(tx, simulated).build();
    const signedXdr = await signTransactionWithFreighter(tx.toXDR(), this.network);
    tx = StellarSdk.TransactionBuilder.fromXDR(signedXdr, this.networkPassphrase) as StellarSdk.Transaction;

    const sent = await this.server.sendTransaction(tx);
    let response = await this.server.getTransaction(sent.hash);
    while (response.status === 'NOT_FOUND') {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      response = await this.server.getTransaction(sent.hash);
    }

    if (response.status === 'SUCCESS' && response.resultMetaXdr) {
      const meta = response.resultMetaXdr;
      return scValToNative(meta.v3().sorobanMeta()?.returnValue()!).toString();
    }

    throw new Error('Transaction failed');
  }

  async removeLiquidity(
    poolId: string,
    provider: string,
    lpShares: string
  ): Promise<[string, string]> {
    const contract = new Contract(poolId);
    const account = await this.server.getAccount(provider);

    let tx = new StellarSdk.TransactionBuilder(account, {
      fee: '100000',
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(
        contract.call(
          'remove_liquidity',
          new Address(provider).toScVal(),
          nativeToScVal(BigInt(lpShares), { type: 'i128' })
        )
      )
      .setTimeout(30)
      .build();

    const simulated = await this.server.simulateTransaction(tx);
    if (!StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated)) {
      throw new Error(`Simulation failed: ${simulated.error || 'Unknown error'}`);
    }

    tx = StellarSdk.SorobanRpc.assembleTransaction(tx, simulated).build();
    const signedXdr = await signTransactionWithFreighter(tx.toXDR(), this.network);
    tx = StellarSdk.TransactionBuilder.fromXDR(signedXdr, this.networkPassphrase) as StellarSdk.Transaction;

    const sent = await this.server.sendTransaction(tx);
    let response = await this.server.getTransaction(sent.hash);
    while (response.status === 'NOT_FOUND') {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      response = await this.server.getTransaction(sent.hash);
    }

    if (response.status === 'SUCCESS' && response.resultMetaXdr) {
      const meta = response.resultMetaXdr;
      const result = scValToNative(meta.v3().sorobanMeta()?.returnValue()!);
      return [result[0].toString(), result[1].toString()];
    }

    throw new Error('Transaction failed');
  }

  // Get LP shares from pool contract
  async getLpShares(poolId: string, provider: string): Promise<string> {
    try {
      const params = [new Address(provider).toScVal()];
      const shares = await this.callContract(poolId, 'get_lp_shares', params);
      return shares.toString();
    } catch (error: any) {
      console.warn(`LP shares not found for ${poolId}, returning 0:`, error.message);
      return '0';
    }
  }

  // Get user balances
  async getUserBalances(
    tokenId: string,
    usdcId: string,
    poolId: string,
    userAddress: string
  ): Promise<UserBalances> {
    const [dob, usdc, lpShares] = await Promise.all([
      this.getBalance(tokenId, userAddress),
      this.getBalance(usdcId, userAddress),
      this.getLpShares(poolId, userAddress),
    ]);

    return {
      dob,
      usdc,
      lpShares,
    };
  }
}

export default ContractService;
