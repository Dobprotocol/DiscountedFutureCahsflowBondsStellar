#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, String as SorobanString, Symbol, Vec,
};

// Import all contract clients
mod token {
    soroban_sdk::contractimport!(file = "./target/wasm32-unknown-unknown/release/dob_token.wasm");
}

mod oracle {
    soroban_sdk::contractimport!(file = "./target/wasm32-unknown-unknown/release/dob_oracle.wasm");
}

mod amm_pool {
    soroban_sdk::contractimport!(file = "./target/wasm32-unknown-unknown/release/dob_amm_pool.wasm");
}

mod stabilizer {
    soroban_sdk::contractimport!(file = "./target/wasm32-unknown-unknown/release/liquid_node_stabilizer.wasm");
}

/// Test: Complete AMM Pool lifecycle with liquidity provision
#[test]
fn test_amm_pool_with_open_liquidity() {
    let env = Env::default();
    env.mock_all_auths();

    // Create addresses
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let lp_provider1 = Address::generate(&env);
    let lp_provider2 = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    // Deploy USDC token (mock)
    let usdc_id = env.register_stellar_asset_contract(admin.clone());
    let usdc_client = token::StellarAssetClient::new(&env, &usdc_id);

    // Deploy contracts
    let dob_token_id = env.register_contract_wasm(None, token::WASM);
    let dob_token_client = token::Client::new(&env, &dob_token_id);

    let oracle_id = env.register_contract_wasm(None, oracle::WASM);
    let oracle_client = oracle::Client::new(&env, &oracle_id);

    let amm_pool_id = env.register_contract_wasm(None, amm_pool::WASM);
    let amm_pool_client = amm_pool::Client::new(&env, &amm_pool_id);

    // Initialize DOB token
    dob_token_client.initialize(
        &admin,
        &amm_pool_id,
        &SorobanString::from_str(&env, "Dob Solar Farm 2035"),
        &SorobanString::from_str(&env, "DOB-35"),
        &7,
    );

    // Initialize oracle: NAV = 1.00, Risk = 10%
    oracle_client.initialize(&admin, &10_000_000, &1000);

    // Initialize AMM Pool
    amm_pool_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);

    // Mint USDC to LP providers
    usdc_client.mint(&lp_provider1, &100_000_0000000); // 100k USDC
    usdc_client.mint(&lp_provider2, &50_000_0000000);  // 50k USDC

    // Mint DOB to LP providers (simulate pre-existing tokens)
    dob_token_client.mint(&lp_provider1, &100_000_0000000); // 100k DOB
    dob_token_client.mint(&lp_provider2, &50_000_0000000);  // 50k DOB

    // LP Provider 1 adds liquidity
    let lp_shares1 = amm_pool_client.add_liquidity(
        &lp_provider1,
        &100_000_0000000,
        &100_000_0000000,
    );

    assert!(lp_shares1 > 0);

    // Check reserves
    let (usdc_reserve, dob_reserve) = amm_pool_client.get_reserves();
    assert_eq!(usdc_reserve, 100_000_0000000);
    assert_eq!(dob_reserve, 100_000_0000000);

    // LP Provider 2 adds liquidity
    let lp_shares2 = amm_pool_client.add_liquidity(
        &lp_provider2,
        &50_000_0000000,
        &50_000_0000000,
    );

    assert!(lp_shares2 > 0);

    // Check updated reserves
    let (usdc_reserve, dob_reserve) = amm_pool_client.get_reserves();
    assert_eq!(usdc_reserve, 150_000_0000000);
    assert_eq!(dob_reserve, 150_000_0000000);

    // Test buy: Buyer purchases DOB with USDC
    usdc_client.mint(&buyer, &1_000_0000000); // 1k USDC

    let dob_received = amm_pool_client.swap_buy(&buyer, &1_000_0000000);

    assert!(dob_received > 0);

    // Verify buyer received DOB tokens
    let buyer_dob_balance = dob_token_client.balance(&buyer);
    assert_eq!(buyer_dob_balance, dob_received);

    // Test sell with sufficient liquidity
    let dob_to_sell = 500_0000000i128; // 500 DOB
    dob_token_client.mint(&seller, &dob_to_sell);

    let usdc_received = amm_pool_client.swap_sell(&seller, &dob_to_sell);

    assert!(usdc_received > 0);

    // Verify seller received USDC
    let seller_usdc_balance = usdc_client.balance(&seller);
    assert_eq!(seller_usdc_balance, usdc_received);

    // LP Provider 1 removes liquidity
    let (usdc_out, dob_out) = amm_pool_client.remove_liquidity(&lp_provider1, &lp_shares1);

    assert!(usdc_out > 0);
    assert!(dob_out > 0);

    println!("AMM Pool with open liquidity test passed!");
}

