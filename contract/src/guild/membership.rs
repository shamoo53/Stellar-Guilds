use crate::events::emit::emit_event;
use crate::events::topics::{
    ACT_CREATED, ACT_JOINED, ACT_MEMBER_ADDED, ACT_MEMBER_REMOVED, ACT_ROLE_UPDATED, MOD_GUILD,
};
use crate::guild::storage;
use crate::guild::types::{
    Guild, GuildCreatedEvent, GuildJoinedEvent, Member, MemberAddedEvent, MemberRemovedEvent, Role,
    RoleUpdatedEvent,
};
use soroban_sdk::{Address, Env, String, Vec};

/// Create a new guild
///
/// # Events emitted
/// - `(guild, created)` â†’ `GuildCreatedEvent`
///
/// # Arguments
/// * `env` - The contract environment
/// * `name` - The name of the guild (1â€“256 chars)
/// * `description` - The description of the guild (max 512 chars)
/// * `owner` - The address of the guild owner
///
/// # Returns
/// The ID of the newly created guild
///
/// # Errors
/// Returns `Err` if name or description violate length constraints.
pub fn create_guild(
    env: &Env,
    name: String,
    description: String,
    owner: Address,
) -> Result<u64, String> {
    if name.len() == 0 || name.len() > 256 {
        return Err(String::from_str(
            env,
            "Guild name must be between 1 and 256 characters",
        ));
    }
    if description.len() > 512 {
        return Err(String::from_str(
            env,
            "Guild description must be at most 512 characters",
        ));
    }

    let guild_id = storage::get_next_guild_id(env);
    let timestamp = env.ledger().timestamp();

    let guild = Guild {
        id: guild_id,
        name: name.clone(),
        description,
        owner: owner.clone(),
        created_at: timestamp,
        member_count: 1,
    };
    storage::store_guild(env, &guild);

    let owner_member = Member {
        address: owner.clone(),
        role: Role::Owner,
        joined_at: timestamp,
    };
    storage::store_member(env, guild_id, &owner_member);

    emit_event(
        env,
        MOD_GUILD,
        ACT_CREATED,
        GuildCreatedEvent {
            guild_id,
            owner,
            name,
            created_at: timestamp,
        },
    );

    Ok(guild_id)
}

/// Add a member to a guild
///
/// # Events emitted
/// - `(guild, member_added)` â†’ `MemberAddedEvent`
///
/// # Arguments
/// * `env`      - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address`  - The address of the member to add
/// * `role`     - The role to assign
/// * `caller`   - The address making the request (must have permission)
///
/// # Errors
/// - Guild not found
/// - Member already exists
/// - Caller lacks permission for the requested role
pub fn add_member(
    env: &Env,
    guild_id: u64,
    address: Address,
    role: Role,
    caller: Address,
) -> Result<bool, String> {
    let guild =
        storage::get_guild(env, guild_id).ok_or(String::from_str(env, "Guild not found"))?;

    if storage::has_member(env, guild_id, &address) {
        return Err(String::from_str(env, "Member already exists in guild"));
    }

    let caller_member = storage::get_member(env, guild_id, &caller)
        .ok_or(String::from_str(env, "Caller is not a member of the guild"))?;

    match role {
        Role::Owner => {
            if caller_member.role != Role::Owner {
                return Err(String::from_str(env, "Only owner can add new owners"));
            }
        }
        Role::Admin => {
            if caller_member.role != Role::Owner && caller_member.role != Role::Admin {
                return Err(String::from_str(env, "Only owner or admin can add admins"));
            }
        }
        Role::Member | Role::Contributor => {
            if !caller_member.role.has_permission(&Role::Member) {
                return Err(String::from_str(
                    env,
                    "Insufficient permissions to add members",
                ));
            }
        }
    }

    let timestamp = env.ledger().timestamp();
    let member = Member {
        address: address.clone(),
        role: role.clone(),
        joined_at: timestamp,
    };
    storage::store_member(env, guild_id, &member);

    let mut updated_guild = guild;
    updated_guild.member_count += 1;
    storage::update_guild(env, &updated_guild);

    emit_event(
        env,
        MOD_GUILD,
        ACT_MEMBER_ADDED,
        MemberAddedEvent {
            guild_id,
            address,
            role,
            joined_at: timestamp,
        },
    );

    Ok(true)
}

/// Remove a member from a guild
///
/// # Events emitted
/// - `(guild, member_removed)` â†’ `MemberRemovedEvent`
///
/// # Arguments
/// * `env`      - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address`  - The address of the member to remove
/// * `caller`   - The address making the request (self-removal is always allowed)
///
/// # Errors
/// - Guild or member not found
/// - Attempting to remove the last owner
/// - Caller lacks permission to remove the target member
pub fn remove_member(
    env: &Env,
    guild_id: u64,
    address: Address,
    caller: Address,
) -> Result<bool, String> {
    let guild =
        storage::get_guild(env, guild_id).ok_or(String::from_str(env, "Guild not found"))?;

    let member = storage::get_member(env, guild_id, &address)
        .ok_or(String::from_str(env, "Member not found"))?;

    let is_self_removal = caller == address;

    if member.role == Role::Owner {
        let owner_count = storage::count_owners(env, guild_id);
        if owner_count <= 1 {
            return Err(String::from_str(env, "Cannot remove the last owner"));
        }
    }

    if !is_self_removal {
        let caller_member = storage::get_member(env, guild_id, &caller)
            .ok_or(String::from_str(env, "Caller is not a member of the guild"))?;

        match member.role {
            Role::Owner => {
                if caller_member.role != Role::Owner {
                    return Err(String::from_str(env, "Only owner can remove owners"));
                }
                let owner_count = storage::count_owners(env, guild_id);
                if owner_count <= 1 {
                    return Err(String::from_str(env, "Cannot remove the last owner"));
                }
            }
            Role::Admin => {
                if caller_member.role != Role::Owner && caller_member.role != Role::Admin {
                    return Err(String::from_str(
                        env,
                        "Only owner or admin can remove admins",
                    ));
                }
            }
            Role::Member | Role::Contributor => {
                if !caller_member.role.has_permission(&Role::Member) {
                    return Err(String::from_str(
                        env,
                        "Insufficient permissions to remove members",
                    ));
                }
            }
        }
    }

    storage::remove_member(env, guild_id, &address);

    let mut updated_guild = guild;
    updated_guild.member_count = updated_guild.member_count.saturating_sub(1);
    storage::update_guild(env, &updated_guild);

    emit_event(
        env,
        MOD_GUILD,
        ACT_MEMBER_REMOVED,
        MemberRemovedEvent { guild_id, address },
    );

    Ok(true)
}

