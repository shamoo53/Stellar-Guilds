use soroban_sdk::{contracttype, Address, Env, Vec};
use crate::payment::types::{PaymentPool, Recipient, DistributionStatus};

/// Storage key for the next pool ID counter
#[contracttype]
pub enum PaymentStorageKey {
    NextPoolId,
    Pool(u64),
    Recipients(u64),
}

/// Initialize payment distribution storage
pub fn initialize_payment_storage(env: &Env) {
    if !env.storage().persistent().has(&PaymentStorageKey::NextPoolId) {
        env.storage().persistent().set(&PaymentStorageKey::NextPoolId, &1u64);
    }
}

/// Get the next available pool ID and increment the counter
pub fn get_next_pool_id(env: &Env) -> u64 {
    let mut next_id: u64 = env.storage().persistent().get(&PaymentStorageKey::NextPoolId).unwrap_or(1);
    let current_id = next_id;
    next_id += 1;
    env.storage().persistent().set(&PaymentStorageKey::NextPoolId, &next_id);
    current_id
}

/// Store a payment pool
pub fn store_payment_pool(env: &Env, pool: &PaymentPool) {
    env.storage().persistent().set(&PaymentStorageKey::Pool(pool.id), pool);
}

/// Get a payment pool by ID
pub fn get_payment_pool(env: &Env, pool_id: u64) -> Option<PaymentPool> {
    env.storage().persistent().get(&PaymentStorageKey::Pool(pool_id))
}

/// Check if a pool exists
pub fn pool_exists(env: &Env, pool_id: u64) -> bool {
    env.storage().persistent().has(&PaymentStorageKey::Pool(pool_id))
}

/// Update pool status
pub fn update_pool_status(env: &Env, pool_id: u64, status: DistributionStatus) {
    if let Some(mut pool) = get_payment_pool(env, pool_id) {
        pool.status = status;
        store_payment_pool(env, &pool);
    }
}

/// Add a recipient to a pool
pub fn add_recipient_to_pool(env: &Env, pool_id: u64, recipient: &Recipient) {
    let key = PaymentStorageKey::Recipients(pool_id);
    let mut recipients: Vec<Recipient> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    recipients.push_back(recipient.clone());
    env.storage().persistent().set(&key, &recipients);
}

/// Get all recipients for a pool
pub fn get_pool_recipients(env: &Env, pool_id: u64) -> Vec<Recipient> {
    let key = PaymentStorageKey::Recipients(pool_id);
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

/// Check if a recipient already exists in a pool
pub fn recipient_exists_in_pool(env: &Env, pool_id: u64, address: &Address) -> bool {
    let recipients = get_pool_recipients(env, pool_id);
    for recipient in recipients.iter() {
        if &recipient.address == address {
            return true;
        }
    }
    false
}

/// Remove all recipients from a pool (used when cancelling)
pub fn clear_pool_recipients(env: &Env, pool_id: u64) {
    let key = PaymentStorageKey::Recipients(pool_id);
    env.storage().persistent().remove(&key);
}

/// Get total number of pools created
pub fn get_total_pools(env: &Env) -> u64 {
    let next_id: u64 = env.storage().persistent().get(&PaymentStorageKey::NextPoolId).unwrap_or(1);
    next_id.saturating_sub(1)
}