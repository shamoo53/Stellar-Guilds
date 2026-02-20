use soroban_sdk::{contracterror, contracttype, Address};

// ── Operation Types ──────────────────────────────────────────────────────────

/// Defines which operations the allowance permits.
/// `Any` permits all operation types.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AllowanceOperation {
    Any,
    Withdrawal,
    BountyFunding,
    MilestonePayment,
    Escrow,
}

// ── Core Allowance Struct ────────────────────────────────────────────────────

/// ERC-20 style token allowance with expiration and per-operation granularity.
///
/// Key: `(owner, spender, token_key)` where `token_key` is a deterministic u64
/// derived from the token address (0 for native XLM).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenAllowance {
    /// Address that owns the tokens and granted the approval
    pub owner: Address,
    /// Address permitted to spend on behalf of the owner
    pub spender: Address,
    /// Token address (None = native XLM)
    pub token: Option<Address>,
    /// Total amount approved
    pub amount: i128,
    /// Amount already consumed
    pub spent: i128,
    /// Ledger timestamp after which this allowance is invalid (0 = no expiry)
    pub expires_at: u64,
    /// Which operation type(s) this allowance permits
    pub operation: AllowanceOperation,
    /// Ledger timestamp when this allowance was created/last modified
    pub created_at: u64,
}

impl TokenAllowance {
    /// Remaining spendable amount.
    pub fn remaining(&self) -> i128 {
        self.amount.saturating_sub(self.spent)
    }

    /// Whether the allowance has expired relative to `now`.
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at > 0 && now >= self.expires_at
    }

    /// Whether this allowance permits the given operation type.
    pub fn permits_operation(&self, op: &AllowanceOperation) -> bool {
        matches!(self.operation, AllowanceOperation::Any) || self.operation == *op
    }
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AllowanceError {
    NotFound = 101,
    Expired = 102,
    InsufficientAllowance = 103,
    Unauthorized = 104,
    OperationNotPermitted = 105,
    InvalidAmount = 106,
}

// ── Events ───────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceApprovedEvent {
    pub owner: Address,
    pub spender: Address,
    pub token: Option<Address>,
    pub amount: i128,
    pub expires_at: u64,
    pub operation: AllowanceOperation,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceSpentEvent {
    pub owner: Address,
    pub spender: Address,
    pub token: Option<Address>,
    pub amount_spent: i128,
    pub remaining: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceRevokedEvent {
    pub owner: Address,
    pub spender: Address,
    pub token: Option<Address>,
}
