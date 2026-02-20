pub mod computations;
pub mod storage;
pub mod types;

pub use computations::{
    compute_budget_utilization, compute_category_breakdown, compute_forecast,
    compute_spending_summary, compute_trend,
};

pub use storage::{get_snapshot_count, get_snapshots, store_snapshot};

pub use types::{
    BudgetUtilization, CategoryBreakdown, SpendingForecast, SpendingSummary, SpendingTrend,
    TreasurySnapshot,
};

#[cfg(test)]
mod tests;
