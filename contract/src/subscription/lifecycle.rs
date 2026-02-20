use crate::subscription::storage::{
    add_active_subscription, add_guild_revenue, add_plan_to_guild, get_next_plan_id,
    get_next_revenue_record_id, get_next_subscription_id, get_plan, get_retry_config,
    get_subscription, get_user_subscription, remove_active_subscription, store_plan,
    store_revenue_record, store_subscription, store_user_subscription,
};
use crate::subscription::types::{
    GracePeriodStartedEvent, MembershipTier, PaymentProcessedEvent, PlanCreatedEvent,
    ProrationResult, RetryConfig, RevenueRecord, RevenueRecordedEvent, Subscription,
    SubscriptionCancelledEvent, SubscriptionChange, SubscriptionCreatedEvent, SubscriptionError,
    SubscriptionPlan, SubscriptionStatus, TierChangedEvent,
};
use soroban_sdk::{token, Address, Env, String, Vec};

/// Create a new subscription plan
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - Guild ID (0 for platform-wide plans)
/// * `name` - Plan name
/// * `description` - Plan description
/// * `tier` - Membership tier level
/// * `price` - Price amount
/// * `token` - Token address (None for native XLM)
/// * `billing_cycle` - Billing cycle type
/// * `benefits` - List of benefits
/// * `created_by` - Creator address
///
/// # Returns
/// The ID of the newly created plan
pub fn create_plan(
    env: &Env,
    guild_id: u64,
    name: String,
    description: String,
    tier: MembershipTier,
    price: i128,
    token: Option<Address>,
    billing_cycle: crate::subscription::types::BillingCycle,
    benefits: Vec<String>,
    created_by: Address,
) -> Result<u64, SubscriptionError> {
    // Validate price
    if price <= 0 {
        return Err(SubscriptionError::InvalidPrice);
    }

    let plan_id = get_next_plan_id(env);

    let plan = SubscriptionPlan {
        id: plan_id,
        guild_id,
        name: name.clone(),
        description,
        tier: tier.clone(),
        price,
        token: token.clone(),
        billing_cycle: billing_cycle.clone(),
        is_active: true,
        benefits,
        created_by: created_by.clone(),
        created_at: env.ledger().timestamp(),
    };

    store_plan(env, &plan);

    // Add to guild's plan list if guild-specific
    if guild_id > 0 {
        add_plan_to_guild(env, guild_id, plan_id);
    }

    // Emit event
    let event = PlanCreatedEvent {
        plan_id,
        guild_id,
        name,
        tier,
        price,
        billing_cycle,
    };
    env.events().publish(("PlanCreated",), event);

    Ok(plan_id)
}

/// Subscribe to a plan
///
/// # Arguments
/// * `env` - The contract environment
/// * `plan_id` - ID of the plan to subscribe to
/// * `subscriber` - Address subscribing
/// * `auto_renew` - Whether to auto-renew
///
/// # Returns
/// The ID of the newly created subscription
pub fn subscribe(
    env: &Env,
    plan_id: u64,
    subscriber: Address,
    auto_renew: bool,
) -> Result<u64, SubscriptionError> {
    let plan = get_plan(env, plan_id).ok_or(SubscriptionError::PlanNotFound)?;

    if !plan.is_active {
        return Err(SubscriptionError::PlanNotActive);
    }

    // Check if user already has an active subscription for this guild
    if let Some(existing) = get_user_subscription(env, &subscriber, plan.guild_id) {
        if existing.status == SubscriptionStatus::Active
            || existing.status == SubscriptionStatus::Paused
        {
            return Err(SubscriptionError::SubscriptionAlreadyExists);
        }
    }

    let subscription_id = get_next_subscription_id(env);
    let now = env.ledger().timestamp();
    let cycle_duration = plan.billing_cycle.duration_seconds();

    let subscription = Subscription {
        id: subscription_id,
        plan_id,
        subscriber: subscriber.clone(),
        status: SubscriptionStatus::Active,
        current_tier: plan.tier.clone(),
        started_at: now,
        ends_at: None,
        next_billing_at: now + cycle_duration,
        last_payment_at: None,
        last_payment_amount: None,
        failed_payment_count: 0,
        grace_period_ends_at: None,
        auto_renew,
        cancelled_at: None,
        cancellation_reason: None,
    };

    store_subscription(env, &subscription);
    store_user_subscription(env, &subscriber, plan.guild_id, subscription_id);
    add_active_subscription(env, subscription_id);

    // Emit event
    let event = SubscriptionCreatedEvent {
        subscription_id,
        plan_id,
        subscriber,
        tier: plan.tier,
        next_billing_at: subscription.next_billing_at,
    };
    env.events().publish(("SubscriptionCreated",), event);

    Ok(subscription_id)
}

