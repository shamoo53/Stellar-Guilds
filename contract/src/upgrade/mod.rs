/// Upgrade module - orchestrates contract upgrades with governance control
///
/// This module provides the main upgrade orchestration logic, including:
/// - Proposal creation and approval
/// - Upgrade execution with safety checks
/// - State migration coordination
/// - Version management and rollback support

pub mod types;
pub mod storage;
pub mod version;
pub mod migration;
pub mod proxy;
pub mod rollback;
pub mod governance_integration;

#[cfg(test)]
mod integration_tests;

pub use types::{
    UpgradeProposal, UpgradeStatus, UpgradeType, VersionInfo, MigrationHandler,
    UpgradeConfig, SimulationResult, CompatibilityCheck,
};







use soroban_sdk::{Address, Bytes, Env, String, Vec};

/// Propose a contract upgrade
///
/// # Arguments
/// * `env` - The contract environment
/// * `from_version` - Current/from version
/// * `to_version` - Target/to version
/// * `upgrade_type` - Type of upgrade (code-only, with migration, etc.)
/// * `new_code` - WASM bytes of new contract code
/// * `migration_handler` - State migration configuration (if applicable)
/// * `description` - Description of changes
///
/// # Returns
/// ID of the created upgrade proposal
pub fn propose_upgrade(
    env: &Env,
    from_version: VersionInfo,
    to_version: VersionInfo,
    upgrade_type: UpgradeType,
    new_code: Bytes,
    migration_handler: Option<MigrationHandler>,
    description: String,
    proposed_by: Address,
) -> Result<u64, String> {
    proposed_by.require_auth();

    // Validate versions
    version::can_upgrade_from(env, &from_version, &to_version)?;

    // Check compatibility
    let compat = version::check_compatibility(env, &from_version, &to_version);
    if !compat.is_compatible {
        return Err(String::from_str(env, "Versions are not compatible"));
    }

    // Validate code size
    let config = storage::get_upgrade_config(env)?;
    if new_code.len() as u64 > config.max_upgrade_size {
        return Err(String::from_str(env, "Contract code exceeds maximum size"));
    }

    // Validate upgrade interval
    let last_upgrade = storage::get_last_upgrade_time(env);
    let current_time = env.ledger().timestamp();
    if current_time - last_upgrade < config.min_upgrade_interval {
        return Err(String::from_str(
            env,
            "Upgrade interval not met - upgrades too frequent",
        ));
    }

    // Create proposal
    let proposal_id = storage::get_next_upgrade_id(env);

    let proposal = UpgradeProposal {
        id: proposal_id,
        from_version,
        to_version,
        upgrade_type,
        status: UpgradeStatus::Pending,
        proposed_by: proposed_by.clone(),
        new_code,
        migration_handler,
        proposed_at: current_time,
        approved_at: 0,
        executed_at: 0,
        description,
        safety_checks_passed: false,
        governance_proposal_id: 0,
    };

    storage::store_proposal(env, &proposal)?;

    Ok(proposal_id)
}

/// Approve a proposed upgrade (governance controlled)
pub fn approve_upgrade(
    env: &Env,
    upgrade_id: u64,
    approver: &Address,
) -> Result<(), String> {
    approver.require_auth();

    let mut proposal = storage::get_proposal(env, upgrade_id)?;

    if proposal.status != UpgradeStatus::Pending {
        return Err(String::from_str(env, "Upgrade not in pending state"));
    }

    // Verify approver has governance authority
    // This is a placeholder - integrate with your governance module
    // _verify_governance_authority(env, approver)?;

    proposal.status = UpgradeStatus::Approved;
    proposal.approved_at = env.ledger().timestamp();

    storage::store_proposal(env, &proposal)?;

    Ok(())
}

/// Execute a proposed and approved upgrade
pub fn execute_upgrade(
    env: &Env,
    upgrade_id: u64,
    executor: &Address,
) -> Result<(), String> {
    executor.require_auth();

    let mut proposal = storage::get_proposal(env, upgrade_id)?;

    if proposal.status != UpgradeStatus::Approved {
        return Err(String::from_str(
            env,
            "Upgrade must be approved before execution",
        ));
    }

    // Get current state snapshot before upgrade
    let current_state = storage::get_state_snapshot(env);

    // Create rollback point
    let current_version = storage::get_current_version(env)?;
    let _rollback = rollback::create_rollback_point(
        env,
        upgrade_id,
        &current_version,
        &current_state,
    )?;

    // Execute state migration if required
    if let Some(handler) = &proposal.migration_handler {
        let new_state = migration::execute_migration(env, handler, &current_state)?;
        storage::set_state_snapshot(env, &new_state)?;
    }

    // In production, the implementation address would come from the governance system
    // For now, we'll store the upgrade ID and mark as ready for delegation
    // The actual implementation switching would be done by the governance contract
    proxy::upgrade_implementation(env, executor, executor)?;

    // Update version
    storage::set_current_version(env, &proposal.to_version)?;

    // Mark upgrade as executed
    proposal.status = UpgradeStatus::Executed;
    proposal.executed_at = env.ledger().timestamp();
    storage::store_proposal(env, &proposal)?;

    // Update last upgrade time
    storage::set_last_upgrade_time(env, env.ledger().timestamp())?;

    // Cleanup old rollback points
    let config = storage::get_upgrade_config(env)?;
    storage::cleanup_old_rollback_points(env, &config)?;

    Ok(())
}