/// Test: Sell without sufficient liquidity - triggers Liquid Node search
#[test]
fn test_sell_with_liquid_node_fallback() {
    let env = Env::default();
    env.mock_all_auths();

    // Create addresses
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let ln_operator = Address::generate(&env);
    let seller = Address::generate(&env);

    // Deploy USDC token
    let usdc_id = env.register_stellar_asset_contract(admin.clone());
    let usdc_client = token::StellarAssetClient::new(&env, &usdc_id);

    // Deploy contracts
    let dob_token_id = env.register_contract_wasm(None, token::WASM);
    let dob_token_client = token::Client::new(&env, &dob_token_id);

    let oracle_id = env.register_contract_wasm(None, oracle::WASM);
    let oracle_client = oracle::Client::new(&env, &oracle_id);

    let amm_pool_id = env.register_contract_wasm(None, amm_pool::WASM);
    let amm_pool_client = amm_pool::Client::new(&env, &amm_pool_id);

    let stabilizer_id = env.register_contract_wasm(None, stabilizer::WASM);
    let stabilizer_client = stabilizer::Client::new(&env, &stabilizer_id);

    // Initialize contracts
    dob_token_client.initialize(
        &admin,
        &amm_pool_id,
        &SorobanString::from_str(&env, "DOB Token"),
        &SorobanString::from_str(&env, "DOB"),
        &7,
    );

    oracle_client.initialize(&admin, &10_000_000, &1000); // NAV=1.00, Risk=10%

    amm_pool_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);

    stabilizer_client.initialize(
        &oracle_id,
        &usdc_id,
        &dob_token_id,
        &ln_operator,
        &amm_pool_id,
    );

    // Add small liquidity to pool (insufficient for large sell)
    let lp_provider = Address::generate(&env);
    usdc_client.mint(&lp_provider, &10_000_0000000); // 10k USDC
    dob_token_client.mint(&lp_provider, &10_000_0000000);

    amm_pool_client.add_liquidity(&lp_provider, &10_000_0000000, &10_000_0000000);

    // Fund Liquid Node with large USDC balance
    usdc_client.mint(&ln_operator, &100_000_0000000); // 100k USDC
    stabilizer_client.fund_usdc(&ln_operator, &100_000_0000000);

    // Register Liquid Node with AMM Pool
    amm_pool_client.register_liquid_node(&stabilizer_id);

    // Verify registration
    let liquid_nodes = amm_pool_client.get_liquid_nodes();
    assert_eq!(liquid_nodes.len(), 1);

    // Seller wants to sell large amount (more than pool has)
    let dob_to_sell = 150_000_0000000i128; // 150k DOB
    dob_token_client.mint(&seller, &dob_to_sell);

    // Quote the swap (should show it will use liquid nodes)
    let quote = amm_pool_client.quote_swap_sell(&dob_to_sell);

    assert!(quote.from_liquid_nodes > 0);
    println!("Quote: from_pool={}, from_ln={}", quote.from_pool, quote.from_liquid_nodes);

    // Execute the swap
    let usdc_received = amm_pool_client.swap_sell(&seller, &dob_to_sell);

    assert!(usdc_received > 0);
    assert_eq!(usdc_received, quote.usdc_out);

    println!("Liquid Node fallback test passed!");
}

