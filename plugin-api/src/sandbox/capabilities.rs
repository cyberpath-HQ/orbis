/// Linux capability dropping for privilege restriction
use nix::sys::prctl;
use tracing::{info, debug, warn};
use crate::PluginError;

// Note: Full capabilities support requires caps crate or manual syscalls
// This is a simplified interface

#[cfg(target_os = "linux")]
use caps::Capability;

/// Drop all capabilities except essential ones
#[cfg(target_os = "linux")]
pub fn drop_capabilities() -> Result<(), PluginError> {
    use caps::CapSet;

    info!("Dropping dangerous capabilities");

    // Set NO_NEW_PRIVS to prevent gaining privileges
    prctl::set_no_new_privs()
        .map_err(|e| PluginError::LoadError(format!("Failed to set NO_NEW_PRIVS: {}", e)))?;

    debug!("Set NO_NEW_PRIVS flag");

    // Get list of capabilities to drop
    let to_drop = get_capabilities_to_drop_enum();

    // Drop from all capability sets (effective, permitted, inheritable)
    for cap in to_drop {
        // Drop from effective set
        if let Err(e) = caps::drop(None, CapSet::Effective, cap) {
            warn!("Failed to drop {} from effective: {}", format!("{:?}", cap), e);
        }

        // Drop from permitted set
        if let Err(e) = caps::drop(None, CapSet::Permitted, cap) {
            warn!("Failed to drop {} from permitted: {}", format!("{:?}", cap), e);
        }

        // Drop from inheritable set
        if let Err(e) = caps::drop(None, CapSet::Inheritable, cap) {
            warn!("Failed to drop {} from inheritable: {}", format!("{:?}", cap), e);
        }
    }

    info!("Capabilities dropped successfully");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn drop_capabilities() -> Result<(), PluginError> {
    warn!("Capability dropping not available on this platform");
    Ok(())
}

/// Get capabilities to drop as enum values
#[cfg(target_os = "linux")]
fn get_capabilities_to_drop_enum() -> Vec<Capability> {
    use caps::Capability::*;

    vec![
        CAP_SYS_ADMIN,
        CAP_SYS_MODULE,
        CAP_SYS_RAWIO,
        CAP_SYS_BOOT,
        CAP_SYS_TIME,
        CAP_SYS_PTRACE,
        CAP_SYS_CHROOT,
        CAP_MAC_ADMIN,
        CAP_MAC_OVERRIDE,
        CAP_NET_ADMIN,
        CAP_SETUID,
        CAP_SETGID,
        CAP_SETPCAP,
        CAP_SYS_NICE,
        CAP_SYS_RESOURCE,
    ]
}

/// List of capabilities that should be dropped for plugins
pub fn get_capabilities_to_drop() -> Vec<&'static str> {
    vec![
        "CAP_SYS_ADMIN",        // System administration
        "CAP_SYS_MODULE",       // Load/unload kernel modules
        "CAP_SYS_RAWIO",        // Raw I/O operations
        "CAP_SYS_BOOT",         // Reboot system
        "CAP_SYS_TIME",         // Set system time
        "CAP_SYS_PTRACE",       // Trace processes
        "CAP_SYS_CHROOT",       // Use chroot()
        "CAP_MAC_ADMIN",        // Override MAC
        "CAP_MAC_OVERRIDE",     // Override MAC
        "CAP_NET_ADMIN",        // Network administration (if not needed)
        "CAP_SETUID",           // Set UID
        "CAP_SETGID",           // Set GID
        "CAP_SETPCAP",          // Modify capabilities
        "CAP_SYS_NICE",         // Modify process priority
        "CAP_SYS_RESOURCE",     // Override resource limits
    ]
}

/// List of capabilities that plugins might need (minimal set)
pub fn get_allowed_capabilities() -> Vec<&'static str> {
    vec![
        "CAP_NET_BIND_SERVICE",  // Bind to ports < 1024 (if needed)
        "CAP_DAC_OVERRIDE",      // Bypass file permission checks (carefully)
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_lists() {
        let to_drop = get_capabilities_to_drop();
        assert!(to_drop.contains(&"CAP_SYS_ADMIN"));
        assert!(to_drop.contains(&"CAP_SYS_MODULE"));

        let allowed = get_allowed_capabilities();
        assert!(allowed.contains(&"CAP_NET_BIND_SERVICE"));
    }
}

