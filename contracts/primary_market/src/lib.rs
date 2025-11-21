#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, token, Address, Env, IntoVal, Symbol, Vec};

/// Storage keys for the contract
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    DobToken,    // DOB token contract address
    UsdcToken,   // USDC token contract address
    Oracle,      // Oracle contract address
    Operator,    // Operator receiving revenues
    TotalBought, // Total USDC spent on buys
    TotalSold,   // Total DOB sold
}

/// Buy event data
#[contracttype]
#[derive(Clone, Debug)]
pub struct BuyEvent {
    pub buyer: Address,
    pub usdc_in: i128,
    pub dob_minted: i128,
}

/// Sell event data
#[contracttype]
#[derive(Clone, Debug)]
pub struct SellEvent {
    pub seller: Address,
    pub dob_in: i128,
    pub usdc_out: i128,
    pub penalty_bps: u32,
}

/// Redemption quote
#[contracttype]
#[derive(Clone, Debug)]
pub struct RedemptionQuote {
    pub usdc_out: i128,
    pub penalty_bps: u32,
}

/// Errors that can be returned by the contract
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InsufficientLiquidity = 1,
    InvalidAmount = 2,
    TransferFailed = 3,
}

// Constants
const OPERATOR_SHARE: u32 = 99; // 99% to operator on buys
const BPS: u32 = 10000; // Basis points denominator

#[contract]
pub struct DobPrimaryMarket;