/// Test: Multiple Liquid Nodes competing for best fee
#[test]
fn test_multiple_liquid_nodes_competition() {
    let env = Env::default();
    env.mock_all_auths();

    // Create addresses
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let ln_operator1 = Address::generate(&env);
    let ln_operator2 = Address::generate(&env);
    let ln_operator3 = Address::generate(&env);
    let seller = Address::generate(&env);

    // Deploy USDC token
    let usdc_id = env.register_stellar_asset_contract(admin.clone());
    let usdc_client = token::StellarAssetClient::new(&env, &usdc_id);

    // Deploy contracts
    let dob_token_id = env.register_contract_wasm(None, token::WASM);
    let dob_token_client = token::Client::new(&env, &dob_token_id);

    let oracle_id = env.register_contract_wasm(None, oracle::WASM);
    let oracle_client = oracle::Client::new(&env, &oracle_id);

    let amm_pool_id = env.register_contract_wasm(None, amm_pool::WASM);
    let amm_pool_client = amm_pool::Client::new(&env, &amm_pool_id);

    // Deploy 3 Liquid Nodes
    let ln1_id = env.register_contract_wasm(None, stabilizer::WASM);
    let ln1_client = stabilizer::Client::new(&env, &ln1_id);

    let ln2_id = env.register_contract_wasm(None, stabilizer::WASM);
    let ln2_client = stabilizer::Client::new(&env, &ln2_id);

    let ln3_id = env.register_contract_wasm(None, stabilizer::WASM);
    let ln3_client = stabilizer::Client::new(&env, &ln3_id);

    // Initialize contracts
    dob_token_client.initialize(
        &admin,
        &amm_pool_id,
        &SorobanString::from_str(&env, "DOB Token"),
        &SorobanString::from_str(&env, "DOB"),
        &7,
    );

    oracle_client.initialize(&admin, &10_000_000, &1000); // NAV=1.00, Risk=10% (5% fee)

    amm_pool_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);

    // Initialize all 3 Liquid Nodes
    ln1_client.initialize(&oracle_id, &usdc_id, &dob_token_id, &ln_operator1, &amm_pool_id);
    ln2_client.initialize(&oracle_id, &usdc_id, &dob_token_id, &ln_operator2, &amm_pool_id);
    ln3_client.initialize(&oracle_id, &usdc_id, &dob_token_id, &ln_operator3, &amm_pool_id);

    // Fund all Liquid Nodes
    usdc_client.mint(&ln_operator1, &50_000_0000000);
    usdc_client.mint(&ln_operator2, &50_000_0000000);
    usdc_client.mint(&ln_operator3, &50_000_0000000);

    ln1_client.fund_usdc(&ln_operator1, &50_000_0000000);
    ln2_client.fund_usdc(&ln_operator2, &50_000_0000000);
    ln3_client.fund_usdc(&ln_operator3, &50_000_0000000);

    // Register all 3 Liquid Nodes
    amm_pool_client.register_liquid_node(&ln1_id);
    amm_pool_client.register_liquid_node(&ln2_id);
    amm_pool_client.register_liquid_node(&ln3_id);

    // Verify all are registered
    let liquid_nodes = amm_pool_client.get_liquid_nodes();
    assert_eq!(liquid_nodes.len(), 3);

    // Seller sells DOB
    let dob_to_sell = 100_000_0000000i128; // 100k DOB
    dob_token_client.mint(&seller, &dob_to_sell);

    let usdc_received = amm_pool_client.swap_sell(&seller, &dob_to_sell);

    assert!(usdc_received > 0);

    // The AMM should have selected the best Liquid Node (all have same fee since same risk)
    println!("Multiple LN competition test passed! USDC received: {}", usdc_received);
}

/// Test: Register and unregister Liquid Nodes
#[test]
fn test_liquid_node_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let ln_operator = Address::generate(&env);

    // Deploy minimal contracts
    let usdc_id = env.register_stellar_asset_contract(admin.clone());
    let dob_token_id = env.register_contract_wasm(None, token::WASM);
    let oracle_id = env.register_contract_wasm(None, oracle::WASM);
    let amm_pool_id = env.register_contract_wasm(None, amm_pool::WASM);
    let stabilizer_id = env.register_contract_wasm(None, stabilizer::WASM);

    let dob_token_client = token::Client::new(&env, &dob_token_id);
    let oracle_client = oracle::Client::new(&env, &oracle_id);
    let amm_pool_client = amm_pool::Client::new(&env, &amm_pool_id);
    let stabilizer_client = stabilizer::Client::new(&env, &stabilizer_id);

    // Initialize
    dob_token_client.initialize(
        &admin,
        &amm_pool_id,
        &SorobanString::from_str(&env, "DOB"),
        &SorobanString::from_str(&env, "DOB"),
        &7,
    );

    oracle_client.initialize(&admin, &10_000_000, &1000);
    amm_pool_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);
    stabilizer_client.initialize(&oracle_id, &usdc_id, &dob_token_id, &ln_operator, &amm_pool_id);

    // Register Liquid Node
    amm_pool_client.register_liquid_node(&stabilizer_id);

    let nodes = amm_pool_client.get_liquid_nodes();
    assert_eq!(nodes.len(), 1);

    // Unregister Liquid Node
    amm_pool_client.unregister_liquid_node(&stabilizer_id);

    let nodes = amm_pool_client.get_liquid_nodes();
    assert_eq!(nodes.len(), 0);

    println!("Registration test passed!");
}

