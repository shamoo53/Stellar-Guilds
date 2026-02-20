/// Storage management for upgrade system
///
/// Handles persistent storage of upgrade proposals, version information,
/// and rollback points using the Soroban SDK storage interface.

use soroban_sdk::{Bytes, Env, Map, String, Vec};
use crate::upgrade::types::{
    UpgradeProposal, VersionInfo, RollbackPoint, UpgradeConfig,
    UpgradeStatus,
};

// Storage keys
const UPGRADE_PROPOSALS_KEY: &str = "upgrade_proposals";
const VERSION_HISTORY_KEY: &str = "version_history";
const CURRENT_VERSION_KEY: &str = "current_version";
const ROLLBACK_POINTS_KEY: &str = "rollback_points";
const UPGRADE_CONFIG_KEY: &str = "upgrade_config";
const LAST_UPGRADE_TIME_KEY: &str = "last_upgrade_time";
const UPGRADE_COUNTER_KEY: &str = "upgrade_counter";
const STATE_SNAPSHOT_KEY: &str = "state_snapshot";

/// Initialize upgrade storage
pub fn initialize(env: &Env, initial_version: VersionInfo, config: UpgradeConfig) {
    env.storage()
        .persistent()
        .set(&String::from_str(env, CURRENT_VERSION_KEY), &initial_version);

    env.storage()
        .persistent()
        .set(&String::from_str(env, UPGRADE_CONFIG_KEY), &config);

    env.storage()
        .persistent()
        .set(&String::from_str(env, UPGRADE_COUNTER_KEY), &0u64);

    let mut version_history: Vec<VersionInfo> = Vec::new(env);
    version_history.push_back(initial_version);

    env.storage()
        .persistent()
        .set(&String::from_str(env, VERSION_HISTORY_KEY), &version_history);
}

/// Store an upgrade proposal
pub fn store_proposal(env: &Env, proposal: &UpgradeProposal) -> Result<(), String> {
    let key = String::from_str(env, "upgrade_proposal");
    env.storage().persistent().set(&key, proposal);
    Ok(())
}
/// Retrieve an upgrade proposal
pub fn get_proposal(env: &Env, _upgrade_id: u64) -> Result<UpgradeProposal, String> {
    let key = String::from_str(env, "upgrade_proposal");
    env.storage()
        .persistent()
        .get::<String, UpgradeProposal>(&key)
        .ok_or_else(|| String::from_str(env, "Proposal not found"))
}

/// Update upgrade proposal status
pub fn update_proposal_status(
    env: &Env,
    upgrade_id: u64,
    status: UpgradeStatus,
) -> Result<(), String> {
    let mut proposal = get_proposal(env, upgrade_id)?;
    proposal.status = status;
    store_proposal(env, &proposal)?;
    Ok(())
}

/// Get all proposals
pub fn get_all_proposals(env: &Env) -> Vec<UpgradeProposal> {
    let mut proposals = Vec::new(env);
    let counter = get_upgrade_counter(env);

    for i in 0..counter {
        if let Ok(proposal) = get_proposal(env, i) {
            proposals.push_back(proposal);
        }
    }

    proposals
}

/// Get next upgrade ID
pub fn get_next_upgrade_id(env: &Env) -> u64 {
    let counter = get_upgrade_counter(env);
    let next_id = counter + 1;
    env.storage()
        .persistent()
        .set(&String::from_str(env, UPGRADE_COUNTER_KEY), &next_id);
    counter
}

/// Get upgrade counter
fn get_upgrade_counter(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get::<String, u64>(&String::from_str(env, UPGRADE_COUNTER_KEY))
        .unwrap_or(0)
}

/// Get current version
pub fn get_current_version(env: &Env) -> Result<VersionInfo, String> {
    env.storage()
        .persistent()
        .get::<String, VersionInfo>(&String::from_str(env, CURRENT_VERSION_KEY))
        .ok_or_else(|| String::from_str(env, "Version not found"))
}

/// Update current version
pub fn set_current_version(env: &Env, version: &VersionInfo) -> Result<(), String> {
    env.storage()
        .persistent()
        .set(&String::from_str(env, CURRENT_VERSION_KEY), version);

    // Add to history
    let mut history: Vec<VersionInfo> = env.storage()
        .persistent()
        .get::<String, Vec<VersionInfo>>(&String::from_str(env, VERSION_HISTORY_KEY))
        .unwrap_or_else(|| Vec::new(env));
    
    history.push_back(version.clone());
    env.storage()
        .persistent()
        .set(&String::from_str(env, VERSION_HISTORY_KEY), &history);

    Ok(())
}

