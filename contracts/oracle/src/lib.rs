#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, Address, Env, Symbol};

/// Storage keys for the oracle contract
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Nav,          // Net Asset Value (7 decimals: 1.00 = 10000000)
    DefaultRisk,  // Default risk in basis points (10000 = 100%)
    Updater,      // Address authorized to update values
}

/// Oracle update event data
#[contracttype]
#[derive(Clone, Debug)]
pub struct OracleUpdate {
    pub nav: i128,
    pub default_risk: u32,
}

/// Errors that can be returned by the oracle
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Unauthorized = 1,
}

/// DobOracle - Simple push oracle for NAV and default risk
/// Perfect for testing and MVP - trusted operator updates values
#[contract]
pub struct DobOracle;

#[contractimpl]
impl DobOracle {
    /// Initialize the oracle contract
    pub fn initialize(env: Env, updater: Address, initial_nav: i128, initial_risk: u32) {
        if env.storage().instance().has(&DataKey::Updater) {
            panic!("Already initialized");
        }

        updater.require_auth();

        env.storage().instance().set(&DataKey::Updater, &updater);
        env.storage().instance().set(&DataKey::Nav, &initial_nav);
        env.storage()
            .instance()
            .set(&DataKey::DefaultRisk, &initial_risk);

        env.events().publish(
            (Symbol::new(&env, "initialized"),),
            OracleUpdate {
                nav: initial_nav,
                default_risk: initial_risk,
            },
        );
    }

    /// Get current NAV (Net Asset Value)
    /// Returns value with 7 decimals (e.g., 10000000 = 1.00 USDC per token)
    pub fn nav(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Nav)
            .unwrap_or(10_000_000) // Default: 1.00 with 7 decimals
    }

    /// Get current default risk in basis points
    /// 10000 basis points = 100%
    /// 1000 basis points = 10%
    pub fn default_risk(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::DefaultRisk)
            .unwrap_or(1000) // Default: 10%
    }

    /// Update NAV and default risk (only updater can call)
    pub fn update(env: Env, new_nav: i128, new_default_risk: u32) -> Result<(), Error> {
        let updater: Address = env
            .storage()
            .instance()
            .get(&DataKey::Updater)
            .expect("Updater not set");

        updater.require_auth();

        if new_nav <= 0 {
            panic!("Invalid NAV");
        }

        if new_default_risk > 10000 {
            panic!("Risk cannot exceed 100%");
        }

        env.storage().instance().set(&DataKey::Nav, &new_nav);
        env.storage()
            .instance()
            .set(&DataKey::DefaultRisk, &new_default_risk);

        env.events().publish(
            (Symbol::new(&env, "oracle_updated"),),
            OracleUpdate {
                nav: new_nav,
                default_risk: new_default_risk,
            },
        );

        Ok(())
    }

    /// Get current updater address
    pub fn updater(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Updater)
            .expect("Updater not set")
    }

    /// Transfer updater role to new address (only current updater)
    pub fn set_updater(env: Env, new_updater: Address) -> Result<(), Error> {
        let updater: Address = env
            .storage()
            .instance()
            .get(&DataKey::Updater)
            .expect("Updater not set");

        updater.require_auth();

        env.storage().instance().set(&DataKey::Updater, &new_updater);

        env.events().publish(
            (Symbol::new(&env, "updater_changed"), updater),
            new_updater.clone(),
        );

        Ok(())
    }

    /// Calculate redemption penalty based on current risk
    /// Returns penalty in basis points (10000 = 100%)
    pub fn calculate_penalty(env: Env) -> u32 {
        let risk = Self::default_risk(env);

        // Base penalty 3% + risk factor (risk/10)
        // e.g., 10% default risk = 300 + 100 = 400 bps = 4% penalty
        let penalty = 300 + (risk / 10);

        // Cap penalty at 50% (5000 bps)
        if penalty > 5000 {
            5000
        } else {
            penalty
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobOracle);
        let client = DobOracleClient::new(&env, &contract_id);

        let updater = Address::generate(&env);

        env.mock_all_auths();

        // Initialize with 1.00 NAV and 10% risk
        client.initialize(&updater, &10_000_000, &1000);

        assert_eq!(client.nav(), 10_000_000);
        assert_eq!(client.default_risk(), 1000);
        assert_eq!(client.updater(), updater);
    }

    #[test]
    fn test_update() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobOracle);
        let client = DobOracleClient::new(&env, &contract_id);

        let updater = Address::generate(&env);

        env.mock_all_auths();

        client.initialize(&updater, &10_000_000, &1000);

        // Update to NAV 1.15 and 7% risk
        client.update(&11_500_000, &700);

        assert_eq!(client.nav(), 11_500_000);
        assert_eq!(client.default_risk(), 700);
    }

    #[test]
    fn test_calculate_penalty() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobOracle);
        let client = DobOracleClient::new(&env, &contract_id);

        let updater = Address::generate(&env);

        env.mock_all_auths();

        // 10% default risk
        client.initialize(&updater, &10_000_000, &1000);
        // Penalty = 300 + 1000/10 = 400 bps = 4%
        assert_eq!(client.calculate_penalty(), 400);

        // 35% default risk
        client.update(&10_000_000, &3500);
        // Penalty = 300 + 3500/10 = 650 bps = 6.5%
        assert_eq!(client.calculate_penalty(), 650);

        // Very high risk (60%)
        client.update(&10_000_000, &6000);
        // Penalty = 300 + 6000/10 = 900, but capped at 5000 (50%)
        assert_eq!(client.calculate_penalty(), 900);
    }

    #[test]
    fn test_set_updater() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobOracle);
        let client = DobOracleClient::new(&env, &contract_id);

        let updater1 = Address::generate(&env);
        let updater2 = Address::generate(&env);

        env.mock_all_auths();

        client.initialize(&updater1, &10_000_000, &1000);
        assert_eq!(client.updater(), updater1);

        // Transfer updater role
        client.set_updater(&updater2);
        assert_eq!(client.updater(), updater2);
    }

    #[test]
    #[should_panic(expected = "Invalid NAV")]
    fn test_invalid_nav() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobOracle);
        let client = DobOracleClient::new(&env, &contract_id);

        let updater = Address::generate(&env);

        env.mock_all_auths();

        client.initialize(&updater, &10_000_000, &1000);
        client.update(&0, &1000); // Should panic
    }

    #[test]
    #[should_panic(expected = "Risk cannot exceed 100%")]
    fn test_invalid_risk() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobOracle);
        let client = DobOracleClient::new(&env, &contract_id);

        let updater = Address::generate(&env);

        env.mock_all_auths();

        client.initialize(&updater, &10_000_000, &1000);
        client.update(&10_000_000, &10001); // Should panic
    }
}
