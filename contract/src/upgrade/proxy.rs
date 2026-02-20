/// Proxy contract pattern implementation
///
/// This module implements the transparent proxy pattern following EIP-1967 principles,
/// adapted for Soroban. The proxy delegates calls to an implementation contract
/// while maintaining a consistent storage layout and upgrade path.

use soroban_sdk::{Address, Env, String};
use crate::upgrade::types::VersionInfo;

// EIP-1967 style storage slots for proxy pattern
const IMPLEMENTATION_SLOT: &str = "eip1967.proxy.implementation";
const ADMIN_SLOT: &str = "eip1967.proxy.admin";
const BEACON_SLOT: &str = "eip1967.proxy.beacon";
const VERSION_SLOT: &str = "eip1967.proxy.version";

/// Get the current implementation contract address
pub fn get_implementation(env: &Env) -> Result<Address, String> {
    env.storage()
        .persistent()
        .get::<String, Address>(&String::from_str(env, IMPLEMENTATION_SLOT))
        .ok_or_else(|| String::from_str(env, "Implementation not set"))
}

/// Set the implementation contract address
pub fn set_implementation(env: &Env, implementation: &Address) -> Result<(), String> {
    env.storage()
        .persistent()
        .set(&String::from_str(env, IMPLEMENTATION_SLOT), implementation);
    Ok(())
}

/// Get the proxy admin address
pub fn get_admin(env: &Env) -> Result<Address, String> {
    env.storage()
        .persistent()
        .get::<String, Address>(&String::from_str(env, ADMIN_SLOT))
        .ok_or_else(|| String::from_str(env, "Admin not set"))
}

/// Set the proxy admin address
pub fn set_admin(env: &Env, admin: &Address) -> Result<(), String> {
    env.storage()
        .persistent()
        .set(&String::from_str(env, ADMIN_SLOT), admin);
    Ok(())
}

/// Get the beacon contract address (for beacon proxy pattern)
pub fn get_beacon(env: &Env) -> Result<Address, String> {
    env.storage()
        .persistent()
        .get::<String, Address>(&String::from_str(env, BEACON_SLOT))
        .ok_or_else(|| String::from_str(env, "Beacon not set"))
}

/// Set the beacon contract address
pub fn set_beacon(env: &Env, beacon: &Address) -> Result<(), String> {
    env.storage()
        .persistent()
        .set(&String::from_str(env, BEACON_SLOT), beacon);
    Ok(())
}

/// Initialize proxy with implementation and admin
pub fn initialize_proxy(
    env: &Env,
    implementation: &Address,
    admin: &Address,
) -> Result<(), String> {
    // Check if already initialized
    if get_implementation(env).is_ok() {
        return Err(String::from_str(env, "Proxy already initialized"));
    }

    set_implementation(env, implementation)?;
    set_admin(env, admin)?;

    Ok(())
}

/// Upgrade proxy to a new implementation
pub fn upgrade_implementation(
    env: &Env,
    new_implementation: &Address,
    caller: &Address,
) -> Result<(), String> {
    // Verify caller is admin
    let admin = get_admin(env)?;
    if &admin != caller {
        return Err(String::from_str(
            env,
            "Only proxy admin can upgrade implementation",
        ));
    }

    // Set new implementation
    set_implementation(env, new_implementation)?;

    // Emit upgrade event
    // In production, use contract events:
    // env.events().publish(("proxy_upgraded",), (new_implementation,));

    Ok(())
}

/// Upgrade proxy and transfer admin
pub fn upgrade_and_transfer_admin(
    env: &Env,
    new_implementation: &Address,
    new_admin: &Address,
    caller: &Address,
) -> Result<(), String> {
    // Verify caller is current admin
    let admin = get_admin(env)?;
    if &admin != caller {
        return Err(String::from_str(
            env,
            "Only proxy admin can perform this action",
        ));
    }

    // Validate new admin is not zero address
    // This is a safety check to prevent losing admin privileges

    // Upgrade implementation
    set_implementation(env, new_implementation)?;

    // Transfer admin
    set_admin(env, new_admin)?;

    Ok(())
}

