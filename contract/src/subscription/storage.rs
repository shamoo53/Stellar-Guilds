use crate::subscription::types::{
    BillingCycle, MembershipTier, RetryConfig, RevenueRecord, Subscription, SubscriptionPlan,
};
use soroban_sdk::{contracttype, Address, Env, Map, Vec};

/// Storage keys for subscription data
#[contracttype]
pub enum SubscriptionStorageKey {
    /// Next plan ID counter
    NextPlanId,
    /// Next subscription ID counter
    NextSubscriptionId,
    /// Next revenue record ID counter
    NextRevenueRecordId,
    /// Plan storage by ID
    Plan(u64),
    /// Subscription storage by ID
    Subscription(u64),
    /// Revenue record storage by ID
    RevenueRecord(u64),
    /// User subscriptions index: (subscriber, guild_id) -> subscription_id
    UserSubscription(Address, u64),
    /// Guild plans index: guild_id -> Vec<plan_id>
    GuildPlans(u64),
    /// Active subscriptions list
    ActiveSubscriptions,
    /// Revenue records index: (guild_id, period_start) -> Vec<record_id>
    GuildRevenue(u64, u64),
    /// Retry configuration
    RetryConfig,
}

/// Initialize subscription storage
pub fn initialize_subscription_storage(env: &Env) {
    let storage = env.storage().persistent();

    if !storage.has(&SubscriptionStorageKey::NextPlanId) {
        storage.set(&SubscriptionStorageKey::NextPlanId, &1u64);
    }

    if !storage.has(&SubscriptionStorageKey::NextSubscriptionId) {
        storage.set(&SubscriptionStorageKey::NextSubscriptionId, &1u64);
    }

    if !storage.has(&SubscriptionStorageKey::NextRevenueRecordId) {
        storage.set(&SubscriptionStorageKey::NextRevenueRecordId, &1u64);
    }

    if !storage.has(&SubscriptionStorageKey::RetryConfig) {
        storage.set(
            &SubscriptionStorageKey::RetryConfig,
            &RetryConfig::default(),
        );
    }
}

/// Get the next plan ID and increment counter
pub fn get_next_plan_id(env: &Env) -> u64 {
    let key = SubscriptionStorageKey::NextPlanId;
    let current: u64 = env
        .storage()
        .persistent()
        .get(&key)
        .expect("NextPlanId not initialized");
    let next = current + 1;
    env.storage().persistent().set(&key, &next);
    current
}

/// Get the next subscription ID and increment counter
pub fn get_next_subscription_id(env: &Env) -> u64 {
    let key = SubscriptionStorageKey::NextSubscriptionId;
    let current: u64 = env
        .storage()
        .persistent()
        .get(&key)
        .expect("NextSubscriptionId not initialized");
    let next = current + 1;
    env.storage().persistent().set(&key, &next);
    current
}

/// Get the next revenue record ID and increment counter
pub fn get_next_revenue_record_id(env: &Env) -> u64 {
    let key = SubscriptionStorageKey::NextRevenueRecordId;
    let current: u64 = env
        .storage()
        .persistent()
        .get(&key)
        .expect("NextRevenueRecordId not initialized");
    let next = current + 1;
    env.storage().persistent().set(&key, &next);
    current
}

/// Store a subscription plan
pub fn store_plan(env: &Env, plan: &SubscriptionPlan) {
    env.storage()
        .persistent()
        .set(&SubscriptionStorageKey::Plan(plan.id), plan);
}

/// Get a subscription plan by ID
pub fn get_plan(env: &Env, plan_id: u64) -> Option<SubscriptionPlan> {
    env.storage()
        .persistent()
        .get(&SubscriptionStorageKey::Plan(plan_id))
}

/// Store a subscription
pub fn store_subscription(env: &Env, subscription: &Subscription) {
    env.storage().persistent().set(
        &SubscriptionStorageKey::Subscription(subscription.id),
        subscription,
    );
}

/// Get a subscription by ID
pub fn get_subscription(env: &Env, subscription_id: u64) -> Option<Subscription> {
    env.storage()
        .persistent()
        .get(&SubscriptionStorageKey::Subscription(subscription_id))
}

/// Get user's subscription for a specific guild
pub fn get_user_subscription(
    env: &Env,
    subscriber: &Address,
    guild_id: u64,
) -> Option<Subscription> {
    env.storage()
        .persistent()
        .get(&SubscriptionStorageKey::UserSubscription(
            subscriber.clone(),
            guild_id,
        ))
        .and_then(|sub_id: u64| get_subscription(env, sub_id))
}

/// Store user subscription index
pub fn store_user_subscription(
    env: &Env,
    subscriber: &Address,
    guild_id: u64,
    subscription_id: u64,
) {
    env.storage().persistent().set(
        &SubscriptionStorageKey::UserSubscription(subscriber.clone(), guild_id),
        &subscription_id,
    );
}

