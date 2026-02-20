/// Governance Integration for Contract Upgrades
///
/// This module integrates the upgrade system with the governance system,
/// enabling governance-controlled upgrades with proposal tracking.

use soroban_sdk::{Address, Env, String, Vec};

use crate::upgrade::types::UpgradeStatus;
use crate::upgrade::{storage, version};

/// Result of upgrade governance approval check
#[derive(Clone, Debug)]
pub struct GovernanceApprovalResult {
    pub approved: bool,
    pub governance_id: u64,
    pub approval_time: u64,
    pub vote_count: u32,
    pub approval_percentage: u32,
}

/// Link an upgrade proposal to a governance proposal
///
/// # Arguments
/// * `env` - The contract environment
/// * `upgrade_id` - The ID of the upgrade proposal
/// * `governance_proposal_id` - The ID of the linked governance proposal
/// * `governance_contract` - Address of the governance contract
///
/// # Returns
/// Result indicating success or error
pub fn link_governance_proposal(
    env: &Env,
    upgrade_id: u64,
    governance_proposal_id: u64,
) -> Result<(), String> {
    let mut proposal = storage::get_proposal(env, upgrade_id)?;

    if proposal.status != UpgradeStatus::Pending {
        return Err(String::from_str(
            env,
            "Can only link governance proposal to pending upgrades",
        ));
    }

    proposal.governance_proposal_id = governance_proposal_id;
    storage::store_proposal(env, &proposal)?;

    Ok(())
}

/// Check if upgrade has governance approval
///
/// This verifies that a linked governance proposal has been approved
/// and meets the upgrade requirements.
pub fn verify_governance_approval(
    env: &Env,
    upgrade_id: u64,
    _governance_contract_id: &Address,
) -> Result<GovernanceApprovalResult, String> {
    let proposal = storage::get_proposal(env, upgrade_id)?;

    if proposal.governance_proposal_id == 0 {
        return Err(String::from_str(
            env,
            "Upgrade has no linked governance proposal",
        ));
    }

    // Simulate governance approval verification
    // In production, this would call the actual governance contract
    let _current_time = env.ledger().timestamp();
    let approval_time = proposal.approved_at;

    if approval_time == 0 {
        return Err(String::from_str(env, "Upgrade has not been approved"));
    }

    Ok(GovernanceApprovalResult {
        approved: true,
        governance_id: proposal.governance_proposal_id,
        approval_time,
        vote_count: 0, // Would be populated from governance contract
        approval_percentage: 100, // Would be calculated from governance contract
    })
}

/// Create an upgrade-specific governance proposal
///
/// This creates a governance proposal specifically for contract upgrades,
/// with upgrade-specific parameters and safety checks.
pub fn create_upgrade_governance_proposal(
    env: &Env,
    upgrade_id: u64,
    proposer: &Address,
    _quorum_percentage: u32,
    _approval_threshold: u32,
    _voting_period_blocks: u32,
) -> Result<u64, String> {
    proposer.require_auth();

    let proposal = storage::get_proposal(env, upgrade_id)?;

    // Validate upgrade before creating governance proposal
    if proposal.status != UpgradeStatus::Pending {
        return Err(String::from_str(env, "Upgrade must be in pending state"));
    }

    // Verify version compatibility
    let compat = version::check_compatibility(env, &proposal.from_version, &proposal.to_version);
    if !compat.is_compatible {
        return Err(String::from_str(env, "Upgrade versions are incompatible"));
    }

    // In production, this would create an actual governance proposal
    // For now, return a simulated governance proposal ID
    let governance_id = env.ledger().timestamp() as u64;

    Ok(governance_id)
}

/// Execute upgrade after governance approval
///
/// This is the main execution path for governance-approved upgrades,
/// ensuring all governance requirements are met before execution.
pub fn execute_governance_approved_upgrade(
    env: &Env,
    upgrade_id: u64,
    executor: &Address,
) -> Result<(), String> {
    executor.require_auth();

    let mut proposal = storage::get_proposal(env, upgrade_id)?;

    // Check if linked to governance proposal
    if proposal.governance_proposal_id == 0 {
        return Err(String::from_str(
            env,
            "Upgrade must be linked to a governance proposal",
        ));
    }

    // Verify governance approval (simulated check)
    if proposal.approved_at == 0 {
        return Err(String::from_str(
            env,
            "Upgrade has not been approved by governance",
        ));
    }

    // Execute the upgrade using the standard upgrade path
    crate::upgrade::execute_upgrade(env, upgrade_id, executor)?;

    // Mark as governance-executed
    proposal.status = UpgradeStatus::Executed;
    storage::store_proposal(env, &proposal)?;

    Ok(())
}

