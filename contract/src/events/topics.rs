/// Standardized event topic constants for all Stellar Guilds contract modules.
///
/// Topics are two-element tuples of `(module, action)` Symbols that Stellar's
/// event system uses for efficient filtering. All module code must use the
/// constants defined here instead of creating ad-hoc Symbol strings inline.
///
/// # Naming Convention
/// - Module constants: `MOD_<MODULE>` â€” identifies which sub-module fired the event.
/// - Action constants: `ACT_<ACTION>` â€” identifies what happened.
///
/// # Backward Compatibility
/// Symbol values in Soroban are limited to 32 bytes. Keep all strings short.
/// Once a symbol string is deployed and indexed by external tools, treat it as
/// immutable. If a rename is needed, bump EVENT_SCHEMA_VERSION and add a
/// migration note here.
///
/// # How to add a new module or action
/// 1. Add a `pub const MOD_<MODULE>: &str` entry below.
/// 2. Add `pub const ACT_<ACTION>: &str` entries for its actions.
/// 3. Update `emit.rs` if the new module needs special envelope handling.
/// 4. Document the new event in the module's `//! Events` section.

// =========== Module identifiers ===========

pub const MOD_GUILD: &str = "guild";
pub const MOD_BOUNTY: &str = "bounty";
pub const MOD_PAYMENT: &str = "payment";
pub const MOD_TREASURY: &str = "treasury";
pub const MOD_MILESTONE: &str = "milestone";
pub const MOD_GOVERNANCE: &str = "governance";
pub const MOD_REPUTATION: &str = "reputation";
pub const MOD_DISPUTE: &str = "dispute";
pub const MOD_SUBSCRIPTION: &str = "subscription";
pub const MOD_MULTISIG: &str = "multisig";
pub const MOD_ALLOWANCE: &str = "allowance";
pub const MOD_EMERGENCY: &str = "emergency";
pub const MOD_UPGRADE: &str = "upgrade";
pub const MOD_PROXY: &str = "proxy";

// =========== Shared action identifiers (used across multiple modules) ===========

pub const ACT_CREATED: &str = "created";
pub const ACT_UPDATED: &str = "updated";
pub const ACT_CANCELLED: &str = "cancelled";
pub const ACT_EXECUTED: &str = "executed";
pub const ACT_APPROVED: &str = "approved";
pub const ACT_REJECTED: &str = "rejected";
pub const ACT_RELEASED: &str = "released";
pub const ACT_EXPIRED: &str = "expired";
pub const ACT_PAUSED: &str = "paused";
pub const ACT_RESUMED: &str = "resumed";
pub const ACT_FUNDED: &str = "funded";
pub const ACT_FAILED: &str = "failed";

// =========== Guild-specific actions ===========

pub const ACT_MEMBER_ADDED: &str = "member_added";
pub const ACT_MEMBER_REMOVED: &str = "member_removed";
pub const ACT_ROLE_UPDATED: &str = "role_updated";
pub const ACT_JOINED: &str = "joined";

// =========== Bounty-specific actions ===========

pub const ACT_CLAIMED: &str = "claimed";
pub const ACT_SUBMITTED: &str = "submitted";

// =========== Payment-specific actions ===========

pub const ACT_RECIPIENT_ADDED: &str = "recipient_added";
pub const ACT_DISTRIBUTED: &str = "distributed";

// =========== Governance-specific actions ===========

pub const ACT_VOTED: &str = "voted";
pub const ACT_DELEGATED: &str = "delegated";
pub const ACT_FINALIZED: &str = "finalized";
pub const ACT_PROPOSED: &str = "proposed";

// =========== Milestone-specific actions ===========

pub const ACT_STARTED: &str = "started";
pub const ACT_COMPLETED: &str = "completed";

// =========== Reputation-specific actions ===========

pub const ACT_CONTRIBUTION: &str = "contribution";
pub const ACT_BADGE_EARNED: &str = "badge_earned";

// =========== Dispute-specific actions ===========

pub const ACT_EVIDENCE: &str = "evidence";
pub const ACT_VOTE_CAST: &str = "vote_cast";
pub const ACT_RESOLVED: &str = "resolved";

// =========== Subscription-specific actions ===========

pub const ACT_SUBSCRIBED: &str = "subscribed";
pub const ACT_PLAN_CREATED: &str = "plan_created";
pub const ACT_TIER_CHANGED: &str = "tier_changed";
pub const ACT_PAYMENT_PROCESSED: &str = "payment_ok";
pub const ACT_PAYMENT_FAILED: &str = "payment_fail";
pub const ACT_PAYMENT_RETRIED: &str = "payment_retry";

// =========== Multisig-specific actions ===========

pub const ACT_PROPOSED_OP: &str = "proposed_op";
pub const ACT_SIGNED: &str = "signed";
pub const ACT_SIGNER_ADDED: &str = "signer_added";
pub const ACT_SIGNER_REMOVED: &str = "signer_removed";
pub const ACT_FROZEN: &str = "frozen";
pub const ACT_UNFROZEN: &str = "unfrozen";
pub const ACT_POLICY_SET: &str = "policy_set";

// =========== Allowance-specific actions ===========

pub const ACT_GRANTED: &str = "granted";
pub const ACT_REVOKED: &str = "revoked";
pub const ACT_INCREASED: &str = "increased";
pub const ACT_DECREASED: &str = "decreased";

// =========== Upgrade-specific actions ===========

pub const ACT_UPGRADE_PROPOSED: &str = "upgrade_proposed";
pub const ACT_UPGRADE_EXECUTED: &str = "upgrade_executed";
pub const ACT_EMERGENCY_UPGRADE: &str = "emerg_upgrade";