/// Process a subscription payment
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
/// * `retry_attempt` - Current retry attempt (0 for first attempt)
///
/// # Returns
/// true if payment was successful
pub fn process_payment(
    env: &Env,
    subscription_id: u64,
    retry_attempt: u32,
) -> Result<bool, SubscriptionError> {
    let mut subscription =
        get_subscription(env, subscription_id).ok_or(SubscriptionError::SubscriptionNotFound)?;

    let plan = get_plan(env, subscription.plan_id).ok_or(SubscriptionError::PlanNotFound)?;

    // Only process active or grace period subscriptions
    if subscription.status != SubscriptionStatus::Active
        && subscription.status != SubscriptionStatus::GracePeriod
    {
        return Err(SubscriptionError::InvalidState);
    }

    let payment_result = execute_payment(env, &subscription.subscriber, plan.price, &plan.token);

    let now = env.ledger().timestamp();

    match payment_result {
        Ok(()) => {
            // Payment successful
            subscription.last_payment_at = Some(now);
            subscription.last_payment_amount = Some(plan.price);
            subscription.failed_payment_count = 0;
            subscription.grace_period_ends_at = None;
            subscription.status = SubscriptionStatus::Active;

            // Calculate next billing date
            let cycle_duration = plan.billing_cycle.duration_seconds();
            subscription.next_billing_at = now + cycle_duration;

            store_subscription(env, &subscription);

            // Record revenue
            record_revenue(
                env,
                plan.guild_id,
                subscription_id,
                subscription.subscriber.clone(),
                plan.price,
                plan.token.clone(),
                plan.billing_cycle.clone(),
                retry_attempt > 0,
                retry_attempt,
            );

            // Emit success event
            let event = PaymentProcessedEvent {
                subscription_id,
                amount: plan.price,
                success: true,
                retry_attempt,
            };
            env.events().publish(("PaymentProcessed",), event);

            Ok(true)
        }
        Err(_) => {
            // Payment failed
            subscription.failed_payment_count += 1;

            let retry_config = get_retry_config(env);

            // Check if max retries exceeded
            if subscription.failed_payment_count >= retry_config.max_retries {
                // Cancel subscription
                subscription.status = SubscriptionStatus::Cancelled;
                subscription.cancelled_at = Some(now);
                subscription.cancellation_reason =
                    Some(String::from_str(env, "Max payment retries exceeded"));
                remove_active_subscription(env, subscription_id);
            } else {
                // Enter or continue grace period
                subscription.status = SubscriptionStatus::GracePeriod;
                let grace_end = now + retry_config.grace_period_seconds;
                subscription.grace_period_ends_at = Some(grace_end);

                // Emit grace period event
                let event = GracePeriodStartedEvent {
                    subscription_id,
                    grace_period_ends_at: grace_end,
                    failed_payment_count: subscription.failed_payment_count,
                };
                env.events().publish(("GracePeriodStarted",), event);
            }

            store_subscription(env, &subscription);

            // Emit failure event
            let event = PaymentProcessedEvent {
                subscription_id,
                amount: plan.price,
                success: false,
                retry_attempt,
            };
            env.events().publish(("PaymentProcessed",), event);

            Err(SubscriptionError::PaymentFailed)
        }
    }
}

