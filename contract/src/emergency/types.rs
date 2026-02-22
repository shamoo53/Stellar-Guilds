use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmergencyStatus {
    Active,
    Inactive,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyConfig {
    pub status: EmergencyStatus,
    pub paused_at: u64,
    pub expires_at: u64,
    pub paused_by: Option<Address>,
    pub emergency_contact: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyActionLog {
    pub action: String,
    pub performed_by: Address,
    pub timestamp: u64,
    pub reason: String,
}