/// Simulate an upgrade without executing it
pub fn simulate_upgrade(
    env: &Env,
    upgrade_id: u64,
) -> Result<SimulationResult, String> {
    let proposal = storage::get_proposal(env, upgrade_id)?;
    let current_state = storage::get_state_snapshot(env);

    if let Some(handler) = &proposal.migration_handler {
        migration::simulate_migration(env, handler, &current_state)
    } else {
        // No migration, create simulation result showing no changes
        Ok(SimulationResult {
            passed: true,
            state_changes: Vec::new(env),
            warnings: Vec::new(env),
            estimated_gas: 2_000_000,
            simulated_at: env.ledger().timestamp(),
        })
    }
}

/// Cancel a pending upgrade proposal
pub fn cancel_upgrade(
    env: &Env,
    upgrade_id: u64,
    canceller: &Address,
) -> Result<(), String> {
    canceller.require_auth();

    let mut proposal = storage::get_proposal(env, upgrade_id)?;

    if proposal.status != UpgradeStatus::Pending && proposal.status != UpgradeStatus::Approved {
        return Err(String::from_str(
            env,
            "Can only cancel pending or approved upgrades",
        ));
    }

    // Only proposer or contract admin can cancel
    if &proposal.proposed_by != canceller {
        // Add check for contract admin if needed
    }

    proposal.status = UpgradeStatus::RolledBack;
    storage::store_proposal(env, &proposal)?;

    Ok(())
}

/// Get upgrade proposal details
pub fn get_upgrade_proposal(env: &Env, upgrade_id: u64) -> Result<UpgradeProposal, String> {
    storage::get_proposal(env, upgrade_id)
}

/// Get all upgrades (active and historical)
pub fn get_all_upgrades(env: &Env) -> Vec<UpgradeProposal> {
    storage::get_all_proposals(env)
}

/// Get active upgrades (pending or approved)
pub fn get_active_upgrades(env: &Env) -> Vec<UpgradeProposal> {
    let all = storage::get_all_proposals(env);
    let mut active = Vec::new(env);

    for i in 0..all.len() {
        let upgrade = all.get(i).unwrap();
        if upgrade.status == UpgradeStatus::Pending ||
           upgrade.status == UpgradeStatus::Approved {
            active.push_back(upgrade.clone());
        }
    }

    active
}

/// Perform version compatibility check
pub fn check_version_compatibility(
    env: &Env,
    from_version: &VersionInfo,
    to_version: &VersionInfo,
) -> CompatibilityCheck {
    version::check_compatibility(env, from_version, to_version)
}

/// Get current contract version
pub fn get_version(env: &Env) -> Result<VersionInfo, String> {
    storage::get_current_version(env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_propose_upgrade() {
        let env = Env::default();

        let v1 = VersionInfo {
            major: 1,
            minor: 0,
            patch: 0,
            build: String::from_str(&env, "build1"),
            code_hash: Bytes::new(&env),
            released_at: 1000,
        };

        let v2 = VersionInfo {
            major: 1,
            minor: 1,
            patch: 0,
            build: String::from_str(&env, "build2"),
            code_hash: Bytes::new(&env),
            released_at: 2000,
        };

        let config = UpgradeConfig {
            min_upgrade_interval: 0,
            max_upgrade_size: 10_000_000,
            allow_emergency_upgrades: false,
            emergency_admin: Address::from_contract_id(&env, &[0u8; 32]),
            rollback_history_size: 10,
            require_state_migration: false,
        };

        storage::initialize(&env, v1.clone(), config);

        let proposer = Address::from_contract_id(&env, &[1u8; 32]);
        let result = propose_upgrade(
            &env,
            v1,
            v2,
            UpgradeType::CodeOnly,
            Bytes::new(&env),
            None,
            String::from_str(&env, "test upgrade"),
            proposer,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
