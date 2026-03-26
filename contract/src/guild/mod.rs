pub mod membership;
pub mod storage;
/// Guild management module
///
/// This module provides all guild creation, membership management, and role-based
/// permission functionality for the Stellar Guilds platform.
///
/// # Overview
/// - `types`: Defines all core data structures (Guild, Member, Role, Events)
/// - `storage`: Manages persistent storage of guilds and members
/// - `membership`: Core functions for guild and member management
pub mod types;

#[cfg(test)]
pub mod tests;
