use soroban_sdk::{Env, String, Vec};

use crate::analytics::types::{
    BudgetUtilization, CategoryBreakdown, SpendingForecast, SpendingSummary, SpendingTrend,
};
use crate::treasury::storage::{get_treasury_transactions, list_budgets_for_treasury};
use crate::treasury::types::{Transaction, TransactionStatus, TransactionType};

/// Compute an aggregated spending summary for a treasury within [period_start, period_end].
pub fn compute_spending_summary(
    env: &Env,
    treasury_id: u64,
    period_start: u64,
    period_end: u64,
) -> SpendingSummary {
    let txs = get_treasury_transactions(env, treasury_id);

    let mut total_deposits: i128 = 0;
    let mut total_withdrawals: i128 = 0;
    let mut tx_count: u32 = 0;

    for tx in txs.iter() {
        if !is_executed(&tx) {
            continue;
        }
        if tx.created_at < period_start || tx.created_at > period_end {
            continue;
        }

        tx_count += 1;
        match tx.tx_type {
            TransactionType::Deposit => {
                total_deposits += tx.amount;
            }
            TransactionType::Withdrawal
            | TransactionType::BountyFunding
            | TransactionType::MilestonePayment => {
                total_withdrawals += tx.amount;
            }
            TransactionType::AllowanceGrant => {}
        }
    }

    let net_flow = total_deposits - total_withdrawals;
    let avg_tx_amount = if tx_count > 0 {
        (total_deposits + total_withdrawals) / (tx_count as i128)
    } else {
        0
    };

    SpendingSummary {
        treasury_id,
        total_deposits,
        total_withdrawals,
        net_flow,
        tx_count,
        avg_tx_amount,
        period_start,
        period_end,
    }
}

/// Compute budget utilization for all categories of a given treasury.
pub fn compute_budget_utilization(env: &Env, treasury_id: u64) -> Vec<BudgetUtilization> {
    let budgets = list_budgets_for_treasury(env, treasury_id);
    let mut result = Vec::new(env);

    for budget in budgets.iter() {
        let remaining = if budget.allocated_amount > budget.spent_amount {
            budget.allocated_amount - budget.spent_amount
        } else {
            0
        };

        let utilization_bps: u32 = if budget.allocated_amount > 0 {
            // (spent * 10000) / allocated — safe since allocated > 0
            let bps = (budget.spent_amount * 10000) / budget.allocated_amount;
            // Cap at 10000 (100%)
            if bps > 10000 {
                10000u32
            } else {
                bps as u32
            }
        } else {
            0
        };

        result.push_back(BudgetUtilization {
            category: budget.category.clone(),
            allocated: budget.allocated_amount,
            spent: budget.spent_amount,
            remaining,
            utilization_bps,
        });
    }

    result
}

/// Group executed transactions by type and compute totals for each category.
pub fn compute_category_breakdown(
    env: &Env,
    treasury_id: u64,
    period_start: u64,
    period_end: u64,
) -> Vec<CategoryBreakdown> {
    let txs = get_treasury_transactions(env, treasury_id);

    let mut deposit_amount: i128 = 0;
    let mut deposit_count: u32 = 0;
    let mut withdrawal_amount: i128 = 0;
    let mut withdrawal_count: u32 = 0;
    let mut bounty_amount: i128 = 0;
    let mut bounty_count: u32 = 0;
    let mut milestone_amount: i128 = 0;
    let mut milestone_count: u32 = 0;
    let mut allowance_amount: i128 = 0;
    let mut allowance_count: u32 = 0;

    for tx in txs.iter() {
        if !is_executed(&tx) {
            continue;
        }
        if tx.created_at < period_start || tx.created_at > period_end {
            continue;
        }
        match tx.tx_type {
            TransactionType::Deposit => {
                deposit_amount += tx.amount;
                deposit_count += 1;
            }
            TransactionType::Withdrawal => {
                withdrawal_amount += tx.amount;
                withdrawal_count += 1;
            }
            TransactionType::BountyFunding => {
                bounty_amount += tx.amount;
                bounty_count += 1;
            }
            TransactionType::MilestonePayment => {
                milestone_amount += tx.amount;
                milestone_count += 1;
            }
            TransactionType::AllowanceGrant => {
                allowance_amount += tx.amount;
                allowance_count += 1;
            }
        }
    }

    let mut result = Vec::new(env);

    if deposit_count > 0 {
        result.push_back(CategoryBreakdown {
            category_name: String::from_str(env, "Deposit"),
            total_amount: deposit_amount,
            tx_count: deposit_count,
        });
    }
    if withdrawal_count > 0 {
        result.push_back(CategoryBreakdown {
            category_name: String::from_str(env, "Withdrawal"),
            total_amount: withdrawal_amount,
            tx_count: withdrawal_count,
        });
    }
    if bounty_count > 0 {
        result.push_back(CategoryBreakdown {
            category_name: String::from_str(env, "BountyFunding"),
            total_amount: bounty_amount,
            tx_count: bounty_count,
        });
    }
    if milestone_count > 0 {
        result.push_back(CategoryBreakdown {
            category_name: String::from_str(env, "MilestonePayment"),
            total_amount: milestone_amount,
            tx_count: milestone_count,
        });
    }
    if allowance_count > 0 {
        result.push_back(CategoryBreakdown {
            category_name: String::from_str(env, "AllowanceGrant"),
            total_amount: allowance_amount,
            tx_count: allowance_count,
        });
    }

    result
}

