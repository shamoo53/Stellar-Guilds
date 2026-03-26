/// Bounty Escrow Module
///
/// This module handles bounty creation, funding, claiming, and escrow management
/// for the Stellar Guilds platform.
///
/// # Events emitted
/// All events follow the `(module, action)` topic convention defined in
/// `crate::events::topics`. The envelope (version, timestamp, sequence) is
/// automatically attached by `emit_event()`.
///
/// | Action              | Topic                    | Payload struct           |
/// |---------------------|--------------------------|--------------------------|
/// | Create bounty       | `(bounty, created)`      | `BountyCreatedEvent`     |
/// | Fund bounty         | `(bounty, funded)`       | `BountyFundedEvent`      |
/// | Claim bounty        | `(bounty, claimed)`      | `BountyClaimedEvent`     |
/// | Submit work         | `(bounty, submitted)`    | `WorkSubmittedEvent`     |
/// | Approve bounty      | `(bounty, approved)`     | `BountyApprovedEvent`    |
/// | Release escrow      | `(bounty, released)`     | `EscrowReleasedEvent`    |
/// | Cancel bounty       | `(bounty, cancelled)`    | `BountyCancelledEvent`   |
/// | Expire bounty       | `(bounty, expired)`      | `BountyExpiredEvent`     |
pub mod escrow;
pub mod storage;
pub mod types;

use crate::bounty::escrow::{lock_funds, release_funds};
use crate::bounty::storage::{get_bounty, get_guild_bounties, get_next_bounty_id, store_bounty};
use crate::bounty::types::{
    BountyApprovedEvent, BountyCancelledEvent, BountyClaimedEvent, BountyCreatedEvent,
    BountyExpiredEvent, BountyFundedEvent, EscrowReleasedEvent, WorkSubmittedEvent,
};
use crate::dispute::storage as dispute_storage;
use crate::dispute::types::DisputeReference;
use crate::events::emit::emit_event;
use crate::events::topics::{
    ACT_APPROVED, ACT_CANCELLED, ACT_CLAIMED, ACT_CREATED, ACT_EXPIRED, ACT_FUNDED, ACT_RELEASED,
    ACT_SUBMITTED, MOD_BOUNTY,
};
use crate::guild::membership::has_permission;
use crate::guild::types::Role;
use soroban_sdk::{Address, Env, String, Vec};

pub use types::{Bounty, BountyStatus};

/// Create a new bounty
///
/// # Events emitted
/// - `(bounty, created)` â†’ `BountyCreatedEvent`
pub fn create_bounty(
    env: &Env,
    guild_id: u64,
    creator: Address,
    title: String,
    description: String,
    reward_amount: i128,
    token: Address,
    expiry: u64,
) -> u64 {
    creator.require_auth();

    if !has_permission(env, guild_id, creator.clone(), Role::Admin) {
        panic!("Unauthorized: Creator must be a guild admin or owner");
    }
    if reward_amount < 0 {
        panic!("Invalid reward amount: must be non-negative");
    }

    let created_at = env.ledger().timestamp();
    if expiry <= created_at {
        panic!("Expiry must be in the future");
    }
    if title.len() == 0 || title.len() > 256 {
        panic!("Title must be between 1 and 256 characters");
    }
    if description.len() > 2048 {
        panic!("Description must be at most 2048 characters");
    }

    let bounty_id = get_next_bounty_id(env);

    let status = if reward_amount == 0 {
        BountyStatus::Open
    } else {
        BountyStatus::AwaitingFunds
    };

    let bounty = Bounty {
        id: bounty_id,
        guild_id,
        creator: creator.clone(),
        title,
        description,
        reward_amount,
        funded_amount: 0,
        token: token.clone(),
        status,
        claimer: None,
        submission_url: None,
        created_at,
        expires_at: expiry,
    };
    store_bounty(env, &bounty);

    emit_event(
        env,
        MOD_BOUNTY,
        ACT_CREATED,
        BountyCreatedEvent {
            bounty_id,
            guild_id,
            creator,
            reward_amount,
            token,
            expires_at: expiry,
        },
    );

    bounty_id
}

/// Fund a bounty with tokens
///
/// # Events emitted
/// - `(bounty, funded)`  â†’ `BountyFundedEvent`
/// - `(bounty, expired)` â†’ `BountyExpiredEvent`  (if bounty found to be expired)
pub fn fund_bounty(env: &Env, bounty_id: u64, funder: Address, amount: i128) -> bool {
    funder.require_auth();

    if amount <= 0 {
        panic!("Amount must be positive");
    }

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    let now = env.ledger().timestamp();
    if now > bounty.expires_at {
        bounty.status = BountyStatus::Expired;
        store_bounty(env, &bounty);
        emit_event(
            env,
            MOD_BOUNTY,
            ACT_EXPIRED,
            BountyExpiredEvent { bounty_id },
        );
        panic!("Bounty has expired");
    }

    match bounty.status {
        BountyStatus::AwaitingFunds | BountyStatus::Open => {}
        _ => panic!("Bounty cannot be funded in current status"),
    }

    lock_funds(env, &bounty.token, &funder, amount);

    bounty.funded_amount += amount;
    let is_fully_funded = bounty.funded_amount >= bounty.reward_amount;

    if is_fully_funded && bounty.status == BountyStatus::AwaitingFunds {
        bounty.status = BountyStatus::Funded;
    }
    store_bounty(env, &bounty);

    emit_event(
        env,
        MOD_BOUNTY,
        ACT_FUNDED,
        BountyFundedEvent {
            bounty_id,
            funder,
            amount,
            total_funded: bounty.funded_amount,
            is_fully_funded,
        },
    );

    true
}

