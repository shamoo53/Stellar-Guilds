#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

mod guild;
use guild::membership::{
    add_member, create_guild, get_all_members, get_member, has_permission, is_member,
    remove_member, update_role,
};
use guild::storage;
use guild::types::{Member, Role};

mod bounty;
use bounty::{
    approve_completion, cancel_bounty, claim_bounty, create_bounty, expire_bounty, fund_bounty,
    get_bounty_data, get_guild_bounties_list, release_escrow, submit_work, Bounty,
};

mod treasury;
use treasury::{
    approve_transaction as core_approve_transaction, deposit as core_deposit,
    emergency_pause as core_emergency_pause, execute_transaction as core_execute_transaction,
    get_balance as core_get_balance, get_transaction_history as core_get_transaction_history,
    grant_allowance as core_grant_allowance, initialize_treasury as core_initialize_treasury,
    propose_withdrawal as core_propose_withdrawal, set_budget as core_set_budget, Transaction,
};

mod analytics;
use analytics::{
    compute_budget_utilization, compute_category_breakdown, compute_forecast,
    compute_spending_summary, compute_trend, get_snapshots, store_snapshot, BudgetUtilization,
    CategoryBreakdown, SpendingForecast, SpendingSummary, SpendingTrend, TreasurySnapshot,
};

mod reputation;
use reputation::{
    compute_governance_weight as rep_governance_weight, get_badges as rep_get_badges,
    get_contributions as rep_get_contributions, get_decayed_profile, get_global_reputation,
    record_contribution as rep_record_contribution, Badge, ContributionRecord, ContributionType,
    ReputationProfile,
};

mod governance;
use governance::{
    cancel_proposal as gov_cancel_proposal, create_proposal as gov_create_proposal,
    delegate_vote as gov_delegate_vote, execute_proposal as gov_execute_proposal,
    finalize_proposal as gov_finalize_proposal, get_active_proposals as gov_get_active_proposals,
    get_proposal as gov_get_proposal, undelegate_vote as gov_undelegate_vote,
    update_governance_config as gov_update_governance_config, vote as gov_vote, ExecutionPayload,
    GovernanceConfig, Proposal, ProposalStatus, ProposalType, VoteDecision,
};

mod milestone;
use milestone::{
    add_milestone as ms_add_milestone, approve_milestone as ms_approve_milestone,
    cancel_project as ms_cancel_project, create_project as ms_create_project,
    extend_milestone_deadline as ms_extend_deadline, get_milestone_view as ms_get_milestone,
    get_project_progress as ms_get_progress, reject_milestone as ms_reject_milestone,
    release_milestone_payment as ms_release_payment, start_milestone as ms_start_milestone,
    submit_milestone as ms_submit_milestone, Milestone, MilestoneInput,
};

mod payment;
use payment::{
    add_recipient as pay_add_recipient, batch_distribute as pay_batch_distribute,
    cancel_distribution as pay_cancel_distribution, create_payment_pool as pay_create_payment_pool,
    execute_distribution as pay_execute_distribution, get_pool_status as pay_get_pool_status,
    get_recipient_amount as pay_get_recipient_amount,
    validate_distribution as pay_validate_distribution, DistributionRule, DistributionStatus,
};

mod subscription;
use subscription::{
    cancel_subscription as sub_cancel_subscription, change_tier as sub_change_tier,
    create_plan as sub_create_plan, days_until_billing as sub_days_until_billing,
    get_subscription_status as sub_get_subscription_status,
    is_subscription_active as sub_is_subscription_active,
    pause_subscription as sub_pause_subscription,
    process_due_subscriptions as sub_process_due_subscriptions,
    process_payment as sub_process_payment, resume_subscription as sub_resume_subscription,
    retry_payment as sub_retry_payment, subscribe as sub_subscribe, BillingCycle, MembershipTier,
    ProrationResult, Subscription, SubscriptionChange, SubscriptionError, SubscriptionPlan,
    SubscriptionStatus,
};

mod dispute;
use dispute::{
    calculate_vote_weight as dispute_calculate_vote_weight, cast_vote as dispute_cast_vote,
    create_dispute as dispute_create_dispute, execute_resolution as dispute_execute_resolution,
    resolve_dispute as dispute_resolve_dispute, submit_evidence as dispute_submit_evidence,
    tally_votes as dispute_tally_votes,
};

mod allowance;
use allowance::{
    approve as allowance_approve, decrease_allowance as allowance_decrease,
    get_allowance_detail as allowance_get, get_owner_allowances as allowance_list_owner,
    get_spender_allowances as allowance_list_spender, increase_allowance as allowance_increase,
    revoke as allowance_revoke, AllowanceOperation, TokenAllowance,
};

mod multisig;
use multisig::{
    // Registrar aliases to prevent recursive naming collisions
    ms_add_signer as internal_add_signer,
    // Signing aliases
    ms_cancel_operation as internal_cancel_operation,
    ms_check_and_expire as internal_check_and_expire,
    ms_emergency_expire_operation as internal_emergency_expire_operation,
    ms_emergency_extend_timeout as internal_emergency_extend_timeout,
    ms_execute_operation as internal_execute_operation,
    ms_freeze_account as internal_freeze_account,
    // Policy aliases
    ms_get_operation_policy as internal_get_operation_policy,
    ms_get_operation_status as internal_get_operation_status,
    ms_get_pending_operations as internal_get_pending_operations,
    ms_get_safe_account as internal_get_safe_account,
    ms_list_accounts_by_owner as internal_list_accounts_by_owner,
    ms_propose_operation as internal_propose_operation,
    ms_register_account as internal_register_account,
    ms_remove_signer as internal_remove_signer,
    ms_require_executed_operation as internal_require_executed_operation,
    ms_reset_operation_policy as internal_reset_operation_policy,
    ms_rotate_signer as internal_rotate_signer,
    ms_set_operation_policy as internal_set_operation_policy,

    ms_sign_operation as internal_sign_operation,
    ms_sweep_expired_operations as internal_sweep_expired_operations,

    ms_unfreeze_account as internal_unfreeze_account,
    ms_update_threshold as internal_update_threshold,

    // Types
    MultiSigAccount,
    MultiSigOperation,
    OperationPolicy,
    OperationType,
};

/// Stellar Guilds - Main Contract Entry Point
///
/// This is the foundational contract for the Stellar Guilds platform.
/// It enables users to create guilds, add members, assign roles, and manage
/// permissions on-chain. This forms the foundation for decentralized communities,
/// voting, and role-based governance.
///
/// # Core Features
/// - Guild creation with metadata
/// - Member management with role assignments
/// - Permission-based access control
/// - Event tracking for all state changes
/// - Efficient on-chain storage management

#[contract]
pub struct StellarGuildsContract;

#[contractimpl]
impl StellarGuildsContract {
    pub fn initialize(env: Env) -> bool {
        storage::initialize(&env);
        subscription::storage::initialize_subscription_storage(&env);
        true
    }

    /// Get contract version
    pub fn version(_env: Env) -> String {
        String::from_str(&_env, "0.1.0")
    }

    /// Create a new guild
    ///
    /// # Arguments
    /// * `name` - The name of the guild
    /// * `description` - The description of the guild
    /// * `owner` - The address of the guild owner
    ///
    /// # Returns
    /// The ID of the newly created guild
    pub fn create_guild(env: Env, name: String, description: String, owner: Address) -> u64 {
        owner.require_auth();
        match create_guild(&env, name, description, owner) {
            Ok(id) => id,
            Err(_) => panic!("create_guild error"),
        }
    }

    /// Add a member to a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member to add
    /// * `role` - The role to assign
    /// * `caller` - The address making the request (must have permission)
    ///
    /// # Returns
    /// true if successful, panics with error message otherwise
    pub fn add_member(
        env: Env,
        guild_id: u64,
        address: Address,
        role: Role,
        caller: Address,
    ) -> bool {
        caller.require_auth();
        match add_member(&env, guild_id, address, role, caller) {
            Ok(result) => result,
            Err(_) => panic!("add_member error"),
        }
    }

    /// Remove a member from a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member to remove
    /// * `caller` - The address making the request
    ///
    /// # Returns
    /// true if successful, panics with error message otherwise
    pub fn remove_member(env: Env, guild_id: u64, address: Address, caller: Address) -> bool {
        caller.require_auth();
        match remove_member(&env, guild_id, address, caller) {
            Ok(result) => result,
            Err(_) => panic!("remove_member error"),
        }
    }

