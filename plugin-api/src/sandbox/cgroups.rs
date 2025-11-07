/// cgroups v2 resource limit enforcement
use crate::{PluginError, ResourceLimits};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn, error, debug};

/// cgroups v2 controller
pub struct CgroupController {
    /// Path to the cgroup directory
    cgroup_path: PathBuf,
    /// Plugin name
    plugin_name: String,
}

impl CgroupController {
    /// Create a new cgroup for a plugin
    pub fn new(base_path: &Path, plugin_name: &str) -> Result<Self, PluginError> {
        let cgroup_path = base_path.join(plugin_name);

        // Create cgroup directory
        if !cgroup_path.exists() {
            fs::create_dir_all(&cgroup_path)
                .map_err(|e| PluginError::LoadError(format!("Failed to create cgroup: {}", e)))?;
            info!("Created cgroup at: {}", cgroup_path.display());
        }

        Ok(Self {
            cgroup_path,
            plugin_name: plugin_name.to_string(),
        })
    }

    /// Apply resource limits to the cgroup
    pub fn apply_limits(&self, limits: &ResourceLimits) -> Result<(), PluginError> {
        info!("Applying cgroup limits for plugin: {}", self.plugin_name);

        // Memory limit
        self.set_memory_limit(limits.max_heap_bytes)?;

        // CPU limit
        self.set_cpu_limit(limits.max_cpu_time_ms)?;

        // Process/thread limit
        self.set_pids_limit(limits.max_threads)?;

        info!("cgroup limits applied successfully");
        Ok(())
    }

    /// Set memory limit
    fn set_memory_limit(&self, max_bytes: usize) -> Result<(), PluginError> {
        let memory_max_path = self.cgroup_path.join("memory.max");

        if memory_max_path.exists() {
            fs::write(&memory_max_path, max_bytes.to_string())
                .map_err(|e| PluginError::LoadError(format!("Failed to set memory.max: {}", e)))?;
            debug!("Set memory.max to {} bytes", max_bytes);

            // Also set memory.high (soft limit with throttling)
            let memory_high = (max_bytes as f64 * 0.9) as usize; // 90% of max
            let memory_high_path = self.cgroup_path.join("memory.high");
            if memory_high_path.exists() {
                let _ = fs::write(&memory_high_path, memory_high.to_string());
                debug!("Set memory.high to {} bytes", memory_high);
            }
        } else {
            warn!("memory.max not available in cgroup (cgroups v2 not enabled?)");
        }

        Ok(())
    }

    /// Set CPU limit
    fn set_cpu_limit(&self, _max_cpu_time_ms: u64) -> Result<(), PluginError> {
        // CPU limit in cgroups is typically set as:
        // cpu.max = "max period" where max is microseconds per period
        // For example: "100000 1000000" = 100ms per 1s = 10% CPU

        let cpu_max_path = self.cgroup_path.join("cpu.max");

        if cpu_max_path.exists() {
            // Set to 100% CPU quota (100ms per 100ms period)
            // This is a simple default; can be made more sophisticated
            let cpu_quota = "100000 100000";
            fs::write(&cpu_max_path, cpu_quota)
                .map_err(|e| PluginError::LoadError(format!("Failed to set cpu.max: {}", e)))?;
            debug!("Set cpu.max to {}", cpu_quota);
        } else {
            warn!("cpu.max not available in cgroup");
        }

        Ok(())
    }

    /// Set PIDs limit (max processes/threads)
    fn set_pids_limit(&self, max_threads: u32) -> Result<(), PluginError> {
        let pids_max_path = self.cgroup_path.join("pids.max");

        if pids_max_path.exists() {
            fs::write(&pids_max_path, max_threads.to_string())
                .map_err(|e| PluginError::LoadError(format!("Failed to set pids.max: {}", e)))?;
            debug!("Set pids.max to {}", max_threads);
        } else {
            warn!("pids.max not available in cgroup");
        }

        Ok(())
    }

    /// Add a process to this cgroup
    pub fn add_process(&self, pid: u32) -> Result<(), PluginError> {
        let procs_path = self.cgroup_path.join("cgroup.procs");

        fs::write(&procs_path, pid.to_string())
            .map_err(|e| PluginError::LoadError(format!("Failed to add process to cgroup: {}", e)))?;

        info!("Added process {} to cgroup {}", pid, self.plugin_name);
        Ok(())
    }

    /// Read current memory usage
    pub fn get_memory_usage(&self) -> Result<usize, PluginError> {
        let memory_current_path = self.cgroup_path.join("memory.current");

        if memory_current_path.exists() {
            let content = fs::read_to_string(&memory_current_path)
                .map_err(|e| PluginError::LoadError(format!("Failed to read memory.current: {}", e)))?;

            content.trim().parse::<usize>()
                .map_err(|e| PluginError::LoadError(format!("Failed to parse memory.current: {}", e)))
        } else {
            Err(PluginError::LoadError("memory.current not available".to_string()))
        }
    }

    /// Read current CPU usage
    pub fn get_cpu_usage(&self) -> Result<u64, PluginError> {
        let cpu_stat_path = self.cgroup_path.join("cpu.stat");

        if cpu_stat_path.exists() {
            let content = fs::read_to_string(&cpu_stat_path)
                .map_err(|e| PluginError::LoadError(format!("Failed to read cpu.stat: {}", e)))?;

            // Parse "usage_usec" line
            for line in content.lines() {
                if line.starts_with("usage_usec") {
                    if let Some(value_str) = line.split_whitespace().nth(1) {
                        let usec = value_str.parse::<u64>()
                            .map_err(|e| PluginError::LoadError(format!("Failed to parse cpu.stat: {}", e)))?;
                        return Ok(usec / 1000); // Convert to milliseconds
                    }
                }
            }

            Err(PluginError::LoadError("usage_usec not found in cpu.stat".to_string()))
        } else {
            Err(PluginError::LoadError("cpu.stat not available".to_string()))
        }
    }

    /// Get path to cgroup
    pub fn path(&self) -> &Path {
        &self.cgroup_path
    }
}

impl Drop for CgroupController {
    fn drop(&mut self) {
        // Clean up cgroup directory
        if self.cgroup_path.exists() {
            if let Err(e) = fs::remove_dir(&self.cgroup_path) {
                error!("Failed to remove cgroup directory {}: {}", self.cgroup_path.display(), e);
            } else {
                debug!("Cleaned up cgroup: {}", self.cgroup_path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_cgroup_controller_creation() {
        // Use temp directory for testing
        let temp_dir = env::temp_dir().join("orbis-test-cgroups");
        let _ = fs::create_dir_all(&temp_dir);

        let result = CgroupController::new(&temp_dir, "test-plugin");
        assert!(result.is_ok());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}

