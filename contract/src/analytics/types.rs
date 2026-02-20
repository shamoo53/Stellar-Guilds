use soroban_sdk::{contracttype, String};

/// Reporting period options for analytics queries
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReportingPeriod {
    Weekly,
    Monthly,
    Quarterly,
    Custom,
}

/// Aggregated spending statistics for a given time period
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpendingSummary {
    pub treasury_id: u64,
    pub total_deposits: i128,
    pub total_withdrawals: i128,
    pub net_flow: i128,
    pub tx_count: u32,
    pub avg_tx_amount: i128,
    pub period_start: u64,
    pub period_end: u64,
}

/// Per-category budget utilization tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetUtilization {
    pub category: String,
    pub allocated: i128,
    pub spent: i128,
    pub remaining: i128,
    /// Utilization percentage in basis points (e.g., 7500 = 75.00%)
    pub utilization_bps: u32,
}

/// Spending breakdown grouped by transaction type
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CategoryBreakdown {
    pub category_name: String,
    pub total_amount: i128,
    pub tx_count: u32,
}

/// Point-in-time treasury balance snapshot
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasurySnapshot {
    pub treasury_id: u64,
    pub timestamp: u64,
    pub balance_xlm: i128,
    pub total_deposits: i128,
    pub total_withdrawals: i128,
    pub snapshot_index: u32,
}

/// Trend comparison between two periods (values in basis points, e.g. 1500 = +15%)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpendingTrend {
    pub deposits_change_bps: i64,
    pub withdrawals_change_bps: i64,
    pub net_flow_change_bps: i64,
}

/// Simple linear forecast for next period
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpendingForecast {
    pub projected_deposits: i128,
    pub projected_withdrawals: i128,
    pub projected_net_flow: i128,
    pub periods_analyzed: u32,
}