/// Validate upgrade is safe for governance approval
///
/// Performs comprehensive safety checks to ensure upgrade can be safely
/// approved and executed.
pub fn validate_upgrade_safety(
    env: &Env,
    upgrade_id: u64,
) -> Result<UpgradeSafetyReport, String> {
    let proposal = storage::get_proposal(env, upgrade_id)?;

    let mut report = UpgradeSafetyReport {
        upgrade_id,
        is_safe: true,
        passed_checks: Vec::new(env),
        failed_checks: Vec::new(env),
        warnings: Vec::new(env),
        compatibility_score: 0,
        timestamp: env.ledger().timestamp(),
    };

    // Check 1: Version compatibility
    let compat = version::check_compatibility(env, &proposal.from_version, &proposal.to_version);
    if compat.is_compatible {
        report.passed_checks.push_back(String::from_str(env, "Version compatibility check passed"));
        report.compatibility_score += 25;
    } else {
        report.is_safe = false;
        report.failed_checks.push_back(String::from_str(env, "Version compatibility check failed"));
    }

    // Check 2: Code hash validation
    if proposal.to_version.code_hash.len() > 0 {
        report.passed_checks.push_back(String::from_str(env, "Code hash present"));
        report.compatibility_score += 25;
    } else {
        report.warnings.push_back(String::from_str(env, "No code hash provided"));
    }

    // Check 3: Migration handler validation
    if proposal.migration_handler.is_some() {
        report.passed_checks.push_back(String::from_str(env, "Migration handler configured"));
        report.compatibility_score += 25;
    } else if proposal.upgrade_type == crate::upgrade::types::UpgradeType::WithMigration {
        report
            .warnings
            .push_back(String::from_str(env, "Migration upgrade without handler"));
    } else {
        report.compatibility_score += 25;
    }

    // Check 4: Code size validation
    let config = storage::get_upgrade_config(env)?;
    if (proposal.new_code.len() as u64) <= config.max_upgrade_size {
        report.passed_checks.push_back(String::from_str(env, "Code size validation passed"));
        report.compatibility_score += 25;
    } else {
        report.is_safe = false;
        report.failed_checks.push_back(String::from_str(
            env,
            "Code size exceeds maximum allowed",
        ));
    }

    Ok(report)
}

/// Report of upgrade safety validation
#[derive(Clone, Debug)]
pub struct UpgradeSafetyReport {
    pub upgrade_id: u64,
    pub is_safe: bool,
    pub passed_checks: soroban_sdk::Vec<String>,
    pub failed_checks: soroban_sdk::Vec<String>,
    pub warnings: soroban_sdk::Vec<String>,
    pub compatibility_score: u32,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upgrade::types::{UpgradeConfig, UpgradeProposal, UpgradeType, VersionInfo};
    use soroban_sdk::Env;

    #[test]
    fn test_link_governance_proposal() {
        let env = Env::default();

        let v1 = VersionInfo {
            major: 1,
            minor: 0,
            patch: 0,
            build: String::from_str(&env, "build1"),
            code_hash: soroban_sdk::Bytes::new(&env),
            released_at: 1000,
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
        let upgrade_id = crate::upgrade::propose_upgrade(
            &env,
            v1.clone(),
            v1,
            UpgradeType::CodeOnly,
            soroban_sdk::Bytes::new(&env),
            None,
            String::from_str(&env, "test"),
            proposer,
        )
        .unwrap();

        let result = link_governance_proposal(&env, upgrade_id, 42);
        assert!(result.is_ok());

        let proposal = storage::get_proposal(&env, upgrade_id).unwrap();
        assert_eq!(proposal.governance_proposal_id, 42);
    }

    #[test]
    fn test_validate_upgrade_safety() {
        let env = Env::default();

        let v1 = VersionInfo {
            major: 1,
            minor: 0,
            patch: 0,
            build: String::from_str(&env, "build1"),
            code_hash: soroban_sdk::Bytes::from_array(&env, &[1u8; 32]),
            released_at: 1000,
        };

        let v2 = VersionInfo {
            major: 1,
            minor: 1,
            patch: 0,
            build: String::from_str(&env, "build2"),
            code_hash: soroban_sdk::Bytes::from_array(&env, &[2u8; 32]),
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
        let upgrade_id = crate::upgrade::propose_upgrade(
            &env,
            v1,
            v2,
            UpgradeType::CodeOnly,
            soroban_sdk::Bytes::new(&env),
            None,
            String::from_str(&env, "test"),
            proposer,
        )
        .unwrap();

        let report = validate_upgrade_safety(&env, upgrade_id).unwrap();
        assert!(report.is_safe);
        assert!(report.passed_checks.len() > 0);
        assert!(report.failed_checks.len() == 0);
    }
}
