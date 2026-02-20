use soroban_sdk::{Address, Env, Symbol};

use crate::governance::storage::{get_all_votes, get_config, get_delegate, get_proposal as load_proposal, set_delegation, store_proposal, store_vote, remove_delegation};
use crate::governance::types::{Proposal, ProposalFinalizedEvent, ProposalStatus, Vote, VoteCastEvent, VoteDecision};
use crate::guild::storage as guild_storage;
use crate::governance::types::role_weight;

const EVENT_TOPIC_VOTE_CAST: &str = "vote_cast";
const EVENT_TOPIC_VOTE_DELEGATED: &str = "vote_delegated";
const EVENT_TOPIC_VOTE_UNDELEGATED: &str = "vote_undelegated";
const EVENT_TOPIC_PROPOSAL_FINALIZED: &str = "proposal_finalized";

fn resolve_delegate(env: &Env, guild_id: u64, addr: &Address) -> Address {
    let mut current = addr.clone();
    // prevent infinite loops in case of unexpected cycles
    for _ in 0..16 {
        if let Some(next) = get_delegate(env, guild_id, &current) {
            if next == current {
                break;
            }
            current = next;
        } else {
            break;
        }
    }
    current
}

fn compute_total_weight_and_tallies(env: &Env, proposal: &Proposal) -> (i128, i128, i128, i128) {
    // returns (total_votes_weight, for_weight, against_weight, abstain_weight)
    let votes_map = get_all_votes(env, proposal.id);
    let _cfg = get_config(env, proposal.guild_id);

    let members = guild_storage::get_all_members(env, proposal.guild_id);

    let mut total_votes_weight: i128 = 0;
    let mut for_weight: i128 = 0;
    let mut against_weight: i128 = 0;
    let mut abstain_weight: i128 = 0;

    for member in members.iter() {
        let rep = resolve_delegate(env, proposal.guild_id, &member.address);
        let weight = role_weight(&member.role);

        let decision_opt = if rep == member.address {
            votes_map.get(member.address.clone()).map(|v| v.decision)
        } else {
            votes_map.get(rep).map(|v| v.decision)
        };

        if let Some(decision) = decision_opt {
            total_votes_weight += weight;
            match decision {
                VoteDecision::For => for_weight += weight,
                VoteDecision::Against => against_weight += weight,
                VoteDecision::Abstain => abstain_weight += weight,
            }
        }
    }

    (total_votes_weight, for_weight, against_weight, abstain_weight)
}

pub fn vote(env: &Env, proposal_id: u64, voter: Address, decision: VoteDecision) -> bool {
    voter.require_auth();

    let proposal = load_proposal(env, proposal_id).unwrap_or_else(|| panic!("proposal not found"));

    if !matches!(proposal.status, ProposalStatus::Active) {
        panic!("proposal not active");
    }

    let now = env.ledger().timestamp();
    if now < proposal.voting_start || now > proposal.voting_end {
        panic!("voting period closed");
    }

    // must be guild member
    let member = guild_storage::get_member(env, proposal.guild_id, &voter)
        .unwrap_or_else(|| panic!("voter must be guild member"));

    let weight = role_weight(&member.role);

    let vote = Vote {
        voter: voter.clone(),
        proposal_id,
        decision: decision.clone(),
        weight,
        timestamp: now,
    };

    store_vote(env, &vote);

    let event = VoteCastEvent {
        proposal_id,
        voter,
        decision,
    };
    env.events().publish(
        (Symbol::new(env, EVENT_TOPIC_VOTE_CAST), Symbol::new(env, "v0")),
        event,
    );

    true
}

pub fn delegate_vote(env: &Env, guild_id: u64, delegator: Address, delegate: Address) -> bool {
    delegator.require_auth();

    if delegator == delegate {
        panic!("cannot delegate to self");
    }

    // both must be guild members
    let _d1 = guild_storage::get_member(env, guild_id, &delegator)
        .unwrap_or_else(|| panic!("delegator must be guild member"));
    let _d2 = guild_storage::get_member(env, guild_id, &delegate)
        .unwrap_or_else(|| panic!("delegate must be guild member"));

    // check for cycles: walk starting from delegate
    let mut current = delegate.clone();
    for _ in 0..16 {
        if current == delegator {
            panic!("delegation cycle detected");
        }
        if let Some(next) = get_delegate(env, guild_id, &current) {
            if next == current {
                break;
            }
            current = next;
        } else {
            break;
        }
    }

    set_delegation(env, guild_id, &delegator, &delegate);

    let event = crate::governance::types::VoteDelegatedEvent {
        guild_id,
        delegator,
        delegate,
    };
    env.events().publish(
        (Symbol::new(env, EVENT_TOPIC_VOTE_DELEGATED), Symbol::new(env, "v0")),
        event,
    );

    true
}

pub fn undelegate_vote(env: &Env, guild_id: u64, delegator: Address) -> bool {
    delegator.require_auth();

    remove_delegation(env, guild_id, &delegator);

    let event = crate::governance::types::VoteUndelegatedEvent { guild_id, delegator };
    env.events().publish(
        (Symbol::new(env, EVENT_TOPIC_VOTE_UNDELEGATED), Symbol::new(env, "v0")),
        event,
    );

    true
}

pub fn finalize_proposal(env: &Env, proposal_id: u64) -> ProposalStatus {
    let mut proposal = load_proposal(env, proposal_id).unwrap_or_else(|| panic!("proposal not found"));

    if !matches!(proposal.status, ProposalStatus::Active) {
        return proposal.status;
    }

    let now = env.ledger().timestamp();
    if now < proposal.voting_end {
        panic!("voting period not finished");
    }

    let cfg = get_config(env, proposal.guild_id);

    let members = guild_storage::get_all_members(env, proposal.guild_id);
    let mut total_possible_weight: i128 = 0;
    for member in members.iter() {
        total_possible_weight += role_weight(&member.role);
    }

    let quorum_threshold: i128 = (total_possible_weight * (cfg.quorum_percentage as i128)) / 100;

    let (total_votes_weight, for_weight, against_weight, abstain_weight) =
        compute_total_weight_and_tallies(env, &proposal);

    proposal.votes_for = for_weight;
    proposal.votes_against = against_weight;
    proposal.votes_abstain = abstain_weight;

    if total_votes_weight < quorum_threshold {
        proposal.status = ProposalStatus::Rejected;
    } else {
        let counted = for_weight + against_weight;
        if counted == 0 {
            proposal.status = ProposalStatus::Rejected;
        } else {
            let approval_pct = (for_weight * 100) / counted;
            if approval_pct >= (cfg.approval_threshold as i128) {
                proposal.status = ProposalStatus::Passed;
                if proposal.passed_at.is_none() {
                    proposal.passed_at = Some(now);
                }
            } else {
                proposal.status = ProposalStatus::Rejected;
            }
        }
    }

    store_proposal(env, &proposal);

    let event = ProposalFinalizedEvent {
        proposal_id,
        status: proposal.status.clone(),
        votes_for: proposal.votes_for,
        votes_against: proposal.votes_against,
        votes_abstain: proposal.votes_abstain,
    };
    env.events().publish(
        (Symbol::new(env, EVENT_TOPIC_PROPOSAL_FINALIZED), Symbol::new(env, "v0")),
        event,
    );

    proposal.status
}
