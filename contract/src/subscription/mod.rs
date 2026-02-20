pub mod lifecycle;
pub mod storage;
pub mod types;

/// Subscription management module
///
/// This module provides comprehensive subscription management functionality for the Stellar Guilds
/// platform, enabling sustainable revenue models and recurring income streams.
///
/// # Overview
/// - `types`: Defines all core data structures, events, and error types
/// - `storage`: Manages persistent storage of plans, subscriptions, and revenue records
/// - `lifecycle`: Core functions for subscription lifecycle management
///
/// # Key Features
/// - **Tiered Membership**: Basic, Standard, Premium, and Enterprise tiers with different benefits
/// - **Flexible Billing Cycles**: Weekly, Monthly, Quarterly, and Annual billing options
/// - **Automatic Payment Processing**: Built-in retry logic with configurable parameters
/// - **Grace Periods**: Automatic handling of failed payments with grace periods
/// - **Proration Support**: Automatic calculations for mid-cycle tier changes
/// - **Revenue Tracking**: Comprehensive revenue recording and reporting
/// - **Pause/Resume**: Subscribers can pause and resume subscriptions
///
/// # Subscription Lifecycle
/// 1. **Plan Creation**: Guilds or platform create subscription plans with pricing and benefits
/// 2. **Subscription**: Users subscribe to a plan, creating an active subscription
/// 3. **Payment Processing**: Automatic payment processing at billing cycle intervals
/// 4. **Retry Logic**: Failed payments trigger retry attempts with grace periods
/// 5. **Tier Changes**: Support for upgrades/downgrades with proration
/// 6. **Cancellation**: Subscribers can cancel at any time
///
/// # Edge Cases Handled
/// - Payment failures with configurable retry logic
/// - Grace period expiration handling
/// - Subscription upgrade/downgrade with proration
/// - Pause/resume functionality
/// - Auto-renewal management
// Re-export main types
pub use types::{
    BillingCycle, GracePeriodStartedEvent, MembershipTier, PaymentProcessedEvent, PlanCreatedEvent,
    ProrationResult, RetryConfig, RevenueRecord, RevenueRecordedEvent, Subscription,
    SubscriptionCancelledEvent, SubscriptionChange, SubscriptionCreatedEvent, SubscriptionError,
    SubscriptionPlan, SubscriptionStatus, TierChangedEvent,
};

// Re-export storage functions
pub use storage::{
    add_guild_revenue, get_all_plans, get_guild_plans, get_guild_revenue_records, get_plan,
    get_retry_config, get_revenue_record, get_subscription, get_subscriptions_by_plan,
    get_user_subscription, initialize_subscription_storage, set_retry_config, store_plan,
    store_subscription,
};

// Re-export lifecycle functions
pub use lifecycle::{
    cancel_subscription, change_tier, create_plan, days_until_billing, get_subscription_status,
    is_subscription_active, pause_subscription, process_due_subscriptions, process_payment,
    resume_subscription, retry_payment, subscribe,
};

#[cfg(test)]
mod tests;
