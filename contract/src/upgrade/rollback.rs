/// Rollback mechanisms for handling failed upgrades
///
/// Provides functionality to revert to previous contract versions in case of
/// issues during or after an upgrade.

use soroban_sdk::{Bytes, Env, Map, String, Vec};
use crate::upgrade::types::{RollbackPoint, VersionInfo};
use crate::upgrade::storage;

/// Create a rollback point before performing an upgrade
pub fn create_rollback_point(
    env: &Env,
    upgrade_id: u64,
    previous_version: &VersionInfo,
    state_snapshot: &Map<String, Bytes>,
) -> Result<RollbackPoint, String> {
    let rollback_id = storage::get_next_rollback_id(env);

    let rollback = RollbackPoint {
        id: rollback_id,
        upgrade_id,
        previous_version: previous_version.clone(),
        state_snapshot: state_snapshot.clone(),
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    storage::store_rollback_point(env, &rollback)?;

    Ok(rollback)
}

/// Execute rollback to a specific rollback point
pub fn execute_rollback(
    env: &Env,
    rollback_id: u64,
    _caller: &String,
) -> Result<VersionInfo, String> {
    // Get the rollback point
    let rollback = storage::get_rollback_point(env, rollback_id)?;

    // Verify rollback is still active
    if !rollback.is_active {
        return Err(String::from_str(env, "Rollback point is no longer active"));
    }

    // Restore state from snapshot
    let _restored_state = restore_state_from_snapshot(env, &rollback.state_snapshot)?;

    // Update version back to previous
    storage::set_current_version(env, &rollback.previous_version)?;

    // Mark rollback as inactive (done)
    let mut inactive_rollback = rollback.clone();
    inactive_rollback.is_active = false;
    storage::store_rollback_point(env, &inactive_rollback)?;

    Ok(rollback.previous_version.clone())
}

/// Restore state from a snapshot
fn restore_state_from_snapshot(
    env: &Env,
    snapshot: &Map<String, Bytes>,
) -> Result<(), String> {
    // In production, you would:
    // 1. Iterate through all keys in the snapshot
    // 2. Restore each key-value pair to storage
    // 3. Handle any special cases for complex data types

    // For now, this is a simplified version
    if snapshot.is_empty() {
        return Err(String::from_str(env, "Empty snapshot - nothing to restore"));
    }

    Ok(())
}

/// Check if an upgrade can be rolled back
pub fn can_rollback(env: &Env, upgrade_id: u64) -> bool {
    // Find rollback point for this upgrade
    let rollback_points = storage::get_active_rollback_points(env);

    for i in 0..rollback_points.len() {
        let point = rollback_points.get(i).unwrap();
        if point.upgrade_id == upgrade_id && point.is_active {
            return true;
        }
    }

    false
}

/// Get all available rollback points for an upgrade
pub fn get_rollback_points_for_upgrade(
    env: &Env,
    upgrade_id: u64,
) -> Vec<RollbackPoint> {
    let mut result = Vec::new(env);
    let all_points = storage::get_active_rollback_points(env);

    for i in 0..all_points.len() {
        let point = all_points.get(i).unwrap();
        if point.upgrade_id == upgrade_id {
            result.push_back(point.clone());
        }
    }

    result
}

/// Automatic rollback on upgrade failure detection
pub fn trigger_automatic_rollback(
    env: &Env,
    upgrade_id: u64,
    error_reason: &String,
) -> Result<VersionInfo, String> {
    // Log the error
    // env.events().publish(("upgrade_failed",), (upgrade_id, error_reason));

    // Get the associated rollback point
    let rollback_points = get_rollback_points_for_upgrade(env, upgrade_id);

    if rollback_points.len() == 0 {
        return Err(String::from_str(
            env,
            "No rollback point available for automatic rollback",
        ));
    }

    // Use the most recent rollback point
    let latest_rollback = rollback_points.get(rollback_points.len() - 1).unwrap();

    // Execute rollback
    execute_rollback(env, latest_rollback.id, error_reason)
}

/// Get rollback history
pub fn get_rollback_history(env: &Env) -> Vec<RollbackPoint> {
    storage::get_active_rollback_points(env)
}

/// Deactivate a rollback point (making it unavailable)
pub fn deactivate_rollback_point(env: &Env, rollback_id: u64) -> Result<(), String> {
    let mut rollback = storage::get_rollback_point(env, rollback_id)?;
    rollback.is_active = false;
    storage::store_rollback_point(env, &rollback)?;
    Ok(())
}

/// Deactivate old rollback points beyond the configured history size
pub fn cleanup_rollback_history(
    env: &Env,
    max_history: u32,
) -> Result<(), String> {
    let all_points = storage::get_active_rollback_points(env);

    if all_points.len() as u32 > max_history {
        let excess = all_points.len() as u32 - max_history;

        // Deactivate oldest rollback points
        for i in 0..excess {
            let point = all_points.get(i).unwrap();
            deactivate_rollback_point(env, point.id)?;
        }
    }

    Ok(())
}

/// Verify rollback point integrity
pub fn verify_rollback_point(env: &Env, rollback_id: u64) -> Result<(), String> {
    let rollback = storage::get_rollback_point(env, rollback_id)?;

    // Verify snapshot is not empty
    if rollback.state_snapshot.is_empty() {
        return Err(String::from_str(env, "Rollback point has empty snapshot"));
    }

    // Verify version info is valid
    if rollback.previous_version.major == 0 &&
       rollback.previous_version.minor == 0 &&
       rollback.previous_version.patch == 0 {
        return Err(String::from_str(
            env,
            "Rollback point has invalid version info",
        ));
    }

    // Verify creation time is in the past
    if rollback.created_at > env.ledger().timestamp() {
        return Err(String::from_str(
            env,
            "Rollback point creation time is in the future",
        ));
    }

    Ok(())
}

/// Get most recent rollback point
pub fn get_latest_rollback_point(env: &Env) -> Result<RollbackPoint, String> {
    let all_points = storage::get_active_rollback_points(env);

    if all_points.len() == 0 {
        return Err(String::from_str(env, "No rollback points available"));
    }

    // Return the last (most recent) rollback point
    Ok(all_points.get(all_points.len() - 1).unwrap().clone())
}

/// Emergency rollback (for critical failures)
pub fn emergency_rollback(
    env: &Env,
    rollback_id: u64,
) -> Result<VersionInfo, String> {
    // Verify rollback point exists and is valid
    verify_rollback_point(env, rollback_id)?;

    // Execute immediate rollback without additional checks
    execute_rollback(env, rollback_id, &String::from_str(env, "emergency_rollback"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_rollback_point_creation() {
        let env = Env::default();

        let version = VersionInfo {
            major: 1,
            minor: 0,
            patch: 0,
            build: String::from_str(&env, "build1"),
            code_hash: soroban_sdk::Bytes::new(&env),
            released_at: 1000,
        };

        let snapshot = Map::new(&env);

        let result = create_rollback_point(&env, 1, &version, &snapshot);
        assert!(result.is_ok());

        let rollback = result.unwrap();
        assert_eq!(rollback.upgrade_id, 1);
        assert!(rollback.is_active);
    }

    #[test]
    fn test_cannot_rollback_empty_history() {
        let env = Env::default();

        let result = trigger_automatic_rollback(
            &env,
            1,
            &String::from_str(&env, "test error"),
        );

        assert!(result.is_err());
    }
}
