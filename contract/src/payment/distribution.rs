use soroban_sdk::{Address, Env, String, Vec, contracterror};
use crate::payment::types::{
    PaymentPool, Recipient, DistributionRule, DistributionStatus,
    PaymentPoolCreatedEvent, RecipientAddedEvent, DistributionExecutedEvent,
    DistributionFailedEvent, PoolCancelledEvent,
};
use crate::payment::storage::{
    get_payment_pool, store_payment_pool, get_pool_recipients, add_recipient_to_pool,
    recipient_exists_in_pool, update_pool_status, clear_pool_recipients, get_next_pool_id,
};

/// Error types for payment distribution operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PaymentError {
    PoolNotFound = 1,
    PoolNotPending = 2,
    Unauthorized = 3,
    InvalidShare = 4,
    DuplicateRecipient = 5,
    SharesNot100Percent = 6,
    NoRecipients = 7,
    InsufficientBalance = 8,
    TransferFailed = 9,
    ArithmeticOverflow = 10,
    InvalidAmount = 11,
}

/// Minimum share amount to avoid dust issues
const MIN_SHARE_AMOUNT: i128 = 1;

/// Create a new payment pool
///
/// # Arguments
/// * `env` - The contract environment
/// * `amount` - Total amount to distribute
/// * `token` - Token contract address (None for native XLM)
/// * `rule` - Distribution rule type
/// * `creator` - Address creating the pool
///
/// # Returns
/// The ID of the newly created pool
pub fn create_payment_pool(
    env: &Env,
    amount: i128,
    token: Option<Address>,
    rule: DistributionRule,
    creator: Address,
) -> Result<u64, PaymentError> {
    // Validate amount
    if amount <= 0 {
        return Err(PaymentError::InvalidAmount);
    }

    let pool_id = get_next_pool_id(env);

    let pool = PaymentPool {
        id: pool_id,
        total_amount: amount,
        token: token.clone(),
        status: DistributionStatus::Pending,
        created_by: creator.clone(),
        rule: rule.clone(),
        created_at: env.ledger().timestamp(),
    };

    store_payment_pool(env, &pool);

    // Emit event
    let event = PaymentPoolCreatedEvent {
        pool_id,
        creator,
        total_amount: amount,
        token,
        rule,
    };
    env.events().publish(("PaymentPoolCreated",), event);

    Ok(pool_id)
}

/// Add a recipient to a payment pool
///
/// # Arguments
/// * `env` - The contract environment
/// * `pool_id` - ID of the pool
/// * `address` - Recipient address
/// * `share` - Share percentage (0-100) or weight
/// * `caller` - Address making the request
///
/// # Returns
/// true if successful
pub fn add_recipient(
    env: &Env,
    pool_id: u64,
    address: Address,
    share: u32,
    caller: Address,
) -> Result<bool, PaymentError> {
    let pool = get_payment_pool(env, pool_id).ok_or(PaymentError::PoolNotFound)?;

    // Only pool creator can add recipients
    if pool.created_by != caller {
        return Err(PaymentError::Unauthorized);
    }

    // Pool must be pending
    if pool.status != DistributionStatus::Pending {
        return Err(PaymentError::PoolNotPending);
    }

    // Check for duplicate recipient
    if recipient_exists_in_pool(env, pool_id, &address) {
        return Err(PaymentError::DuplicateRecipient);
    }

    // Validate share based on rule
    match pool.rule {
        DistributionRule::Percentage => {
            if share == 0 || share > 100 {
                return Err(PaymentError::InvalidShare);
            }
        }
        DistributionRule::EqualSplit => {
            // For equal split, share is ignored but must be > 0
            if share == 0 {
                return Err(PaymentError::InvalidShare);
            }
        }
        DistributionRule::Weighted => {
            if share == 0 {
                return Err(PaymentError::InvalidShare);
            }
        }
    }

    let recipient = Recipient { address: address.clone(), share };
    add_recipient_to_pool(env, pool_id, &recipient);

    // Emit event
    let event = RecipientAddedEvent { pool_id, recipient: address, share };
    env.events().publish(("RecipientAdded",), event);

    Ok(true)
}

