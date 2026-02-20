use soroban_sdk::{symbol_short, Address, Env, Map, Symbol, Vec};

use super::types::TokenAllowance;

// ── Storage Keys ─────────────────────────────────────────────────────────────

/// Main allowance map: (owner, spender, token) → TokenAllowance
const ALLOWANCES_KEY: Symbol = symbol_short!("tkn_alw");

/// Owner index: owner → Vec<(spender, token)>
const OWNER_IDX_KEY: Symbol = symbol_short!("alw_oidx");

/// Spender index: spender → Vec<(owner, token)>
const SPENDER_IDX_KEY: Symbol = symbol_short!("alw_sidx");

// ── Composite Key Type ───────────────────────────────────────────────────────

type AllowanceKey = (Address, Address, Option<Address>); // (owner, spender, token)

// ── CRUD ─────────────────────────────────────────────────────────────────────

pub fn store_allowance(env: &Env, allowance: &TokenAllowance) {
    let key: AllowanceKey = (
        allowance.owner.clone(),
        allowance.spender.clone(),
        allowance.token.clone(),
    );

    // Main map
    let mut map: Map<AllowanceKey, TokenAllowance> = env
        .storage()
        .persistent()
        .get(&ALLOWANCES_KEY)
        .unwrap_or_else(|| Map::new(env));
    map.set(key.clone(), allowance.clone());
    env.storage().persistent().set(&ALLOWANCES_KEY, &map);

    // Owner index
    let mut owner_idx: Map<Address, Vec<(Address, Option<Address>)>> = env
        .storage()
        .persistent()
        .get(&OWNER_IDX_KEY)
        .unwrap_or_else(|| Map::new(env));
    let mut entries = owner_idx
        .get(allowance.owner.clone())
        .unwrap_or_else(|| Vec::new(env));
    let pair = (allowance.spender.clone(), allowance.token.clone());
    if !entries.iter().any(|e| e == pair) {
        entries.push_back(pair.clone());
        owner_idx.set(allowance.owner.clone(), entries);
        env.storage().persistent().set(&OWNER_IDX_KEY, &owner_idx);
    }

    // Spender index
    let mut spender_idx: Map<Address, Vec<(Address, Option<Address>)>> = env
        .storage()
        .persistent()
        .get(&SPENDER_IDX_KEY)
        .unwrap_or_else(|| Map::new(env));
    let mut s_entries = spender_idx
        .get(allowance.spender.clone())
        .unwrap_or_else(|| Vec::new(env));
    let s_pair = (allowance.owner.clone(), allowance.token.clone());
    if !s_entries.iter().any(|e| e == s_pair) {
        s_entries.push_back(s_pair.clone());
        spender_idx.set(allowance.spender.clone(), s_entries);
        env.storage()
            .persistent()
            .set(&SPENDER_IDX_KEY, &spender_idx);
    }
}

pub fn get_allowance(
    env: &Env,
    owner: &Address,
    spender: &Address,
    token: &Option<Address>,
) -> Option<TokenAllowance> {
    let key: AllowanceKey = (owner.clone(), spender.clone(), token.clone());

    let map: Map<AllowanceKey, TokenAllowance> = env
        .storage()
        .persistent()
        .get(&ALLOWANCES_KEY)
        .unwrap_or_else(|| Map::new(env));
    map.get(key)
}

pub fn delete_allowance(env: &Env, owner: &Address, spender: &Address, token: &Option<Address>) {
    let key: AllowanceKey = (owner.clone(), spender.clone(), token.clone());

    // Remove from main map
    let mut map: Map<AllowanceKey, TokenAllowance> = env
        .storage()
        .persistent()
        .get(&ALLOWANCES_KEY)
        .unwrap_or_else(|| Map::new(env));
    map.remove(key);
    env.storage().persistent().set(&ALLOWANCES_KEY, &map);

    // Remove from owner index
    let mut owner_idx: Map<Address, Vec<(Address, Option<Address>)>> = env
        .storage()
        .persistent()
        .get(&OWNER_IDX_KEY)
        .unwrap_or_else(|| Map::new(env));
    if let Some(mut entries) = owner_idx.get(owner.clone()) {
        let pair = (spender.clone(), token.clone());
        let mut new_entries = Vec::new(env);
        for e in entries.iter() {
            if e != pair {
                new_entries.push_back(e);
            }
        }
        entries = new_entries;
        owner_idx.set(owner.clone(), entries);
        env.storage().persistent().set(&OWNER_IDX_KEY, &owner_idx);
    }

    // Remove from spender index
    let mut spender_idx: Map<Address, Vec<(Address, Option<Address>)>> = env
        .storage()
        .persistent()
        .get(&SPENDER_IDX_KEY)
        .unwrap_or_else(|| Map::new(env));
    if let Some(mut entries) = spender_idx.get(spender.clone()) {
        let pair = (owner.clone(), token.clone());
        let mut new_entries = Vec::new(env);
        for e in entries.iter() {
            if e != pair {
                new_entries.push_back(e);
            }
        }
        entries = new_entries;
        spender_idx.set(spender.clone(), entries);
        env.storage()
            .persistent()
            .set(&SPENDER_IDX_KEY, &spender_idx);
    }
}

// ── Index Queries ────────────────────────────────────────────────────────────

/// List all allowances granted by `owner`.
pub fn list_by_owner(env: &Env, owner: &Address) -> Vec<TokenAllowance> {
    let owner_idx: Map<Address, Vec<(Address, Option<Address>)>> = env
        .storage()
        .persistent()
        .get(&OWNER_IDX_KEY)
        .unwrap_or_else(|| Map::new(env));

    let entries = owner_idx
        .get(owner.clone())
        .unwrap_or_else(|| Vec::new(env));

    let map: Map<AllowanceKey, TokenAllowance> = env
        .storage()
        .persistent()
        .get(&ALLOWANCES_KEY)
        .unwrap_or_else(|| Map::new(env));

    let mut result = Vec::new(env);
    for (spender, tk) in entries.iter() {
        let key: AllowanceKey = (owner.clone(), spender, tk);
        if let Some(allowance) = map.get(key) {
            result.push_back(allowance);
        }
    }
    result
}

/// List all allowances where `spender` is the beneficiary.
pub fn list_by_spender(env: &Env, spender: &Address) -> Vec<TokenAllowance> {
    let spender_idx: Map<Address, Vec<(Address, Option<Address>)>> = env
        .storage()
        .persistent()
        .get(&SPENDER_IDX_KEY)
        .unwrap_or_else(|| Map::new(env));

    let entries = spender_idx
        .get(spender.clone())
        .unwrap_or_else(|| Vec::new(env));

    let map: Map<AllowanceKey, TokenAllowance> = env
        .storage()
        .persistent()
        .get(&ALLOWANCES_KEY)
        .unwrap_or_else(|| Map::new(env));

    let mut result = Vec::new(env);
    for (owner, tk) in entries.iter() {
        let key: AllowanceKey = (owner, spender.clone(), tk);
        if let Some(allowance) = map.get(key) {
            result.push_back(allowance);
        }
    }
    result
}
