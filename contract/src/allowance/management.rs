use soroban_sdk::{Address, Env, Symbol, Vec};

use super::storage;
use super::types::{
    AllowanceApprovedEvent, AllowanceError, AllowanceOperation, AllowanceRevokedEvent,
    AllowanceSpentEvent, TokenAllowance,
};

// ── Approve ──────────────────────────────────────────────────────────────────

/// Create or replace an allowance from `owner` to `spender`.
///
/// Requires `owner` authorization. If an existing allowance exists for this
/// (owner, spender, token) triple, it is replaced entirely.
pub fn approve(
    env: &Env,
    owner: Address,
    spender: Address,
    token: Option<Address>,
    amount: i128,
    expires_at: u64,
    operation: AllowanceOperation,
) -> Result<(), AllowanceError> {
    owner.require_auth();

    if amount <= 0 {
        return Err(AllowanceError::InvalidAmount);
    }

    let now = env.ledger().timestamp();
    if expires_at > 0 && expires_at <= now {
        return Err(AllowanceError::Expired);
    }

    let allowance = TokenAllowance {
        owner: owner.clone(),
        spender: spender.clone(),
        token: token.clone(),
        amount,
        spent: 0,
        expires_at,
        operation: operation.clone(),
        created_at: now,
    };

    storage::store_allowance(env, &allowance);

    let event = AllowanceApprovedEvent {
        owner,
        spender,
        token,
        amount,
        expires_at,
        operation,
    };
    env.events().publish(
        (Symbol::new(env, "allowance"), Symbol::new(env, "approved")),
        event,
    );

    Ok(())
}

// ── Increase / Decrease ──────────────────────────────────────────────────────

/// Atomically increase an existing allowance.
pub fn increase_allowance(
    env: &Env,
    owner: Address,
    spender: Address,
    token: Option<Address>,
    delta: i128,
) -> Result<(), AllowanceError> {
    owner.require_auth();

    if delta <= 0 {
        return Err(AllowanceError::InvalidAmount);
    }

    let mut allowance =
        storage::get_allowance(env, &owner, &spender, &token).ok_or(AllowanceError::NotFound)?;

    let now = env.ledger().timestamp();
    if allowance.is_expired(now) {
        return Err(AllowanceError::Expired);
    }

    allowance.amount = allowance.amount.saturating_add(delta);
    storage::store_allowance(env, &allowance);

    let event = AllowanceApprovedEvent {
        owner: allowance.owner,
        spender: allowance.spender,
        token: allowance.token,
        amount: allowance.amount,
        expires_at: allowance.expires_at,
        operation: allowance.operation,
    };
    env.events().publish(
        (Symbol::new(env, "allowance"), Symbol::new(env, "approved")),
        event,
    );

    Ok(())
}

/// Atomically decrease an existing allowance. Floors at `spent` (cannot go
/// below what has already been consumed).
pub fn decrease_allowance(
    env: &Env,
    owner: Address,
    spender: Address,
    token: Option<Address>,
    delta: i128,
) -> Result<(), AllowanceError> {
    owner.require_auth();

    if delta <= 0 {
        return Err(AllowanceError::InvalidAmount);
    }

    let mut allowance =
        storage::get_allowance(env, &owner, &spender, &token).ok_or(AllowanceError::NotFound)?;

    let now = env.ledger().timestamp();
    if allowance.is_expired(now) {
        return Err(AllowanceError::Expired);
    }

    // Floor: amount can't go below what's already spent
    let new_amount = allowance.amount.saturating_sub(delta);
    allowance.amount = if new_amount < allowance.spent {
        allowance.spent
    } else {
        new_amount
    };

    storage::store_allowance(env, &allowance);

    let event = AllowanceApprovedEvent {
        owner: allowance.owner,
        spender: allowance.spender,
        token: allowance.token,
        amount: allowance.amount,
        expires_at: allowance.expires_at,
        operation: allowance.operation,
    };
    env.events().publish(
        (Symbol::new(env, "allowance"), Symbol::new(env, "approved")),
        event,
    );

    Ok(())
}

// ── Revoke ───────────────────────────────────────────────────────────────────

/// Revoke (delete) an allowance. Requires `owner` auth.
pub fn revoke(
    env: &Env,
    owner: Address,
    spender: Address,
    token: Option<Address>,
) -> Result<(), AllowanceError> {
    owner.require_auth();

    let existing = storage::get_allowance(env, &owner, &spender, &token);
    if existing.is_none() {
        return Err(AllowanceError::NotFound);
    }

    storage::delete_allowance(env, &owner, &spender, &token);

    let event = AllowanceRevokedEvent {
        owner,
        spender,
        token,
    };
    env.events().publish(
        (Symbol::new(env, "allowance"), Symbol::new(env, "revoked")),
        event,
    );

    Ok(())
}

// ── Spend ────────────────────────────────────────────────────────────────────

/// Internal: consume `amount` from the allowance.
///
/// Validates expiry, operation type, and remaining balance.
/// Called by treasury execution paths.
pub fn spend(
    env: &Env,
    spender: &Address,
    owner: &Address,
    token: &Option<Address>,
    amount: i128,
    op_type: &AllowanceOperation,
) -> Result<(), AllowanceError> {
    if amount <= 0 {
        return Ok(());
    }

    let mut allowance =
        storage::get_allowance(env, owner, spender, token).ok_or(AllowanceError::NotFound)?;

    // Check expiry
    let now = env.ledger().timestamp();
    if allowance.is_expired(now) {
        return Err(AllowanceError::Expired);
    }

    // Check operation type
    if !allowance.permits_operation(op_type) {
        return Err(AllowanceError::OperationNotPermitted);
    }

    // Check remaining balance
    if allowance.remaining() < amount {
        return Err(AllowanceError::InsufficientAllowance);
    }

    allowance.spent = allowance.spent.saturating_add(amount);
    storage::store_allowance(env, &allowance);

    let event = AllowanceSpentEvent {
        owner: owner.clone(),
        spender: spender.clone(),
        token: token.clone(),
        amount_spent: amount,
        remaining: allowance.remaining(),
    };
    env.events().publish(
        (Symbol::new(env, "allowance"), Symbol::new(env, "spent")),
        event,
    );

    Ok(())
}

// ── Queries ──────────────────────────────────────────────────────────────────

/// Get the full allowance detail for a specific (owner, spender, token) triple.
pub fn get_allowance_detail(
    env: &Env,
    owner: &Address,
    spender: &Address,
    token: &Option<Address>,
) -> Option<TokenAllowance> {
    storage::get_allowance(env, owner, spender, token)
}

/// List all allowances granted by `owner`.
pub fn get_owner_allowances(env: &Env, owner: &Address) -> Vec<TokenAllowance> {
    storage::list_by_owner(env, owner)
}

/// List all allowances where `spender` is the beneficiary.
pub fn get_spender_allowances(env: &Env, spender: &Address) -> Vec<TokenAllowance> {
    storage::list_by_spender(env, spender)
}