#[contractimpl]
impl DobPrimaryMarket {
    /// Initialize the primary market contract
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
        env.storage().instance().set(&DataKey::TotalBought, &0i128);
        env.storage().instance().set(&DataKey::TotalSold, &0i128);
    }

    /// Buy DOB tokens with USDC (Primary Market)
    /// 99% of USDC goes to operator, 1% fee
    /// DOB tokens minted to buyer at NAV rate
    pub fn buy(env: Env, buyer: Address, usdc_amount: i128) -> Result<i128, Error> {
        buyer.require_auth();

        if usdc_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let dob_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::DobToken)
            .expect("DOB token not set");
        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .expect("USDC token not set");
        let oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::Oracle)
            .expect("Oracle not set");
        let operator: Address = env
            .storage()
            .instance()
            .get(&DataKey::Operator)
            .expect("Operator not set");

        // Get current NAV from oracle
        let nav: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "nav"), soroban_sdk::vec![&env]);

        // Transfer USDC from buyer to contract
        let usdc_client = token::Client::new(&env, &usdc_token);
        usdc_client.transfer(&buyer, &env.current_contract_address(), &usdc_amount);

        // 99% to operator
        let operator_amount = (usdc_amount * OPERATOR_SHARE as i128) / 100;
        usdc_client.transfer(&env.current_contract_address(), &operator, &operator_amount);

        // Calculate DOB to mint: (USDC × 0.99) / NAV
        // NAV is in 7 decimals, USDC is in 7 decimals
        // Result should be in 7 decimals for DOB
        let dob_amount = (operator_amount * 10_000_000) / nav;

        // Mint DOB tokens to buyer
        let mint_args: Vec<soroban_sdk::Val> = (buyer.clone(), dob_amount).into_val(&env);
        let _: () = env.invoke_contract(
            &dob_token,
            &Symbol::new(&env, "mint"),
            mint_args,
        );

        // Update stats
        let total_bought: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalBought)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalBought, &(total_bought + usdc_amount));

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "buy"),),
            BuyEvent {
                buyer,
                usdc_in: usdc_amount,
                dob_minted: dob_amount,
            },
        );

        Ok(dob_amount)
    }

    /// Sell DOB tokens for USDC (Secondary Market)
    /// USDC returned = DOB × NAV × (1 - penalty)
    /// Penalty based on default risk: 3% base + risk/10
    pub fn sell(env: Env, seller: Address, dob_amount: i128) -> Result<i128, Error> {
        seller.require_auth();

        if dob_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let dob_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::DobToken)
            .expect("DOB token not set");
        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .expect("USDC token not set");

        // Calculate redemption
        let quote = Self::quote_redemption(env.clone(), dob_amount);

        // Check contract has enough USDC
        let usdc_client = token::Client::new(&env, &usdc_token);
        let contract_balance = usdc_client.balance(&env.current_contract_address());

        if contract_balance < quote.usdc_out {
            return Err(Error::InsufficientLiquidity);
        }

        // Burn DOB tokens from seller
        let burn_args: Vec<soroban_sdk::Val> = (seller.clone(), dob_amount).into_val(&env);
        let _: () = env.invoke_contract(
            &dob_token,
            &Symbol::new(&env, "burn"),
            burn_args,
        );

        // Transfer USDC to seller
        usdc_client.transfer(&env.current_contract_address(), &seller, &quote.usdc_out);

        // Update stats
        let total_sold: i128 = env.storage().instance().get(&DataKey::TotalSold).unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSold, &(total_sold + dob_amount));

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "sell"),),
            SellEvent {
                seller,
                dob_in: dob_amount,
                usdc_out: quote.usdc_out,
                penalty_bps: quote.penalty_bps,
            },
        );

        Ok(quote.usdc_out)
    }

    /// Get quote for selling DOB tokens
    /// Returns expected USDC output and penalty in basis points
    pub fn quote_redemption(env: Env, dob_amount: i128) -> RedemptionQuote {
        let oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::Oracle)
            .expect("Oracle not set");

        // Get NAV and default risk from oracle
        let nav: i128 = env.invoke_contract(&oracle, &Symbol::new(&env, "nav"), soroban_sdk::vec![&env]);
        let risk: u32 = env.invoke_contract(&oracle, &Symbol::new(&env, "default_risk"), soroban_sdk::vec![&env]);

        // Calculate penalty: 3% base + risk/10
        let penalty_bps = 300 + (risk / 10);
        // Cap at 50%
        let penalty_bps = if penalty_bps > 5000 { 5000 } else { penalty_bps };

        // USDC out = DOB × NAV × (1 - penalty)
        // DOB is 7 decimals, NAV is 7 decimals
        // Result should be 7 decimals for USDC
        let value_before_penalty = (dob_amount * nav) / 10_000_000;
        let usdc_out = (value_before_penalty * (BPS - penalty_bps) as i128) / BPS as i128;

        RedemptionQuote {
            usdc_out,
            penalty_bps,
        }
    }

    /// Get current NAV from oracle
    pub fn get_nav(env: Env) -> i128 {
        let oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::Oracle)
            .expect("Oracle not set");

        env.invoke_contract(&oracle, &Symbol::new(&env, "nav"), soroban_sdk::vec![&env])
    }

    /// Get current default risk from oracle
    pub fn get_default_risk(env: Env) -> u32 {
        let oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::Oracle)
            .expect("Oracle not set");

        env.invoke_contract(&oracle, &Symbol::new(&env, "default_risk"), soroban_sdk::vec![&env])
    }

    /// Get contract addresses
    pub fn get_addresses(env: Env) -> (Address, Address, Address, Address) {
        let dob_token = env.storage().instance().get(&DataKey::DobToken).unwrap();
        let usdc_token = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let oracle = env.storage().instance().get(&DataKey::Oracle).unwrap();
        let operator = env.storage().instance().get(&DataKey::Operator).unwrap();

        (dob_token, usdc_token, oracle, operator)
    }

    /// Get trading statistics
    pub fn get_stats(env: Env) -> (i128, i128) {
        let total_bought = env
            .storage()
            .instance()
            .get(&DataKey::TotalBought)
            .unwrap_or(0);
        let total_sold = env.storage().instance().get(&DataKey::TotalSold).unwrap_or(0);

        (total_bought, total_sold)
    }

    /// Fund the contract with USDC for redemptions
    pub fn fund(env: Env, funder: Address, amount: i128) -> Result<(), Error> {
        funder.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .expect("USDC token not set");

        let usdc_client = token::Client::new(&env, &usdc_token);
        usdc_client.transfer(&funder, &env.current_contract_address(), &amount);

        env.events()
            .publish((Symbol::new(&env, "funded"), funder), amount);

        Ok(())
    }

    /// Get contract USDC balance
    pub fn get_balance(env: Env) -> i128 {
        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .expect("USDC token not set");

        let usdc_client = token::Client::new(&env, &usdc_token);
        usdc_client.balance(&env.current_contract_address())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quote_calculation() {
        // This is a simplified test - full integration tests would need
        // deployed oracle and token contracts
        // Penalty = 300 + 1000/10 = 400 bps = 4%
        // NAV = 1.00 (10_000_000 with 7 decimals)
        // DOB amount = 1000 (10_000_000 with 7 decimals = 1000 tokens)
        // Value = 1000 * 1.00 = 1000 USDC
        // After 4% penalty = 1000 * 0.96 = 960 USDC
    }
}
