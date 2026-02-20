pub mod types;
pub mod storage;
pub mod management;
pub mod multisig;

pub use management::{
    approve_transaction,
    deposit,
    emergency_pause,
    execute_transaction,
    get_balance,
    get_transaction_history,
    grant_allowance,
    initialize_treasury,
    propose_withdrawal,
    set_budget,
        execute_milestone_payment, 
};

pub use storage::initialize_treasury_storage;

pub use types::Transaction;
// Tests disabled pending fixes
#[cfg(test)]
mod tests;