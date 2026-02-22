use crate::emergency::storage::{get_emergency_config, log_emergency_action, set_emergency_config};
use crate::emergency::types::{EmergencyConfig, EmergencyStatus};
use crate::multisig::storage::get_operation;
use crate::multisig::types::{MultiSigOperation, OperationStatus, OperationType};
use soroban_sdk::{Address, Env, String};

pub fn pause_contract(
    env: &Env,
    multisig_op_id: u64,
    duration_seconds: u64,
    reason: String,
    emergency_contact: String,
) -> bool {
    let op = get_operation(env, multisig_op_id).expect("Operation not found");

    if op.status != OperationStatus::Executed {
        panic!("Multisig operation not executed");
    }

    if op.op_type != OperationType::EmergencyAction {
        panic!("Invalid operation type");
    }

    const MIN_DURATION: u64 = 7 * 24 * 60 * 60;
    const MAX_DURATION: u64 = 30 * 24 * 60 * 60;

    if duration_seconds < MIN_DURATION || duration_seconds > MAX_DURATION {
        panic!("Duration must be between 7 and 30 days");
    }

    let current_time = env.ledger().timestamp();
    let config = EmergencyConfig {
        status: EmergencyStatus::Active,
        paused_at: current_time,
        expires_at: current_time + duration_seconds,
        paused_by: Some(op.proposer.clone()),
        emergency_contact,
    };

    set_emergency_config(env, &config);

    log_emergency_action(env, String::from_str(env, "Pause"), op.proposer, reason);

    true
}

pub fn resume_contract(env: &Env, multisig_op_id: u64, reason: String) -> bool {
    let op = get_operation(env, multisig_op_id).expect("Operation not found");

    if op.status != OperationStatus::Executed {
        panic!("Multisig operation not executed");
    }

    if op.op_type != OperationType::EmergencyAction {
        panic!("Invalid operation type");
    }

    let mut config = get_emergency_config(env);
    config.status = EmergencyStatus::Inactive;
    config.expires_at = 0;

    set_emergency_config(env, &config);

    log_emergency_action(env, String::from_str(env, "Resume"), op.proposer, reason);

    true
}
