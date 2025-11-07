/// Plugin requirements and permissions declaration system
///
/// Plugins declare what they need (network access, filesystem paths, etc.)
/// and the system enforces these requirements via sandboxing.

use crate::{PluginError, ResourceLimits, NetworkTarget};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin requirements - what the plugin needs to function
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginRequirements {
    /// Network access requirements
    pub network: NetworkRequirements,

    /// Filesystem access requirements
    pub filesystem: FilesystemRequirements,

    /// Context data access permissions
    pub context_permissions: ContextPermissions,

    /// Resource limits (optional - uses defaults if not specified)
    pub resources: Option<ResourceLimits>,

    /// Required capabilities (Linux)
    pub capabilities: Vec<String>,

    /// Required system calls (for seccomp whitelist)
    pub syscalls: Vec<String>,
}

/// Network access requirements
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkRequirements {
    /// Allowed network targets (domains, IPs, IP:ports)
    pub allowed_targets: Vec<NetworkTargetRequirement>,

    /// Needs DNS resolution
    pub needs_dns: bool,

    /// Needs loopback access (127.0.0.1)
    pub needs_loopback: bool,

    /// Description of why network access is needed
    pub reason: Option<String>,
}

/// Network target requirement (serializable version of NetworkTarget)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum NetworkTargetRequirement {
    /// Domain pattern
    Domain(String),
    /// IP address
    Ip(String),
    /// IP:port pair
    IpPort { ip: String, port: u16 },
    /// IP:port range
    IpPortRange { ip: String, start: u16, end: u16 },
}

impl NetworkTargetRequirement {
    /// Convert to NetworkTarget for sandbox configuration
    pub fn to_network_target(&self) -> Result<NetworkTarget, PluginError> {
        match self {
            NetworkTargetRequirement::Domain(d) => Ok(NetworkTarget::domain(d.clone())),
            NetworkTargetRequirement::Ip(ip) => {
                let addr = ip.parse()
                    .map_err(|e| PluginError::LoadError(format!("Invalid IP address '{}': {}", ip, e)))?;
                Ok(NetworkTarget::ip(addr))
            }
            NetworkTargetRequirement::IpPort { ip, port } => {
                let addr = ip.parse()
                    .map_err(|e| PluginError::LoadError(format!("Invalid IP address '{}': {}", ip, e)))?;
                Ok(NetworkTarget::ip_port(addr, *port))
            }
            NetworkTargetRequirement::IpPortRange { ip, start, end } => {
                let addr = ip.parse()
                    .map_err(|e| PluginError::LoadError(format!("Invalid IP address '{}': {}", ip, e)))?;
                Ok(NetworkTarget::ip_port_range(addr, *start, *end))
            }
        }
    }
}

/// Filesystem access requirements
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesystemRequirements {
    /// Paths the plugin needs to read
    pub read_paths: Vec<PathBuf>,

    /// Paths the plugin needs to write
    pub write_paths: Vec<PathBuf>,

    /// Paths the plugin needs to execute
    pub execute_paths: Vec<PathBuf>,

    /// Whether plugin needs /tmp access
    pub needs_tmp: bool,

    /// Temporary storage size limit in bytes
    pub temp_storage_bytes: Option<u64>,

    /// Description of why filesystem access is needed
    pub reason: Option<String>,
}

/// Context data access permissions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextPermissions {
    /// Allowed context keys the plugin can access
    pub allowed_contexts: Vec<ContextPermission>,

    /// Description of why context access is needed
    pub reason: Option<String>,
}

/// Permission to access a specific context data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContextPermission {
    /// The context key to access
    pub key: String,

    /// Access level (read-only vs read-write)
    pub access: ContextAccessLevel,

    /// Description of how this context will be used
    pub reason: Option<String>,
}

/// Access level for context data
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextAccessLevel {
    /// Read-only access
    Read,
    /// Read and write access
    ReadWrite,
}

impl ContextPermission {
    /// Create a read-only permission
    pub fn read(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            access: ContextAccessLevel::Read,
            reason: None,
        }
    }

    /// Create a read-write permission
    pub fn read_write(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            access: ContextAccessLevel::ReadWrite,
            reason: None,
        }
    }

    /// Add a reason description
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

impl ContextPermissions {
    /// Create permissions with specific contexts
    pub fn allow(contexts: Vec<ContextPermission>) -> Self {
        Self {
            allowed_contexts: contexts,
            reason: None,
        }
    }