    /// Update a member's role
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member
    /// * `new_role` - The new role to assign
    /// * `caller` - The address making the request (must have permission)
    ///
    /// # Returns
    /// true if successful, panics with error message otherwise
    pub fn update_role(
        env: Env,
        guild_id: u64,
        address: Address,
        new_role: Role,
        caller: Address,
    ) -> bool {
        caller.require_auth();
        match update_role(&env, guild_id, address, new_role, caller) {
            Ok(result) => result,
            Err(_) => panic!("update_role error"),
        }
    }

    /// Get a member from a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member
    ///
    /// # Returns
    /// The Member if found, panics with error message otherwise
    pub fn get_member(env: Env, guild_id: u64, address: Address) -> Member {
        match get_member(&env, guild_id, address) {
            Ok(member) => member,
            Err(_) => panic!("get_member error"),
        }
    }

    /// Get all members of a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    ///
    /// # Returns
    /// A vector of all members in the guild
    pub fn get_all_members(env: Env, guild_id: u64) -> Vec<Member> {
        get_all_members(&env, guild_id)
    }

    /// Check if an address is a member of a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address to check
    ///
    /// # Returns
    /// true if the address is a member, false otherwise
    pub fn is_member(env: Env, guild_id: u64, address: Address) -> bool {
        is_member(&env, guild_id, address)
    }

    /// Check if a member has permission for a required role
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member
    /// * `required_role` - The required role level
    ///
    /// # Returns
    /// true if the member has the required permission, false otherwise
    pub fn has_permission(env: Env, guild_id: u64, address: Address, required_role: Role) -> bool {
        has_permission(&env, guild_id, address, required_role)
    }

    // ============ Payment Functions ============

