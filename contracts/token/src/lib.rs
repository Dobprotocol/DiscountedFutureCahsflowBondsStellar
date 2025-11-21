#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, Address, Env, String, Symbol};

/// Storage keys for the contract
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,           // Contract administrator
    Hook,            // Hook contract that can mint/burn
    Name,            // Token name
    Symbol,          // Token symbol
    Decimals,        // Token decimals
    TotalSupply,     // Total supply
    Balance(Address), // Balance of an address
    Allowance(Address, Address), // Allowance from owner to spender
}

/// Errors that can be returned by the contract
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Unauthorized = 1,
    InsufficientBalance = 2,
    InsufficientAllowance = 3,
}

/// DobToken - ERC20-like token for RWA revenue streams
/// Only the hook contract can mint and burn tokens
#[contract]
pub struct DobToken;

#[contractimpl]
impl DobToken {
    /// Initialize the token contract
    pub fn initialize(
        env: Env,
        admin: Address,
        hook: Address,
        name: String,
        symbol: String,
        decimals: u32,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Hook, &hook);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
    }

    /// Get token name
    pub fn name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .unwrap_or(String::from_str(&env, "DOB Token"))
    }

    /// Get token symbol
    pub fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .unwrap_or(String::from_str(&env, "DOB"))
    }

    /// Get token decimals
    pub fn decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Decimals)
            .unwrap_or(7)
    }

    /// Get total supply
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    /// Get balance of an address
    pub fn balance(env: Env, account: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(account))
            .unwrap_or(0)
    }

    /// Transfer tokens
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if amount < 0 {
            return Err(Error::InsufficientBalance);
        }

        let from_balance = Self::balance(env.clone(), from.clone());
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let to_balance = Self::balance(env.clone(), to.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));

        env.events().publish(
            (Symbol::new(&env, "transfer"), from, to),
            amount,
        );

        Ok(())
    }

    /// Approve spender
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::Allowance(owner.clone(), spender.clone()), &amount);

        env.events().publish(
            (Symbol::new(&env, "approve"), owner, spender),
            amount,
        );
    }

    /// Get allowance
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Allowance(owner, spender))
            .unwrap_or(0)
    }

    /// Transfer from (with allowance)
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        spender.require_auth();

        if amount < 0 {
            return Err(Error::InsufficientBalance);
        }

        let allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        if allowance < amount {
            return Err(Error::InsufficientAllowance);
        }

        let from_balance = Self::balance(env.clone(), from.clone());
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let to_balance = Self::balance(env.clone(), to.clone());

        // Update balances
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));

        // Update allowance
        env.storage()
            .persistent()
            .set(
                &DataKey::Allowance(from.clone(), spender.clone()),
                &(allowance - amount),
            );

        env.events().publish(
            (Symbol::new(&env, "transfer"), from, to),
            amount,
        );

        Ok(())
    }

    /// Mint new tokens (only callable by hook)
    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        let hook: Address = env
            .storage()
            .instance()
            .get(&DataKey::Hook)
            .expect("Hook not set");

        hook.require_auth();

        if amount < 0 {
            panic!("Invalid amount");
        }

        let to_balance = Self::balance(env.clone(), to.clone());
        let total_supply = Self::total_supply(env.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_balance + amount));
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(total_supply + amount));

        env.events().publish(
            (Symbol::new(&env, "mint"), to),
            amount,
        );

        Ok(())
    }

    /// Burn tokens from an address (only callable by hook)
    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        let hook: Address = env
            .storage()
            .instance()
            .get(&DataKey::Hook)
            .expect("Hook not set");

        hook.require_auth();

        if amount < 0 {
            panic!("Invalid amount");
        }

        let from_balance = Self::balance(env.clone(), from.clone());
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let total_supply = Self::total_supply(env.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_balance - amount));
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(total_supply - amount));

        env.events().publish(
            (Symbol::new(&env, "burn"), from),
            amount,
        );

        Ok(())
    }

    /// Get hook address
    pub fn hook(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Hook)
            .expect("Hook not set")
    }

    /// Get admin address
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set")
    }

    /// Update hook address (only admin)
    pub fn set_hook(env: Env, new_hook: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set");

        admin.require_auth();

        env.storage().instance().set(&DataKey::Hook, &new_hook);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobToken);
        let client = DobTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let hook = Address::generate(&env);

        env.mock_all_auths();

        client.initialize(
            &admin,
            &hook,
            &String::from_str(&env, "Dob Solar Farm 2035"),
            &String::from_str(&env, "DOB-35"),
            &7,
        );

        assert_eq!(client.name(), String::from_str(&env, "Dob Solar Farm 2035"));
        assert_eq!(client.symbol(), String::from_str(&env, "DOB-35"));
        assert_eq!(client.decimals(), 7);
        assert_eq!(client.total_supply(), 0);
    }

    #[test]
    fn test_mint_and_burn() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobToken);
        let client = DobTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let hook = Address::generate(&env);
        let user = Address::generate(&env);

        env.mock_all_auths();

        client.initialize(
            &admin,
            &hook,
            &String::from_str(&env, "DOB Token"),
            &String::from_str(&env, "DOB"),
            &7,
        );

        // Mint tokens
        client.mint(&user, &1000);
        assert_eq!(client.balance(&user), 1000);
        assert_eq!(client.total_supply(), 1000);

        // Burn tokens
        client.burn(&user, &300);
        assert_eq!(client.balance(&user), 700);
        assert_eq!(client.total_supply(), 700);
    }

    #[test]
    fn test_transfer() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DobToken);
        let client = DobTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let hook = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        env.mock_all_auths();

        client.initialize(
            &admin,
            &hook,
            &String::from_str(&env, "DOB Token"),
            &String::from_str(&env, "DOB"),
            &7,
        );

        // Mint to user1
        client.mint(&user1, &1000);

        // Transfer from user1 to user2
        client.transfer(&user1, &user2, &400);

        assert_eq!(client.balance(&user1), 600);
        assert_eq!(client.balance(&user2), 400);
    }
}