/// Add plan to guild's plan list
pub fn add_plan_to_guild(env: &Env, guild_id: u64, plan_id: u64) {
    let key = SubscriptionStorageKey::GuildPlans(guild_id);
    let mut plans: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));

    // Check if already exists
    if !plans.iter().any(|p| p == plan_id) {
        plans.push_back(plan_id);
        env.storage().persistent().set(&key, &plans);
    }
}

/// Get all plans for a guild
pub fn get_guild_plans(env: &Env, guild_id: u64) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&SubscriptionStorageKey::GuildPlans(guild_id))
        .unwrap_or(Vec::new(env))
}

/// Get all active subscription IDs
pub fn get_active_subscriptions(env: &Env) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&SubscriptionStorageKey::ActiveSubscriptions)
        .unwrap_or(Vec::new(env))
}

/// Add subscription to active list
pub fn add_active_subscription(env: &Env, subscription_id: u64) {
    let key = SubscriptionStorageKey::ActiveSubscriptions;
    let mut active: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));

    if !active.iter().any(|id| id == subscription_id) {
        active.push_back(subscription_id);
        env.storage().persistent().set(&key, &active);
    }
}

/// Remove subscription from active list
pub fn remove_active_subscription(env: &Env, subscription_id: u64) {
    let key = SubscriptionStorageKey::ActiveSubscriptions;
    let active: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));

    let mut new_active = Vec::new(env);
    for id in active.iter() {
        if id != subscription_id {
            new_active.push_back(id);
        }
    }

    env.storage().persistent().set(&key, &new_active);
}

/// Store a revenue record
pub fn store_revenue_record(env: &Env, record: &RevenueRecord) {
    env.storage()
        .persistent()
        .set(&SubscriptionStorageKey::RevenueRecord(record.id), record);
}

/// Get a revenue record by ID
pub fn get_revenue_record(env: &Env, record_id: u64) -> Option<RevenueRecord> {
    env.storage()
        .persistent()
        .get(&SubscriptionStorageKey::RevenueRecord(record_id))
}

/// Add revenue record to guild's revenue index
pub fn add_guild_revenue(env: &Env, guild_id: u64, period_start: u64, record_id: u64) {
    let key = SubscriptionStorageKey::GuildRevenue(guild_id, period_start);
    let mut records: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));

    records.push_back(record_id);
    env.storage().persistent().set(&key, &records);
}

/// Get revenue records for a guild in a period
pub fn get_guild_revenue_records(
    env: &Env,
    guild_id: u64,
    period_start: u64,
) -> Vec<RevenueRecord> {
    let key = SubscriptionStorageKey::GuildRevenue(guild_id, period_start);
    let record_ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));

    let mut records = Vec::new(env);
    for id in record_ids.iter() {
        if let Some(record) = get_revenue_record(env, id) {
            records.push_back(record);
        }
    }
    records
}

/// Get retry configuration
pub fn get_retry_config(env: &Env) -> RetryConfig {
    env.storage()
        .persistent()
        .get(&SubscriptionStorageKey::RetryConfig)
        .unwrap_or_default()
}

/// Update retry configuration
pub fn set_retry_config(env: &Env, config: &RetryConfig) {
    env.storage()
        .persistent()
        .set(&SubscriptionStorageKey::RetryConfig, config);
}

/// Get all plans (for platform-wide queries)
pub fn get_all_plans(env: &Env, limit: u32) -> Vec<SubscriptionPlan> {
    let next_id = env
        .storage()
        .persistent()
        .get::<SubscriptionStorageKey, u64>(&SubscriptionStorageKey::NextPlanId)
        .unwrap_or(1);

    let mut plans = Vec::new(env);
    let mut count = 0u32;

    for id in 1..next_id {
        if count >= limit {
            break;
        }
        if let Some(plan) = get_plan(env, id) {
            plans.push_back(plan);
            count += 1;
        }
    }

    plans
}

/// Get subscriptions by plan ID
pub fn get_subscriptions_by_plan(env: &Env, plan_id: u64, limit: u32) -> Vec<Subscription> {
    let next_id = env
        .storage()
        .persistent()
        .get::<SubscriptionStorageKey, u64>(&SubscriptionStorageKey::NextSubscriptionId)
        .unwrap_or(1);

    let mut subscriptions = Vec::new(env);
    let mut count = 0u32;

    for id in 1..next_id {
        if count >= limit {
            break;
        }
        if let Some(sub) = get_subscription(env, id) {
            if sub.plan_id == plan_id {
                subscriptions.push_back(sub);
                count += 1;
            }
        }
    }

    subscriptions
}