/// Validate that distribution rules are met
///
/// # Arguments
/// * `env` - The contract environment
/// * `pool_id` - ID of the pool
///
/// # Returns
/// true if validation passes
pub fn validate_distribution(env: &Env, pool_id: u64) -> Result<bool, PaymentError> {
    let pool = get_payment_pool(env, pool_id).ok_or(PaymentError::PoolNotFound)?;
    let recipients = get_pool_recipients(env, pool_id);

    if recipients.is_empty() {
        return Err(PaymentError::NoRecipients);
    }

    match pool.rule {
        DistributionRule::Percentage => {
            let total_percentage: u32 = recipients.iter().map(|r| r.share).sum();
            if total_percentage != 100 {
                return Err(PaymentError::SharesNot100Percent);
            }
        }
        DistributionRule::EqualSplit | DistributionRule::Weighted => {
            // For equal split and weighted, we just need at least one recipient
            // Validation happens during execution for amount calculations
        }
    }

    Ok(true)
}

/// Calculate the amount a recipient should receive
///
/// # Arguments
/// * `pool` - The payment pool
/// * `recipient` - The recipient
/// * `total_recipients` - Total number of recipients
/// * `total_weight` - Total weight for weighted distribution
///
/// # Returns
/// The calculated amount
fn calculate_recipient_amount(
    pool: &PaymentPool,
    recipient: &Recipient,
    total_recipients: u32,
    total_weight: Option<u32>,
) -> Result<i128, PaymentError> {
    match pool.rule {
        DistributionRule::Percentage => {
            // amount = total_amount * share / 100
            let amount = (pool.total_amount as i128)
                .checked_mul(recipient.share as i128)
                .ok_or(PaymentError::ArithmeticOverflow)?
                .checked_div(100)
                .ok_or(PaymentError::ArithmeticOverflow)?;
            Ok(amount)
        }
        DistributionRule::EqualSplit => {
            // amount = total_amount / total_recipients
            let amount = pool.total_amount
                .checked_div(total_recipients as i128)
                .ok_or(PaymentError::ArithmeticOverflow)?;
            Ok(amount)
        }
        DistributionRule::Weighted => {
            if let Some(total_w) = total_weight {
                // amount = total_amount * recipient_weight / total_weight
                let amount = (pool.total_amount as i128)
                    .checked_mul(recipient.share as i128)
                    .ok_or(PaymentError::ArithmeticOverflow)?
                    .checked_div(total_w as i128)
                    .ok_or(PaymentError::ArithmeticOverflow)?;
                Ok(amount)
            } else {
                Err(PaymentError::ArithmeticOverflow)
            }
        }
    }
}

/// Execute the distribution for a payment pool
///
/// # Arguments
/// * `env` - The contract environment
/// * `pool_id` - ID of the pool
/// * `caller` - Address making the request
///
/// # Returns
/// true if successful
pub fn execute_distribution(env: &Env, pool_id: u64, caller: Address) -> Result<bool, PaymentError> {
    let mut pool = get_payment_pool(env, pool_id).ok_or(PaymentError::PoolNotFound)?;

    // Only pool creator can execute
    if pool.created_by != caller {
        return Err(PaymentError::Unauthorized);
    }

    // Pool must be pending
    if pool.status != DistributionStatus::Pending {
        return Err(PaymentError::PoolNotPending);
    }

    // Validate distribution
    validate_distribution(env, pool_id)?;

    let recipients = get_pool_recipients(env, pool_id);
    let total_recipients = recipients.len() as u32;

    // Calculate total weight for weighted distribution
    let total_weight = if pool.rule == DistributionRule::Weighted {
        Some(recipients.iter().map(|r| r.share).sum())
    } else {
        None
    };

    // Check if contract has sufficient balance
    let contract_balance = if let Some(token_addr) = &pool.token {
        // Custom token balance
        let token_client = soroban_sdk::token::Client::new(env, token_addr);
        token_client.balance(&env.current_contract_address())
    } else {
        // Native XLM balance - for now, assume sufficient (would need ledger integration)
        // TODO: Implement native XLM balance checking
        i128::MAX
    };

    if contract_balance < pool.total_amount {
        update_pool_status(env, pool_id, DistributionStatus::Failed);
        let event = DistributionFailedEvent {
            pool_id,
            reason: String::from_str(env, "Insufficient contract balance"),
        };
        env.events().publish(("DistributionFailed",), event);
        return Err(PaymentError::InsufficientBalance);
    }

    // Execute transfers atomically
    let mut total_distributed = 0i128;

    for recipient in recipients.iter() {
        let amount = calculate_recipient_amount(&pool, &recipient, total_recipients, total_weight)?;

        // Skip dust amounts
        if amount < MIN_SHARE_AMOUNT {
            continue;
        }

        // Execute transfer
        if let Some(token_addr) = &pool.token {
            // Transfer custom token
            let token_client = soroban_sdk::token::Client::new(env, token_addr);
            token_client.transfer(&env.current_contract_address(), &recipient.address, &amount);
        } else {
            // Native XLM transfer - TODO: implement
            // For now, skip
        }

        total_distributed = total_distributed.checked_add(amount).ok_or(PaymentError::ArithmeticOverflow)?;
    }

    // Update pool status
    pool.status = DistributionStatus::Executed;
    store_payment_pool(env, &pool);

    // Emit success event
    let event = DistributionExecutedEvent {
        pool_id,
        total_recipients: total_recipients as u32,
        total_distributed,
    };
    env.events().publish(("DistributionExecuted",), event);

    Ok(true)
}

