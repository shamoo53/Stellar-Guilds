use soroban_sdk::{symbol_short, Address, Env, Map, Symbol, Vec};

use crate::reputation::types::{Badge, ContributionRecord, ReputationProfile};

const PROFILES_KEY: Symbol = symbol_short!("r_prof");
const CONTRIBS_KEY: Symbol = symbol_short!("r_cont");
const CONTRIB_IDX: Symbol = symbol_short!("r_cidx");
const BADGES_KEY: Symbol = symbol_short!("r_badge");
const BADGE_IDX: Symbol = symbol_short!("r_bidx");
const CONTRIB_CNT: Symbol = symbol_short!("r_ccnt");
const BADGE_CNT: Symbol = symbol_short!("r_bcnt");

// ────────────────────── Reputation Profiles ──────────────────────

/// Store or update a reputation profile keyed by (address, guild_id).
pub fn store_profile(env: &Env, profile: &ReputationProfile) {
    let storage = env.storage().persistent();
    let mut profiles: Map<(Address, u64), ReputationProfile> =
        storage.get(&PROFILES_KEY).unwrap_or_else(|| Map::new(env));
    profiles.set((profile.address.clone(), profile.guild_id), profile.clone());
    storage.set(&PROFILES_KEY, &profiles);
}

/// Get a reputation profile for (address, guild_id). Returns None if not found.
pub fn get_profile(env: &Env, address: &Address, guild_id: u64) -> Option<ReputationProfile> {
    let storage = env.storage().persistent();
    let profiles: Map<(Address, u64), ReputationProfile> =
        storage.get(&PROFILES_KEY).unwrap_or_else(|| Map::new(env));
    profiles.get((address.clone(), guild_id))
}

/// Get all guild IDs that an address has reputation in.
pub fn get_all_guild_profiles(env: &Env, address: &Address) -> Vec<ReputationProfile> {
    let storage = env.storage().persistent();
    let profiles: Map<(Address, u64), ReputationProfile> =
        storage.get(&PROFILES_KEY).unwrap_or_else(|| Map::new(env));

    let mut result = Vec::new(env);
    for entry in profiles.iter() {
        let ((addr, _gid), profile) = entry;
        if addr == address.clone() {
            result.push_back(profile);
        }
    }
    result
}

// ────────────────────── Contributions ──────────────────────

/// Get next contribution ID (global counter).
pub fn get_next_contribution_id(env: &Env) -> u64 {
    let storage = env.storage().persistent();
    let count: u64 = storage.get(&CONTRIB_CNT).unwrap_or(0u64);
    storage.set(&CONTRIB_CNT, &(count + 1));
    count + 1
}

/// Store a contribution record.
pub fn store_contribution(env: &Env, record: &ContributionRecord) {
    let storage = env.storage().persistent();

    // Store by ID
    let mut contribs: Map<u64, ContributionRecord> =
        storage.get(&CONTRIBS_KEY).unwrap_or_else(|| Map::new(env));
    contribs.set(record.id, record.clone());
    storage.set(&CONTRIBS_KEY, &contribs);

    // Update per-user per-guild index
    let mut index: Map<(Address, u64), Vec<u64>> =
        storage.get(&CONTRIB_IDX).unwrap_or_else(|| Map::new(env));
    let key = (record.contributor.clone(), record.guild_id);
    let mut ids = index.get(key.clone()).unwrap_or_else(|| Vec::new(env));
    ids.push_back(record.id);
    index.set(key, ids);
    storage.set(&CONTRIB_IDX, &index);
}

/// Get contribution records for a user in a guild, most recent first, limited.
pub fn get_contributions(
    env: &Env,
    address: &Address,
    guild_id: u64,
    limit: u32,
) -> Vec<ContributionRecord> {
    let storage = env.storage().persistent();

    let index: Map<(Address, u64), Vec<u64>> =
        storage.get(&CONTRIB_IDX).unwrap_or_else(|| Map::new(env));

    let ids = index
        .get((address.clone(), guild_id))
        .unwrap_or_else(|| Vec::new(env));

    let contribs: Map<u64, ContributionRecord> =
        storage.get(&CONTRIBS_KEY).unwrap_or_else(|| Map::new(env));

    let mut result = Vec::new(env);
    let len = ids.len();
    let start = if len > limit { len - limit } else { 0 };

    for i in start..len {
        let id = ids.get(i).unwrap();
        if let Some(record) = contribs.get(id) {
            result.push_back(record);
        }
    }
    result
}

/// Count contributions of a specific type for a user in a guild.
pub fn count_contributions_by_type(
    env: &Env,
    address: &Address,
    guild_id: u64,
    contribution_type: &crate::reputation::types::ContributionType,
) -> u32 {
    let storage = env.storage().persistent();

    let index: Map<(Address, u64), Vec<u64>> =
        storage.get(&CONTRIB_IDX).unwrap_or_else(|| Map::new(env));

    let ids = index
        .get((address.clone(), guild_id))
        .unwrap_or_else(|| Vec::new(env));

    let contribs: Map<u64, ContributionRecord> =
        storage.get(&CONTRIBS_KEY).unwrap_or_else(|| Map::new(env));

    let mut count = 0u32;
    for id in ids.iter() {
        if let Some(record) = contribs.get(id) {
            if record.contribution_type == *contribution_type {
                count += 1;
            }
        }
    }
    count
}

// ────────────────────── Badges ──────────────────────

/// Get next badge ID.
pub fn get_next_badge_id(env: &Env) -> u64 {
    let storage = env.storage().persistent();
    let count: u64 = storage.get(&BADGE_CNT).unwrap_or(0u64);
    storage.set(&BADGE_CNT, &(count + 1));
    count + 1
}

/// Store a badge.
pub fn store_badge(env: &Env, badge: &Badge) {
    let storage = env.storage().persistent();

    let mut badges: Map<u64, Badge> = storage.get(&BADGES_KEY).unwrap_or_else(|| Map::new(env));
    badges.set(badge.id, badge.clone());
    storage.set(&BADGES_KEY, &badges);

    // Per-user per-guild index
    let mut index: Map<(Address, u64), Vec<u64>> =
        storage.get(&BADGE_IDX).unwrap_or_else(|| Map::new(env));
    let key = (badge.holder.clone(), badge.guild_id);
    let mut ids = index.get(key.clone()).unwrap_or_else(|| Vec::new(env));
    ids.push_back(badge.id);
    index.set(key, ids);
    storage.set(&BADGE_IDX, &index);
}

/// Get all badges for a user in a guild.
pub fn get_badges(env: &Env, address: &Address, guild_id: u64) -> Vec<Badge> {
    let storage = env.storage().persistent();

    let index: Map<(Address, u64), Vec<u64>> =
        storage.get(&BADGE_IDX).unwrap_or_else(|| Map::new(env));

    let ids = index
        .get((address.clone(), guild_id))
        .unwrap_or_else(|| Vec::new(env));

    let badges: Map<u64, Badge> = storage.get(&BADGES_KEY).unwrap_or_else(|| Map::new(env));

    let mut result = Vec::new(env);
    for id in ids.iter() {
        if let Some(badge) = badges.get(id) {
            result.push_back(badge);
        }
    }
    result
}

/// Check if a user already has a specific badge type in a guild.
pub fn has_badge_type(
    env: &Env,
    address: &Address,
    guild_id: u64,
    badge_type: &crate::reputation::types::BadgeType,
) -> bool {
    let existing = get_badges(env, address, guild_id);
    for badge in existing.iter() {
        if badge.badge_type == *badge_type {
            return true;
        }
    }
    false
}