/// Get version history
pub fn get_version_history(env: &Env) -> Vec<VersionInfo> {
    env.storage()
        .persistent()
        .get::<String, Vec<VersionInfo>>(&String::from_str(env, VERSION_HISTORY_KEY))
        .unwrap_or_else(|| Vec::new(env))
}

/// Store rollback point
pub fn store_rollback_point(env: &Env, rollback: &RollbackPoint) -> Result<(), String> {
    let key = String::from_str(env, "rollback_point");
    env.storage().persistent().set(&key, rollback);
    Ok(())
}

/// Get rollback point
pub fn get_rollback_point(env: &Env, _rollback_id: u64) -> Result<RollbackPoint, String> {
    let key = String::from_str(env, "rollback_point");
    env.storage()
        .persistent()
        .get::<String, RollbackPoint>(&key)
        .ok_or_else(|| String::from_str(env, "Rollback point not found"))
}

/// Get all active rollback points
pub fn get_active_rollback_points(env: &Env) -> Vec<RollbackPoint> {
    let mut rollbacks = Vec::new(env);
    let counter = get_rollback_counter(env);

    for i in 0..counter {
        if let Ok(rollback) = get_rollback_point(env, i) {
            if rollback.is_active {
                rollbacks.push_back(rollback);
            }
        }
    }

    rollbacks
}

/// Get next rollback ID
pub fn get_next_rollback_id(env: &Env) -> u64 {
    let counter = get_rollback_counter(env);
    let next_id = counter + 1;
    env.storage()
        .persistent()
        .set(&String::from_str(env, "rollback_counter"), &next_id);
    counter
}

/// Get rollback counter
fn get_rollback_counter(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get::<String, u64>(&String::from_str(env, "rollback_counter"))
        .unwrap_or(0)
}

/// Get upgrade configuration
pub fn get_upgrade_config(env: &Env) -> Result<UpgradeConfig, String> {
    env.storage()
        .persistent()
        .get::<String, UpgradeConfig>(&String::from_str(env, UPGRADE_CONFIG_KEY))
        .ok_or_else(|| String::from_str(env, "Upgrade config not found"))
}

/// Update upgrade configuration
pub fn set_upgrade_config(env: &Env, config: &UpgradeConfig) -> Result<(), String> {
    env.storage()
        .persistent()
        .set(&String::from_str(env, UPGRADE_CONFIG_KEY), config);
    Ok(())
}

/// Get last upgrade timestamp
pub fn get_last_upgrade_time(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get::<String, u64>(&String::from_str(env, LAST_UPGRADE_TIME_KEY))
        .unwrap_or(0)
}

/// Update last upgrade timestamp
pub fn set_last_upgrade_time(env: &Env, timestamp: u64) -> Result<(), String> {
    env.storage()
        .persistent()
        .set(&String::from_str(env, LAST_UPGRADE_TIME_KEY), &timestamp);
    Ok(())
}

/// Get state snapshot
pub fn get_state_snapshot(env: &Env) -> Map<String, Bytes> {
    env.storage()
        .persistent()
        .get::<String, Map<String, Bytes>>(&String::from_str(env, STATE_SNAPSHOT_KEY))
        .unwrap_or_else(|| Map::new(env))
}

/// Store state snapshot
pub fn set_state_snapshot(env: &Env, snapshot: &Map<String, Bytes>) -> Result<(), String> {
    env.storage()
        .persistent()
        .set(&String::from_str(env, STATE_SNAPSHOT_KEY), snapshot);
    Ok(())
}

/// Clear old rollback points beyond history size limit
pub fn cleanup_old_rollback_points(env: &Env, config: &UpgradeConfig) -> Result<(), String> {
    let all_points = get_active_rollback_points(env);
    
    if all_points.len() as u32 > config.rollback_history_size {
        let excess = all_points.len() as u32 - config.rollback_history_size;
        
        for i in 0..excess {
            let point = all_points.get(i).unwrap();
            let mut updated = point.clone();
            updated.is_active = false;
            store_rollback_point(env, &updated)?;
        }
    }

    Ok(())
}