/// Execute the actual token transfer for payment
fn execute_payment(
    env: &Env,
    from: &Address,
    amount: i128,
    token: &Option<Address>,
) -> Result<(), ()> {
    from.require_auth();

    if let Some(token_addr) = token {
        // Transfer custom token
        let token_client = token::Client::new(env, token_addr);
        token_client.transfer(from, &env.current_contract_address(), &amount);
    } else {
        // For native XLM, we would need additional handling
        // In a real implementation, this would check and transfer XLM
        // For now, we assume the contract has a way to receive native tokens
    }

    Ok(())
}

/// Record revenue from a successful payment
fn record_revenue(
    env: &Env,
    guild_id: u64,
    subscription_id: u64,
    subscriber: Address,
    amount: i128,
    token: Option<Address>,
    billing_cycle: crate::subscription::types::BillingCycle,
    is_retry: bool,
    retry_attempt: u32,
) -> u64 {
    let record_id = get_next_revenue_record_id(env);
    let now = env.ledger().timestamp();

    // Calculate period start (beginning of the cycle this payment covers)
    let cycle_duration = billing_cycle.duration_seconds();
    let period_start = now - cycle_duration;

    let record = RevenueRecord {
        id: record_id,
        guild_id,
        plan_id: 0, // Will be filled from subscription
        subscription_id,
        subscriber,
        amount,
        token,
        paid_at: now,
        billing_cycle,
        is_retry,
        retry_attempt,
    };

    store_revenue_record(env, &record);
    add_guild_revenue(env, guild_id, period_start, record_id);

    // Emit revenue event
    let event = RevenueRecordedEvent {
        record_id,
        guild_id,
        subscription_id,
        amount,
        paid_at: now,
    };
    env.events().publish(("RevenueRecorded",), event);

    record_id
}

/// Pause a subscription
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
/// * `caller` - Address making the request
///
/// # Returns
/// true if successful
pub fn pause_subscription(
    env: &Env,
    subscription_id: u64,
    caller: Address,
) -> Result<bool, SubscriptionError> {
    let mut subscription =
        get_subscription(env, subscription_id).ok_or(SubscriptionError::SubscriptionNotFound)?;

    // Only subscriber can pause their own subscription
    if subscription.subscriber != caller {
        return Err(SubscriptionError::Unauthorized);
    }

    if subscription.status != SubscriptionStatus::Active {
        return Err(SubscriptionError::InvalidState);
    }

    subscription.status = SubscriptionStatus::Paused;
    store_subscription(env, &subscription);
    remove_active_subscription(env, subscription_id);

    Ok(true)
}

/// Resume a paused subscription
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
/// * `caller` - Address making the request
///
/// # Returns
/// true if successful
pub fn resume_subscription(
    env: &Env,
    subscription_id: u64,
    caller: Address,
) -> Result<bool, SubscriptionError> {
    let mut subscription =
        get_subscription(env, subscription_id).ok_or(SubscriptionError::SubscriptionNotFound)?;

    // Only subscriber can resume their own subscription
    if subscription.subscriber != caller {
        return Err(SubscriptionError::Unauthorized);
    }

    if subscription.status != SubscriptionStatus::Paused {
        return Err(SubscriptionError::InvalidState);
    }

    let plan = get_plan(env, subscription.plan_id).ok_or(SubscriptionError::PlanNotFound)?;

    subscription.status = SubscriptionStatus::Active;

    // Adjust next billing date to account for pause period
    let now = env.ledger().timestamp();
    let cycle_duration = plan.billing_cycle.duration_seconds();
    subscription.next_billing_at = now + cycle_duration;

    store_subscription(env, &subscription);
    add_active_subscription(env, subscription_id);

    Ok(true)
}