/// Claim a bounty after approval
///
/// # Events emitted
/// - `(bounty, claimed)`  â†’ `BountyClaimedEvent`
/// - `(bounty, expired)`  â†’ `BountyExpiredEvent`  (if found expired during claim)
pub fn claim_bounty(env: &Env, bounty_id: u64, claimer: Address) -> bool {
    claimer.require_auth();

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    let now = env.ledger().timestamp();
    if now > bounty.expires_at {
        bounty.status = BountyStatus::Expired;
        store_bounty(env, &bounty);
        emit_event(
            env,
            MOD_BOUNTY,
            ACT_EXPIRED,
            BountyExpiredEvent { bounty_id },
        );
        panic!("Bounty has expired");
    }

    if bounty.status != BountyStatus::Open {
        panic!("Bounty is not open for claiming");
    }

    match bounty.claimer.clone() {
        Some(approved_claimer) if approved_claimer == claimer => {}
        Some(_) => panic!("Bounty may only be claimed by the approved address"),
        None => {}
    }

    bounty.status = BountyStatus::Claimed;
    bounty.claimer = Some(claimer.clone());
    store_bounty(env, &bounty);

    emit_event(
        env,
        MOD_BOUNTY,
        ACT_CLAIMED,
        BountyClaimedEvent { bounty_id, claimer },
    );

    true
}

/// Submit work for a claimed bounty
///
/// # Events emitted
/// - `(bounty, submitted)` â†’ `WorkSubmittedEvent`
pub fn submit_work(env: &Env, bounty_id: u64, submission_url: String) -> bool {
    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    let claimer = bounty.claimer.clone().expect("No claimer for this bounty");
    claimer.require_auth();

    if bounty.status != BountyStatus::Claimed {
        panic!("Bounty is not in claimed status");
    }
    if submission_url.len() == 0 || submission_url.len() > 512 {
        panic!("Submission URL must be between 1 and 512 characters");
    }

    bounty.status = BountyStatus::UnderReview;
    bounty.submission_url = Some(submission_url.clone());
    store_bounty(env, &bounty);

    emit_event(
        env,
        MOD_BOUNTY,
        ACT_SUBMITTED,
        WorkSubmittedEvent {
            bounty_id,
            claimer,
            submission_url,
        },
    );

    true
}

/// Approve a funded bounty for a specific claimer
///
/// # Events emitted
/// - `(bounty, approved)` → `BountyApprovedEvent`
pub fn approve_bounty(env: &Env, bounty_id: u64, approver: Address, claimer: Address) -> bool {
    approver.require_auth();

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    if !has_permission(env, bounty.guild_id, approver.clone(), Role::Admin) {
        panic!("Unauthorized: Approver must be a guild admin or owner");
    }
    if bounty.status != BountyStatus::Funded {
        panic!("Bounty is not funded");
    }

    bounty.status = BountyStatus::Open;
    bounty.claimer = Some(claimer.clone());
    store_bounty(env, &bounty);

    emit_event(
        env,
        MOD_BOUNTY,
        ACT_APPROVED,
        BountyApprovedEvent {
            bounty_id,
            approver,
            claimer,
        },
    );

    true
}

/// Approve completion of a bounty
pub fn approve_completion(env: &Env, bounty_id: u64, approver: Address) -> bool {
    approver.require_auth();

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    if !has_permission(env, bounty.guild_id, approver.clone(), Role::Admin) {
        panic!("Unauthorized: Approver must be a guild admin or owner");
    }
    if bounty.status != BountyStatus::UnderReview {
        panic!("Bounty is not under review");
    }

    bounty.status = BountyStatus::Completed;
    store_bounty(env, &bounty);

    emit_event(
        env,
        MOD_BOUNTY,
        ACT_APPROVED,
        BountyApprovedEvent {
            bounty_id,
            approver,
            claimer: bounty.claimer.expect("No claimer for this bounty"),
        },
    );

    true
}

/// Release escrow funds to the bounty claimer
///
/// # Events emitted
/// - `(bounty, released)` â†’ `EscrowReleasedEvent`
pub fn release_escrow(env: &Env, bounty_id: u64) -> bool {
    if dispute_storage::is_reference_locked(env, &DisputeReference::Bounty, bounty_id) {
        panic!("Bounty is in active dispute");
    }

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    if bounty.status != BountyStatus::Completed {
        panic!("Bounty is not completed");
    }

    let claimer = bounty.claimer.clone().expect("No claimer for this bounty");

    if bounty.funded_amount > 0 {
        let amount = bounty.funded_amount;
        release_funds(env, &bounty.token, &claimer, amount);
        bounty.funded_amount = 0;
        store_bounty(env, &bounty);

        emit_event(
            env,
            MOD_BOUNTY,
            ACT_RELEASED,
            EscrowReleasedEvent {
                bounty_id,
                recipient: claimer,
                amount,
                token: bounty.token,
            },
        );
    }

    true
}

