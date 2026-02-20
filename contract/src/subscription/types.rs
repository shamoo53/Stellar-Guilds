use soroban_sdk::{contracterror, contracttype, Address, String, Vec};

/// Billing cycle options for subscriptions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BillingCycle {
    /// Weekly billing (7 days)
    Weekly,
    /// Monthly billing (30 days)
    Monthly,
    /// Quarterly billing (90 days)
    Quarterly,
    /// Annual billing (365 days)
    Annually,
}

impl BillingCycle {
    /// Get the duration in seconds for each billing cycle
    pub fn duration_seconds(&self) -> u64 {
        match self {
            BillingCycle::Weekly => 7 * 24 * 60 * 60, // 604,800 seconds
            BillingCycle::Monthly => 30 * 24 * 60 * 60, // 2,592,000 seconds
            BillingCycle::Quarterly => 90 * 24 * 60 * 60, // 7,776,000 seconds
            BillingCycle::Annually => 365 * 24 * 60 * 60, // 31,536,000 seconds
        }
    }
}

/// Status of a subscription
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    /// Subscription is active and billing normally
    Active,
    /// Subscription is paused (no billing, retains benefits temporarily)
    Paused,
    /// Subscription is in grace period after failed payment
    GracePeriod,
    /// Subscription has been cancelled
    Cancelled,
    /// Subscription has expired (end of term reached)
    Expired,
}

/// Tier level for membership subscriptions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum MembershipTier {
    /// Basic tier with limited features
    Basic,
    /// Standard tier with core features
    Standard,
    /// Premium tier with advanced features
    Premium,
    /// Enterprise tier with full features
    Enterprise,
}

/// A subscription plan definition
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionPlan {
    /// Unique plan identifier
    pub id: u64,
    /// Guild ID this plan belongs to (0 for platform-wide plans)
    pub guild_id: u64,
    /// Plan name
    pub name: String,
    /// Plan description
    pub description: String,
    /// Membership tier level
    pub tier: MembershipTier,
    /// Price amount
    pub price: i128,
    /// Token address (None for native XLM)
    pub token: Option<Address>,
    /// Billing cycle
    pub billing_cycle: BillingCycle,
    /// Whether this plan is active
    pub is_active: bool,
    /// Benefits included in this tier
    pub benefits: Vec<String>,
    /// Creator of the plan
    pub created_by: Address,
    /// Creation timestamp
    pub created_at: u64,
}

/// A user subscription instance
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    /// Unique subscription identifier
    pub id: u64,
    /// Plan ID being subscribed to
    pub plan_id: u64,
    /// Subscriber address
    pub subscriber: Address,
    /// Current status
    pub status: SubscriptionStatus,
    /// Current tier level
    pub current_tier: MembershipTier,
    /// Start timestamp
    pub started_at: u64,
    /// End timestamp (for fixed-term subscriptions)
    pub ends_at: Option<u64>,
    /// Next billing timestamp
    pub next_billing_at: u64,
    /// Last payment timestamp
    pub last_payment_at: Option<u64>,
    /// Last payment amount
    pub last_payment_amount: Option<i128>,
    /// Number of consecutive failed payments
    pub failed_payment_count: u32,
    /// Grace period end timestamp
    pub grace_period_ends_at: Option<u64>,
    /// Whether subscription auto-renews
    pub auto_renew: bool,
    /// Cancellation timestamp
    pub cancelled_at: Option<u64>,
    /// Cancellation reason
    pub cancellation_reason: Option<String>,
}

/// Payment retry configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay in seconds
    pub initial_delay_seconds: u64,
    /// Retry backoff multiplier
    pub backoff_multiplier: u32,
    /// Grace period duration in seconds
    pub grace_period_seconds: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 3,
            initial_delay_seconds: 24 * 60 * 60, // 1 day
            backoff_multiplier: 2,
            grace_period_seconds: 7 * 24 * 60 * 60, // 7 days
        }
    }
}

