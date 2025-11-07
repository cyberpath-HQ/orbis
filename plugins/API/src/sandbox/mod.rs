/// Linux sandboxing for plugin isolation
///
/// This module provides comprehensive Linux sandboxing including:
/// - Namespaces (PID, mount, network, IPC, UTS, user)
/// - Seccomp syscall filtering
/// - cgroups v2 resource limits
/// - Filesystem isolation (chroot/pivot_root)
/// - Capability dropping

#[cfg(target_os = "linux")]
pub mod namespaces;

#[cfg(target_os = "linux")]
pub mod seccomp;

#[cfg(target_os = "linux")]
pub mod cgroups;

#[cfg(target_os = "linux")]
pub mod filesystem;

#[cfg(target_os = "linux")]
pub mod capabilities;

#[cfg(target_os = "linux")]
pub mod network;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::{PluginError, ResourceLimits};
use std::path::PathBuf;

#[cfg(target_os = "linux")]
pub use network::NetworkConfig;

#[cfg(target_os = "linux")]
pub use seccomp::SeccompConfig;

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Enable PID namespace
    pub enable_pid_namespace: bool,
    
    /// Enable mount namespace
    pub enable_mount_namespace: bool,
    
    /// Enable network namespace
    pub enable_network_namespace: bool,
    
    /// Enable IPC namespace
    pub enable_ipc_namespace: bool,
    
    /// Enable UTS namespace
    pub enable_uts_namespace: bool,
    
    /// Enable user namespace
    pub enable_user_namespace: bool,
    
    /// Enable seccomp filtering
    pub enable_seccomp: bool,
    
    /// Enable cgroups resource limits
    pub enable_cgroups: bool,
    
    /// Enable filesystem isolation
    pub enable_filesystem_isolation: bool,
    
    /// Enable capability dropping
    pub enable_capability_dropping: bool,
    
    /// Network configuration
    #[cfg(target_os = "linux")]
    pub network_config: NetworkConfig,
    
    /// Seccomp filter configuration
    #[cfg(target_os = "linux")]
    pub seccomp_config: SeccompConfig,
    
    /// Root directory for chroot
    pub chroot_dir: Option<PathBuf>,
    
    /// cgroups base path
    pub cgroups_path: PathBuf,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_pid_namespace: true,
            enable_mount_namespace: true,
            enable_network_namespace: true,
            enable_ipc_namespace: true,
            enable_uts_namespace: true,
            enable_user_namespace: false, // Requires root or user_namespaces enabled
            enable_seccomp: true,
            enable_cgroups: true,
            enable_filesystem_isolation: false, // Requires setup
            enable_capability_dropping: true,
            #[cfg(target_os = "linux")]
            network_config: NetworkConfig::default(),
            #[cfg(target_os = "linux")]
            seccomp_config: SeccompConfig::default(),
            chroot_dir: None,
            cgroups_path: PathBuf::from("/sys/fs/cgroup/orbis-plugins"),
        }
    }
}

impl SandboxConfig {
    /// Create a minimal sandbox config (namespaces only)
    pub fn minimal() -> Self {
        Self {
            enable_seccomp: false,
            enable_cgroups: false,
            enable_filesystem_isolation: false,
            #[cfg(target_os = "linux")]
            network_config: NetworkConfig::permissive(),
            #[cfg(target_os = "linux")]
            seccomp_config: SeccompConfig::minimal(),
            ..Default::default()
        }
    }
    
    /// Create a strict sandbox config (all features enabled)
    pub fn strict() -> Self {
        Self {
            enable_pid_namespace: true,
            enable_mount_namespace: true,
            enable_network_namespace: true,
            enable_ipc_namespace: true,
            enable_uts_namespace: true,
            enable_user_namespace: false,
            enable_seccomp: true,
            enable_cgroups: true,
            enable_filesystem_isolation: true,
            enable_capability_dropping: true,
            #[cfg(target_os = "linux")]
            network_config: NetworkConfig::restrictive(),
            #[cfg(target_os = "linux")]
            seccomp_config: SeccompConfig::strict(),
            ..Default::default()
        }
    }
}

