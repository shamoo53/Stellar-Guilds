use soroban_sdk::{contracttype, Address, String};

/// Represents a Guild's configuration in Soroban storage
#[contracttype]
#[derive(Clone, Debug)]
pub struct GuildConfig {
    pub name: String,
    pub admin: Address,
    pub member_limits: u32,
}

/// Enum for Guild related states/types
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildStatus {
    Active = 0,
    Inactive = 1,
}
