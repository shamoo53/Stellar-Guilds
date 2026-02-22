use crate::emergency::types::{EmergencyConfig, EmergencyStatus};
use soroban_sdk::{contracttype, Address, Env, String};

#[contracttype]
pub enum DataKey {
    EmergencyConfig,
    EmergencyLog(u64),
    LogCounter,
}

pub fn get_emergency_config(env: &Env) -> EmergencyConfig {
    env.storage()
        .instance()
        .get(&DataKey::EmergencyConfig)
        .unwrap_or(EmergencyConfig {
            status: EmergencyStatus::Inactive,
            paused_at: 0,
            expires_at: 0,
            paused_by: None,
            emergency_contact: String::from_str(env, ""),
        })
}

pub fn set_emergency_config(env: &Env, config: &EmergencyConfig) {
    env.storage()
        .instance()
        .set(&DataKey::EmergencyConfig, config);
}

pub fn is_paused(env: &Env) -> bool {
    let config = get_emergency_config(env);
    if config.status == EmergencyStatus::Active {
        if env.ledger().timestamp() > config.expires_at {
            let mut new_config = config.clone();
            new_config.status = EmergencyStatus::Inactive;
            set_emergency_config(env, &new_config);
            return false;
        }
        return true;
    }
    false
}

pub fn next_log_id(env: &Env) -> u64 {
    let mut count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::LogCounter)
        .unwrap_or(0);
    count += 1;
    env.storage().instance().set(&DataKey::LogCounter, &count);
    count
}

pub fn log_emergency_action(env: &Env, action: String, performed_by: Address, reason: String) {
    use crate::emergency::types::EmergencyActionLog;

    let log = EmergencyActionLog {
        action,
        performed_by,
        timestamp: env.ledger().timestamp(),
        reason,
    };

    let id = next_log_id(env);
    env.storage()
        .persistent()
        .set(&DataKey::EmergencyLog(id), &log);

    env.events().publish(
        (
            String::from_str(env, "emergency_action"),
            log.action.clone(),
        ),
        log,
    );
}
