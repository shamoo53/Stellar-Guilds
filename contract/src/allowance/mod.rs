pub mod management;
pub mod storage;
pub mod types;

pub use management::{
    approve, decrease_allowance, get_allowance_detail, get_owner_allowances,
    get_spender_allowances, increase_allowance, revoke, spend,
};

pub use types::{AllowanceError, AllowanceOperation, TokenAllowance};

#[cfg(test)]
mod tests;