    /// Add a reason description
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Check if a context key is allowed
    pub fn is_allowed(&self, key: &str, required_access: ContextAccessLevel) -> bool {
        self.allowed_contexts.iter().any(|perm| {
            perm.key == key && match (perm.access, required_access) {
                (ContextAccessLevel::ReadWrite, _) => true,
                (ContextAccessLevel::Read, ContextAccessLevel::Read) => true,
                _ => false,
            }
        })
    }
}

impl PluginRequirements {
    /// Create minimal requirements (isolated plugin)
    pub fn minimal() -> Self {
        Self {
            network: NetworkRequirements {
                needs_loopback: true,
                ..Default::default()
            },
            filesystem: FilesystemRequirements {
                needs_tmp: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create requirements for a database plugin
    pub fn database(db_host: &str, db_port: u16) -> Self {
        Self {
            network: NetworkRequirements {
                allowed_targets: vec![
                    NetworkTargetRequirement::IpPort {
                        ip: db_host.to_string(),
                        port: db_port,
                    },
                ],
                needs_dns: true,
                needs_loopback: true,
                reason: Some("Database connection".to_string()),
            },
            filesystem: FilesystemRequirements {
                needs_tmp: true,
                ..Default::default()
            },
            resources: Some(ResourceLimits {
                max_connections: 20,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Create requirements for an API client plugin
    pub fn api_client(domains: Vec<&str>) -> Self {
        Self {
            network: NetworkRequirements {
                allowed_targets: domains
                    .into_iter()
                    .map(|d| NetworkTargetRequirement::Domain(d.to_string()))
                    .collect(),
                needs_dns: true,
                needs_loopback: true,
                reason: Some("External API access".to_string()),
            },
            filesystem: FilesystemRequirements {
                needs_tmp: true,
                ..Default::default()
            },
            resources: Some(ResourceLimits {
                max_connections: 50,
                max_external_api_ms: 30000,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Merge with default requirements
    pub fn with_defaults(mut self) -> Self {
        // Ensure loopback if not specified
        if !self.network.needs_loopback {
            self.network.needs_loopback = true;
        }

        // Ensure tmp access if not specified
        if !self.filesystem.needs_tmp {
            self.filesystem.needs_tmp = true;
        }

        self
    }

    /// Validate requirements (ensure they're reasonable)
    pub fn validate(&self) -> Result<(), PluginError> {
        // Check for conflicting requirements
        if self.network.allowed_targets.is_empty() && !self.network.needs_dns && !self.network.needs_loopback {
            // Completely isolated - this is fine
        }

        // Validate IP addresses in network targets
        for target in &self.network.allowed_targets {
            match target {
                NetworkTargetRequirement::Ip(ip) |
                NetworkTargetRequirement::IpPort { ip, .. } |
                NetworkTargetRequirement::IpPortRange { ip, .. } => {
                    // Try to parse IP
                    ip.parse::<std::net::IpAddr>()
                        .map_err(|e| PluginError::LoadError(format!("Invalid IP in requirements: {}", e)))?;
                }
                _ => {}
            }
        }

        // Validate filesystem paths
        for path in &self.filesystem.read_paths {
            if !path.is_absolute() {
                return Err(PluginError::LoadError(format!(
                    "Filesystem paths must be absolute: {}",
                    path.display()
                )));
            }
        }

        Ok(())
    }
}

/// Helper trait for plugins to declare requirements via macro
pub trait HasRequirements {
    fn requirements() -> PluginRequirements;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_requirements() {
        let req = PluginRequirements::minimal();
        assert!(req.network.needs_loopback);
        assert!(req.filesystem.needs_tmp);
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_database_requirements() {
        let req = PluginRequirements::database("192.168.1.100", 5432);
        assert_eq!(req.network.allowed_targets.len(), 1);
        assert!(req.network.needs_dns);
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_api_client_requirements() {
        let req = PluginRequirements::api_client(vec!["api.stripe.com", "api.github.com"]);
        assert_eq!(req.network.allowed_targets.len(), 2);
        assert!(req.network.needs_dns);
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_network_target_conversion() {
        let target = NetworkTargetRequirement::Domain("example.com".to_string());
        assert!(target.to_network_target().is_ok());

        let target = NetworkTargetRequirement::IpPort {
            ip: "192.168.1.1".to_string(),
            port: 443,
        };
        assert!(target.to_network_target().is_ok());
    }
}

