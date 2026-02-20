use soroban_sdk::{Address, Env, String, Symbol};

use crate::guild::types::Role;
use crate::reputation::storage::{
    count_contributions_by_type, get_badges, get_next_badge_id, get_next_contribution_id,
    get_profile, has_badge_type, store_badge, store_contribution, store_profile,
};
use crate::reputation::types::{
    points_for_contribution, Badge, BadgeAwardedEvent, BadgeType, ContributionRecord,
    ContributionType, ReputationProfile, ReputationUpdatedEvent, DECAY_DENOMINATOR,
    DECAY_NUMERATOR, DECAY_PERIOD_SECS,
};

use crate::governance::types::role_weight;

// ────────────────────── Core Scoring ──────────────────────

/// Record a contribution and update the user's reputation profile.
/// Awards badges if thresholds are met.
pub fn record_contribution(
    env: &Env,
    guild_id: u64,
    contributor: &Address,
    contribution_type: ContributionType,
    reference_id: u64,
) {
    let points = points_for_contribution(&contribution_type);
    let now = env.ledger().timestamp();

    // Store the contribution record
    let contrib_id = get_next_contribution_id(env);
    let record = ContributionRecord {
        id: contrib_id,
        guild_id,
        contributor: contributor.clone(),
        contribution_type: contribution_type.clone(),
        points,
        timestamp: now,
        reference_id,
    };
    store_contribution(env, &record);

    // Update or create reputation profile
    let mut profile = get_profile(env, contributor, guild_id).unwrap_or(ReputationProfile {
        address: contributor.clone(),
        guild_id,
        total_score: 0,
        decayed_score: 0,
        contributions_count: 0,
        last_activity: now,
        last_decay_applied: now,
    });

    // Apply pending decay before adding new points
    apply_decay_to_profile(&mut profile, now);

    profile.total_score += points as u64;
    profile.decayed_score += points as u64;
    profile.contributions_count += 1;
    profile.last_activity = now;
    store_profile(env, &profile);

    // Emit reputation updated event
    let event = ReputationUpdatedEvent {
        guild_id,
        contributor: contributor.clone(),
        points_earned: points,
        new_total_score: profile.total_score,
        contribution_type,
    };
    env.events().publish(
        (Symbol::new(env, "reputation"), Symbol::new(env, "updated")),
        event,
    );

    // Check and award badges
    check_and_award_badges(env, guild_id, contributor, &profile);
}

// ────────────────────── Decay ──────────────────────

/// Apply time-based decay to a profile's decayed_score.
/// Uses iterative multiplication by 99/100 per elapsed period.
fn apply_decay_to_profile(profile: &mut ReputationProfile, now: u64) {
    if now <= profile.last_decay_applied {
        return;
    }

    let elapsed = now - profile.last_decay_applied;
    let periods = elapsed / DECAY_PERIOD_SECS;

    if periods == 0 {
        return;
    }

    // Cap iterations to avoid excessive gas usage
    let capped_periods = if periods > 52 { 52 } else { periods };

    let mut score = profile.decayed_score;
    for _ in 0..capped_periods {
        score = (score * DECAY_NUMERATOR) / DECAY_DENOMINATOR;
    }

    profile.decayed_score = score;
    profile.last_decay_applied = now;
}

/// Get a profile with decay applied (read-only, does not persist).
pub fn get_decayed_profile(
    env: &Env,
    address: &Address,
    guild_id: u64,
) -> Option<ReputationProfile> {
    let mut profile = get_profile(env, address, guild_id)?;
    let now = env.ledger().timestamp();
    apply_decay_to_profile(&mut profile, now);
    Some(profile)
}

// ────────────────────── Governance Weight ──────────────────────

/// Compute governance weight: role_weight + integer_sqrt(decayed_score).
/// Falls back to role_weight only if no reputation profile exists.
pub fn compute_governance_weight(env: &Env, address: &Address, guild_id: u64, role: &Role) -> i128 {
    let base = role_weight(role);

    let reputation_bonus = match get_decayed_profile(env, address, guild_id) {
        Some(profile) => integer_sqrt(profile.decayed_score) as i128,
        None => 0,
    };

    base + reputation_bonus
}

/// Get the global (cross-guild) reputation for a user.
pub fn get_global_reputation(env: &Env, address: &Address) -> u64 {
    let profiles = crate::reputation::storage::get_all_guild_profiles(env, address);
    let now = env.ledger().timestamp();

    let mut total: u64 = 0;
    for mut profile in profiles.iter() {
        apply_decay_to_profile(&mut profile, now);
        total += profile.decayed_score;
    }
    total
}

// ────────────────────── Badge Logic ──────────────────────

/// Check badge criteria and award any newly earned badges.
fn check_and_award_badges(
    env: &Env,
    guild_id: u64,
    contributor: &Address,
    profile: &ReputationProfile,
) {
    let now = env.ledger().timestamp();

    // FirstContribution — contributions_count >= 1
    if profile.contributions_count >= 1 {
        maybe_award_badge(
            env,
            guild_id,
            contributor,
            BadgeType::FirstContribution,
            "First Contribution",
            now,
        );
    }

    // BountyHunter — 5+ bounties completed
    let bounty_count = count_contributions_by_type(
        env,
        contributor,
        guild_id,
        &ContributionType::BountyCompleted,
    );
    if bounty_count >= 5 {
        maybe_award_badge(
            env,
            guild_id,
            contributor,
            BadgeType::BountyHunter,
            "Bounty Hunter",
            now,
        );
    }

    // Mentor — 10+ milestones approved
    let milestone_count = count_contributions_by_type(
        env,
        contributor,
        guild_id,
        &ContributionType::MilestoneApproved,
    );
    if milestone_count >= 10 {
        maybe_award_badge(env, guild_id, contributor, BadgeType::Mentor, "Mentor", now);
    }

    // Governor — 10+ votes cast
    let vote_count =
        count_contributions_by_type(env, contributor, guild_id, &ContributionType::VoteCast);
    if vote_count >= 10 {
        maybe_award_badge(
            env,
            guild_id,
            contributor,
            BadgeType::Governor,
            "Governor",
            now,
        );
    }

    // Veteran — score > 1000
    if profile.total_score > 1000 {
        maybe_award_badge(
            env,
            guild_id,
            contributor,
            BadgeType::Veteran,
            "Veteran",
            now,
        );
    }
}

/// Award a badge if the user doesn't already have it.
fn maybe_award_badge(
    env: &Env,
    guild_id: u64,
    holder: &Address,
    badge_type: BadgeType,
    name: &str,
    timestamp: u64,
) {
    if has_badge_type(env, holder, guild_id, &badge_type) {
        return;
    }

    let badge_id = get_next_badge_id(env);
    let badge_name = String::from_str(env, name);
    let badge = Badge {
        id: badge_id,
        guild_id,
        holder: holder.clone(),
        badge_type: badge_type.clone(),
        name: badge_name.clone(),
        awarded_at: timestamp,
    };
    store_badge(env, &badge);

    let event = BadgeAwardedEvent {
        guild_id,
        holder: holder.clone(),
        badge_type,
        badge_name,
    };
    env.events().publish(
        (Symbol::new(env, "reputation"), Symbol::new(env, "badge")),
        event,
    );
}

// ────────────────────── Helpers ──────────────────────

/// Integer square root using Newton's method.
fn integer_sqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
