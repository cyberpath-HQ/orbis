/// Main Linux sandbox implementation
use super::{SandboxConfig, namespaces, seccomp, cgroups, capabilities, filesystem};
use crate::{PluginError, ResourceLimits};
use tracing::{debug, info, warn};

/// Linux sandbox for plugin isolation
pub struct LinuxSandbox {
    config: SandboxConfig,
    plugin_name: String,
    resource_limits: ResourceLimits,
    cgroup_controller: Option<cgroups::CgroupController>,
}

impl LinuxSandbox {
    /// Create a new Linux sandbox
    pub fn new(
        plugin_name: &str,
        resource_limits: ResourceLimits,
        config: SandboxConfig,
    ) -> Result<Self, PluginError> {
        info!("Creating Linux sandbox for plugin: {}", plugin_name);

        // Create cgroup controller if enabled
        let cgroup_controller = if config.enable_cgroups {
            match cgroups::CgroupController::new(&config.cgroups_path, plugin_name) {
                Ok(controller) => Some(controller),
                Err(e) => {
                    warn!("Failed to create cgroup controller: {} (continuing without cgroups)", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            config,
            plugin_name: plugin_name.to_string(),
            resource_limits,
            cgroup_controller,
        })
    }

    /// Apply sandbox before spawning worker
    /// This is called BEFORE fork/spawn
    pub fn prepare(&mut self) -> Result<(), PluginError> {
        info!("Preparing sandbox for plugin: {}", self.plugin_name);

        // Apply cgroup limits
        if let Some(controller) = &self.cgroup_controller {
            controller.apply_limits(&self.resource_limits)?;
        }

        info!("Sandbox prepared successfully");
        Ok(())
    }

    /// Apply sandbox AFTER spawning worker
    /// This adds the spawned process to the cgroup
    pub fn apply_to_process(&self, pid: u32) -> Result<(), PluginError> {
        info!("Applying sandbox to process {} (plugin: {})", pid, self.plugin_name);

        // Add process to cgroup
        if let Some(controller) = &self.cgroup_controller {
            controller.add_process(pid)?;
        }

        info!("Sandbox applied to process successfully");
        Ok(())
    }

    /// Apply sandbox inside the worker process
    /// This is called INSIDE the worker after it starts
    pub fn apply_inside_worker() -> Result<(), PluginError> {
        info!("Applying sandbox inside worker process");

        // Note: This would be called from worker
        // Currently worker doesn't support this yet

        warn!("apply_inside_worker is a placeholder - needs worker integration");
        Ok(())
    }

    /// Setup sandbox environment inside worker (called IN the worker process)
    pub fn setup_worker_sandbox(config: &SandboxConfig, plugin_name: &str) -> Result<(), PluginError> {
        info!("Setting up worker sandbox environment for: {}", plugin_name);

        // Create namespaces
        if config.enable_pid_namespace || config.enable_mount_namespace ||
           config.enable_network_namespace || config.enable_ipc_namespace ||
           config.enable_uts_namespace || config.enable_user_namespace {

            let ns_config = namespaces::NamespaceConfig {
                pid: config.enable_pid_namespace,
                mount: config.enable_mount_namespace,
                network: config.enable_network_namespace,
                ipc: config.enable_ipc_namespace,
                uts: config.enable_uts_namespace,
                user: config.enable_user_namespace,
            };

            namespaces::create_namespaces(&ns_config)?;
        }

        // Apply network isolation (after namespace creation)
        if config.enable_network_namespace {
            use super::network;
            network::apply_network_isolation(&config.network_config, plugin_name)?;
        }

        // Apply filesystem isolation
        if config.enable_filesystem_isolation {
            let fs_config = filesystem::FilesystemConfig {
                root_dir: config.chroot_dir.as_ref().map(|p| p.to_string_lossy().to_string()),
                readonly_root: true,
            };
            filesystem::apply_filesystem_isolation(&fs_config)?;
        }

        // Drop capabilities
        if config.enable_capability_dropping {
            capabilities::drop_capabilities()?;
        }

        // Apply seccomp filter (LAST - after everything else is set up)
        if config.enable_seccomp {
            seccomp::apply_seccomp_filter(&config.seccomp_config)?;
        }

        info!("Worker sandbox environment setup complete for: {}", plugin_name);
        Ok(())
    }

    /// Get current memory usage from cgroup
    pub fn get_memory_usage(&self) -> Result<usize, PluginError> {
        if let Some(controller) = &self.cgroup_controller {
            controller.get_memory_usage()
        } else {
            Err(PluginError::LoadError("cgroups not enabled".to_string()))
        }
    }

    /// Get current CPU usage from cgroup
    pub fn get_cpu_usage(&self) -> Result<u64, PluginError> {
        if let Some(controller) = &self.cgroup_controller {
            controller.get_cpu_usage()
        } else {
            Err(PluginError::LoadError("cgroups not enabled".to_string()))
        }
    }

    /// Get plugin name
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    /// Check if cgroups are enabled
    pub fn has_cgroups(&self) -> bool {
        self.cgroup_controller.is_some()
    }

    /// Explicit cleanup method to release all sandbox resources
    pub fn cleanup(&mut self) {
        info!("Cleaning up sandbox for plugin: {}", self.plugin_name);

        // Cgroup controller will be cleaned up by its Drop implementation
        if let Some(controller) = self.cgroup_controller.take() {
            debug!("Dropping cgroup controller for plugin: {}", self.plugin_name);
            drop(controller);
        }

        info!("Sandbox cleanup complete for plugin: {}", self.plugin_name);
    }
}

impl Drop for LinuxSandbox {
    fn drop(&mut self) {
        debug!("LinuxSandbox dropping for plugin: {}", self.plugin_name);
        // Ensure cleanup happens even if not called explicitly
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let config = SandboxConfig::minimal();
        let limits = ResourceLimits::default();

        // This will fail without proper cgroups setup, but we can test creation
        let sandbox = LinuxSandbox::new("test-plugin", limits, config);
        // Just verify it doesn't panic
        let _ = sandbox;
    }

    #[test]
    fn test_sandbox_config_minimal() {
        let config = SandboxConfig::minimal();
        assert!(config.enable_pid_namespace);
        assert!(!config.enable_seccomp);
        assert!(!config.enable_cgroups);
    }

    #[test]
    fn test_sandbox_config_strict() {
        let config = SandboxConfig::strict();
        assert!(config.enable_pid_namespace);
        assert!(config.enable_seccomp);
        assert!(config.enable_cgroups);
    }
}

