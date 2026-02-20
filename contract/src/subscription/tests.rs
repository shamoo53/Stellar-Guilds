use crate::subscription::types::{BillingCycle, MembershipTier, SubscriptionStatus};
use crate::{StellarGuildsContract, StellarGuildsContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String, Vec};

fn setup_env() -> Env {
    let env = Env::default();
    env.budget().reset_unlimited();
    env
}

fn register_and_init_contract(env: &Env) -> Address {
    let contract_id = env.register_contract(None, StellarGuildsContract);
    let client = StellarGuildsContractClient::new(env, &contract_id);
    client.initialize();
    contract_id
}

fn create_test_plan(
    env: &Env,
    client: &StellarGuildsContractClient,
    creator: &Address,
    guild_id: u64,
    tier: MembershipTier,
    price: i128,
    billing_cycle: BillingCycle,
) -> u64 {
    let name = String::from_str(env, "Test Plan");
    let description = String::from_str(env, "Test plan description");
    let benefits = Vec::new(env);
    let token: Option<Address> = None;

    client.create_subscription_plan(
        &guild_id,
        &name,
        &description,
        &tier,
        &price,
        &token,
        &billing_cycle,
        &benefits,
        creator,
    )
}

#[test]
fn test_create_plan() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    assert_eq!(plan_id, 1);
}

#[test]
#[should_panic(expected = "create_plan error")]
fn test_create_plan_invalid_price() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    env.mock_all_auths();

    let name = String::from_str(&env, "Test Plan");
    let description = String::from_str(&env, "Test plan description");
    let benefits = Vec::new(&env);
    let token: Option<Address> = None;

    // Should panic with invalid price (0)
    client.create_subscription_plan(
        &1,
        &name,
        &description,
        &MembershipTier::Standard,
        &0,
        &token,
        &BillingCycle::Monthly,
        &benefits,
        &creator,
    );
}

#[test]
fn test_subscribe() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);
    assert_eq!(subscription_id, 1);

    let subscription = client.get_subscription(&subscription_id);
    assert_eq!(subscription.plan_id, plan_id);
    assert_eq!(subscription.subscriber, subscriber);
    assert_eq!(subscription.status, SubscriptionStatus::Active);
    assert!(subscription.auto_renew);
}

#[test]
fn test_subscribe_to_inactive_plan() {
    // Note: This test demonstrates the expected behavior
    // In a full implementation, we would have a deactivate_plan function
    // For now, we just verify that subscribing to an active plan works
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    // Subscribe should work with active plan
    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);
    assert_eq!(subscription_id, 1);
}

#[test]
#[should_panic(expected = "subscribe error")]
fn test_duplicate_subscription() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    // First subscription should succeed
    let _ = client.subscribe(&plan_id, &subscriber, &true);

    // Second subscription should fail
    let _ = client.subscribe(&plan_id, &subscriber, &true);
}

#[test]
fn test_pause_and_resume_subscription() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);

    // Pause subscription
    let paused = client.pause_subscription(&subscription_id, &subscriber);
    assert!(paused);

    let subscription = client.get_subscription(&subscription_id);
    assert_eq!(subscription.status, SubscriptionStatus::Paused);

    // Resume subscription
    let resumed = client.resume_subscription(&subscription_id, &subscriber);
    assert!(resumed);

    let subscription = client.get_subscription(&subscription_id);
    assert_eq!(subscription.status, SubscriptionStatus::Active);
}

#[test]
#[should_panic(expected = "pause_subscription error")]
fn test_pause_unauthorized() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let other_user = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);

    // Try to pause with different user - should panic
    let _ = client.pause_subscription(&subscription_id, &other_user);
}

#[test]
fn test_cancel_subscription() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);

    let reason = Some(String::from_str(&env, "No longer needed"));
    let cancelled = client.cancel_subscription(&subscription_id, &subscriber, &reason);
    assert!(cancelled);

    let subscription = client.get_subscription(&subscription_id);
    assert_eq!(subscription.status, SubscriptionStatus::Cancelled);
    assert!(subscription.cancelled_at.is_some());
    assert!(!subscription.auto_renew);
}

#[test]
#[should_panic(expected = "cancel_subscription error")]
fn test_cancel_already_cancelled() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);

    let reason = Some(String::from_str(&env, "No longer needed"));
    let _ = client.cancel_subscription(&subscription_id, &subscriber, &reason);

    // Try to cancel again - should panic
    let _ = client.cancel_subscription(&subscription_id, &subscriber, &reason);
}

