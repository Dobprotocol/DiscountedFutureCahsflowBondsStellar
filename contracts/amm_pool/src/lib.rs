#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, token, Address, Env, IntoVal, Symbol, Vec,
};

/// Integer square root using Newton's method
fn isqrt(n: i128) -> i128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Storage keys for the AMM pool
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    DobToken,              // DOB token contract address
    UsdcToken,             // USDC token contract address
    Oracle,                // Oracle contract address
    Operator,              // Operator receiving revenues
    TotalLpShares,         // Total LP shares issued
    LpShares(Address),     // LP shares per address
    LiquidNodes,           // Vec<Address> of registered Liquid Nodes
    UsdcReserve,           // USDC reserves in pool
    DobReserve,            // DOB reserves in pool
    TotalBought,           // Total USDC spent on buys
    TotalSold,             // Total DOB sold
    DexFeeCollected,       // Total DEX fee collected (1%)
}

/// LP provision event
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityAddedEvent {
    pub provider: Address,
    pub usdc_amount: i128,
    pub dob_amount: i128,
    pub lp_shares: i128,
}

/// LP removal event
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityRemovedEvent {
    pub provider: Address,
    pub usdc_amount: i128,
    pub dob_amount: i128,
    pub lp_shares: i128,
}

/// Swap event for buys (AfterSwap)
#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapBuyEvent {
    pub buyer: Address,
    pub usdc_in: i128,
    pub dob_out: i128,
    pub fair_price: i128,
    pub pool_price: i128,
}

/// Swap event for sells (BeforeSwap)
#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapSellEvent {
    pub seller: Address,
    pub dob_in: i128,
    pub usdc_out: i128,
    pub fair_price: i128,
    pub pool_price: i128,
    pub fee_bps: u32,
    pub liquid_nodes_used: bool,
}

/// Liquid Node quote
#[contracttype]
#[derive(Clone, Debug)]
pub struct LnQuote {
    pub node_address: Address,
    pub usdc_provided: i128,
    pub dob_taken: i128,
    pub fee_bps: u32,
}

/// Swap quote for user
#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapQuote {
    pub usdc_out: i128,
    pub total_fee_bps: u32,
    pub from_pool: i128,
    pub from_liquid_nodes: i128,
}

/// Errors
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InsufficientLiquidity = 1,
    InvalidAmount = 2,
    TransferFailed = 3,
    NoLiquidityAvailable = 4,
    InvalidLpShares = 5,
    Unauthorized = 6,
    AlreadyRegistered = 7,
    NotRegistered = 8,
}

// Constants
const OPERATOR_SHARE: u32 = 99; // 99% to operator on buys
const DEX_FEE: u32 = 100; // 1% DEX fee in basis points
const BPS: u32 = 10000; // Basis points denominator

#[contract]
pub struct AmmPool;

#[contractimpl]
impl AmmPool {
    /// Initialize the AMM pool contract
    pub fn initialize(
        env: Env,
        dob_token: Address,
        usdc_token: Address,
        oracle: Address,
        operator: Address,
    ) {
        if env.storage().instance().has(&DataKey::DobToken) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::DobToken, &dob_token);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage().instance().set(&DataKey::Oracle, &oracle);
        env.storage().instance().set(&DataKey::Operator, &operator);
        env.storage().instance().set(&DataKey::TotalLpShares, &0i128);
        env.storage().instance().set(&DataKey::UsdcReserve, &0i128);
        env.storage().instance().set(&DataKey::DobReserve, &0i128);
        env.storage().instance().set(&DataKey::TotalBought, &0i128);
        env.storage().instance().set(&DataKey::TotalSold, &0i128);
        env.storage().instance().set(&DataKey::DexFeeCollected, &0i128);