    pub fn create_payment_pool(
        env: Env,
        total_amount: i128,
        token: Option<Address>,
        rule: DistributionRule,
        creator: Address,
    ) -> u64 {
        match pay_create_payment_pool(&env, total_amount, token, rule, creator) {
            Ok(id) => id,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn add_recipient(
        env: Env,
        pool_id: u64,
        recipient: Address,
        share: u32,
        caller: Address,
    ) -> bool {
        match pay_add_recipient(&env, pool_id, recipient, share, caller) {
            Ok(result) => result,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn validate_distribution(env: Env, pool_id: u64) -> bool {
        match pay_validate_distribution(&env, pool_id) {
            Ok(result) => result,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn get_recipient_amount(env: Env, pool_id: u64, recipient: Address) -> i128 {
        match pay_get_recipient_amount(&env, pool_id, recipient) {
            Ok(amount) => amount,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn cancel_distribution(env: Env, pool_id: u64, caller: Address) -> bool {
        match pay_cancel_distribution(&env, pool_id, caller) {
            Ok(result) => result,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn get_pool_status(env: Env, pool_id: u64) -> DistributionStatus {
        match pay_get_pool_status(&env, pool_id) {
            Ok(status) => status,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    /// Execute distribution for a payment pool
    ///
    /// # Arguments
    /// * `pool_id` - The ID of the pool to execute
    /// * `caller` - The address executing the distribution (must be pool creator)
    ///
    /// # Returns
    /// `true` if distribution was successful
    pub fn execute_distribution(env: Env, pool_id: u64, caller: Address) -> bool {
        match pay_execute_distribution(&env, pool_id, caller) {
            Ok(result) => result,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    /// Execute distribution for multiple payment pools in batch
    ///
    /// # Arguments
    /// * `pool_ids` - Vector of pool IDs to execute
    /// * `caller` - The address executing the distributions (must be pool creator for each)
    ///
    /// # Returns
    /// Vector of results (true for success, false for failure) for each pool
    pub fn batch_distribute(env: Env, pool_ids: Vec<u64>, caller: Address) -> Vec<bool> {
        pay_batch_distribute(&env, pool_ids, caller)
    }

    // ============ Dispute Functions ============

    /// Create a dispute for a bounty or milestone
    ///
    /// # Arguments
    /// * `reference_id` - Bounty or milestone ID
    /// * `plaintiff` - Address opening the dispute
    /// * `defendant` - Address responding to the dispute
    /// * `reason` - Dispute reason
    /// * `evidence_url` - Initial evidence URL
    ///
    /// # Returns
    /// The ID of the newly created dispute
    pub fn create_dispute(
        env: Env,
        reference_id: u64,
        plaintiff: Address,
        defendant: Address,
        reason: String,
        evidence_url: String,
    ) -> u64 {
        dispute_create_dispute(
            &env,
            reference_id,
            plaintiff,
            defendant,
            reason,
            evidence_url,
        )
    }

    /// Submit evidence for an active dispute
    pub fn submit_evidence(
        env: Env,
        dispute_id: u64,
        party: Address,
        evidence_url: String,
    ) -> bool {
        dispute_submit_evidence(&env, dispute_id, party, evidence_url)
    }

    /// Cast a weighted vote on a dispute
    pub fn cast_dispute_vote(
        env: Env,
        dispute_id: u64,
        voter: Address,
        decision: dispute::types::VoteDecision,
    ) -> bool {
        dispute_cast_vote(&env, dispute_id, voter, decision)
    }

    /// Calculate voting weight for a guild member
    pub fn calculate_dispute_vote_weight(env: Env, guild_id: u64, voter: Address) -> u32 {
        dispute_calculate_vote_weight(&env, guild_id, voter)
    }

    /// Tally votes for a dispute
    pub fn tally_dispute_votes(env: Env, dispute_id: u64) -> dispute::types::Resolution {
        dispute_tally_votes(&env, dispute_id)
    }

    /// Resolve a dispute and execute fund distribution
    pub fn resolve_dispute(env: Env, dispute_id: u64) -> dispute::types::Resolution {
        dispute_resolve_dispute(&env, dispute_id)
    }

    /// Execute a resolved dispute payout
    pub fn execute_dispute_resolution(
        env: Env,
        dispute_id: u64,
    ) -> Vec<dispute::types::FundDistribution> {
        dispute_execute_resolution(&env, dispute_id)
    }

    // ============ Treasury Functions ============

    /// Initialize a new treasury for a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `signers` - Vector of signer addresses (first is owner)
    /// * `approval_threshold` - Number of approvals required for transactions
    ///
    /// # Returns
    /// The ID of the newly created treasury
    pub fn initialize_treasury(
        env: Env,
        guild_id: u64,
        signers: Vec<Address>,
        approval_threshold: u32,
    ) -> u64 {
        core_initialize_treasury(&env, guild_id, signers, approval_threshold)
    }

    /// Deposit funds into a treasury
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `depositor` - Address making the deposit
    /// * `amount` - Amount to deposit
    /// * `token` - Token address (None for XLM)
    ///
    /// # Returns
    /// `true` if deposit was successful
    pub fn deposit_treasury(
        env: Env,
        treasury_id: u64,
        depositor: Address,
        amount: i128,
        token: Option<Address>,
    ) -> bool {
        core_deposit(&env, treasury_id, depositor, amount, token)
    }

    /// Propose a withdrawal from treasury
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `proposer` - Address proposing the withdrawal
    /// * `recipient` - Address to receive the funds
    /// * `amount` - Amount to withdraw
    /// * `token` - Token address (None for XLM)
    /// * `reason` - Reason for the withdrawal
    ///
    /// # Returns
    /// The ID of the proposed transaction
    pub fn propose_withdrawal(
        env: Env,
        treasury_id: u64,
        proposer: Address,
        recipient: Address,
        amount: i128,
        token: Option<Address>,
        reason: String,
    ) -> u64 {
        core_propose_withdrawal(
            &env,
            treasury_id,
            proposer,
            recipient,
            amount,
            token,
            reason,
        )
    }

    /// Approve a proposed transaction
    ///
    /// # Arguments
    /// * `tx_id` - The ID of the transaction to approve
    /// * `approver` - Address approving the transaction
    ///
    /// # Returns
    /// `true` if approval was successful
    pub fn approve_transaction(env: Env, tx_id: u64, approver: Address) -> bool {
        core_approve_transaction(&env, tx_id, approver)
    }

    /// Execute an approved transaction
    ///
    /// # Arguments
    /// * `tx_id` - The ID of the transaction to execute
    /// * `executor` - Address executing the transaction
    ///
    /// # Returns
    /// `true` if execution was successful
    pub fn execute_transaction(env: Env, tx_id: u64, executor: Address) -> bool {
        core_execute_transaction(&env, tx_id, executor)
    }

    /// Set a budget for a treasury category
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `category` - Budget category name
    /// * `amount` - Budget amount
    /// * `period_seconds` - Budget period in seconds
    /// * `caller` - Address making the request (must be signer)
    ///
    /// # Returns
    /// `true` if budget was set successfully
    pub fn set_budget(
        env: Env,
        treasury_id: u64,
        category: String,
        amount: i128,
        period_seconds: u64,
        caller: Address,
    ) -> bool {
        core_set_budget(&env, treasury_id, caller, category, amount, period_seconds)
    }

    /// Get treasury balance for a token
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `token` - Token address (None for XLM)
    ///
    /// # Returns
    /// The balance amount
    pub fn get_treasury_balance(env: Env, treasury_id: u64, token: Option<Address>) -> i128 {
        core_get_balance(&env, treasury_id, token)
    }

    /// Get transaction history for a treasury
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `limit` - Maximum number of transactions to return
    ///
    /// # Returns
    /// Vector of transactions
    pub fn get_transaction_history(env: Env, treasury_id: u64, limit: u32) -> Vec<Transaction> {
        core_get_transaction_history(&env, treasury_id, limit)
    }

    /// Grant an allowance to an admin
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `admin` - Address to grant allowance to
    /// * `amount` - Allowance amount per period
    /// * `token` - Token address (None for XLM)
    /// * `period_seconds` - Allowance period in seconds
    /// * `owner` - Treasury owner making the request
    ///
    /// # Returns
    /// `true` if allowance was granted successfully
    pub fn grant_allowance(
        env: Env,
        treasury_id: u64,
        admin: Address,
        amount: i128,
        token: Option<Address>,
        period_seconds: u64,
        owner: Address,
    ) -> bool {
        core_grant_allowance(
            &env,
            treasury_id,
            owner,
            admin,
            amount,
            token,
            period_seconds,
        )
    }

    /// Emergency pause treasury operations
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `signer` - Address making the request (must be signer)
    /// * `paused` - Whether to pause or unpause
    ///
    /// # Returns
    /// `true` if pause state was changed successfully
    pub fn emergency_pause(env: Env, treasury_id: u64, signer: Address, paused: bool) -> bool {
        core_emergency_pause(&env, treasury_id, signer, paused)
    }

    // ============ Token Allowance Functions ============

    /// Approve a token allowance from owner to spender.
    ///
    /// Creates or replaces an allowance with optional expiration and
    /// per-operation granularity.
    ///
    /// # Arguments
    /// * `owner` - Address granting the allowance (requires auth)
    /// * `spender` - Address permitted to spend
    /// * `token` - Token address (None for XLM)
    /// * `amount` - Maximum spendable amount
    /// * `expires_at` - Ledger timestamp expiry (0 = no expiry)
    /// * `operation` - Operation type filter
    pub fn approve_token_allowance(
        env: Env,
        owner: Address,
        spender: Address,
        token: Option<Address>,
        amount: i128,
        expires_at: u64,
        operation: AllowanceOperation,
    ) -> bool {
        allowance_approve(&env, owner, spender, token, amount, expires_at, operation)
            .unwrap_or_else(|e| {
                let msg = match e {
                    allowance::AllowanceError::InvalidAmount => "invalid amount",
                    allowance::AllowanceError::Expired => "already expired",
                    _ => "allowance error",
                };
                panic!("{}", msg);
            });
        true
    }

    /// Atomically increase an existing allowance.
    pub fn increase_token_allowance(
        env: Env,
        owner: Address,
        spender: Address,
        token: Option<Address>,
        delta: i128,
    ) -> bool {
        allowance_increase(&env, owner, spender, token, delta).unwrap_or_else(|e| {
            let msg = match e {
                allowance::AllowanceError::NotFound => "allowance not found",
                allowance::AllowanceError::Expired => "allowance expired",
                allowance::AllowanceError::InvalidAmount => "invalid amount",
                _ => "allowance error",
            };
            panic!("{}", msg);
        });
        true
    }

    /// Atomically decrease an existing allowance.
    pub fn decrease_token_allowance(
        env: Env,
        owner: Address,
        spender: Address,
        token: Option<Address>,
        delta: i128,
    ) -> bool {
        allowance_decrease(&env, owner, spender, token, delta).unwrap_or_else(|e| {
            let msg = match e {
                allowance::AllowanceError::NotFound => "allowance not found",
                allowance::AllowanceError::Expired => "allowance expired",
                allowance::AllowanceError::InvalidAmount => "invalid amount",
                _ => "allowance error",
            };
            panic!("{}", msg);
        });
        true
    }

    /// Revoke (delete) a token allowance.
    pub fn revoke_token_allowance(
        env: Env,
        owner: Address,
        spender: Address,
        token: Option<Address>,
    ) -> bool {
        allowance_revoke(&env, owner, spender, token).unwrap_or_else(|e| {
            let msg = match e {
                allowance::AllowanceError::NotFound => "allowance not found",
                _ => "allowance error",
            };
            panic!("{}", msg);
        });
        true
    }

    /// Get allowance details for a specific (owner, spender, token) triple.
    pub fn get_token_allowance(
        env: Env,
        owner: Address,
        spender: Address,
        token: Option<Address>,
    ) -> TokenAllowance {
        allowance_get(&env, &owner, &spender, &token)
            .unwrap_or_else(|| panic!("allowance not found"))
    }

    /// List all allowances granted by an owner.
    pub fn get_owner_allowances(env: Env, owner: Address) -> Vec<TokenAllowance> {
        allowance_list_owner(&env, &owner)
    }

    /// List all allowances where the given address is the spender.
    pub fn get_spender_allowances(env: Env, spender: Address) -> Vec<TokenAllowance> {
        allowance_list_spender(&env, &spender)
    }

    // ============ Analytics Functions ============

    /// Get spending summary for a treasury within a time range.
    ///
    /// # Arguments
    /// * `treasury_id` - The treasury to analyze
    /// * `period_start` - Start timestamp (unix seconds)
    /// * `period_end` - End timestamp (unix seconds)
    ///
    /// # Returns
    /// `SpendingSummary` with aggregated deposit/withdrawal totals
    pub fn get_spending_summary(
        env: Env,
        treasury_id: u64,
        period_start: u64,
        period_end: u64,
    ) -> SpendingSummary {
        compute_spending_summary(&env, treasury_id, period_start, period_end)
    }

    /// Get budget utilization for all categories of a treasury.
    ///
    /// # Returns
    /// `Vec<BudgetUtilization>` with allocated/spent/remaining per category
    pub fn get_budget_utilization(env: Env, treasury_id: u64) -> Vec<BudgetUtilization> {
        compute_budget_utilization(&env, treasury_id)
    }

    /// Get spending breakdown by transaction type for a time range.
    ///
    /// # Returns
    /// `Vec<CategoryBreakdown>` grouped by transaction type
    pub fn get_category_breakdown(
        env: Env,
        treasury_id: u64,
        period_start: u64,
        period_end: u64,
    ) -> Vec<CategoryBreakdown> {
        compute_category_breakdown(&env, treasury_id, period_start, period_end)
    }

    /// Compare spending between two time periods.
    ///
    /// # Returns
    /// `SpendingTrend` with percentage changes in basis points
    pub fn get_spending_trend(
        env: Env,
        treasury_id: u64,
        p1_start: u64,
        p1_end: u64,
        p2_start: u64,
        p2_end: u64,
    ) -> SpendingTrend {
        let period1 = compute_spending_summary(&env, treasury_id, p1_start, p1_end);
        let period2 = compute_spending_summary(&env, treasury_id, p2_start, p2_end);
        compute_trend(&env, &period1, &period2)
    }

    /// Forecast future spending using moving average of historical periods.
    ///
    /// # Arguments
    /// * `num_periods` - Number of past periods to analyze
    /// * `period_length_secs` - Length of each period in seconds
    ///
    /// # Returns
    /// `SpendingForecast` with projected deposits/withdrawals
    pub fn get_spending_forecast(
        env: Env,
        treasury_id: u64,
        num_periods: u32,
        period_length_secs: u64,
    ) -> SpendingForecast {
        let current_time = env.ledger().timestamp();
        compute_forecast(
            &env,
            treasury_id,
            num_periods,
            period_length_secs,
            current_time,
        )
    }

    /// Get recent treasury balance snapshots.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of snapshots to return
    ///
    /// # Returns
    /// `Vec<TreasurySnapshot>` ordered oldest to newest
    pub fn get_treasury_snapshots(env: Env, treasury_id: u64, limit: u32) -> Vec<TreasurySnapshot> {
        get_snapshots(&env, treasury_id, limit)
    }

    /// Manually record a treasury snapshot (signer-only).
    ///
    /// # Returns
    /// `true` if snapshot was recorded
    pub fn record_treasury_snapshot(env: Env, treasury_id: u64, caller: Address) -> bool {
        caller.require_auth();
        let treasury =
            treasury::storage::get_treasury(&env, treasury_id).expect("treasury not found");
        treasury::multisig::ensure_is_signer(&treasury, &caller);

        let index = analytics::get_snapshot_count(&env, treasury_id);
        let snapshot = TreasurySnapshot {
            treasury_id,
            timestamp: env.ledger().timestamp(),
            balance_xlm: treasury.balance_xlm,
            total_deposits: treasury.total_deposits,
            total_withdrawals: treasury.total_withdrawals,
            snapshot_index: index,
        };
        store_snapshot(&env, &snapshot);
        true
    }

    // ============ Reputation Functions ============

    /// Record a contribution and update reputation score.
    /// Awards badges automatically if thresholds are met.
    pub fn record_contribution(
        env: Env,
        guild_id: u64,
        contributor: Address,
        contribution_type: ContributionType,
        reference_id: u64,
    ) {
        contributor.require_auth();
        rep_record_contribution(
            &env,
            guild_id,
            &contributor,
            contribution_type,
            reference_id,
        );
    }

    /// Get a user's reputation profile for a specific guild (with decay applied).
    pub fn get_reputation(env: Env, guild_id: u64, address: Address) -> ReputationProfile {
        get_decayed_profile(&env, &address, guild_id)
            .unwrap_or_else(|| panic!("no reputation profile found"))
    }

    /// Get a user's aggregate reputation across all guilds.
    pub fn get_reputation_global(env: Env, address: Address) -> u64 {
        get_global_reputation(&env, &address)
    }

    /// Get contribution history for a user in a guild.
    pub fn get_reputation_contributions(
        env: Env,
        guild_id: u64,
        address: Address,
        limit: u32,
    ) -> Vec<ContributionRecord> {
        rep_get_contributions(&env, &address, guild_id, limit)
    }

    /// Get badges earned by a user in a guild.
    pub fn get_reputation_badges(env: Env, guild_id: u64, address: Address) -> Vec<Badge> {
        rep_get_badges(&env, &address, guild_id)
    }

    /// Get computed governance weight for a user (role + reputation).
    pub fn get_governance_weight_for(env: Env, guild_id: u64, address: Address) -> i128 {
        let member = guild::storage::get_member(&env, guild_id, &address)
            .unwrap_or_else(|| panic!("not a guild member"));
        rep_governance_weight(&env, &address, guild_id, &member.role)
    }

    // ============ Milestone Tracking Functions ============

    /// Create a new project with milestones
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `contributor` - Address of the project contributor
    /// * `milestones` - Vector of milestone definitions
    /// * `total_amount` - Total project budget
    /// * `treasury_id` - Treasury ID for payments
    /// * `token` - Token address (None for XLM)
    /// * `is_sequential` - Whether milestones must be completed in order
    ///
    /// # Returns
    /// The ID of the newly created project
    pub fn create_project(
        env: Env,
        guild_id: u64,
        contributor: Address,
        milestones: Vec<MilestoneInput>,
        total_amount: i128,
        treasury_id: u64,
        token: Option<Address>,
        is_sequential: bool,
    ) -> u64 {
        ms_create_project(
            &env,
            guild_id,
            contributor,
            milestones,
            total_amount,
            treasury_id,
            token,
            is_sequential,
        )
    }

    /// Add a new milestone to an existing project
    ///
    /// # Arguments
    /// * `project_id` - The ID of the project
    /// * `title` - Milestone title
    /// * `description` - Milestone description
    /// * `amount` - Payment amount for this milestone
    /// * `deadline` - Deadline timestamp
    /// * `caller` - Address making the request (must be guild admin)
    ///
    /// # Returns
    /// The ID of the newly created milestone
    pub fn add_milestone(
        env: Env,
        project_id: u64,
        title: String,
        description: String,
        amount: i128,
        deadline: u64,
        caller: Address,
    ) -> u64 {
        ms_add_milestone(
            &env,
            project_id,
            title,
            description,
            amount,
            deadline,
            caller,
        )
    }

    /// Start working on a milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `contributor` - Address of the contributor starting work
    ///
    /// # Returns
    /// `true` if successful
    pub fn start_milestone(env: Env, milestone_id: u64, contributor: Address) -> bool {
        ms_start_milestone(&env, milestone_id, contributor)
    }

    /// Submit completed work for a milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `proof_url` - URL to proof of work
    ///
    /// # Returns
    /// `true` if successful
    pub fn submit_milestone(env: Env, milestone_id: u64, proof_url: String) -> bool {
        ms_submit_milestone(&env, milestone_id, proof_url)
    }

    /// Approve a submitted milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `approver` - Address of the approver (must be guild admin)
    ///
    /// # Returns
    /// `true` if successful
    pub fn approve_milestone(env: Env, milestone_id: u64, approver: Address) -> bool {
        ms_approve_milestone(&env, milestone_id, approver)
    }

    /// Reject a submitted milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `approver` - Address of the approver (must be guild admin)
    /// * `reason` - Reason for rejection
    ///
    /// # Returns
    /// `true` if successful
    pub fn reject_milestone(
        env: Env,
        milestone_id: u64,
        approver: Address,
        reason: String,
    ) -> bool {
        ms_reject_milestone(&env, milestone_id, approver, reason)
    }

    /// Get project progress statistics
    ///
    /// # Arguments
    /// * `project_id` - The ID of the project
    ///
    /// # Returns
    /// Tuple of (completed_count, total_count, percentage)
    pub fn get_project_progress(env: Env, project_id: u64) -> (u32, u32, u32) {
        ms_get_progress(&env, project_id)
    }

    /// Get milestone details
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    ///
    /// # Returns
    /// The Milestone struct
    pub fn get_milestone(env: Env, milestone_id: u64) -> Milestone {
        ms_get_milestone(&env, milestone_id)
    }

    /// Release payment for an approved milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    ///
    /// # Returns
    /// `true` if successful
    pub fn release_milestone_payment(env: Env, milestone_id: u64) -> bool {
        ms_release_payment(&env, milestone_id)
    }

    /// Extend the deadline of a milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `new_deadline` - New deadline timestamp
    /// * `caller` - Address making the request (must be guild admin)
    ///
    /// # Returns
    /// `true` if successful
    pub fn extend_milestone_deadline(
        env: Env,
        milestone_id: u64,
        new_deadline: u64,
        caller: Address,
    ) -> bool {
        ms_extend_deadline(&env, milestone_id, new_deadline, caller)
    }

    /// Cancel a project
    ///
    /// # Arguments
    /// * `project_id` - The ID of the project
    /// * `caller` - Address making the request (must be guild admin)
    ///
    /// # Returns
    /// `true` if successful
    pub fn cancel_project(env: Env, project_id: u64, caller: Address) -> bool {
        ms_cancel_project(&env, project_id, caller)
    }

    // ============ Governance Functions ============

    /// Create a new governance proposal
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `proposer` - Address of the proposer
    /// * `proposal_type` - Type of the proposal
    /// * `title` - Proposal title
    /// * `description` - Detailed description
    ///
    /// # Returns
    /// The ID of the newly created proposal
    pub fn create_proposal(
        env: Env,
        guild_id: u64,
        proposer: Address,
        proposal_type: ProposalType,
        title: String,
        description: String,
    ) -> u64 {
        gov_create_proposal(
            &env,
            guild_id,
            proposer,
            proposal_type,
            title,
            description,
            ExecutionPayload::GeneralDecision,
        )
    }

    /// Get a proposal by ID
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// The Proposal struct
    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        gov_get_proposal(&env, proposal_id)
    }

    /// Get all active proposals for a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    ///
    /// # Returns
    /// Vector of active proposals
    pub fn get_active_proposals(env: Env, guild_id: u64) -> Vec<Proposal> {
        gov_get_active_proposals(&env, guild_id)
    }

    /// Cast a vote on a proposal
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal
    /// * `voter` - Address of the voter
    /// * `decision` - Vote decision (For, Against, Abstain)
    ///
    /// # Returns
    /// `true` if successful
    pub fn vote(env: Env, proposal_id: u64, voter: Address, decision: VoteDecision) -> bool {
        gov_vote(&env, proposal_id, voter, decision)
    }

    /// Delegate voting power to another member
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `delegator` - Address delegating their vote
    /// * `delegate` - Address receiving the delegation
    ///
    /// # Returns
    /// `true` if successful
    pub fn delegate_vote(env: Env, guild_id: u64, delegator: Address, delegate: Address) -> bool {
        gov_delegate_vote(&env, guild_id, delegator, delegate)
    }

    /// Remove vote delegation
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `delegator` - Address removing their delegation
    ///
    /// # Returns
    /// `true` if successful
    pub fn undelegate_vote(env: Env, guild_id: u64, delegator: Address) -> bool {
        gov_undelegate_vote(&env, guild_id, delegator)
    }

    /// Finalize a proposal after voting period ends
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// The final status of the proposal
    pub fn finalize_proposal(env: Env, proposal_id: u64) -> ProposalStatus {
        gov_finalize_proposal(&env, proposal_id)
    }

    /// Execute a passed proposal
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal to execute
    /// * `executor` - Address executing the proposal
    ///
    /// # Returns
    /// `true` if execution was successful
    pub fn execute_proposal(env: Env, proposal_id: u64, executor: Address) -> bool {
        gov_execute_proposal(&env, proposal_id, executor)
    }

    /// Cancel a proposal
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal
    /// * `caller` - Address making the request (must be proposer or admin)
    ///
    /// # Returns
    /// `true` if successful
    pub fn cancel_proposal(env: Env, proposal_id: u64, caller: Address) -> bool {
        gov_cancel_proposal(&env, proposal_id, caller)
    }

    /// Update governance configuration
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `caller` - Address making the request (must be owner)
    /// * `config` - New governance configuration
    ///
    /// # Returns
    /// `true` if successful
    pub fn update_governance_config(
        env: Env,
        guild_id: u64,
        caller: Address,
        config: GovernanceConfig,
    ) -> bool {
        gov_update_governance_config(&env, guild_id, caller, config)
    }

    // ============ Bounty Escrow Functions ============

    /// Create a new bounty
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild creating the bounty
    /// * `creator` - Address of the bounty creator (must be guild admin/owner)
    /// * `title` - Short title for the bounty
    /// * `description` - Detailed description of the task
    /// * `reward_amount` - Amount of tokens as reward
    /// * `token` - Address of the token contract
    /// * `expiry` - Absolute timestamp when the bounty expires
    ///
    /// # Returns
    /// The ID of the newly created bounty
    pub fn create_bounty(
        env: Env,
        guild_id: u64,
        creator: Address,
        title: String,
        description: String,
        reward_amount: i128,
        token: Address,
        expiry: u64,
    ) -> u64 {
        create_bounty(
            &env,
            guild_id,
            creator,
            title,
            description,
            reward_amount,
            token,
            expiry,
        )
    }

    /// Fund a bounty with tokens
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to fund
    /// * `funder` - Address providing the funds
    /// * `amount` - Amount of tokens to fund
    ///
    /// # Returns
    /// `true` if funding was successful
    pub fn fund_bounty(env: Env, bounty_id: u64, funder: Address, amount: i128) -> bool {
        fund_bounty(&env, bounty_id, funder, amount)
    }

    /// Claim a bounty (first-come-first-served)
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to claim
    /// * `claimer` - Address of the contributor claiming the bounty
    ///
    /// # Returns
    /// `true` if claiming was successful
    pub fn claim_bounty(env: Env, bounty_id: u64, claimer: Address) -> bool {
        claim_bounty(&env, bounty_id, claimer)
    }

    /// Submit work for a claimed bounty
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty
    /// * `submission_url` - URL or reference to the submitted work
    ///
    /// # Returns
    /// `true` if submission was successful
    pub fn submit_work(env: Env, bounty_id: u64, submission_url: String) -> bool {
        submit_work(&env, bounty_id, submission_url)
    }

    /// Approve completion of a bounty
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to approve
    /// * `approver` - Address of the approver (must be guild admin/owner)
    ///
    /// # Returns
    /// `true` if approval was successful
    pub fn approve_completion(env: Env, bounty_id: u64, approver: Address) -> bool {
        approve_completion(&env, bounty_id, approver)
    }

    /// Release escrow funds to the bounty claimer
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the completed bounty
    ///
    /// # Returns
    /// `true` if release was successful
    pub fn release_escrow(env: Env, bounty_id: u64) -> bool {
        release_escrow(&env, bounty_id)
    }

    /// Cancel a bounty and refund escrowed funds
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to cancel
    /// * `canceller` - Address attempting to cancel (must be creator or guild admin)
    ///
    /// # Returns
    /// `true` if cancellation was successful
    pub fn cancel_bounty(env: Env, bounty_id: u64, canceller: Address) -> bool {
        cancel_bounty(&env, bounty_id, canceller)
    }

    /// Handle expired bounty - refund funds and update status
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to check/expire
    ///
    /// # Returns
    /// `true` if bounty was expired and refunded
    pub fn expire_bounty(env: Env, bounty_id: u64) -> bool {
        expire_bounty(&env, bounty_id)
    }

    /// Get bounty by ID
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty
    ///
    /// # Returns
    /// The Bounty struct
    pub fn get_bounty(env: Env, bounty_id: u64) -> Bounty {
        get_bounty_data(&env, bounty_id)
    }

    /// Get all bounties for a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    ///
    /// # Returns
    /// Vector of all bounties belonging to the guild
    pub fn get_guild_bounties(env: Env, guild_id: u64) -> Vec<Bounty> {
        get_guild_bounties_list(&env, guild_id)
    }

    // ════════════════════════════════════════════════════════════════════════
    //  Multi-Signature Framework
    //  Provides M-of-N signing, configurable policies, and emergency controls.
    // ════════════════════════════════════════════════════════════════════════

    /// Register a new multi-signature safe account.
    pub fn ms_register_account(
        env: Env,
        owner: Address,
        signers: Vec<Address>,
        threshold: u32,
        guild_id: Option<u64>,
        timeout_seconds: u64,
    ) -> u64 {
        match internal_register_account(&env, owner, signers, threshold, guild_id, timeout_seconds)
        {
            Ok(id) => id,
            Err(e) => panic!("ms_register_account error: {}", e as u32),
        }
    }

    /// Add a new signer to a multi-sig account (owner only).
    pub fn ms_add_signer(env: Env, account_id: u64, new_signer: Address, caller: Address) -> bool {
        match internal_add_signer(&env, account_id, new_signer, caller) {
            Ok(()) => true,
            Err(e) => panic!("ms_add_signer error: {}", e as u32),
        }
    }

    /// Remove a signer from a multi-sig account (owner only).
    pub fn ms_remove_signer(
        env: Env,
        account_id: u64,
        signer: Address,
        caller: Address,
        new_threshold: u32,
    ) -> bool {
        match internal_remove_signer(&env, account_id, signer, caller, new_threshold) {
            Ok(()) => true,
            Err(e) => panic!("ms_remove_signer error: {}", e as u32),
        }
    }

    /// Atomically replace a compromised signer key with a new one (owner only).
    pub fn ms_rotate_signer(
        env: Env,
        account_id: u64,
        old_signer: Address,
        new_signer: Address,
        caller: Address,
    ) -> bool {
        match internal_rotate_signer(&env, account_id, old_signer, new_signer, caller) {
            Ok(()) => true,
            Err(e) => panic!("ms_rotate_signer error: {}", e as u32),
        }
    }

    /// Update the signing threshold for an account (owner only).
    pub fn ms_update_threshold(
        env: Env,
        account_id: u64,
        new_threshold: u32,
        caller: Address,
    ) -> bool {
        match internal_update_threshold(&env, account_id, new_threshold, caller) {
            Ok(()) => true,
            Err(e) => panic!("ms_update_threshold error: {}", e as u32),
        }
    }

    /// Freeze a multi-sig account, blocking all new operations (owner only).
    pub fn ms_freeze_account(env: Env, account_id: u64, caller: Address) -> bool {
        match internal_freeze_account(&env, account_id, caller) {
            Ok(()) => true,
            Err(e) => panic!("ms_freeze_account error: {}", e as u32),
        }
    }

    /// Unfreeze a previously frozen account (owner only).
    pub fn ms_unfreeze_account(env: Env, account_id: u64, caller: Address) -> bool {
        match internal_unfreeze_account(&env, account_id, caller) {
            Ok(()) => true,
            Err(e) => panic!("ms_unfreeze_account error: {}", e as u32),
        }
    }

    /// Retrieve a multi-sig account by ID.
    pub fn ms_get_account(env: Env, account_id: u64) -> MultiSigAccount {
        match internal_get_safe_account(&env, account_id) {
            Ok(a) => a,
            Err(e) => panic!("ms_get_account error: {}", e as u32),
        }
    }

    /// List all multi-sig accounts owned by a given address.
    pub fn ms_list_accounts(env: Env, owner: Address) -> Vec<MultiSigAccount> {
        internal_list_accounts_by_owner(&env, owner)
    }

    // ─── Multi-Sig Operations ─────────────────────────────────────────────

    /// Propose a new operation requiring multi-sig approval.
    pub fn ms_propose_operation(
        env: Env,
        account_id: u64,
        operation_type: OperationType,
        description: String,
        proposer: Address,
    ) -> u64 {
        match internal_propose_operation(&env, account_id, operation_type, description, proposer) {
            Ok(id) => id,
            Err(e) => panic!("ms_propose_operation error: {}", e as u32),
        }
    }

    /// Submit a signature for a pending operation.
    pub fn ms_sign_operation(env: Env, operation_id: u64, signer: Address) -> u32 {
        match internal_sign_operation(&env, operation_id, signer) {
            Ok(n) => n,
            Err(e) => panic!("ms_sign_operation error: {}", e as u32),
        }
    }

    /// Execute a fully-signed operation.
    pub fn ms_execute_operation(env: Env, operation_id: u64, executor: Address) -> bool {
        match internal_execute_operation(&env, operation_id, executor) {
            Ok(()) => true,
            Err(e) => panic!("ms_execute_operation error: {}", e as u32),
        }
    }

    /// Execute a treasury withdrawal proposal only after a multisig treasury operation is executed.
    /// This preserves backward compatibility while enabling strict multisig-gated flows.
    pub fn ms_propose_treasury_withdrawal(
        env: Env,
        multisig_operation_id: u64,
        treasury_id: u64,
        proposer: Address,
        recipient: Address,
        amount: i128,
        token: Option<Address>,
        reason: String,
    ) -> u64 {
        if let Err(e) = internal_require_executed_operation(
            &env,
            multisig_operation_id,
            OperationType::TreasuryWithdrawal,
        ) {
            panic!("ms_propose_treasury_withdrawal gate error: {}", e as u32);
        }

        core_propose_withdrawal(
            &env,
            treasury_id,
            proposer,
            recipient,
            amount,
            token,
            reason,
        )
    }

    /// Execute a governance proposal only after an executed governance multisig operation.
    pub fn ms_execute_governance_proposal(
        env: Env,
        multisig_operation_id: u64,
        proposal_id: u64,
        executor: Address,
    ) -> bool {
        if let Err(e) = internal_require_executed_operation(
            &env,
            multisig_operation_id,
            OperationType::GovernanceUpdate,
        ) {
            panic!("ms_execute_governance_proposal gate error: {}", e as u32);
        }
        gov_execute_proposal(&env, proposal_id, executor)
    }

    /// Cancel a pending operation (proposer or account owner only).
    pub fn ms_cancel_operation(env: Env, operation_id: u64, caller: Address) -> bool {
        match internal_cancel_operation(&env, operation_id, caller) {
            Ok(()) => true,
            Err(e) => panic!("ms_cancel_operation error: {}", e as u32),
        }
    }

    /// Lazily check and mark a single operation as expired if its timeout passed.
    pub fn ms_check_and_expire(env: Env, operation_id: u64) -> bool {
        match internal_check_and_expire(&env, operation_id) {
            Ok(expired) => expired,
            Err(e) => panic!("ms_check_and_expire error: {}", e as u32),
        }
    }

    /// Sweep all pending operations for an account and expire timed-out ones.
    pub fn ms_sweep_expired(env: Env, account_id: u64) -> u32 {
        internal_sweep_expired_operations(&env, account_id)
    }

    /// Retrieve the full state of an operation (with lazy expiry applied).
    pub fn ms_get_operation(env: Env, operation_id: u64) -> MultiSigOperation {
        match internal_get_operation_status(&env, operation_id) {
            Ok(op) => op,
            Err(e) => panic!("ms_get_operation error: {}", e as u32),
        }
    }

    /// List all currently pending (non-expired) operations for an account.
    pub fn ms_get_pending_ops(env: Env, account_id: u64) -> Vec<MultiSigOperation> {
        internal_get_pending_operations(&env, account_id)
    }

    // ─── Emergency Controls ───────────────────────────────────────────────

    /// Extend or shorten the expiry of a pending operation (owner only).
    pub fn ms_emergency_extend_timeout(
        env: Env,
        operation_id: u64,
        new_timeout_seconds: u64,
        owner: Address,
    ) -> bool {
        match internal_emergency_extend_timeout(&env, operation_id, new_timeout_seconds, owner) {
            Ok(()) => true,
            Err(e) => panic!("ms_emergency_extend_timeout error: {}", e as u32),
        }
    }

    /// Immediately expire a pending operation (owner-only kill-switch).
    pub fn ms_emergency_expire(env: Env, operation_id: u64, owner: Address) -> bool {
        match internal_emergency_expire_operation(&env, operation_id, owner) {
            Ok(()) => true,
            Err(e) => panic!("ms_emergency_expire error: {}", e as u32),
        }
    }

    // ─── Policy Management ────────────────────────────────────────────────

    /// Set a custom signing policy for a specific operation type (owner only).
    pub fn ms_set_policy(
        env: Env,
        account_id: u64,
        operation_type: OperationType,
        min_signatures: u32,
        require_all_signers: bool,
        timeout_seconds: u64,
        require_owner_signature: bool,
        caller: Address,
    ) -> bool {
        match internal_set_operation_policy(
            &env,
            account_id,
            operation_type,
            min_signatures,
            require_all_signers,
            timeout_seconds,
            require_owner_signature,
            caller,
        ) {
            Ok(()) => true,
            Err(e) => panic!("ms_set_policy error: {}", e as u32),
        }
    }

    /// Get the effective policy for an operation type (returns default if none set).
    pub fn ms_get_policy(
        env: Env,
        account_id: u64,
        operation_type: OperationType,
    ) -> OperationPolicy {
        internal_get_operation_policy(&env, account_id, operation_type)
    }

    /// Reset a policy back to its secure defaults (owner only).
    pub fn ms_reset_policy(
        env: Env,
        account_id: u64,
        operation_type: OperationType,
        caller: Address,
    ) -> bool {
        match internal_reset_operation_policy(&env, account_id, operation_type, caller) {
            Ok(()) => true,
            Err(e) => panic!("ms_reset_policy error: {}", e as u32),
        }
    }

    // ============ Subscription Functions ============

    /// Create a new subscription plan
    ///
    /// # Arguments
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
    pub fn create_subscription_plan(
        env: Env,
        guild_id: u64,
        name: String,
        description: String,
        tier: MembershipTier,
        price: i128,
        token: Option<Address>,
        billing_cycle: BillingCycle,
        benefits: Vec<String>,
        created_by: Address,
    ) -> u64 {
        created_by.require_auth();
        match sub_create_plan(
            &env,
            guild_id,
            name,
            description,
            tier,
            price,
            token,
            billing_cycle,
            benefits,
            created_by,
        ) {
            Ok(id) => id,
            Err(e) => panic!("create_plan error: {}", e as u32),
        }
    }

    /// Subscribe to a plan
    ///
    /// # Arguments
    /// * `plan_id` - ID of the plan to subscribe to
    /// * `subscriber` - Address subscribing
    /// * `auto_renew` - Whether to auto-renew
    ///
    /// # Returns
    /// The ID of the newly created subscription
    pub fn subscribe(env: Env, plan_id: u64, subscriber: Address, auto_renew: bool) -> u64 {
        subscriber.require_auth();
        match sub_subscribe(&env, plan_id, subscriber, auto_renew) {
            Ok(id) => id,
            Err(e) => panic!("subscribe error: {}", e as u32),
        }
    }

    /// Process a subscription payment
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    ///
    /// # Returns
    /// true if payment was successful
    pub fn process_subscription_payment(env: Env, subscription_id: u64) -> bool {
        match sub_process_payment(&env, subscription_id, 0) {
            Ok(result) => result,
            Err(e) => panic!("process_payment error: {}", e as u32),
        }
    }

    /// Retry a failed payment
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    ///
    /// # Returns
    /// true if payment was successful
    pub fn retry_subscription_payment(env: Env, subscription_id: u64) -> bool {
        match sub_retry_payment(&env, subscription_id) {
            Ok(result) => result,
            Err(e) => panic!("retry_payment error: {}", e as u32),
        }
    }

    /// Pause a subscription
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    /// * `caller` - Address making the request
    ///
    /// # Returns
    /// true if successful
    pub fn pause_subscription(env: Env, subscription_id: u64, caller: Address) -> bool {
        caller.require_auth();
        match sub_pause_subscription(&env, subscription_id, caller) {
            Ok(result) => result,
            Err(e) => panic!("pause_subscription error: {}", e as u32),
        }
    }

    /// Resume a paused subscription
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    /// * `caller` - Address making the request
    ///
    /// # Returns
    /// true if successful
    pub fn resume_subscription(env: Env, subscription_id: u64, caller: Address) -> bool {
        caller.require_auth();
        match sub_resume_subscription(&env, subscription_id, caller) {
            Ok(result) => result,
            Err(e) => panic!("resume_subscription error: {}", e as u32),
        }
    }

    /// Cancel a subscription
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    /// * `caller` - Address making the request
    /// * `reason` - Optional cancellation reason
    ///
    /// # Returns
    /// true if successful
    pub fn cancel_subscription(
        env: Env,
        subscription_id: u64,
        caller: Address,
        reason: Option<String>,
    ) -> bool {
        caller.require_auth();
        match sub_cancel_subscription(&env, subscription_id, caller, reason) {
            Ok(result) => result,
            Err(e) => panic!("cancel_subscription error: {}", e as u32),
        }
    }

    /// Change subscription tier (upgrade/downgrade)
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    /// * `new_plan_id` - New plan ID
    /// * `effective_immediately` - Whether to apply immediately or at next cycle
    /// * `caller` - Address making the request
    ///
    /// # Returns
    /// Proration amount (0 if no proration)
    pub fn change_subscription_tier(
        env: Env,
        subscription_id: u64,
        new_plan_id: u64,
        effective_immediately: bool,
        caller: Address,
    ) -> i128 {
        caller.require_auth();
        let change = SubscriptionChange {
            new_plan_id,
            effective_immediately,
            reason: None,
        };
        match sub_change_tier(&env, subscription_id, change, caller) {
            Ok(proration) => proration.map(|p| p.amount).unwrap_or(0),
            Err(e) => panic!("change_tier error: {}", e as u32),
        }
    }

    /// Get subscription status
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    ///
    /// # Returns
    /// Subscription details
    pub fn get_subscription(env: Env, subscription_id: u64) -> Subscription {
        match sub_get_subscription_status(&env, subscription_id) {
            Some(sub) => sub,
            None => panic!("subscription not found"),
        }
    }

    /// Check if a subscription is active
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    ///
    /// # Returns
    /// true if subscription is active
    pub fn is_subscription_active(env: Env, subscription_id: u64) -> bool {
        sub_is_subscription_active(&env, subscription_id)
    }

    /// Get days until next billing
    ///
    /// # Arguments
    /// * `subscription_id` - ID of the subscription
    ///
    /// # Returns
    /// Days until next billing (0 if past due)
    pub fn days_until_billing(env: Env, subscription_id: u64) -> u64 {
        sub_days_until_billing(&env, subscription_id)
    }

    /// Process due subscriptions (can be called by anyone)
    ///
    /// # Arguments
    /// * `limit` - Maximum number of subscriptions to process
    ///
    /// # Returns
    /// Number of subscriptions processed
    pub fn process_due_subscriptions(env: Env, limit: u32) -> u32 {
        sub_process_due_subscriptions(&env, limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.budget().reset_unlimited();

        let owner = Address::generate(&env);
        let admin = Address::generate(&env);
        let member = Address::generate(&env);
        let non_member = Address::generate(&env);

        (env, owner, admin, member, non_member)
    }

    fn register_and_init_contract(env: &Env) -> Address {
        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(env, &contract_id);

        client.initialize();

        contract_id
    }

    // ============ Initialization Tests ============

    #[test]
    fn test_initialize() {
        let (env, _, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);

        // Verify initialization was successful
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let result = client.initialize();
        assert_eq!(result, true);
    }

    #[test]
    fn test_version() {
        let (env, _, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);

        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let version = client.version();
        assert_eq!(version, String::from_str(&env, "0.1.0"));
    }

    // ============ Guild Creation Tests ============

    #[test]
    fn test_create_guild_success() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Test Guild");
        let description = String::from_str(&env, "A test guild");

        let guild_id = client.create_guild(&name, &description, &owner);
        assert_eq!(guild_id, 1u64);
    }

    #[test]
    fn test_create_guild_owner_is_member() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Owner should be a member after creation
        let is_member = client.is_member(&guild_id, &owner);
        assert_eq!(is_member, true);

        let member = client.get_member(&guild_id, &owner);
        assert_eq!(member.role, Role::Owner);
    }

    #[test]
    #[should_panic]
    fn test_create_guild_invalid_name_empty() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "");
        let description = String::from_str(&env, "Description");

        client.create_guild(&name, &description, &owner);
    }

    #[test]
    #[should_panic]
    fn test_create_guild_invalid_description_too_long() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        // Create a description that is too long (> 512 chars)
        let long_desc = "x".repeat(513);
        let description = String::from_str(&env, &long_desc);

        client.create_guild(&name, &description, &owner);
    }

    #[test]
    fn test_create_multiple_guilds() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name1 = String::from_str(&env, "Guild 1");
        let description1 = String::from_str(&env, "First guild");

        let guild_id_1 = client.create_guild(&name1, &description1, &owner);

        let name2 = String::from_str(&env, "Guild 2");
        let description2 = String::from_str(&env, "Second guild");

        let guild_id_2 = client.create_guild(&name2, &description2, &owner);

        // Guild IDs should be unique and incremental
        assert_eq!(guild_id_1, 1u64);
        assert_eq!(guild_id_2, 2u64);
    }

    // ============ Member Addition Tests ============

    #[test]
    fn test_add_member_by_owner() {
        let (env, owner, admin, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Owner adds admin
        let result = client.add_member(&guild_id, &admin, &Role::Admin, &owner);
        assert_eq!(result, true);

        let member = client.get_member(&guild_id, &admin);
        assert_eq!(member.role, Role::Admin);
    }

    #[test]
    #[should_panic]
    fn test_add_member_duplicate() {
        let (env, owner, admin, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member once
        client.add_member(&guild_id, &admin, &Role::Member, &owner);

        // Try to add same member again - should panic
        client.add_member(&guild_id, &admin, &Role::Member, &owner);
    }

    #[test]
    #[should_panic]
    fn test_add_member_permission_denied() {
        let (env, owner, admin, member, non_member) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Non-member tries to add someone - should panic
        client.add_member(&guild_id, &non_member, &Role::Member, &non_member);
    }

    #[test]
    #[should_panic]
    fn test_add_admin_by_non_owner() {
        let (env, owner, _admin, member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Member tries to add an owner - should panic
        let new_owner = Address::generate(&env);
        env.mock_all_auths();

        client.add_member(&guild_id, &new_owner, &Role::Owner, &member);
    }

    // ============ Member Removal Tests ============

    #[test]
    fn test_remove_member_by_owner() {
        let (env, owner, member, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Verify member exists
        let is_member = client.is_member(&guild_id, &member);
        assert_eq!(is_member, true);

        // Remove member
        let result = client.remove_member(&guild_id, &member, &owner);
        assert_eq!(result, true);

        // Verify member no longer exists
        let is_member = client.is_member(&guild_id, &member);
        assert_eq!(is_member, false);
    }

    #[test]
    fn test_self_removal() {
        let (env, owner, member, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Member removes themselves
        let result = client.remove_member(&guild_id, &member, &member);
        assert_eq!(result, true);

        // Verify member no longer exists
        let is_member = client.is_member(&guild_id, &member);
        assert_eq!(is_member, false);
    }

    #[test]
    #[should_panic]
    fn test_remove_last_owner_fails() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Try to remove the only owner - should panic
        client.remove_member(&guild_id, &owner, &owner);
    }

    #[test]
    #[should_panic]
    fn test_remove_non_owner_by_non_owner_fails() {
        let (env, owner, admin, member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member and admin
        client.add_member(&guild_id, &member, &Role::Member, &owner);
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Member tries to remove admin - should panic
        client.remove_member(&guild_id, &admin, &member);
    }

    // ============ Role Update Tests ============

    #[test]
    fn test_update_role_by_owner() {
        let (env, owner, member, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Update to admin
        let result = client.update_role(&guild_id, &member, &Role::Admin, &owner);
        assert_eq!(result, true);

        let updated_member = client.get_member(&guild_id, &member);
        assert_eq!(updated_member.role, Role::Admin);
    }

    #[test]
    #[should_panic]
    fn test_update_role_permission_denied() {
        let (env, owner, member1, member2, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add members
        client.add_member(&guild_id, &member1, &Role::Member, &owner);
        client.add_member(&guild_id, &member2, &Role::Member, &owner);

        // Member1 tries to change member2's role - should panic
        client.update_role(&guild_id, &member2, &Role::Admin, &member1);
    }

    #[test]
    #[should_panic]
    fn test_cannot_demote_last_owner() {
        let (env, owner, admin, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Try to demote the last owner - should panic
        client.update_role(&guild_id, &owner, &Role::Admin, &owner);
    }

    #[test]
    fn test_can_demote_owner_if_multiple() {
        let (env, owner1, owner2, _member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner1);

        // Add owner2
        client.add_member(&guild_id, &owner2, &Role::Owner, &owner1);

        // Now owner1 can be demoted
        let result = client.update_role(&guild_id, &owner1, &Role::Admin, &owner1);
        assert_eq!(result, true);
    }

    // ============ Member Query Tests ============

    #[test]
    fn test_get_member() {
        let (env, owner, member, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        client.add_member(&guild_id, &member, &Role::Member, &owner);

        let member_data = client.get_member(&guild_id, &member);
        assert_eq!(member_data.address, member);
        assert_eq!(member_data.role, Role::Member);
    }

    #[test]
    #[should_panic]
    fn test_get_member_not_found() {
        let (env, owner, member, non_member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        client.add_member(&guild_id, &member, &Role::Member, &owner);

        client.get_member(&guild_id, &non_member);
    }

    #[test]
    fn test_get_all_members() {
        let (env, owner, member1, member2, member3) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Initially should have 1 member (owner)
        let members = client.get_all_members(&guild_id);
        assert_eq!(members.len(), 1);

        // Add more members
        client.add_member(&guild_id, &member1, &Role::Member, &owner);
        client.add_member(&guild_id, &member2, &Role::Admin, &owner);
        client.add_member(&guild_id, &member3, &Role::Contributor, &owner);

        // Should now have 4 members
        let members = client.get_all_members(&guild_id);
        assert_eq!(members.len(), 4);
    }

    #[test]
    fn test_is_member() {
        let (env, owner, member, non_member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        assert_eq!(client.is_member(&guild_id, &owner), true);
        assert_eq!(client.is_member(&guild_id, &member), false);

        client.add_member(&guild_id, &member, &Role::Member, &owner);
        assert_eq!(client.is_member(&guild_id, &member), true);
        assert_eq!(client.is_member(&guild_id, &non_member), false);
    }

    // ============ Permission Tests ============

    #[test]
    fn test_has_permission() {
        let (env, owner, admin, member, contributor) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        client.add_member(&guild_id, &admin, &Role::Admin, &owner);
        client.add_member(&guild_id, &member, &Role::Member, &owner);
        client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

        // Owner has all permissions
        assert_eq!(client.has_permission(&guild_id, &owner, &Role::Owner), true);
        assert_eq!(client.has_permission(&guild_id, &owner, &Role::Admin), true);
        assert_eq!(
            client.has_permission(&guild_id, &owner, &Role::Member),
            true
        );
        assert_eq!(
            client.has_permission(&guild_id, &owner, &Role::Contributor),
            true
        );

        // Admin has admin and below permissions
        assert_eq!(
            client.has_permission(&guild_id, &admin, &Role::Owner),
            false
        );
        assert_eq!(client.has_permission(&guild_id, &admin, &Role::Admin), true);
        assert_eq!(
            client.has_permission(&guild_id, &admin, &Role::Member),
            true
        );
        assert_eq!(
            client.has_permission(&guild_id, &admin, &Role::Contributor),
            true
        );

        // Member has member and below permissions
        assert_eq!(
            client.has_permission(&guild_id, &member, &Role::Owner),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &member, &Role::Admin),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &member, &Role::Member),
            true
        );
        assert_eq!(
            client.has_permission(&guild_id, &member, &Role::Contributor),
            true
        );

        // Contributor has only contributor permissions
        assert_eq!(
            client.has_permission(&guild_id, &contributor, &Role::Owner),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &contributor, &Role::Admin),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &contributor, &Role::Member),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &contributor, &Role::Contributor),
            true
        );
    }

    // ============ Guild Lifecycle Integration Tests ============

    #[test]
    fn test_full_guild_lifecycle() {
        let (env, owner, admin, member1, member2) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Create guild
        let name = String::from_str(&env, "Community Guild");
        let description = String::from_str(&env, "A thriving community");

        let guild_id = client.create_guild(&name, &description, &owner);
        assert_eq!(guild_id, 1u64);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Add members
        client.add_member(&guild_id, &member1, &Role::Member, &admin);
        client.add_member(&guild_id, &member2, &Role::Contributor, &owner);

        // Verify all members exist
        let members = client.get_all_members(&guild_id);
        assert_eq!(members.len(), 4);

        // Promote member1 to member
        client.update_role(&guild_id, &member1, &Role::Member, &admin);

        // member1 removes themselves
        client.remove_member(&guild_id, &member1, &member1);

        // Verify member1 is gone
        let members = client.get_all_members(&guild_id);
        assert_eq!(members.len(), 3);

        assert_eq!(client.is_member(&guild_id, &member1), false);
        assert_eq!(client.is_member(&guild_id, &member2), true);
    }

    #[test]
    fn test_admin_can_add_members_and_contributors() {
        let (env, owner, admin, member, contributor) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Admin adds member and contributor
        let result1 = client.add_member(&guild_id, &member, &Role::Member, &admin);
        assert_eq!(result1, true);

        let result2 = client.add_member(&guild_id, &contributor, &Role::Contributor, &admin);
        assert_eq!(result2, true);

        // Verify they were added
        assert_eq!(client.is_member(&guild_id, &member), true);
        assert_eq!(client.is_member(&guild_id, &contributor), true);
    }

    #[test]
    #[should_panic]
    fn test_admin_cannot_add_owner() {
        let (env, owner, admin, new_owner, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Admin tries to add owner - should panic
        client.add_member(&guild_id, &new_owner, &Role::Owner, &admin);
    }

    // ============ Payment Distribution Tests ============

    #[test]
    fn test_create_payment_pool_percentage() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
        assert_eq!(pool_id, 1u64);
    }

    #[test]
    fn test_add_recipient_and_validate() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let recipient1 = Address::generate(&env);
        let recipient2 = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

        client.add_recipient(&pool_id, &recipient1, &50u32, &creator);
        client.add_recipient(&pool_id, &recipient2, &50u32, &creator);

        let is_valid = client.validate_distribution(&pool_id);
        assert_eq!(is_valid, true);
    }

    #[test]
    fn test_get_recipient_amount_percentage() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

        client.add_recipient(&pool_id, &recipient, &25u32, &creator);

        let amount = client.get_recipient_amount(&pool_id, &recipient);
        assert_eq!(amount, 250i128);
    }

    #[test]
    fn test_equal_split_distribution() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let recipient1 = Address::generate(&env);
        let recipient2 = Address::generate(&env);
        let recipient3 = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::EqualSplit, &creator);

        client.add_recipient(&pool_id, &recipient1, &1u32, &creator);
        client.add_recipient(&pool_id, &recipient2, &1u32, &creator);
        client.add_recipient(&pool_id, &recipient3, &1u32, &creator);

        let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
        let amount2 = client.get_recipient_amount(&pool_id, &recipient2);
        let amount3 = client.get_recipient_amount(&pool_id, &recipient3);

        assert_eq!(amount1, 333i128);
        assert_eq!(amount2, 333i128);
        assert_eq!(amount3, 333i128);
    }

    #[test]
    fn test_cancel_distribution() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

        let result = client.cancel_distribution(&pool_id, &creator);
        assert_eq!(result, true);

        let status = client.get_pool_status(&pool_id);
        assert_eq!(status, DistributionStatus::Cancelled);
    }
}