/// Compute the percentage change (in basis points) between two periods.
/// Positive = increase, negative = decrease.
pub fn compute_trend(
    _env: &Env,
    period1: &SpendingSummary,
    period2: &SpendingSummary,
) -> SpendingTrend {
    SpendingTrend {
        deposits_change_bps: pct_change_bps(period1.total_deposits, period2.total_deposits),
        withdrawals_change_bps: pct_change_bps(
            period1.total_withdrawals,
            period2.total_withdrawals,
        ),
        net_flow_change_bps: pct_change_bps(period1.net_flow, period2.net_flow),
    }
}

/// Simple moving average forecast: takes N historical periods and projects the next.
pub fn compute_forecast(
    env: &Env,
    treasury_id: u64,
    num_periods: u32,
    period_length_secs: u64,
    current_time: u64,
) -> SpendingForecast {
    if num_periods == 0 || period_length_secs == 0 {
        return SpendingForecast {
            projected_deposits: 0,
            projected_withdrawals: 0,
            projected_net_flow: 0,
            periods_analyzed: 0,
        };
    }

    let mut total_deposits: i128 = 0;
    let mut total_withdrawals: i128 = 0;
    let mut valid_periods: u32 = 0;

    for i in 0..num_periods {
        let period_end = current_time.saturating_sub((i as u64) * period_length_secs);
        let period_start = period_end.saturating_sub(period_length_secs);

        if period_start >= period_end {
            continue;
        }

        let summary = compute_spending_summary(env, treasury_id, period_start, period_end);
        if summary.tx_count > 0 {
            total_deposits += summary.total_deposits;
            total_withdrawals += summary.total_withdrawals;
            valid_periods += 1;
        }
    }

    if valid_periods == 0 {
        return SpendingForecast {
            projected_deposits: 0,
            projected_withdrawals: 0,
            projected_net_flow: 0,
            periods_analyzed: 0,
        };
    }

    let projected_deposits = total_deposits / (valid_periods as i128);
    let projected_withdrawals = total_withdrawals / (valid_periods as i128);

    SpendingForecast {
        projected_deposits,
        projected_withdrawals,
        projected_net_flow: projected_deposits - projected_withdrawals,
        periods_analyzed: valid_periods,
    }
}

// ────────────────────── Helpers ──────────────────────

fn is_executed(tx: &Transaction) -> bool {
    matches!(tx.status, TransactionStatus::Executed)
}

/// Compute percentage change from `old` to `new` in basis points.
/// Returns 0 if old is 0 (avoids division by zero).
fn pct_change_bps(old: i128, new: i128) -> i64 {
    if old == 0 {
        if new == 0 {
            return 0;
        }
        // Infinite increase represented as max i64 value capped at 100_00 bps (100%)
        return 10000;
    }
    let change = ((new - old) * 10000) / old;
    change as i64
}