/// Cancel a bounty and refund escrowed funds to the creator
///
/// # Events emitted
/// - `(bounty, cancelled)` â†’ `BountyCancelledEvent`
pub fn cancel_bounty(env: &Env, bounty_id: u64, canceller: Address) -> bool {
    canceller.require_auth();

    if dispute_storage::is_reference_locked(env, &DisputeReference::Bounty, bounty_id) {
        panic!("Bounty is in active dispute");
    }

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    match bounty.status {
        BountyStatus::Completed | BountyStatus::Cancelled => {
            panic!("Bounty cannot be cancelled in current status");
        }
        _ => {}
    }

    let is_creator = bounty.creator == canceller;
    let is_admin = has_permission(env, bounty.guild_id, canceller.clone(), Role::Admin);

    if !is_creator && !is_admin {
        panic!("Unauthorized: Only creator or guild admin can cancel");
    }

    let refund_amount = bounty.funded_amount;
    let refund_recipient = bounty.creator.clone();

    if refund_amount > 0 {
        release_funds(env, &bounty.token, &refund_recipient, refund_amount);
        bounty.funded_amount = 0;
    }

    bounty.status = BountyStatus::Cancelled;
    store_bounty(env, &bounty);

    emit_event(
        env,
        MOD_BOUNTY,
        ACT_CANCELLED,
        BountyCancelledEvent {
            bounty_id,
            canceller,
            refund_amount,
            refund_recipient,
        },
    );

    true
}

/// Expire a bounty and refund escrowed funds if past its expiry timestamp
///
/// # Events emitted
/// - `(bounty, expired)` â†’ `BountyExpiredEvent`
pub fn expire_bounty(env: &Env, bounty_id: u64) -> bool {
    if dispute_storage::is_reference_locked(env, &DisputeReference::Bounty, bounty_id) {
        panic!("Bounty is in active dispute");
    }

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    if bounty.status == BountyStatus::Expired
        || bounty.status == BountyStatus::Completed
        || bounty.status == BountyStatus::Cancelled
    {
        return false;
    }

    let now = env.ledger().timestamp();
    if now <= bounty.expires_at {
        return false;
    }

    if bounty.funded_amount > 0 {
        release_funds(env, &bounty.token, &bounty.creator, bounty.funded_amount);
        bounty.funded_amount = 0;
    }

    bounty.status = BountyStatus::Expired;
    store_bounty(env, &bounty);

    emit_event(
        env,
        MOD_BOUNTY,
        ACT_EXPIRED,
        BountyExpiredEvent { bounty_id },
    );

    true
}

/// Claim bounty payout - allows claimer to pull funds from escrow to their own address
///
/// This is called by the claimer (assignee) after bounty completion approval.
/// Uses checks-effects-interactions pattern: state is updated first to prevent reentrancy.
///
/// # Events emitted
/// - `(bounty, released)` → `EscrowReleasedEvent`
pub fn claim_payout(env: &Env, bounty_id: u64, claimer: Address) -> bool {
    claimer.require_auth();

    if dispute_storage::is_reference_locked(env, &DisputeReference::Bounty, bounty_id) {
        panic!("Bounty is in active dispute");
    }

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    if bounty.status != BountyStatus::Completed {
        panic!("Bounty is not completed");
    }

    let stored_claimer = bounty.claimer.clone().expect("No claimer for this bounty");
    if stored_claimer != claimer {
        panic!("Unauthorized: Only the approved claimer can claim payout");
    }

    // EFFECTS: Update state FIRST to prevent reentrancy
    let payout_amount = bounty.funded_amount;
    bounty.funded_amount = 0;
    store_bounty(env, &bounty);

    // INTERACTIONS: Only transfer after state is updated
    if payout_amount > 0 {
        release_funds(env, &bounty.token, &claimer, payout_amount);

        emit_event(
            env,
            MOD_BOUNTY,
            ACT_RELEASED,
            EscrowReleasedEvent {
                bounty_id,
                recipient: claimer,
                amount: payout_amount,
                token: bounty.token,
            },
        );
    }

    true
}

// â"€â"€â"€ Query helpers â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€â"€

pub fn get_bounty_data(env: &Env, bounty_id: u64) -> Bounty {
    get_bounty(env, bounty_id).expect("Bounty not found")
}



pub fn get_guild_bounties_list(env: &Env, guild_id: u64) -> Vec<Bounty> {
    get_guild_bounties(env, guild_id)
}

#[allow(dead_code)]
pub fn cancel_bounty_auth(env: &Env, bounty_id: u64, canceller: Address) -> bool {
    cancel_bounty(env, bounty_id, canceller)
}

#[cfg(test)]
mod tests;
