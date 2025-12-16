//! Sandbox configuration for plugin security.

use super::PluginPermission;
use serde::{Deserialize, Serialize};

/// Sandbox configuration for controlling plugin capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Allow database read operations.
    pub allow_db_read: bool,

    /// Allow database write operations.
    pub allow_db_write: bool,

    /// Allow file read operations.
    pub allow_file_read: bool,

    /// Allow file write operations.
    pub allow_file_write: bool,

    /// Allow network operations.
    pub allow_network: bool,

    /// Allow system information access.
    pub allow_system: bool,

    /// Allow shell execution (dangerous).
    pub allow_shell: bool,

    /// Allow environment variable access.
    pub allow_environment: bool,

    /// Memory limit in bytes.
    pub memory_limit: usize,

    /// Execution time limit in milliseconds.
    pub time_limit_ms: u64,

    /// Maximum number of function calls.
    pub max_calls: u64,

    /// Allowed file paths (if file access is enabled).
    pub allowed_paths: Vec<String>,

    /// Allowed network hosts (if network is enabled).
    pub allowed_hosts: Vec<String>,
}

impl SandboxConfig {
    /// Create a minimal sandbox with no permissions.
    #[must_use]
    pub const fn minimal() -> Self {
        Self {
            allow_db_read: false,
            allow_db_write: false,
            allow_file_read: false,
            allow_file_write: false,
            allow_network: false,
            allow_system: false,
            allow_shell: false,
            allow_environment: false,
            memory_limit: 16 * 1024 * 1024, // 16MB
            time_limit_ms: 5000,            // 5 seconds
            max_calls: 10000,
            allowed_paths: Vec::new(),
            allowed_hosts: Vec::new(),
        }
    }

    /// Create sandbox config from plugin permissions.
    #[must_use]
    pub fn from_permissions(permissions: &[PluginPermission]) -> Self {
        let mut config = Self::minimal();

        for permission in permissions {
            match permission {
                PluginPermission::DatabaseRead => config.allow_db_read = true,
                PluginPermission::DatabaseWrite => config.allow_db_write = true,
                PluginPermission::FileRead => config.allow_file_read = true,
                PluginPermission::FileWrite => config.allow_file_write = true,
                PluginPermission::Network => config.allow_network = true,
                PluginPermission::System => config.allow_system = true,
                PluginPermission::Shell => config.allow_shell = true,
                PluginPermission::Environment => config.allow_environment = true,
                PluginPermission::Custom(_) => {}
            }
        }

        config
    }

    /// Check if a permission is allowed.
    #[must_use]
    pub fn is_allowed(&self, permission: &PluginPermission) -> bool {
        match permission {
            PluginPermission::DatabaseRead => self.allow_db_read,
            PluginPermission::DatabaseWrite => self.allow_db_write,
            PluginPermission::FileRead => self.allow_file_read,
            PluginPermission::FileWrite => self.allow_file_write,
            PluginPermission::Network => self.allow_network,
            PluginPermission::System => self.allow_system,
            PluginPermission::Shell => self.allow_shell,
            PluginPermission::Environment => self.allow_environment,
            PluginPermission::Custom(_) => true, // Custom permissions are app-specific
        }
    }

    /// Set memory limit.
    #[must_use]
    pub const fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// Set time limit.
    #[must_use]
    pub const fn with_time_limit(mut self, limit_ms: u64) -> Self {
        self.time_limit_ms = limit_ms;
        self
    }

    /// Add allowed path.
    #[must_use]
    pub fn with_allowed_path(mut self, path: impl Into<String>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Add allowed host.
    #[must_use]
    pub fn with_allowed_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.push(host.into());
        self
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::minimal()
    }
}
