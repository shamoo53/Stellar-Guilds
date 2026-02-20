/// Upgrade types and data structures for the Stellar Guilds contract
///
/// This module defines all types used in the upgrade system, including:
/// - UpgradeProposal: Configuration for upgrade proposals
/// - UpgradeStatus: Current state of an upgrade
/// - VersionInfo: Version tracking information
/// - MigrationHandler: State migration configuration

use soroban_sdk::{Address, Bytes, Map, String, Vec, contracttype};

/// Status of an upgrade operation
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpgradeStatus {
    /// Upgrade is pending governance approval
    Pending = 0,
    /// Upgrade has been approved
    Approved = 1,
    /// Upgrade is ready to execute
    ReadyToExecute = 2,
    /// Upgrade has been executed
    Executed = 3,
    /// Upgrade has been rolled back
    RolledBack = 4,
    /// Upgrade failed during execution
    Failed = 5,
}

impl UpgradeStatus {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => UpgradeStatus::Pending,
            1 => UpgradeStatus::Approved,
            2 => UpgradeStatus::ReadyToExecute,
            3 => UpgradeStatus::Executed,
            4 => UpgradeStatus::RolledBack,
            5 => UpgradeStatus::Failed,
            _ => UpgradeStatus::Pending,
        }
    }

    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}

/// Type of upgrade being performed
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpgradeType {
    /// Code-only upgrade (no state changes)
    CodeOnly = 0,
    /// Code upgrade with state migration
    WithMigration = 1,
    /// Emergency patch upgrade
    Emergency = 2,
    /// Security fix upgrade
    SecurityFix = 3,
}

impl UpgradeType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => UpgradeType::CodeOnly,
            1 => UpgradeType::WithMigration,
            2 => UpgradeType::Emergency,
            3 => UpgradeType::SecurityFix,
            _ => UpgradeType::CodeOnly,
        }
    }

    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}

/// Version information for the contract
#[contracttype]
#[derive(Clone, Debug)]
pub struct VersionInfo {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (features)
    pub minor: u32,
    /// Patch version (fixes)
    pub patch: u32,
    /// Build metadata
    pub build: String,
    /// Version hash for integrity checking
    pub code_hash: Bytes,
    /// Timestamp of release
    pub released_at: u64,
}

/// Upgrade proposal configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct UpgradeProposal {
    /// Unique identifier for the upgrade
    pub id: u64,
    /// From version
    pub from_version: VersionInfo,
    /// To version
    pub to_version: VersionInfo,
    /// Type of upgrade
    pub upgrade_type: UpgradeType,
    /// Current status
    pub status: UpgradeStatus,
    /// Address proposing the upgrade
    pub proposed_by: Address,
    /// New contract code (WASM bytes)
    pub new_code: Bytes,
    /// Migration instructions if applicable
    pub migration_handler: Option<MigrationHandler>,
    /// Timestamp when proposed
    pub proposed_at: u64,
    /// Timestamp when approved (if applicable)
    pub approved_at: u64,
    /// Timestamp when executed (if applicable)
    pub executed_at: u64,
    /// Description of changes
    pub description: String,
    /// Safety checks passed
    pub safety_checks_passed: bool,
    /// Governance proposal ID (if applicable)
    pub governance_proposal_id: u64,
}

/// Configuration for state migration
#[contracttype]
#[derive(Clone, Debug)]
pub struct MigrationHandler {
    /// Migration script identifier
    pub migration_id: String,
    /// Mapping of old keys to new keys
    pub key_mappings: Vec<KeyMapping>,
    /// Data transformations to apply
    pub transformations: Vec<DataTransformation>,
    /// Verification hash
    pub verification_hash: Bytes,
}

/// Key mapping for state migration
#[contracttype]
#[derive(Clone, Debug)]
pub struct KeyMapping {
    /// Old storage key
    pub old_key: String,
    /// New storage key
    pub new_key: String,
    /// Whether to preserve the old key
    pub preserve_old: bool,
}

/// Data transformation configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct DataTransformation {
    /// Target key to transform
    pub key: String,
    /// Type of transformation
    pub transformation_type: String,
    /// Parameters for transformation (e.g., scaling factors)
    pub parameters: Vec<u64>,
}

/// Upgrade simulation result
#[contracttype]
#[derive(Clone, Debug)]
pub struct SimulationResult {
    /// Whether simulation passed
    pub passed: bool,
    /// State changes that would occur
    pub state_changes: Vec<StateChange>,
    /// Warnings or issues found
    pub warnings: Vec<String>,
    /// Gas estimate for execution
    pub estimated_gas: u64,
    /// Timestamp of simulation
    pub simulated_at: u64,
}

/// Single state change during migration
#[contracttype]
#[derive(Clone, Debug)]
pub struct StateChange {
    /// Storage key being changed
    pub key: String,
    /// Old value (if applicable)
    pub old_value: Option<Bytes>,
    /// New value
    pub new_value: Bytes,
    /// Change type
    pub change_type: String,
}

/// Rollback point for reverting upgrades
#[contracttype]
#[derive(Clone, Debug)]
pub struct RollbackPoint {
    /// Identifier for the rollback point
    pub id: u64,
    /// Associated upgrade ID
    pub upgrade_id: u64,
    /// Previous version that can be reverted to
    pub previous_version: VersionInfo,
    /// Snapshot of critical state
    pub state_snapshot: Map<String, Bytes>,
    /// Timestamp of creation
    pub created_at: u64,
    /// Whether this rollback point is active
    pub is_active: bool,
}

/// Configuration for upgrade system
#[contracttype]
#[derive(Clone, Debug)]
pub struct UpgradeConfig {
    /// Minimum time between upgrades (in seconds)
    pub min_upgrade_interval: u64,
    /// Maximum upgrade size in bytes
    pub max_upgrade_size: u64,
    /// Whether emergency upgrades are allowed
    pub allow_emergency_upgrades: bool,
    /// Address that can authorize emergency upgrades
    pub emergency_admin: Address,
    /// Number of rollback points to maintain
    pub rollback_history_size: u32,
    /// Whether state migration is required for this contract
    pub require_state_migration: bool,
}

/// Compatibility check result
#[contracttype]
#[derive(Clone, Debug)]
pub struct CompatibilityCheck {
    /// Whether versions are compatible
    pub is_compatible: bool,
    /// Specific incompatibilities found
    pub incompatibilities: Vec<String>,
    /// Required migrations for compatibility
    pub required_migrations: Vec<String>,
    /// Breaking changes identified
    pub breaking_changes: Vec<String>,
}