/// Cancel a subscription
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
/// * `caller` - Address making the request
/// * `reason` - Optional cancellation reason
///
/// # Returns
/// true if successful
pub fn cancel_subscription(
    env: &Env,
    subscription_id: u64,
    caller: Address,
    reason: Option<String>,
) -> Result<bool, SubscriptionError> {
    let mut subscription =
        get_subscription(env, subscription_id).ok_or(SubscriptionError::SubscriptionNotFound)?;

    // Only subscriber can cancel their own subscription
    if subscription.subscriber != caller {
        return Err(SubscriptionError::Unauthorized);
    }

    if subscription.status == SubscriptionStatus::Cancelled {
        return Err(SubscriptionError::AlreadyCancelled);
    }

    let now = env.ledger().timestamp();
    subscription.status = SubscriptionStatus::Cancelled;
    subscription.cancelled_at = Some(now);
    subscription.cancellation_reason = reason.clone();
    subscription.auto_renew = false;

    store_subscription(env, &subscription);
    remove_active_subscription(env, subscription_id);

    // Emit cancellation event
    let event = SubscriptionCancelledEvent {
        subscription_id,
        cancelled_by: caller,
        reason,
    };
    env.events().publish(("SubscriptionCancelled",), event);

    Ok(true)
}

/// Change subscription tier (upgrade/downgrade)
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
/// * `change` - Subscription change details
/// * `caller` - Address making the request
///
/// # Returns
/// Proration result if applicable
pub fn change_tier(
    env: &Env,
    subscription_id: u64,
    change: SubscriptionChange,
    caller: Address,
) -> Result<Option<ProrationResult>, SubscriptionError> {
    let mut subscription =
        get_subscription(env, subscription_id).ok_or(SubscriptionError::SubscriptionNotFound)?;

    // Only subscriber can change their tier
    if subscription.subscriber != caller {
        return Err(SubscriptionError::Unauthorized);
    }

    if subscription.status != SubscriptionStatus::Active
        && subscription.status != SubscriptionStatus::Paused
    {
        return Err(SubscriptionError::InvalidState);
    }

    let current_plan =
        get_plan(env, subscription.plan_id).ok_or(SubscriptionError::PlanNotFound)?;

    let new_plan = get_plan(env, change.new_plan_id).ok_or(SubscriptionError::PlanNotFound)?;

    // Validate tier change direction
    let is_upgrade = new_plan.tier > current_plan.tier;
    let is_downgrade = new_plan.tier < current_plan.tier;

    if !is_upgrade && !is_downgrade {
        return Err(SubscriptionError::InvalidTierChange);
    }

    let old_tier = subscription.current_tier.clone();
    subscription.current_tier = new_plan.tier.clone();
    subscription.plan_id = change.new_plan_id;

    let proration = if change.effective_immediately {
        // Calculate proration
        let now = env.ledger().timestamp();
        let cycle_duration = current_plan.billing_cycle.duration_seconds();
        let time_remaining = subscription.next_billing_at.saturating_sub(now);

        if time_remaining > 0 && cycle_duration > 0 {
            let days_remaining = time_remaining / (24 * 60 * 60);
            let total_days = cycle_duration / (24 * 60 * 60);

            // Calculate prorated amounts
            let current_plan_daily_rate = current_plan.price / total_days as i128;
            let new_plan_daily_rate = new_plan.price / total_days as i128;

            let remaining_value = current_plan_daily_rate * days_remaining as i128;
            let new_plan_value = new_plan_daily_rate * days_remaining as i128;

            let proration_amount = if is_upgrade {
                // Charge difference for upgrade
                new_plan_value.saturating_sub(remaining_value)
            } else {
                // Credit difference for downgrade
                remaining_value.saturating_sub(new_plan_value)
            };

            Some(ProrationResult {
                amount: proration_amount,
                is_charge: is_upgrade,
                days_remaining,
                total_days,
            })
        } else {
            None
        }
    } else {
        // Change effective at next billing cycle - no proration
        None
    };

    store_subscription(env, &subscription);

    // Emit tier change event
    let event = TierChangedEvent {
        subscription_id,
        old_tier,
        new_tier: new_plan.tier,
        proration_amount: proration.as_ref().map(|p| p.amount).unwrap_or(0),
    };
    env.events().publish(("TierChanged",), event);

    Ok(proration)
}