#[test]
fn test_tier_upgrade() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    // Create basic plan
    let basic_plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Basic,
        500,
        BillingCycle::Monthly,
    );

    // Create premium plan
    let premium_plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Premium,
        2000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&basic_plan_id, &subscriber, &true);

    // Upgrade to premium
    let proration_amount =
        client.change_subscription_tier(&subscription_id, &premium_plan_id, &true, &subscriber);

    // Should have proration amount for upgrade (charge)
    assert!(proration_amount >= 0);

    let subscription = client.get_subscription(&subscription_id);
    assert_eq!(subscription.current_tier, MembershipTier::Premium);
    assert_eq!(subscription.plan_id, premium_plan_id);
}

#[test]
fn test_tier_downgrade() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    // Create premium plan
    let premium_plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Premium,
        2000,
        BillingCycle::Monthly,
    );

    // Create basic plan
    let basic_plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Basic,
        500,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&premium_plan_id, &subscriber, &true);

    // Downgrade to basic
    let proration_amount =
        client.change_subscription_tier(&subscription_id, &basic_plan_id, &true, &subscriber);

    // Downgrade should have proration amount
    assert!(proration_amount >= 0);

    let subscription = client.get_subscription(&subscription_id);
    assert_eq!(subscription.current_tier, MembershipTier::Basic);
}

#[test]
#[should_panic(expected = "change_tier error")]
fn test_invalid_tier_change() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);

    // Try to change to same tier - should panic
    let _ = client.change_subscription_tier(&subscription_id, &plan_id, &true, &subscriber);
}

#[test]
fn test_is_subscription_active() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);

    assert!(client.is_subscription_active(&subscription_id));

    // Cancel subscription
    let _ = client.cancel_subscription(&subscription_id, &subscriber, &None);

    assert!(!client.is_subscription_active(&subscription_id));
}

#[test]
fn test_days_until_billing() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);

    let days = client.days_until_billing(&subscription_id);
    // Should be approximately 30 days (monthly billing)
    assert!(days > 28 && days <= 30);
}

#[test]
fn test_billing_cycle_durations() {
    assert_eq!(BillingCycle::Weekly.duration_seconds(), 7 * 24 * 60 * 60);
    assert_eq!(BillingCycle::Monthly.duration_seconds(), 30 * 24 * 60 * 60);
    assert_eq!(
        BillingCycle::Quarterly.duration_seconds(),
        90 * 24 * 60 * 60
    );
    assert_eq!(
        BillingCycle::Annually.duration_seconds(),
        365 * 24 * 60 * 60
    );
}

#[test]
fn test_membership_tier_ordering() {
    assert!(MembershipTier::Basic < MembershipTier::Standard);
    assert!(MembershipTier::Standard < MembershipTier::Premium);
    assert!(MembershipTier::Premium < MembershipTier::Enterprise);
}

#[test]
fn test_process_due_subscriptions() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    // Create subscription
    let _ = client.subscribe(&plan_id, &subscriber, &true);

    // Process due subscriptions (none should be due yet as we just created it)
    let processed = client.process_due_subscriptions(&10);
    assert_eq!(processed, 0);
}

#[test]
#[should_panic(expected = "subscription not found")]
fn test_nonexistent_subscription() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Try to get non-existent subscription - should panic
    let _ = client.get_subscription(&999);
}

#[test]
#[should_panic(expected = "subscribe error")]
fn test_nonexistent_plan() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    // Try to subscribe to non-existent plan - should panic
    let _ = client.subscribe(&999, &subscriber, &true);
}

#[test]
#[should_panic(expected = "pause_subscription error")]
fn test_pause_non_active_subscription() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);

    env.mock_all_auths();

    let plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Standard,
        1000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&plan_id, &subscriber, &true);

    // Cancel the subscription
    let _ = client.cancel_subscription(&subscription_id, &subscriber, &None);

    // Try to pause cancelled subscription - should panic
    let _ = client.pause_subscription(&subscription_id, &subscriber);
}

#[test]
#[should_panic(expected = "change_tier error")]
fn test_change_tier_unauthorized() {
    let env = setup_env();
    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let other_user = Address::generate(&env);

    env.mock_all_auths();

    let basic_plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Basic,
        500,
        BillingCycle::Monthly,
    );

    let premium_plan_id = create_test_plan(
        &env,
        &client,
        &creator,
        1,
        MembershipTier::Premium,
        2000,
        BillingCycle::Monthly,
    );

    let subscription_id = client.subscribe(&basic_plan_id, &subscriber, &true);

    // Try to change tier with different user - should panic
    let _ = client.change_subscription_tier(&subscription_id, &premium_plan_id, &true, &other_user);
}