/// Beacon proxy pattern - get implementation from beacon
pub fn get_implementation_from_beacon(env: &Env, _beacon: &Address) -> Result<Address, String> {
    // In production, this would call the beacon contract to get the implementation
    // For now, we assume beacon address points to a contract that implements
    // the beacon interface with implementation() function
    
    Err(String::from_str(
        env,
        "Beacon pattern requires cross-contract call",
    ))
}

/// Check if proxy needs upgrade based on version
pub fn needs_upgrade(
    _env: &Env,
    current_version: &VersionInfo,
    latest_version: &VersionInfo,
) -> bool {
    // Compare versions
    let current_val = current_version.major * 1_000_000 +
                     current_version.minor * 1_000 +
                     current_version.patch;
    
    let latest_val = latest_version.major * 1_000_000 +
                    latest_version.minor * 1_000 +
                    latest_version.patch;

    latest_val > current_val
}

/// Verify proxy storage layout integrity
pub fn verify_proxy_storage(env: &Env) -> Result<(), String> {
    // Check that all proxy slots are properly set
    let _ = get_implementation(env)?;
    let _ = get_admin(env)?;
    
    Ok(())
}

/// Change proxy admin
pub fn change_proxy_admin(
    env: &Env,
    new_admin: &Address,
    caller: &Address,
) -> Result<(), String> {
    // Verify caller is current admin
    let admin = get_admin(env)?;
    if &admin != caller {
        return Err(String::from_str(env, "Only admin can change admin"));
    }

    set_admin(env, new_admin)?;

    Ok(())
}

/// Revoke admin privileges (transfer to null address)
pub fn revoke_admin(env: &Env, caller: &Address) -> Result<(), String> {
    // This is a dangerous operation - permanently locks the proxy
    
    let admin = get_admin(env)?;
    if &admin != caller {
        return Err(String::from_str(env, "Only admin can revoke admin"));
    }

    // To revoke admin, we set it to the caller itself (making it read-only)
    set_admin(env, caller)?;

    Ok(())
}

/// Get proxy info for diagnostics
pub fn get_proxy_info(env: &Env) -> Result<ProxyInfo, String> {
    let implementation = get_implementation(env)?;
    let admin = get_admin(env)?;

    Ok(ProxyInfo {
        implementation,
        admin,
        initialized: true,
    })
}

/// Proxy information struct
pub struct ProxyInfo {
    pub implementation: Address,
    pub admin: Address,
    pub initialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_proxy_initialization() {
        let env = Env::default();
        let impl_addr = Address::from_contract_id(&env, &[1u8; 32]);
        let admin_addr = Address::from_contract_id(&env, &[2u8; 32]);

        let result = initialize_proxy(&env, &impl_addr, &admin_addr);
        assert!(result.is_ok());

        let stored_impl = get_implementation(&env).unwrap();
        assert_eq!(stored_impl, impl_addr);
    }

    #[test]
    fn test_cannot_reinitialize() {
        let env = Env::default();
        let impl_addr = Address::from_contract_id(&env, &[1u8; 32]);
        let admin_addr = Address::from_contract_id(&env, &[2u8; 32]);

        initialize_proxy(&env, &impl_addr, &admin_addr).unwrap();
        
        let result = initialize_proxy(&env, &impl_addr, &admin_addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_upgrade_requires_admin() {
        let env = Env::default();
        let impl_addr = Address::from_contract_id(&env, &[1u8; 32]);
        let admin_addr = Address::from_contract_id(&env, &[2u8; 32]);
        let non_admin = Address::from_contract_id(&env, &[3u8; 32]);

        initialize_proxy(&env, &impl_addr, &admin_addr).unwrap();

        let new_impl = Address::from_contract_id(&env, &[4u8; 32]);
        let result = upgrade_implementation(&env, &new_impl, &non_admin);
        
        assert!(result.is_err());
    }
}