/// Retry a failed payment
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
///
/// # Returns
/// true if payment was successful
pub fn retry_payment(env: &Env, subscription_id: u64) -> Result<bool, SubscriptionError> {
    let subscription =
        get_subscription(env, subscription_id).ok_or(SubscriptionError::SubscriptionNotFound)?;

    if subscription.status != SubscriptionStatus::GracePeriod {
        return Err(SubscriptionError::NotInGracePeriod);
    }

    let retry_attempt = subscription.failed_payment_count;
    process_payment(env, subscription_id, retry_attempt)
}

/// Check and process due subscriptions (can be called by anyone)
///
/// # Arguments
/// * `env` - The contract environment
/// * `limit` - Maximum number of subscriptions to process
///
/// # Returns
/// Number of subscriptions processed
pub fn process_due_subscriptions(env: &Env, limit: u32) -> u32 {
    use crate::subscription::storage::get_active_subscriptions;

    let active_ids = get_active_subscriptions(env);
    let now = env.ledger().timestamp();
    let mut processed = 0u32;

    for sub_id in active_ids.iter() {
        if processed >= limit {
            break;
        }

        if let Some(subscription) = get_subscription(env, sub_id) {
            // Check if subscription is due for payment
            if subscription.status == SubscriptionStatus::Active
                && subscription.next_billing_at <= now
                && subscription.auto_renew
            {
                // Attempt payment
                let _ = process_payment(env, sub_id, 0);
                processed += 1;
            }
            // Check if grace period has expired
            else if subscription.status == SubscriptionStatus::GracePeriod {
                if let Some(grace_end) = subscription.grace_period_ends_at {
                    if now > grace_end {
                        // Grace period expired - cancel subscription
                        let mut sub = subscription;
                        sub.status = SubscriptionStatus::Cancelled;
                        sub.cancelled_at = Some(now);
                        sub.cancellation_reason =
                            Some(String::from_str(env, "Grace period expired"));
                        store_subscription(env, &sub);
                        remove_active_subscription(env, sub_id);

                        // Emit cancellation event
                        let event = SubscriptionCancelledEvent {
                            subscription_id: sub_id,
                            cancelled_by: sub.subscriber.clone(),
                            reason: sub.cancellation_reason.clone(),
                        };
                        env.events().publish(("SubscriptionCancelled",), event);
                        processed += 1;
                    }
                }
            }
        }
    }

    processed
}

/// Get subscription status summary
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
///
/// # Returns
/// Subscription details
pub fn get_subscription_status(env: &Env, subscription_id: u64) -> Option<Subscription> {
    get_subscription(env, subscription_id)
}

/// Check if a subscription is active and valid
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
///
/// # Returns
/// true if subscription is active
pub fn is_subscription_active(env: &Env, subscription_id: u64) -> bool {
    if let Some(subscription) = get_subscription(env, subscription_id) {
        subscription.status == SubscriptionStatus::Active
    } else {
        false
    }
}

/// Calculate days until next billing
///
/// # Arguments
/// * `env` - The contract environment
/// * `subscription_id` - ID of the subscription
///
/// # Returns
/// Days until next billing (0 if past due)
pub fn days_until_billing(env: &Env, subscription_id: u64) -> u64 {
    if let Some(subscription) = get_subscription(env, subscription_id) {
        let now = env.ledger().timestamp();
        if subscription.next_billing_at > now {
            (subscription.next_billing_at - now) / (24 * 60 * 60)
        } else {
            0
        }
    } else {
        0
    }
}
