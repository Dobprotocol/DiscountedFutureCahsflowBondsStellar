#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    token, Address, Env, String as SorobanString,
};

// Import contract clients
mod dob_token {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/dob_token.wasm"
    );
}

mod dob_oracle {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/dob_oracle.wasm"
    );
}

mod dob_primary_market {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/dob_primary_market.wasm"
    );
}

#[test]
fn test_basic_buy_sell_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    // Setup addresses
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let alice = Address::generate(&env);

    // Create USDC token
    let usdc_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let usdc_admin = token::StellarAssetClient::new(&env, &usdc_id);
    let usdc_client = token::Client::new(&env, &usdc_id);

    // Deploy contracts
    let dob_token_id = env.register_contract_wasm(None, dob_token::WASM);
    let oracle_id = env.register_contract_wasm(None, dob_oracle::WASM);
    let primary_market_id = env.register_contract_wasm(None, dob_primary_market::WASM);

    // Initialize Oracle (NAV = 1.00, Risk = 10%)
    let oracle_client = dob_oracle::Client::new(&env, &oracle_id);
    oracle_client.initialize(&admin, &10_000_000, &1000);

    // Initialize DOB Token
    let token_client = dob_token::Client::new(&env, &dob_token_id);
    token_client.initialize(
        &admin,
        &primary_market_id,
        &SorobanString::from_str(&env, "Dob Token"),
        &SorobanString::from_str(&env, "DOB"),
        &7,
    );

    // Initialize Primary Market
    let market_client = dob_primary_market::Client::new(&env, &primary_market_id);
    market_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);

    // Mint USDC to Alice and market
    usdc_admin.mint(&alice, &10_000_0000000);
    usdc_admin.mint(&primary_market_id, &5_000_0000000);

    println!("\n✅ Setup complete");

    // TEST 1: Buy DOB tokens
    println!("\n=== TEST 1: Alice buys $1,000 of DOB ===");
    let alice_usdc_before = usdc_client.balance(&alice);
    let dob_received = market_client.buy(&alice, &1_000_0000000);
    let alice_dob = token_client.balance(&alice);

    assert_eq!(alice_dob, dob_received);
    assert_eq!(alice_dob, 990_0000000); // 1000 * 0.99 / 1.00 = 990
    println!("✅ Alice received {} DOB", alice_dob);

    // Verify operator revenue
    let operator_usdc = usdc_client.balance(&operator);
    assert_eq!(operator_usdc, 990_0000000); // 99% of 1000
    println!("✅ Operator received {} USDC", operator_usdc);

    // TEST 2: Get redemption quote
    println!("\n=== TEST 2: Quote for selling 500 DOB ===");
    let quote = market_client.quote_redemption(&500_0000000);
    println!("USDC out: {}, Penalty: {} bps", quote.usdc_out, quote.penalty_bps);

    // Penalty = 300 + 1000/10 = 400 bps = 4%
    assert_eq!(quote.penalty_bps, 400);
    // 500 * 1.00 * 0.96 = 480
    assert_eq!(quote.usdc_out, 480_0000000);
    println!("✅ Quote calculated correctly");

    // TEST 3: Sell DOB tokens
    println!("\n=== TEST 3: Alice sells 500 DOB ===");
    let usdc_received = market_client.sell(&alice, &500_0000000);
    let alice_dob_after = token_client.balance(&alice);

    assert_eq!(usdc_received, 480_0000000);
    assert_eq!(alice_dob_after, 490_0000000); // 990 - 500 = 490
    println!("✅ Alice sold 500 DOB for {} USDC", usdc_received);

    // TEST 4: Update oracle and verify new prices
    println!("\n=== TEST 4: Oracle update to NAV $1.20, Risk 5% ===");
    oracle_client.update(&12_000_000, &500);

    let new_nav = oracle_client.nav();
    let new_risk = oracle_client.default_risk();
    assert_eq!(new_nav, 12_000_000);
    assert_eq!(new_risk, 500);

    let new_quote = market_client.quote_redemption(&100_0000000);
    // Penalty = 300 + 500/10 = 350 bps = 3.5%
    assert_eq!(new_quote.penalty_bps, 350);
    // 100 * 1.20 * 0.965 = 115.8
    assert_eq!(new_quote.usdc_out, 115_8000000);
    println!("✅ Oracle updated and quotes reflect new prices");

    // TEST 5: Buy at new price
    println!("\n=== TEST 5: Alice buys $1,000 more at new NAV ===");
    let dob_received_2 = market_client.buy(&alice, &1_000_0000000);
    // 1000 * 0.99 / 1.20 = 825
    assert_eq!(dob_received_2, 825_0000000);
    println!("✅ Alice received {} DOB at higher NAV", dob_received_2);

    let final_balance = token_client.balance(&alice);
    assert_eq!(final_balance, 490_0000000 + 825_0000000);

    println!("\n=== ALL TESTS PASSED ===");
    println!("Final Alice DOB balance: {}", final_balance);
    println!("Final Alice USDC balance: {}", usdc_client.balance(&alice));
}

