use soroban_sdk::{symbol_short, Env, Map, Symbol, Vec};

use crate::analytics::types::TreasurySnapshot;

const SNAPSHOTS_KEY: Symbol = symbol_short!("a_snaps");
const SNAP_CNT_KEY: Symbol = symbol_short!("a_scnt");

/// Maximum number of snapshots retained per treasury to bound storage growth.
const MAX_SNAPSHOTS_PER_TREASURY: u32 = 200;

/// Store a new treasury snapshot. Older snapshots are evicted when the cap is reached.
pub fn store_snapshot(env: &Env, snapshot: &TreasurySnapshot) {
    let storage = env.storage().persistent();

    let mut all_snaps: Map<u64, Vec<TreasurySnapshot>> = storage
        .get(&SNAPSHOTS_KEY)
        .unwrap_or_else(|| Map::new(env));

    let mut snaps = all_snaps
        .get(snapshot.treasury_id)
        .unwrap_or_else(|| Vec::new(env));

    // Evict oldest if we hit the cap
    if snaps.len() >= MAX_SNAPSHOTS_PER_TREASURY {
        // Remove the first (oldest) element
        let mut new_snaps = Vec::new(env);
        for (i, s) in snaps.iter().enumerate() {
            if i > 0 {
                new_snaps.push_back(s);
            }
        }
        snaps = new_snaps;
    }

    snaps.push_back(snapshot.clone());
    all_snaps.set(snapshot.treasury_id, snaps);
    storage.set(&SNAPSHOTS_KEY, &all_snaps);

    // Update counter
    let mut counts: Map<u64, u32> = storage
        .get(&SNAP_CNT_KEY)
        .unwrap_or_else(|| Map::new(env));
    let current = counts.get(snapshot.treasury_id).unwrap_or(0u32);
    counts.set(snapshot.treasury_id, current + 1);
    storage.set(&SNAP_CNT_KEY, &counts);
}

/// Retrieve the latest `limit` snapshots for a treasury (most recent last).
pub fn get_snapshots(env: &Env, treasury_id: u64, limit: u32) -> Vec<TreasurySnapshot> {
    let storage = env.storage().persistent();

    let all_snaps: Map<u64, Vec<TreasurySnapshot>> = storage
        .get(&SNAPSHOTS_KEY)
        .unwrap_or_else(|| Map::new(env));

    let snaps = all_snaps
        .get(treasury_id)
        .unwrap_or_else(|| Vec::new(env));

    let len = snaps.len();
    if len <= limit {
        return snaps;
    }

    let start = len - limit;
    let mut result = Vec::new(env);
    for (idx, s) in snaps.iter().enumerate() {
        if (idx as u32) >= start {
            result.push_back(s);
        }
    }
    result
}

/// Get the total number of snapshots ever recorded for a treasury.
pub fn get_snapshot_count(env: &Env, treasury_id: u64) -> u32 {
    let storage = env.storage().persistent();

    let counts: Map<u64, u32> = storage
        .get(&SNAP_CNT_KEY)
        .unwrap_or_else(|| Map::new(env));

    counts.get(treasury_id).unwrap_or(0u32)
}
