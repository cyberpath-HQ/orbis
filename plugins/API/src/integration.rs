/// Integration module - converts plugin requirements to sandbox configuration
///
/// This is the glue that connects:
/// - Plugin requirements (what plugins declare they need)
/// - Sandbox configuration (how to enforce those requirements)
/// - Security validation (whether to allow those requirements)

use crate::{PluginError, PluginRequirements, ResourceLimits};

#[cfg(all(feature = "sandboxing", target_os = "linux"))]
use crate::sandbox::{SandboxConfig, NetworkConfig, seccomp::SeccompConfig};

#[cfg(not(all(feature = "sandboxing", target_os = "linux")))]
pub fn requirements_to_sandbox_config(
    _requirements: &PluginRequirements,
) -> Result<(), PluginError> {
    Err(PluginError::LoadError(
        "Sandboxing features not available on this platform or not enabled".to_string()
    ))
}

#[cfg(all(feature = "sandboxing", target_os = "linux"))]

/// Convert plugin requirements into a sandbox configuration
pub fn requirements_to_sandbox_config(
    requirements: &PluginRequirements,
) -> Result<SandboxConfig, PluginError> {
    // Validate requirements first
    requirements.validate()?;

    // Start with default sandbox config
    let mut config = SandboxConfig::default();

    // Configure network based on requirements
    #[cfg(target_os = "linux")]
    {
        config.network_config = build_network_config(&requirements.network)?;
    }

    // Configure seccomp if custom syscalls specified
    #[cfg(target_os = "linux")]
    {
        if !requirements.syscalls.is_empty() {
            config.seccomp_config = build_seccomp_config(&requirements.syscalls);
        }
    }

    // Configure filesystem if paths specified
    if requirements.filesystem.needs_filesystem_isolation() {
        config.enable_filesystem_isolation = true;
        // TODO: Set chroot_dir based on requirements
    }

    Ok(config)
}

/// Build network configuration from requirements
#[cfg(target_os = "linux")]
fn build_network_config(
    requirements: &crate::requirements::NetworkRequirements,
) -> Result<NetworkConfig, PluginError> {
    let mut config = if requirements.allowed_targets.is_empty()
        && !requirements.needs_dns
        && requirements.needs_loopback {
        // Isolated - only loopback
        NetworkConfig::restrictive()
    } else {
        // Has external requirements
        NetworkConfig::default()
    };

    // Set DNS
    config.allow_dns = requirements.needs_dns;

    // Set loopback
    config.allow_loopback = requirements.needs_loopback;

    // Convert and add targets
    for target_req in &requirements.allowed_targets {
        let target = target_req.to_network_target()?;
        config.allow_target(target);
    }

    Ok(config)
}

/// Build seccomp configuration from syscall requirements
#[cfg(target_os = "linux")]
fn build_seccomp_config(syscalls: &[String]) -> SeccompConfig {
    let mut config = SeccompConfig::default();

    // Add custom syscalls to the allowed list
    config.allowed_syscalls.extend(syscalls.iter().cloned());

    // Remove duplicates
    config.allowed_syscalls.sort();
    config.allowed_syscalls.dedup();

    config
}

/// Get resource limits from requirements or use defaults
pub fn get_resource_limits(requirements: &PluginRequirements) -> ResourceLimits {
    requirements.resources.clone().unwrap_or_default()
}

/// Validate that requirements are allowed by security policy
#[cfg(feature = "security")]
pub fn validate_requirements_against_policy(
    requirements: &PluginRequirements,
    _security_policy: &crate::security::SecurityPolicy,
) -> Result<(), PluginError> {
    // For now, validate that requirements are reasonable
    requirements.validate()?;

    // Additional policy checks could go here:
    // - Maximum number of allowed network targets
    // - Forbidden domains/IPs
    // - Forbidden filesystem paths
    // - Maximum resource limits

    Ok(())
}

/// Validate that requirements are allowed (without security policy - basic validation only)
#[cfg(not(feature = "security"))]
pub fn validate_requirements_against_policy(
    requirements: &PluginRequirements,
) -> Result<(), PluginError> {
    // Just validate basic requirements structure
    requirements.validate()
}

impl crate::requirements::FilesystemRequirements {
    /// Check if filesystem isolation is needed
    pub fn needs_filesystem_isolation(&self) -> bool {
        !self.read_paths.is_empty()
            || !self.write_paths.is_empty()
            || !self.execute_paths.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_requirements_to_config() {
        let req = PluginRequirements::minimal();
        let config = requirements_to_sandbox_config(&req);
        assert!(config.is_ok());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_network_requirements_conversion() {
        let req = PluginRequirements::api_client(vec!["api.github.com"]);
        let config = requirements_to_sandbox_config(&req).unwrap();
        assert!(config.network_config.allow_dns);
        assert_eq!(config.network_config.allowed_targets.len(), 1);
    }

    #[test]
    fn test_resource_limits_extraction() {
        let req = PluginRequirements::database("localhost", 5432);
        let limits = get_resource_limits(&req);
        assert_eq!(limits.max_connections, 20);
    }
}