/// Get the calculated amount for a specific recipient
///
/// # Arguments
/// * `env` - The contract environment
/// * `pool_id` - ID of the pool
/// * `address` - Recipient address
///
/// # Returns
/// The amount the recipient should receive
pub fn get_recipient_amount(env: &Env, pool_id: u64, address: Address) -> Result<i128, PaymentError> {
    let pool = get_payment_pool(env, pool_id).ok_or(PaymentError::PoolNotFound)?;
    let recipients = get_pool_recipients(env, pool_id);

    // Find the recipient
    let recipient = recipients.iter().find(|r| r.address == address).ok_or(PaymentError::PoolNotFound)?;

    let total_recipients = recipients.len() as u32;
    let total_weight = if pool.rule == DistributionRule::Weighted {
        Some(recipients.iter().map(|r| r.share).sum())
    } else {
        None
    };

    calculate_recipient_amount(&pool, &recipient, total_recipients, total_weight)
}

/// Cancel a payment pool
///
/// # Arguments
/// * `env` - The contract environment
/// * `pool_id` - ID of the pool
/// * `caller` - Address making the request
///
/// # Returns
/// true if successful
pub fn cancel_distribution(env: &Env, pool_id: u64, caller: Address) -> Result<bool, PaymentError> {
    let pool = get_payment_pool(env, pool_id).ok_or(PaymentError::PoolNotFound)?;

    // Only pool creator can cancel
    if pool.created_by != caller {
        return Err(PaymentError::Unauthorized);
    }

    // Can only cancel pending pools
    if pool.status != DistributionStatus::Pending {
        return Err(PaymentError::PoolNotPending);
    }

    update_pool_status(env, pool_id, DistributionStatus::Cancelled);
    clear_pool_recipients(env, pool_id);

    // Emit event
    let event = PoolCancelledEvent {
        pool_id,
        cancelled_by: caller,
    };
    env.events().publish(("PoolCancelled",), event);

    Ok(true)
}

/// Get the status of a payment pool
///
/// # Arguments
/// * `env` - The contract environment
/// * `pool_id` - ID of the pool
///
/// # Returns
/// The distribution status
pub fn get_pool_status(env: &Env, pool_id: u64) -> Result<DistributionStatus, PaymentError> {
    let pool = get_payment_pool(env, pool_id).ok_or(PaymentError::PoolNotFound)?;
    Ok(pool.status)
}

/// Batch distribute multiple pools
///
/// # Arguments
/// * `env` - The contract environment
/// * `pool_ids` - List of pool IDs to distribute
/// * `caller` - Address making the request
///
/// # Returns
/// Vector of results (true for success, false for failure)
pub fn batch_distribute(env: &Env, pool_ids: Vec<u64>, caller: Address) -> Vec<bool> {
    let mut results = Vec::new(env);

    for pool_id in pool_ids.iter() {
        let result = execute_distribution(env, pool_id, caller.clone()).is_ok();
        results.push_back(result);
    }

    results
}