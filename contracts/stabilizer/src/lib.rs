#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, token, Address, Env, IntoVal, Symbol};

/// Storage keys for the stabilizer contract
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Oracle,           // Oracle contract address
    UsdcToken,        // USDC token address
    DobToken,         // DOB token address
    Operator,         // Operator address
    TotalFeesEarned,  // Total fees earned from interventions
    AmmPool,          // AMM Pool address (for registration)
}

/// Liquidity provision event
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityProvidedEvent {
    pub seller: Address,
    pub dob_amount: i128,
    pub usdc_provided: i128,
    pub fee_bps: u32,
}

/// Errors that can be returned by the contract
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Unauthorized = 1,
    InsufficientBalance = 2,
    InvalidAmount = 3,
}

const BPS: u32 = 10000;

/// LiquidNodeStabilizer
/// Pre-funded buffer that provides instant liquidity on-demand
/// Dynamically calculates fees based on oracle risk assessment
#[contract]
pub struct LiquidNodeStabilizer;

#[contractimpl]
impl LiquidNodeStabilizer {
    /// Initialize the stabilizer contract
    pub fn initialize(
        env: Env,
        oracle: Address,
        usdc_token: Address,
        dob_token: Address,
        operator: Address,
        amm_pool: Address,
    ) {
        if env.storage().instance().has(&DataKey::Oracle) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Oracle, &oracle);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage().instance().set(&DataKey::DobToken, &dob_token);
        env.storage().instance().set(&DataKey::Operator, &operator);
        env.storage().instance().set(&DataKey::AmmPool, &amm_pool);
        env.storage().instance().set(&DataKey::TotalFeesEarned, &0i128);
    }

    /// Fund the Liquid Node with USDC
    pub fn fund_usdc(env: Env, funder: Address, amount: i128) -> Result<(), Error> {
        funder.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

        let usdc_client = token::Client::new(&env, &usdc_token);
        usdc_client.transfer(&funder, &env.current_contract_address(), &amount);

        env.events()
            .publish((Symbol::new(&env, "funded_usdc"), funder), amount);

        Ok(())
    }

    /// Fund the Liquid Node with DOB tokens
    pub fn fund_dob(env: Env, funder: Address, amount: i128) -> Result<(), Error> {
        funder.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let dob_token: Address = env.storage().instance().get(&DataKey::DobToken).unwrap();

        let dob_client = token::Client::new(&env, &dob_token);
        dob_client.transfer(&funder, &env.current_contract_address(), &amount);

        env.events()
            .publish((Symbol::new(&env, "funded_dob"), funder), amount);

        Ok(())
    }

    /// Request quote for liquidity provision (called by AMM Pool)
    /// Returns (usdc_provided, fee_bps)
    /// Fee is dynamically calculated based on oracle risk
    pub fn request_quote(env: Env, dob_amount: i128) -> Result<(i128, u32), Error> {
        if dob_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let oracle: Address = env.storage().instance().get(&DataKey::Oracle).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

        // Get NAV and risk from oracle
        let nav: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "fair_price"), soroban_sdk::vec![&env]);
        let risk: u32 = env.invoke_contract(&oracle, &Symbol::new(&env, "default_risk"), soroban_sdk::vec![&env]);

        // Dynamic fee calculation based on risk
        // Low risk (<15%): 5% fee
        // Medium risk (15-30%): 10% fee
        // High risk (30-50%): 20% fee
        // Very high risk (>50%): 30% fee
        let fee_bps = if risk < 1500 {
            500  // 5%
        } else if risk < 3000 {
            1000 // 10%
        } else if risk < 5000 {
            2000 // 20%
        } else {
            3000 // 30%
        };

        // Calculate USDC we can provide: DOB × NAV × (1 - fee)
        let value = (dob_amount * nav) / 10_000_000; // 7 decimals
        let usdc_provided = (value * (BPS - fee_bps) as i128) / BPS as i128;

        // Check if we have enough USDC
        let usdc_client = token::Client::new(&env, &usdc_token);
        let usdc_balance = usdc_client.balance(&env.current_contract_address());

        if usdc_balance < usdc_provided {
            return Err(Error::InsufficientBalance);
        }

        Ok((usdc_provided, fee_bps))
    }

    /// Execute liquidity provision (called by AMM Pool after accepting quote)
    /// AMM Pool has already transferred DOB to this contract
    /// Returns USDC amount sent to seller
    pub fn execute_liquidity(env: Env, seller: Address, dob_amount: i128) -> Result<i128, Error> {
        // Verify caller is the AMM Pool
        let amm_pool: Address = env.storage().instance().get(&DataKey::AmmPool).unwrap();
        amm_pool.require_auth();

        if dob_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let oracle: Address = env.storage().instance().get(&DataKey::Oracle).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

        // Get NAV and risk from oracle
        let nav: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "fair_price"), soroban_sdk::vec![&env]);
        let risk: u32 = env.invoke_contract(&oracle, &Symbol::new(&env, "default_risk"), soroban_sdk::vec![&env]);

        // Calculate fee (same logic as request_quote)
        let fee_bps = if risk < 1500 {
            500
        } else if risk < 3000 {
            1000
        } else if risk < 5000 {
            2000
        } else {
            3000
        };

        // Calculate USDC to provide
        let value = (dob_amount * nav) / 10_000_000;
        let usdc_provided = (value * (BPS - fee_bps) as i128) / BPS as i128;

        // Check balance
        let usdc_client = token::Client::new(&env, &usdc_token);
        let usdc_balance = usdc_client.balance(&env.current_contract_address());

        if usdc_balance < usdc_provided {
            return Err(Error::InsufficientBalance);
        }

        // Transfer USDC to seller
        usdc_client.transfer(&env.current_contract_address(), &seller, &usdc_provided);

        // Track fees earned
        let fee_amount = value - usdc_provided;
        let total_fees: i128 = env.storage().instance().get(&DataKey::TotalFeesEarned).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalFeesEarned, &(total_fees + fee_amount));

        env.events().publish(
            (Symbol::new(&env, "liquidity_provided"),),
            LiquidityProvidedEvent {
                seller,
                dob_amount,
                usdc_provided,
                fee_bps,
            },
        );

        Ok(usdc_provided)
    }

    /// Provide instant liquidity directly (alternative to AMM pool)
    /// User can call this directly if they want to skip the pool
    pub fn provide_liquidity_direct(
        env: Env,
        seller: Address,
        dob_amount: i128,
    ) -> Result<i128, Error> {
        seller.require_auth();

        if dob_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let dob_token: Address = env.storage().instance().get(&DataKey::DobToken).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let oracle: Address = env.storage().instance().get(&DataKey::Oracle).unwrap();

        // Get NAV and risk from oracle
        let nav: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "fair_price"), soroban_sdk::vec![&env]);
        let risk: u32 = env.invoke_contract(&oracle, &Symbol::new(&env, "default_risk"), soroban_sdk::vec![&env]);

        // Calculate fee
        let fee_bps = if risk < 1500 {
            500
        } else if risk < 3000 {
            1000
        } else if risk < 5000 {
            2000
        } else {
            3000
        };

        let value = (dob_amount * nav) / 10_000_000;
        let usdc_provided = (value * (BPS - fee_bps) as i128) / BPS as i128;

        // Check balance
        let usdc_client = token::Client::new(&env, &usdc_token);
        let usdc_balance = usdc_client.balance(&env.current_contract_address());

        if usdc_balance < usdc_provided {
            return Err(Error::InsufficientBalance);
        }

        // Transfer DOB from seller to contract
        let dob_client = token::Client::new(&env, &dob_token);
        dob_client.transfer(&seller, &env.current_contract_address(), &dob_amount);

        // Transfer USDC to seller
        usdc_client.transfer(&env.current_contract_address(), &seller, &usdc_provided);

        // Track fees
        let fee_amount = value - usdc_provided;
        let total_fees: i128 = env.storage().instance().get(&DataKey::TotalFeesEarned).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalFeesEarned, &(total_fees + fee_amount));

        env.events().publish(
            (Symbol::new(&env, "liquidity_provided"),),
            LiquidityProvidedEvent {
                seller,
                dob_amount,
                usdc_provided,
                fee_bps,
            },
        );

        Ok(usdc_provided)
    }

    /// Get quote for direct liquidity provision
    pub fn quote_liquidity_direct(env: Env, dob_amount: i128) -> (i128, u32) {
        let oracle: Address = env.storage().instance().get(&DataKey::Oracle).unwrap();

        let nav: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "fair_price"), soroban_sdk::vec![&env]);
        let risk: u32 = env.invoke_contract(&oracle, &Symbol::new(&env, "default_risk"), soroban_sdk::vec![&env]);

        let fee_bps = if risk < 1500 {
            500
        } else if risk < 3000 {
            1000
        } else if risk < 5000 {
            2000
        } else {
            3000
        };

        let value = (dob_amount * nav) / 10_000_000;
        let usdc_provided = (value * (BPS - fee_bps) as i128) / BPS as i128;

        (usdc_provided, fee_bps)
    }

    /// Withdraw accumulated fees (operator only)
    pub fn withdraw_fees(env: Env) -> Result<i128, Error> {
        let operator: Address = env.storage().instance().get(&DataKey::Operator).unwrap();
        operator.require_auth();

        let total_fees: i128 = env.storage().instance().get(&DataKey::TotalFeesEarned).unwrap_or(0);

        if total_fees == 0 {
            return Ok(0);
        }

        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

        // Reset fees
        env.storage().instance().set(&DataKey::TotalFeesEarned, &0i128);

        // Transfer fees to operator
        let usdc_client = token::Client::new(&env, &usdc_token);
        usdc_client.transfer(&env.current_contract_address(), &operator, &total_fees);

        env.events().publish(
            (Symbol::new(&env, "fees_withdrawn"), operator),
            total_fees,
        );

        Ok(total_fees)
    }

    /// Get current balances
    pub fn get_balances(env: Env) -> (i128, i128) {
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let dob_token: Address = env.storage().instance().get(&DataKey::DobToken).unwrap();

        let usdc_client = token::Client::new(&env, &usdc_token);
        let dob_client = token::Client::new(&env, &dob_token);

        let usdc_balance = usdc_client.balance(&env.current_contract_address());
        let dob_balance = dob_client.balance(&env.current_contract_address());

        (usdc_balance, dob_balance)
    }

    /// Get total fees earned
    pub fn total_fees_earned(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalFeesEarned).unwrap_or(0)
    }

    /// Get contract addresses
    pub fn get_addresses(env: Env) -> (Address, Address, Address, Address, Address) {
        let oracle = env.storage().instance().get(&DataKey::Oracle).unwrap();
        let usdc_token = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let dob_token = env.storage().instance().get(&DataKey::DobToken).unwrap();
        let operator = env.storage().instance().get(&DataKey::Operator).unwrap();
        let amm_pool = env.storage().instance().get(&DataKey::AmmPool).unwrap();

        (oracle, usdc_token, dob_token, operator, amm_pool)
    }

    /// Register this Liquid Node with an AMM Pool
    pub fn register_with_pool(env: Env, pool: Address) -> Result<(), Error> {
        let operator: Address = env.storage().instance().get(&DataKey::Operator).unwrap();
        operator.require_auth();

        // Call AMM pool's register function
        let _: () = env.invoke_contract(
            &pool,
            &Symbol::new(&env, "register_liquid_node"),
            (env.current_contract_address(),).into_val(&env),
        );

        env.storage().instance().set(&DataKey::AmmPool, &pool);

        env.events().publish(
            (Symbol::new(&env, "registered_with_pool"),),
            pool,
        );

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dynamic_fee_calculation() {
        // Risk 10% (1000 bps) -> 5% fee (500 bps)
        // Risk 20% (2000 bps) -> 10% fee (1000 bps)
        // Risk 35% (3500 bps) -> 20% fee (2000 bps)
        // Risk 60% (6000 bps) -> 30% fee (3000 bps)
    }
}
