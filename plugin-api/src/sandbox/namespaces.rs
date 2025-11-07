/// Linux namespace management
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{Pid, getpid};
use tracing::{info, debug, error};
use crate::PluginError;

/// Namespace configuration
#[derive(Debug, Clone)]
pub struct NamespaceConfig {
    pub pid: bool,
    pub mount: bool,
    pub network: bool,
    pub ipc: bool,
    pub uts: bool,
    pub user: bool,
}

impl NamespaceConfig {
    /// Create all namespaces enabled
    pub fn all() -> Self {
        Self {
            pid: true,
            mount: true,
            network: true,
            ipc: true,
            uts: true,
            user: false, // Usually requires special setup
        }
    }

    /// Convert to CloneFlags
    fn to_clone_flags(&self) -> CloneFlags {
        let mut flags = CloneFlags::empty();

        if self.pid {
            flags |= CloneFlags::CLONE_NEWPID;
        }
        if self.mount {
            flags |= CloneFlags::CLONE_NEWNS;
        }
        if self.network {
            flags |= CloneFlags::CLONE_NEWNET;
        }
        if self.ipc {
            flags |= CloneFlags::CLONE_NEWIPC;
        }
        if self.uts {
            flags |= CloneFlags::CLONE_NEWUTS;
        }
        if self.user {
            flags |= CloneFlags::CLONE_NEWUSER;
        }

        flags
    }
}

/// Create namespaces for the current process
pub fn create_namespaces(config: &NamespaceConfig) -> Result<(), PluginError> {
    info!("Creating namespaces: {:?}", config);

    let flags = config.to_clone_flags();

    if flags.is_empty() {
        debug!("No namespaces to create");
        return Ok(());
    }

    unshare(flags)
        .map_err(|e| PluginError::LoadError(format!("Failed to create namespaces: {}", e)))?;

    info!("Namespaces created successfully");
    Ok(())
}

/// Get current PID (useful for debugging)
pub fn current_pid() -> Pid {
    getpid()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_config_creation() {
        let config = NamespaceConfig::all();
        assert!(config.pid);
        assert!(config.mount);
        assert!(config.network);
    }

    #[test]
    fn test_clone_flags_conversion() {
        let config = NamespaceConfig {
            pid: true,
            mount: false,
            network: true,
            ipc: false,
            uts: false,
            user: false,
        };

        let flags = config.to_clone_flags();
        assert!(flags.contains(CloneFlags::CLONE_NEWPID));
        assert!(flags.contains(CloneFlags::CLONE_NEWNET));
        assert!(!flags.contains(CloneFlags::CLONE_NEWNS));
    }
}