#[test]
fn test_penalty_tiers() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    // Setup
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let alice = Address::generate(&env);

    let usdc_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let usdc_admin = token::StellarAssetClient::new(&env, &usdc_id);

    let dob_token_id = env.register_contract_wasm(None, dob_token::WASM);
    let oracle_id = env.register_contract_wasm(None, dob_oracle::WASM);
    let primary_market_id = env.register_contract_wasm(None, dob_primary_market::WASM);

    let oracle_client = dob_oracle::Client::new(&env, &oracle_id);
    oracle_client.initialize(&admin, &10_000_000, &1000);

    let token_client = dob_token::Client::new(&env, &dob_token_id);
    token_client.initialize(
        &admin,
        &primary_market_id,
        &SorobanString::from_str(&env, "Dob Token"),
        &SorobanString::from_str(&env, "DOB"),
        &7,
    );

    let market_client = dob_primary_market::Client::new(&env, &primary_market_id);
    market_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);

    usdc_admin.mint(&alice, &10_000_0000000);
    usdc_admin.mint(&primary_market_id, &5_000_0000000);

    // Buy some DOB
    market_client.buy(&alice, &1_000_0000000);

    println!("\n=== PENALTY TIER TESTS ===");

    // Low risk
    oracle_client.update(&10_000_000, &500); // 5%
    let quote1 = market_client.quote_redemption(&100_0000000);
    assert_eq!(quote1.penalty_bps, 350); // 300 + 50 = 350
    println!("✅ Risk 5%: Penalty {} bps", quote1.penalty_bps);

    // Medium risk
    oracle_client.update(&10_000_000, &1500); // 15%
    let quote2 = market_client.quote_redemption(&100_0000000);
    assert_eq!(quote2.penalty_bps, 450); // 300 + 150 = 450
    println!("✅ Risk 15%: Penalty {} bps", quote2.penalty_bps);

    // High risk
    oracle_client.update(&10_000_000, &3000); // 30%
    let quote3 = market_client.quote_redemption(&100_0000000);
    assert_eq!(quote3.penalty_bps, 600); // 300 + 300 = 600
    println!("✅ Risk 30%: Penalty {} bps", quote3.penalty_bps);

    println!("\n=== PENALTY TESTS PASSED ===");
}

#[test]
fn test_multiple_users() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    // Setup
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let usdc_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let usdc_admin = token::StellarAssetClient::new(&env, &usdc_id);
    let usdc_client = token::Client::new(&env, &usdc_id);

    let dob_token_id = env.register_contract_wasm(None, dob_token::WASM);
    let oracle_id = env.register_contract_wasm(None, dob_oracle::WASM);
    let primary_market_id = env.register_contract_wasm(None, dob_primary_market::WASM);

    let oracle_client = dob_oracle::Client::new(&env, &oracle_id);
    oracle_client.initialize(&admin, &10_000_000, &1000);

    let token_client = dob_token::Client::new(&env, &dob_token_id);
    token_client.initialize(
        &admin,
        &primary_market_id,
        &SorobanString::from_str(&env, "Dob Token"),
        &SorobanString::from_str(&env, "DOB"),
        &7,
    );

    let market_client = dob_primary_market::Client::new(&env, &primary_market_id);
    market_client.initialize(&dob_token_id, &usdc_id, &oracle_id, &operator);

    usdc_admin.mint(&alice, &5_000_0000000);
    usdc_admin.mint(&bob, &3_000_0000000);
    usdc_admin.mint(&primary_market_id, &5_000_0000000);

    println!("\n=== MULTIPLE USERS TEST ===");

    // Alice buys
    println!("\n1. Alice buys $1,000 DOB");
    market_client.buy(&alice, &1_000_0000000);
    let alice_dob = token_client.balance(&alice);
    assert_eq!(alice_dob, 990_0000000);

    // Bob buys
    println!("2. Bob buys $500 DOB");
    market_client.buy(&bob, &500_0000000);
    let bob_dob = token_client.balance(&bob);
    assert_eq!(bob_dob, 495_0000000);

    // Check total supply
    let total_supply = token_client.total_supply();
    assert_eq!(total_supply, alice_dob + bob_dob);
    println!("✅ Total supply: {}", total_supply);

    // Alice sells half
    println!("\n3. Alice sells 250 DOB");
    market_client.sell(&alice, &250_0000000);
    let alice_remaining = token_client.balance(&alice);
    assert_eq!(alice_remaining, 740_0000000);

    // Bob sells all
    println!("4. Bob sells all {} DOB", bob_dob);
    market_client.sell(&bob, &bob_dob);
    let bob_remaining = token_client.balance(&bob);
    assert_eq!(bob_remaining, 0);

    println!("\n✅ Multiple users test passed");
    println!("Alice remaining: {}", alice_remaining);
    println!("Bob remaining: {}", bob_remaining);
}