        // Initialize empty liquid nodes vec
        let liquid_nodes: Vec<Address> = Vec::new(&env);
        env.storage().instance().set(&DataKey::LiquidNodes, &liquid_nodes);
    }

    /// Register a Liquid Node (callable by operator)
    pub fn register_liquid_node(env: Env, node: Address) -> Result<(), Error> {
        let operator: Address = env.storage().instance().get(&DataKey::Operator).unwrap();
        operator.require_auth();

        let mut liquid_nodes: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::LiquidNodes)
            .unwrap_or(Vec::new(&env));

        // Check if already registered
        for i in 0..liquid_nodes.len() {
            if let Some(existing) = liquid_nodes.get(i) {
                if existing == node {
                    return Err(Error::AlreadyRegistered);
                }
            }
        }

        liquid_nodes.push_back(node.clone());
        env.storage().instance().set(&DataKey::LiquidNodes, &liquid_nodes);

        env.events().publish(
            (Symbol::new(&env, "ln_registered"),),
            node,
        );

        Ok(())
    }

    /// Unregister a Liquid Node (callable by operator)
    pub fn unregister_liquid_node(env: Env, node: Address) -> Result<(), Error> {
        let operator: Address = env.storage().instance().get(&DataKey::Operator).unwrap();
        operator.require_auth();

        let mut liquid_nodes: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::LiquidNodes)
            .unwrap_or(Vec::new(&env));

        let mut found = false;
        let mut found_index = 0u32;

        for i in 0..liquid_nodes.len() {
            if let Some(existing) = liquid_nodes.get(i) {
                if existing == node {
                    found = true;
                    found_index = i;
                    break;
                }
            }
        }

        if !found {
            return Err(Error::NotRegistered);
        }

        liquid_nodes.remove(found_index);
        env.storage().instance().set(&DataKey::LiquidNodes, &liquid_nodes);

        env.events().publish(
            (Symbol::new(&env, "ln_unregistered"),),
            node,
        );

        Ok(())
    }

    /// Get registered Liquid Nodes
    pub fn get_liquid_nodes(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::LiquidNodes)
            .unwrap_or(Vec::new(&env))
    }

    /// Add liquidity to the pool (open to anyone)
    /// Returns LP shares minted
    pub fn add_liquidity(
        env: Env,
        provider: Address,
        usdc_amount: i128,
        dob_amount: i128,
    ) -> Result<i128, Error> {
        provider.require_auth();

        if usdc_amount <= 0 || dob_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let dob_token: Address = env.storage().instance().get(&DataKey::DobToken).unwrap();

        let usdc_reserve: i128 = env.storage().instance().get(&DataKey::UsdcReserve).unwrap_or(0);
        let dob_reserve: i128 = env.storage().instance().get(&DataKey::DobReserve).unwrap_or(0);
        let total_lp: i128 = env.storage().instance().get(&DataKey::TotalLpShares).unwrap_or(0);

        // Calculate LP shares to mint
        let lp_shares = if total_lp == 0 {
            // First liquidity provider gets shares = sqrt(usdc * dob)
            // Simplified: geometric mean
            isqrt(usdc_amount * dob_amount)
        } else {
            // Proportional to existing reserves
            let usdc_share = (usdc_amount * total_lp) / usdc_reserve;
            let dob_share = (dob_amount * total_lp) / dob_reserve;
            // Take minimum to maintain ratio
            if usdc_share < dob_share { usdc_share } else { dob_share }
        };

        if lp_shares <= 0 {
            return Err(Error::InvalidLpShares);
        }

        // Transfer tokens from provider to pool
        let usdc_client = token::Client::new(&env, &usdc_token);
        let dob_client = token::Client::new(&env, &dob_token);

        usdc_client.transfer(&provider, &env.current_contract_address(), &usdc_amount);
        dob_client.transfer(&provider, &env.current_contract_address(), &dob_amount);

        // Update reserves
        env.storage().instance().set(&DataKey::UsdcReserve, &(usdc_reserve + usdc_amount));
        env.storage().instance().set(&DataKey::DobReserve, &(dob_reserve + dob_amount));

        // Update LP shares
        let provider_shares: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::LpShares(provider.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::LpShares(provider.clone()), &(provider_shares + lp_shares));

        env.storage().instance().set(&DataKey::TotalLpShares, &(total_lp + lp_shares));

        env.events().publish(
            (Symbol::new(&env, "liquidity_added"),),
            LiquidityAddedEvent {
                provider,
                usdc_amount,
                dob_amount,
                lp_shares,
            },
        );

        Ok(lp_shares)
    }

    /// Remove liquidity from the pool
    /// Burns LP shares and returns proportional assets
    pub fn remove_liquidity(
        env: Env,
        provider: Address,
        lp_shares: i128,
    ) -> Result<(i128, i128), Error> {
        provider.require_auth();

        if lp_shares <= 0 {
            return Err(Error::InvalidAmount);
        }

        let provider_shares: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::LpShares(provider.clone()))
            .unwrap_or(0);

        if provider_shares < lp_shares {
            return Err(Error::InvalidLpShares);
        }

        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let dob_token: Address = env.storage().instance().get(&DataKey::DobToken).unwrap();

        let usdc_reserve: i128 = env.storage().instance().get(&DataKey::UsdcReserve).unwrap_or(0);
        let dob_reserve: i128 = env.storage().instance().get(&DataKey::DobReserve).unwrap_or(0);
        let total_lp: i128 = env.storage().instance().get(&DataKey::TotalLpShares).unwrap_or(0);

        // Calculate proportional amounts
        let usdc_out = (usdc_reserve * lp_shares) / total_lp;
        let dob_out = (dob_reserve * lp_shares) / total_lp;

        // Update reserves
        env.storage().instance().set(&DataKey::UsdcReserve, &(usdc_reserve - usdc_out));
        env.storage().instance().set(&DataKey::DobReserve, &(dob_reserve - dob_out));

        // Update LP shares
        env.storage()
            .persistent()
            .set(&DataKey::LpShares(provider.clone()), &(provider_shares - lp_shares));

        env.storage().instance().set(&DataKey::TotalLpShares, &(total_lp - lp_shares));

        // Transfer tokens back to provider
        let usdc_client = token::Client::new(&env, &usdc_token);
        let dob_client = token::Client::new(&env, &dob_token);

        usdc_client.transfer(&env.current_contract_address(), &provider, &usdc_out);
        dob_client.transfer(&env.current_contract_address(), &provider, &dob_out);

        env.events().publish(
            (Symbol::new(&env, "liquidity_removed"),),
            LiquidityRemovedEvent {
                provider,
                usdc_amount: usdc_out,
                dob_amount: dob_out,
                lp_shares,
            },
        );

        Ok((usdc_out, dob_out))
    }

    /// Buy DOB tokens with USDC (AfterSwap hook)
    /// Mints new tokens at fair price, sends USDC to operator
    pub fn swap_buy(env: Env, buyer: Address, usdc_amount: i128) -> Result<i128, Error> {
        buyer.require_auth();

        if usdc_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let dob_token: Address = env.storage().instance().get(&DataKey::DobToken).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let oracle: Address = env.storage().instance().get(&DataKey::Oracle).unwrap();
        let operator: Address = env.storage().instance().get(&DataKey::Operator).unwrap();

        // Get fair price from oracle
        let fair_price: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "fair_price"), soroban_sdk::vec![&env]);

        // Transfer USDC from buyer to contract
        let usdc_client = token::Client::new(&env, &usdc_token);
        usdc_client.transfer(&buyer, &env.current_contract_address(), &usdc_amount);

        // Calculate DEX fee (1%)
        let dex_fee = (usdc_amount * DEX_FEE as i128) / BPS as i128;
        let amount_after_fee = usdc_amount - dex_fee;

        // 99% to operator
        let operator_amount = (amount_after_fee * OPERATOR_SHARE as i128) / 100;
        usdc_client.transfer(&env.current_contract_address(), &operator, &operator_amount);

        // Calculate DOB to mint based on fair price
        // DOB amount = (USDC × 0.99) / fair_price
        let dob_amount = (operator_amount * 10_000_000) / fair_price;

        // AfterSwap: Mint DOB tokens to buyer
        let mint_args: Vec<soroban_sdk::Val> = (buyer.clone(), dob_amount).into_val(&env);
        let _: () = env.invoke_contract(
            &dob_token,
            &Symbol::new(&env, "mint"),
            mint_args,
        );

        // Update stats
        let total_bought: i128 = env.storage().instance().get(&DataKey::TotalBought).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalBought, &(total_bought + usdc_amount));

        let dex_fee_collected: i128 = env.storage().instance().get(&DataKey::DexFeeCollected).unwrap_or(0);
        env.storage().instance().set(&DataKey::DexFeeCollected, &(dex_fee_collected + dex_fee));

        // Calculate pool price for comparison (if there's liquidity)
        let usdc_reserve: i128 = env.storage().instance().get(&DataKey::UsdcReserve).unwrap_or(0);
        let dob_reserve: i128 = env.storage().instance().get(&DataKey::DobReserve).unwrap_or(0);

        let pool_price = if dob_reserve > 0 {
            (usdc_reserve * 10_000_000) / dob_reserve
        } else {
            fair_price
        };

        env.events().publish(
            (Symbol::new(&env, "swap_buy"),),
            SwapBuyEvent {
                buyer,
                usdc_in: usdc_amount,
                dob_out: dob_amount,
                fair_price,
                pool_price,
            },
        );

        Ok(dob_amount)
    }

    /// Sell DOB tokens for USDC (BeforeSwap hook)
    /// First tries to use pool liquidity, then calls Liquid Nodes if needed
    pub fn swap_sell(env: Env, seller: Address, dob_amount: i128) -> Result<i128, Error> {
        seller.require_auth();

        if dob_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let dob_token: Address = env.storage().instance().get(&DataKey::DobToken).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let oracle: Address = env.storage().instance().get(&DataKey::Oracle).unwrap();

        // Get fair price and risk from oracle
        let fair_price: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "fair_price"), soroban_sdk::vec![&env]);
        let risk: u32 = env.invoke_contract(&oracle, &Symbol::new(&env, "default_risk"), soroban_sdk::vec![&env]);

        // Calculate base fee from oracle
        let base_fee_bps = 300 + (risk / 10); // 3% base + risk/10
        let base_fee_bps = if base_fee_bps > 5000 { 5000 } else { base_fee_bps };

        // Calculate how much USDC needed at fair price
        let usdc_needed_at_fair_price = (dob_amount * fair_price) / 10_000_000;
        let usdc_after_fee = (usdc_needed_at_fair_price * (BPS - base_fee_bps) as i128) / BPS as i128;

        // Check pool liquidity
        let usdc_reserve: i128 = env.storage().instance().get(&DataKey::UsdcReserve).unwrap_or(0);
        let dob_reserve: i128 = env.storage().instance().get(&DataKey::DobReserve).unwrap_or(0);

        // Transfer all DOB from seller to this contract first
        let dob_client = token::Client::new(&env, &dob_token);
        dob_client.transfer(&seller, &env.current_contract_address(), &dob_amount);

        let mut from_pool = 0i128;
        let mut from_liquid_nodes = 0i128;
        let mut liquid_nodes_used = false;
        let mut total_fee_bps = base_fee_bps;

        // BeforeSwap: Check if pool has enough liquidity
        if usdc_reserve >= usdc_after_fee {
            // Pool has enough liquidity
            from_pool = usdc_after_fee;

            // Update reserves (DOB was already transferred to contract)
            env.storage().instance().set(&DataKey::UsdcReserve, &(usdc_reserve - usdc_after_fee));
            env.storage().instance().set(&DataKey::DobReserve, &(dob_reserve + dob_amount));

            // Transfer USDC to seller
            let usdc_client = token::Client::new(&env, &usdc_token);
            usdc_client.transfer(&env.current_contract_address(), &seller, &usdc_after_fee);
        } else {
            // Pool doesn't have enough liquidity - call Liquid Nodes
            // BeforeSwap triggers Liquid Node search

            let shortage = usdc_after_fee - usdc_reserve;
            // Account for LN fee (estimate ~10% fee buffer to ensure enough USDC)
            // LN will charge 5-30% fee depending on risk, so we request extra DOB
            let dob_for_shortage_base = (shortage * 10_000_000) / fair_price;
            let dob_for_shortage = (dob_for_shortage_base * 11000) / 10000; // Add 10% buffer
            let dob_for_pool = if dob_amount > dob_for_shortage {
                dob_amount - dob_for_shortage
            } else {
                0
            };

            // Get quotes from all registered Liquid Nodes
            let liquid_nodes: Vec<Address> = env
                .storage()
                .instance()
                .get(&DataKey::LiquidNodes)
                .unwrap_or(Vec::new(&env));

            if liquid_nodes.is_empty() {
                return Err(Error::NoLiquidityAvailable);
            }

            // Collect quotes from all LN
            let mut best_quote: Option<LnQuote> = None;
            let mut best_fee = u32::MAX;

            for i in 0..liquid_nodes.len() {
                if let Some(ln_address) = liquid_nodes.get(i) {
                    // Request quote from Liquid Node
                    let quote_result = env.try_invoke_contract::<(i128, u32), Error>(
                        &ln_address,
                        &Symbol::new(&env, "request_quote"),
                        (dob_for_shortage,).into_val(&env),
                    );

                    if let Ok(Ok((usdc_provided, fee_bps))) = quote_result {
                        if fee_bps < best_fee && usdc_provided >= shortage {
                            best_fee = fee_bps;
                            best_quote = Some(LnQuote {
                                node_address: ln_address.clone(),
                                usdc_provided,
                                dob_taken: dob_for_shortage,
                                fee_bps,
                            });
                        }
                    }
                }
            }

            if best_quote.is_none() {
                return Err(Error::NoLiquidityAvailable);
            }

            let best_ln = best_quote.unwrap();

            // Execute swap with pool portion
            if usdc_reserve > 0 && dob_for_pool > 0 {
                // DOB already in contract, just update reserves
                env.storage().instance().set(&DataKey::UsdcReserve, &0i128);
                env.storage().instance().set(&DataKey::DobReserve, &(dob_reserve + dob_for_pool));

                from_pool = usdc_reserve;
            }

            // Transfer DOB to Liquid Node from this contract
            dob_client.transfer(&env.current_contract_address(), &best_ln.node_address, &dob_for_shortage);

            // Call Liquid Node to fulfill
            let ln_usdc: i128 = env.invoke_contract(
                &best_ln.node_address,
                &Symbol::new(&env, "execute_liquidity"),
                (seller.clone(), dob_for_shortage).into_val(&env),
            );

            from_liquid_nodes = ln_usdc;
            liquid_nodes_used = true;

            // Calculate weighted average fee
            total_fee_bps = if from_pool > 0 {
                ((base_fee_bps as i128 * from_pool + best_ln.fee_bps as i128 * from_liquid_nodes)
                    / (from_pool + from_liquid_nodes)) as u32
            } else {
                best_ln.fee_bps
            };
        }

        let total_usdc_out = from_pool + from_liquid_nodes;

        // Burn DOB tokens from this contract (not from seller, since we already transferred them)
        let burn_args: Vec<soroban_sdk::Val> = (env.current_contract_address(), dob_amount).into_val(&env);
        let _: () = env.invoke_contract(
            &dob_token,
            &Symbol::new(&env, "burn"),
            burn_args,
        );

        // Update stats
        let total_sold: i128 = env.storage().instance().get(&DataKey::TotalSold).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalSold, &(total_sold + dob_amount));

        // Calculate pool price
        let usdc_reserve_after: i128 = env.storage().instance().get(&DataKey::UsdcReserve).unwrap_or(0);
        let dob_reserve_after: i128 = env.storage().instance().get(&DataKey::DobReserve).unwrap_or(0);

        let pool_price = if dob_reserve_after > 0 {
            (usdc_reserve_after * 10_000_000) / dob_reserve_after
        } else {
            fair_price
        };

        env.events().publish(
            (Symbol::new(&env, "swap_sell"),),
            SwapSellEvent {
                seller,
                dob_in: dob_amount,
                usdc_out: total_usdc_out,
                fair_price,
                pool_price,
                fee_bps: total_fee_bps,
                liquid_nodes_used,
            },
        );

        Ok(total_usdc_out)
    }

    /// Quote swap sell (read-only)
    pub fn quote_swap_sell(env: Env, dob_amount: i128) -> SwapQuote {
        let oracle: Address = env.storage().instance().get(&DataKey::Oracle).unwrap();

        let fair_price: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "fair_price"), soroban_sdk::vec![&env]);
        let risk: u32 = env.invoke_contract(&oracle, &Symbol::new(&env, "default_risk"), soroban_sdk::vec![&env]);

        let base_fee_bps = 300 + (risk / 10);
        let base_fee_bps = if base_fee_bps > 5000 { 5000 } else { base_fee_bps };

        let usdc_needed = (dob_amount * fair_price) / 10_000_000;
        let usdc_after_fee = (usdc_needed * (BPS - base_fee_bps) as i128) / BPS as i128;

        let usdc_reserve: i128 = env.storage().instance().get(&DataKey::UsdcReserve).unwrap_or(0);

        if usdc_reserve >= usdc_after_fee {
            SwapQuote {
                usdc_out: usdc_after_fee,
                total_fee_bps: base_fee_bps,
                from_pool: usdc_after_fee,
                from_liquid_nodes: 0,
            }
        } else {
            let from_pool = usdc_reserve;
            let from_ln = usdc_after_fee - usdc_reserve;

            SwapQuote {
                usdc_out: usdc_after_fee,
                total_fee_bps: base_fee_bps,
                from_pool,
                from_liquid_nodes: from_ln,
            }
        }
    }

    /// Get pool reserves
    pub fn get_reserves(env: Env) -> (i128, i128) {
        let usdc_reserve: i128 = env.storage().instance().get(&DataKey::UsdcReserve).unwrap_or(0);
        let dob_reserve: i128 = env.storage().instance().get(&DataKey::DobReserve).unwrap_or(0);
        (usdc_reserve, dob_reserve)
    }

    /// Get LP shares for an address
    pub fn get_lp_shares(env: Env, provider: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::LpShares(provider))
            .unwrap_or(0)
    }

    /// Get total LP shares
    pub fn get_total_lp_shares(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalLpShares).unwrap_or(0)
    }

    /// Get trading statistics
    pub fn get_stats(env: Env) -> (i128, i128, i128) {
        let total_bought = env.storage().instance().get(&DataKey::TotalBought).unwrap_or(0);
        let total_sold = env.storage().instance().get(&DataKey::TotalSold).unwrap_or(0);
        let dex_fee = env.storage().instance().get(&DataKey::DexFeeCollected).unwrap_or(0);
        (total_bought, total_sold, dex_fee)
    }

    /// Get contract addresses
    pub fn get_addresses(env: Env) -> (Address, Address, Address, Address) {
        let dob_token = env.storage().instance().get(&DataKey::DobToken).unwrap();
        let usdc_token = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let oracle = env.storage().instance().get(&DataKey::Oracle).unwrap();
        let operator = env.storage().instance().get(&DataKey::Operator).unwrap();
        (dob_token, usdc_token, oracle, operator)
    }
}
