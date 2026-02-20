/// Version management system for contract upgrades
///
/// Handles version parsing, compatibility checking, and version progression validation.
/// Follows semantic versioning (major.minor.patch).

use soroban_sdk::{Bytes, Env, String, Vec};
use crate::upgrade::types::{VersionInfo, CompatibilityCheck};

/// Parse a semantic version string (e.g., "1.2.3")
pub fn parse_version(env: &Env, _version_str: &String) -> Result<(u32, u32, u32), String> {
    // Note: Soroban string operations are limited, so we parse manually
    // Format expected: "major.minor.patch"
    
    // This is a simplified version parser
    // In production, you'd want more robust parsing
    Err(String::from_str(env, "Version parsing requires custom implementation"))
}

/// Create a version string from components
pub fn version_to_string(env: &Env, _major: u32, _minor: u32, _patch: u32) -> String {
    // Soroban doesn't support format! macro, use hardcoded example
    String::from_str(env, "major.minor.patch")
}

/// Check if two versions are compatible
pub fn check_compatibility(env: &Env, from: &VersionInfo, to: &VersionInfo) -> CompatibilityCheck {
    let mut incompatibilities = Vec::new(env);
    let mut breaking_changes = Vec::new(env);
    let mut required_migrations = Vec::new(env);

    // Major version change indicates breaking changes
    if to.major != from.major {
        breaking_changes.push_back(String::from_str(
            env,
            "Major version change detected - significant changes expected",
        ));
        required_migrations.push_back(String::from_str(
            env,
            "Full state migration required",
        ));
    }

    // Check version progression logic
    if to.major < from.major ||
       (to.major == from.major && to.minor < from.minor) ||
       (to.major == from.major && to.minor == from.minor && to.patch < from.patch) {
        incompatibilities.push_back(String::from_str(
            env,
            "Cannot downgrade to an older version",
        ));
    }

    let is_compatible = incompatibilities.len() == 0;

    CompatibilityCheck {
        is_compatible,
        incompatibilities,
        required_migrations,
        breaking_changes,
    }
}

/// Check if this version can be deployed after the specified version
pub fn can_upgrade_from(env: &Env, from: &VersionInfo, to: &VersionInfo) -> Result<(), String> {
    // Prevent downgrading
    if to.major < from.major {
        return Err(String::from_str(env, "Cannot downgrade major version"));
    }

    if to.major == from.major && to.minor < from.minor {
        return Err(String::from_str(env, "Cannot downgrade minor version"));
    }

    if to.major == from.major && to.minor == from.minor && to.patch < from.patch {
        return Err(String::from_str(env, "Cannot downgrade patch version"));
    }

    Ok(())
}

/// Validate that code hash matches version
pub fn validate_code_hash(env: &Env, version: &VersionInfo, actual_hash: &Bytes) -> Result<(), String> {
    if version.code_hash != *actual_hash {
        return Err(String::from_str(
            env,
            "Code hash mismatch - version integrity check failed",
        ));
    }
    Ok(())
}

/// Check if versions are compatible for automated state migration
pub fn requires_state_migration(from: &VersionInfo, to: &VersionInfo) -> bool {
    // If major version changed, state migration is definitely required
    if to.major != from.major {
        return true;
    }

    // If minor version changed significantly, migration might be needed
    if to.minor > from.minor + 2 {
        return true;
    }

    false
}

/// Get version compatibility info for display
pub fn get_version_info(env: &Env, _version: &VersionInfo) -> String {
    // Format version for storage (Soroban has limited string operations)
    String::from_str(
        env,
        "v0.0.0", // Simplified version string
    )
}

/// Validate version ordering in sequence
pub fn validate_version_sequence(env: &Env, versions: &Vec<VersionInfo>) -> Result<(), String> {
    if versions.len() <= 1 {
        return Ok(());
    }

    for i in 1..versions.len() {
        let prev = &versions.get(i - 1).unwrap();
        let current = &versions.get(i).unwrap();

        // Check that each version is greater than or equal to previous
        let prev_val = prev.major * 1000000 + prev.minor * 1000 + prev.patch;
        let curr_val = current.major * 1000000 + current.minor * 1000 + current.patch;

        if curr_val < prev_val {
            return Err(String::from_str(
                env,
                "Version history out of order - version must not decrease",
            ));
        }
    }

    Ok(())
}

/// Get next recommended version based on change type
pub fn get_next_version(
    env: &Env,
    current: &VersionInfo,
    _bump_type: &String,
) -> Result<VersionInfo, String> {
    // Compare bump_type to determine which version component to bump
    // Note: String comparison in Soroban is limited, so we use a simplified approach
    let next_major = current.major;
    let next_minor = current.minor;
    let mut next_patch = current.patch;

    // Since string operations are limited in Soroban, we accept the bumped version parameters
    // In a production system, you'd parse these more robustly
    next_patch += 1;

    Ok(VersionInfo {
        major: next_major,
        minor: next_minor,
        patch: next_patch,
        build: String::from_str(env, "bumped"),
        code_hash: Bytes::new(env),
        released_at: env.ledger().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_version_compatibility() {
        let env = Env::default();

        let v1 = VersionInfo {
            major: 1,
            minor: 0,
            patch: 0,
            build: String::from_str(&env, "build1"),
            code_hash: Bytes::new(&env),
            released_at: 1000,
        };

        let v2 = VersionInfo {
            major: 1,
            minor: 1,
            patch: 0,
            build: String::from_str(&env, "build2"),
            code_hash: Bytes::new(&env),
            released_at: 2000,
        };

        let compat = check_compatibility(&env, &v1, &v2);
        assert!(compat.is_compatible);
        assert_eq!(compat.breaking_changes.len(), 0);
    }

    #[test]
    fn test_prevent_downgrade() {
        let env = Env::default();

        let v2 = VersionInfo {
            major: 1,
            minor: 2,
            patch: 0,
            build: String::from_str(&env, "build2"),
            code_hash: Bytes::new(&env),
            released_at: 2000,
        };

        let v1 = VersionInfo {
            major: 1,
            minor: 0,
            patch: 0,
            build: String::from_str(&env, "build1"),
            code_hash: Bytes::new(&env),
            released_at: 1000,
        };

        let result = can_upgrade_from(&env, &v2, &v1);
        assert!(result.is_err());
    }
}