/// Update a member's role
///
/// # Events emitted
/// - `(guild, role_updated)` â†’ `RoleUpdatedEvent`
///
/// # Arguments
/// * `env`      - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address`  - The address of the member whose role is changing
/// * `new_role` - The new role to assign
/// * `caller`   - The address making the request (must have permission)
///
/// # Errors
/// - Guild or member not found
/// - Caller lacks permission
/// - Attempting to demote the last owner
pub fn update_role(
    env: &Env,
    guild_id: u64,
    address: Address,
    new_role: Role,
    caller: Address,
) -> Result<bool, String> {
    let _guild =
        storage::get_guild(env, guild_id).ok_or(String::from_str(env, "Guild not found"))?;

    let member = storage::get_member(env, guild_id, &address)
        .ok_or(String::from_str(env, "Member not found"))?;

    let caller_member = storage::get_member(env, guild_id, &caller)
        .ok_or(String::from_str(env, "Caller is not a member of the guild"))?;

    match member.role {
        Role::Owner => {
            if caller_member.role != Role::Owner {
                return Err(String::from_str(env, "Only owner can change owner role"));
            }
            if new_role != Role::Owner {
                let owner_count = storage::count_owners(env, guild_id);
                if owner_count <= 1 {
                    return Err(String::from_str(env, "Cannot demote the last owner"));
                }
            }
        }
        Role::Admin => {
            if caller_member.role != Role::Owner && caller_member.role != Role::Admin {
                return Err(String::from_str(
                    env,
                    "Only owner or admin can change admin role",
                ));
            }
        }
        Role::Member | Role::Contributor => {
            if caller_member.role != Role::Owner && caller_member.role != Role::Admin {
                return Err(String::from_str(
                    env,
                    "Insufficient permissions to change member role",
                ));
            }
        }
    }

    let old_role = member.role.clone();

    let updated_member = Member {
        address: address.clone(),
        role: new_role.clone(),
        joined_at: member.joined_at,
    };
    storage::store_member(env, guild_id, &updated_member);

    emit_event(
        env,
        MOD_GUILD,
        ACT_ROLE_UPDATED,
        RoleUpdatedEvent {
            guild_id,
            address,
            old_role,
            new_role,
        },
    );

    Ok(true)
}

/// Self-join a guild
///
/// Allows any address to add themselves to an existing guild as a `Member`.
/// The caller **must** have signed the transaction — `caller.require_auth()`
/// is enforced inside this function so that the Soroban runtime verifies the
/// authorization before any state is written.
///
/// # Events emitted
/// - `(guild, joined)` → `GuildJoinedEvent`
///
/// # Arguments
/// * `env`      - The contract environment
/// * `guild_id` - The ID of the guild to join
/// * `caller`   - The address of the account joining (must sign the transaction)
///
/// # Returns
/// `Ok(true)` on success.
///
/// # Errors
/// - `"Guild not found"` — no guild exists with the given `guild_id`.
/// - `"Already a member of this guild"` — `caller` is already in the membership map.
pub fn join_guild(env: &Env, guild_id: u64, caller: Address) -> Result<bool, String> {
    // Soroban auth: the caller must have signed this invocation.
    caller.require_auth();

    let guild =
        storage::get_guild(env, guild_id).ok_or(String::from_str(env, "Guild not found"))?;

    if storage::has_member(env, guild_id, &caller) {
        return Err(String::from_str(env, "Already a member of this guild"));
    }

    let timestamp = env.ledger().timestamp();
    let member = Member {
        address: caller.clone(),
        role: Role::Member,
        joined_at: timestamp,
    };
    storage::store_member(env, guild_id, &member);

    let mut updated_guild = guild;
    updated_guild.member_count += 1;
    storage::update_guild(env, &updated_guild);

    emit_event(
        env,
        MOD_GUILD,
        ACT_JOINED,
        GuildJoinedEvent {
            guild_id,
            caller,
            joined_at: timestamp,
        },
    );

    Ok(true)
}

// ─── Query helpers (no events) ────────────────────────────────────────────────

pub fn get_member(env: &Env, guild_id: u64, address: Address) -> Result<Member, String> {
    storage::get_member(env, guild_id, &address).ok_or(String::from_str(env, "Member not found"))
}

pub fn get_all_members(env: &Env, guild_id: u64) -> Vec<Member> {
    storage::get_all_members(env, guild_id)
}

pub fn is_member(env: &Env, guild_id: u64, address: Address) -> bool {
    storage::has_member(env, guild_id, &address)
}

pub fn has_permission(env: &Env, guild_id: u64, address: Address, required_role: Role) -> bool {
    if let Some(member) = storage::get_member(env, guild_id, &address) {
        member.role.has_permission(&required_role)
    } else {
        false
    }
}