/// Revenue tracking record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevenueRecord {
    /// Record ID
    pub id: u64,
    /// Guild ID (0 for platform-wide)
    pub guild_id: u64,
    /// Plan ID
    pub plan_id: u64,
    /// Subscription ID
    pub subscription_id: u64,
    /// Subscriber address
    pub subscriber: Address,
    /// Payment amount
    pub amount: i128,
    /// Token address (None for native XLM)
    pub token: Option<Address>,
    /// Payment timestamp
    pub paid_at: u64,
    /// Billing cycle this payment covers
    pub billing_cycle: BillingCycle,
    /// Whether this was a retry payment
    pub is_retry: bool,
    /// Retry attempt number (0 for first attempt)
    pub retry_attempt: u32,
}

/// Proration calculation result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProrationResult {
    /// Amount to charge/credit
    pub amount: i128,
    /// Whether this is a charge (true) or credit (false)
    pub is_charge: bool,
    /// Days remaining in current cycle
    pub days_remaining: u64,
    /// Total days in cycle
    pub total_days: u64,
}

/// Subscription change request (upgrade/downgrade)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionChange {
    /// New plan ID
    pub new_plan_id: u64,
    /// Whether to apply immediately or at next cycle
    pub effective_immediately: bool,
    /// Reason for change
    pub reason: Option<String>,
}

/// Error types for subscription operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SubscriptionError {
    /// Plan not found
    PlanNotFound = 1,
    /// Subscription not found
    SubscriptionNotFound = 2,
    /// Invalid billing cycle
    InvalidBillingCycle = 3,
    /// Invalid price amount
    InvalidPrice = 4,
    /// Plan is not active
    PlanNotActive = 5,
    /// Subscription is not active
    SubscriptionNotActive = 6,
    /// Subscription already exists
    SubscriptionAlreadyExists = 7,
    /// Unauthorized operation
    Unauthorized = 8,
    /// Payment failed
    PaymentFailed = 9,
    /// Max retries exceeded
    MaxRetriesExceeded = 10,
    /// Invalid tier change
    InvalidTierChange = 11,
    /// Arithmetic overflow
    ArithmeticOverflow = 12,
    /// Subscription already cancelled
    AlreadyCancelled = 13,
    /// Subscription not in grace period
    NotInGracePeriod = 14,
    /// Invalid subscription state for operation
    InvalidState = 15,
    /// Revenue record not found
    RevenueRecordNotFound = 16,
}

/// Event emitted when a subscription plan is created
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanCreatedEvent {
    pub plan_id: u64,
    pub guild_id: u64,
    pub name: String,
    pub tier: MembershipTier,
    pub price: i128,
    pub billing_cycle: BillingCycle,
}

/// Event emitted when a subscription is created
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionCreatedEvent {
    pub subscription_id: u64,
    pub plan_id: u64,
    pub subscriber: Address,
    pub tier: MembershipTier,
    pub next_billing_at: u64,
}

/// Event emitted when a payment is processed
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentProcessedEvent {
    pub subscription_id: u64,
    pub amount: i128,
    pub success: bool,
    pub retry_attempt: u32,
}

/// Event emitted when a subscription enters grace period
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GracePeriodStartedEvent {
    pub subscription_id: u64,
    pub grace_period_ends_at: u64,
    pub failed_payment_count: u32,
}

/// Event emitted when a subscription is cancelled
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionCancelledEvent {
    pub subscription_id: u64,
    pub cancelled_by: Address,
    pub reason: Option<String>,
}

/// Event emitted when a subscription tier is changed
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierChangedEvent {
    pub subscription_id: u64,
    pub old_tier: MembershipTier,
    pub new_tier: MembershipTier,
    pub proration_amount: i128,
}

/// Event emitted when revenue is recorded
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevenueRecordedEvent {
    pub record_id: u64,
    pub guild_id: u64,
    pub subscription_id: u64,
    pub amount: i128,
    pub paid_at: u64,
}
