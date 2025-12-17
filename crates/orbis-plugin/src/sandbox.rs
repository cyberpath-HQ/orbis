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

    /// Check if a string permission is allowed.
    #[must_use]
    pub fn has_permission(&self, permission: &str) -> bool {
        match permission.to_lowercase().as_str() {
            "database_read" | "db_read" => self.allow_db_read,
            "database_write" | "db_write" => self.allow_db_write,
            "file_read" => self.allow_file_read,
            "file_write" => self.allow_file_write,
            "network" => self.allow_network,
            "system" => self.allow_system,
            "shell" => self.allow_shell,
            "environment" | "env" => self.allow_environment,
            _ => false,
        }
    }

    /// Check if a network host is accessible.
    #[must_use]
    pub fn can_access_network(&self, host: &str) -> bool {
        if !self.allow_network {
            return false;
        }
        // Empty allowed_hosts means all hosts are allowed when network is enabled
        if self.allowed_hosts.is_empty() {
            return true;
        }
        self.allowed_hosts.iter().any(|h| host.contains(h) || h == "*")
    }

    /// Check if a file path is accessible.
    #[must_use]
    pub fn can_access_path(&self, path: &str) -> bool {
        if !self.allow_file_read && !self.allow_file_write {
            return false;
        }
        // Empty allowed_paths means no paths are allowed even with permission
        if self.allowed_paths.is_empty() {
            return false;
        }
        self.allowed_paths.iter().any(|p| path.starts_with(p) || p == "*")
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::minimal()
    }
}
