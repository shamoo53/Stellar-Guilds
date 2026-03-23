use soroban_sdk::{contracttype, Address};

/// Role enum for guild members
/// - Owner: Full control over the guild
/// - Admin: Can manage members and contributors
/// - Member: Can participate in guild activities
/// - Contributor: Limited access, read-only in most cases
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Owner = 0,
    Admin = 1,
    Member = 2,
    Contributor = 3,
}

impl Role {
    /// Check if a role has permission for a required role level
    /// Hierarchy: Owner > Admin > Member > Contributor
    pub fn has_permission(&self, required_role: &Role) -> bool {
        match (self, required_role) {
            (Role::Owner, _) => true,
            (Role::Admin, Role::Admin)
            | (Role::Admin, Role::Member)
            | (Role::Admin, Role::Contributor) => true,
            (Role::Member, Role::Member) | (Role::Member, Role::Contributor) => true,
            (Role::Contributor, Role::Contributor) => true,
            _ => false,
        }
    }
}

/// Guild struct containing guild metadata
#[contracttype]
#[derive(Clone, Debug)]
pub struct Guild {
    /// Unique identifier for the guild
    pub id: u64,
    /// Guild name
    pub name: soroban_sdk::String,
    /// Guild description
    pub description: soroban_sdk::String,
    /// Address of the guild owner
    pub owner: Address,
    /// Timestamp when the guild was created (in seconds)
    pub created_at: u64,
    /// Total member count
    pub member_count: u32,
}

/// Guild configuration settings
#[contracttype]
#[derive(Clone, Debug)]
pub struct GuildConfig {
    /// Guild name
    pub name: soroban_sdk::String,
    /// Address of the guild admin/owner
    pub admin: Address,
    /// Maximum number of members allowed
    pub member_limit: u32,
}

/// Member struct representing a guild member
#[contracttype]
#[derive(Clone, Debug)]
pub struct Member {
    /// Address of the member
    pub address: Address,
    /// Role assigned to this member
    pub role: Role,
    /// Timestamp when the member joined (in seconds)
    pub joined_at: u64,
}

/// Event emitted when a guild is created
#[contracttype]
#[derive(Clone, Debug)]
pub struct GuildCreatedEvent {
    pub guild_id: u64,
    pub owner: Address,
    pub name: soroban_sdk::String,
    pub created_at: u64,
}

/// Event emitted when a member is added
#[contracttype]
#[derive(Clone, Debug)]
pub struct MemberAddedEvent {
    pub guild_id: u64,
    pub address: Address,
    pub role: Role,
    pub joined_at: u64,
}

/// Event emitted when a member is removed
#[contracttype]
#[derive(Clone, Debug)]
pub struct MemberRemovedEvent {
    pub guild_id: u64,
    pub address: Address,
}

/// Event emitted when a member's role is updated
#[contracttype]
#[derive(Clone, Debug)]
pub struct RoleUpdatedEvent {
    pub guild_id: u64,
    pub address: Address,
    pub old_role: Role,
    pub new_role: Role,
}