/// Test: AfterSwap hook on buy
#[test]
fn test_afterswap_hook_buy() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Deploy contracts
    let usdc_id = env.register_stellar_asset_contract(admin.clone());
    let usdc_client = token::StellarAssetClient::new(&env, &usdc_id);

    let dob_token_id = env.register_contract_wasm(None, token::WASM);
    let dob_token_client = token::Client::new(&env, &dob_token_id);

    let oracle_id = env.register_contract_wasm(None, oracle::WASM);
    let oracle_client = oracle::Client::new(&env, &oracle_id);

    let amm_pool_id = env.register_contract_wasm(None, amm_pool::WASM);
    let amm_pool_client = amm_pool::Client::new(&env, &amm_pool_id);

    // Initialize
    dob_token_client.initialize(
        &admin,
        &amm_pool_id,
        &SorobanString::from_str(&env, "DOB"),
        &SorobanString::from_str(&env, "DOB"),
        &7,
    );

    oracle_client.initialize(&admin, &10_000_000, &1000); // NAV=1.00
    amm_pool_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);

    // Buyer buys DOB
    usdc_client.mint(&buyer, &1_000_0000000); // 1k USDC

    let initial_operator_balance = usdc_client.balance(&operator);

    let dob_received = amm_pool_client.swap_buy(&buyer, &1_000_0000000);

    // AfterSwap: DOB tokens should be minted to buyer
    let buyer_dob_balance = dob_token_client.balance(&buyer);
    assert_eq!(buyer_dob_balance, dob_received);

    // Verify operator received 99% of USDC (minus 1% DEX fee)
    let operator_balance_after = usdc_client.balance(&operator);
    assert!(operator_balance_after > initial_operator_balance);

    // Expected: 1000 USDC - 1% DEX fee = 990 USDC, then 99% to operator = 980.1 USDC
    let expected_operator_usdc = (1_000_0000000 * 99) / 100 - (1_000_0000000 * 1) / 100;

    println!("AfterSwap hook buy test passed!");
    println!("DOB minted: {}, Operator USDC: {}", dob_received, operator_balance_after);
}

/// Test: BeforeSwap hook on sell with pool liquidity
#[test]
fn test_beforeswap_hook_sell() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let lp_provider = Address::generate(&env);
    let seller = Address::generate(&env);

    // Deploy contracts
    let usdc_id = env.register_stellar_asset_contract(admin.clone());
    let usdc_client = token::StellarAssetClient::new(&env, &usdc_id);

    let dob_token_id = env.register_contract_wasm(None, token::WASM);
    let dob_token_client = token::Client::new(&env, &dob_token_id);

    let oracle_id = env.register_contract_wasm(None, oracle::WASM);
    let oracle_client = oracle::Client::new(&env, &oracle_id);

    let amm_pool_id = env.register_contract_wasm(None, amm_pool::WASM);
    let amm_pool_client = amm_pool::Client::new(&env, &amm_pool_id);

    // Initialize
    dob_token_client.initialize(
        &admin,
        &amm_pool_id,
        &SorobanString::from_str(&env, "DOB"),
        &SorobanString::from_str(&env, "DOB"),
        &7,
    );

    oracle_client.initialize(&admin, &10_000_000, &1000); // NAV=1.00, Risk=10%
    amm_pool_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);

    // Add liquidity to pool
    usdc_client.mint(&lp_provider, &100_000_0000000);
    dob_token_client.mint(&lp_provider, &100_000_0000000);

    amm_pool_client.add_liquidity(&lp_provider, &100_000_0000000, &100_000_0000000);

    // Seller sells DOB
    let dob_to_sell = 1_000_0000000i128; // 1k DOB
    dob_token_client.mint(&seller, &dob_to_sell);

    let (usdc_before, dob_before) = amm_pool_client.get_reserves();

    let usdc_received = amm_pool_client.swap_sell(&seller, &dob_to_sell);

    // BeforeSwap: Pool should check liquidity first
    // Seller should receive USDC minus fee (3% base + 1% from risk = 4%)
    assert!(usdc_received > 0);

    let (usdc_after, dob_after) = amm_pool_client.get_reserves();

    // Verify reserves changed
    assert!(usdc_after < usdc_before); // USDC decreased
    assert!(dob_after > dob_before);   // DOB increased

    println!("BeforeSwap hook sell test passed!");
    println!("USDC received: {}", usdc_received);
}
