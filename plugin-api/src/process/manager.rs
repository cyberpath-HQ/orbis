/// Plugin Process Manager - orchestrates multiple plugin processes
use crate::ipc::IpcConfig;
use crate::process::{PluginProcess, ProcessConfig, ProcessStatus};
use crate::{PluginError, ResourceLimits};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Manages multiple plugin processes
pub struct PluginProcessManager {
    /// Active plugin processes
    processes: Arc<RwLock<HashMap<String, PluginProcess>>>,
    
    /// Process configuration
    config: ProcessConfig,
    
    /// IPC configuration
    ipc_config: IpcConfig,
}

impl PluginProcessManager {
    /// Create a new process manager
    pub fn new(config: ProcessConfig, ipc_config: IpcConfig) -> Self {
        info!("Creating plugin process manager");
        
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            config,
            ipc_config,
        }
    }
    
    /// Spawn a new plugin process
    pub async fn spawn_plugin(
        &self,
        plugin_name: String,
        plugin_path: PathBuf,
        resource_limits: ResourceLimits,
    ) -> Result<(), PluginError> {
        info!("Spawning plugin process: {} from {}", plugin_name, plugin_path.display());
        
        // Validate plugin path
        if !plugin_path.exists() {
            return Err(PluginError::LoadError(format!(
                "Plugin file not found: {}",
                plugin_path.display()
            )));
        }
        
        // Check if already running
        {
            let processes = self.processes.read().await;
            if processes.contains_key(&plugin_name) {
                return Err(PluginError::AlreadyLoaded(plugin_name));
            }
        }
        
        // Create plugin process
        let mut process = PluginProcess::new(
            plugin_name.clone(),
            plugin_path,
            resource_limits,
            self.ipc_config.clone(),
        ).await?;
        
        // Start the process
        let startup_timeout = Duration::from_millis(self.config.startup_timeout_ms);
        process.start(&self.config.worker_binary, startup_timeout).await?;
        
        // Store in map
        {
            let mut processes = self.processes.write().await;
            processes.insert(plugin_name.clone(), process);
        }
        
        info!("Plugin '{}' spawned and running", plugin_name);
        Ok(())
    }
    
    /// Stop a plugin process
    pub async fn stop_plugin(&self, plugin_name: &str) -> Result<(), PluginError> {
        info!("Stopping plugin: {}", plugin_name);
        
        let mut process = {
            let mut processes = self.processes.write().await;
            processes.remove(plugin_name)
                .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?
        };
        
        let grace_period = Duration::from_millis(self.config.shutdown_grace_period_ms);
        process.shutdown(grace_period).await?;
        
        info!("Plugin '{}' stopped", plugin_name);
        Ok(())
    }
    
    /// Execute a hook in a plugin
    pub async fn execute_hook(
        &self,
        plugin_name: &str,
        hook_name: &str,
        data: Vec<u8>,
        timeout_ms: u64,
    ) -> Result<Vec<u8>, PluginError> {
        let mut processes = self.processes.write().await;
        
        let process = processes.get_mut(plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;
        
        if process.status != ProcessStatus::Running {
            return Err(PluginError::LoadError(format!(
                "Plugin '{}' not in running state: {}",
                plugin_name, process.status
            )));
        }
        
        process.execute_hook(hook_name, data, timeout_ms).await
    }
    
    /// Perform health check on a plugin
    pub async fn health_check(&self, plugin_name: &str) -> Result<bool, PluginError> {
        let mut processes = self.processes.write().await;
        
        let process = processes.get_mut(plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;
        
        process.health_check().await
    }
    
    /// Get list of running plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        let processes = self.processes.read().await;
        processes.keys().cloned().collect()
    }
    
    /// Get plugin status
    pub async fn get_status(&self, plugin_name: &str) -> Option<ProcessStatus> {
        let processes = self.processes.read().await;
        processes.get(plugin_name).map(|p| p.status)
    }
    
    /// Get plugin PID
    pub async fn get_pid(&self, plugin_name: &str) -> Option<u32> {
        let processes = self.processes.read().await;
        processes.get(plugin_name).and_then(|p| p.pid)
    }
    
    /// Get plugin uptime
    pub async fn get_uptime(&self, plugin_name: &str) -> Option<Duration> {
        let processes = self.processes.read().await;
        processes.get(plugin_name).and_then(|p| p.uptime())
    }
    
    /// Restart a crashed or failed plugin
    pub async fn restart_plugin(&self, plugin_name: &str) -> Result<(), PluginError> {
        info!("Restarting plugin: {}", plugin_name);
        
        // Get plugin info before stopping
        let (plugin_path, resource_limits, restart_attempts) = {
            let processes = self.processes.read().await;
            let process = processes.get(plugin_name)
                .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;
            
            (
                process.plugin_path.clone(),
                process.resource_limits.clone(),
                process.restart_attempts(),
            )
        };
        
        // Check restart attempt limit
        if restart_attempts >= self.config.max_restart_attempts {
            error!(
                "Plugin '{}' exceeded max restart attempts ({})",
                plugin_name, self.config.max_restart_attempts
            );
            return Err(PluginError::LoadError(format!(
                "Max restart attempts exceeded: {}",
                self.config.max_restart_attempts
            )));
        }
        
        // Stop the plugin (ignore errors if already stopped)
        let _ = self.stop_plugin(plugin_name).await;
        
        // Wait a bit before restarting
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Spawn new instance
        self.spawn_plugin(plugin_name.to_string(), plugin_path, resource_limits).await?;
        
        // Increment restart counter
        {
            let mut processes = self.processes.write().await;
            if let Some(process) = processes.get_mut(plugin_name) {
                process.increment_restart_attempts();
            }
        }
        
        info!("Plugin '{}' restarted (attempt {})", plugin_name, restart_attempts + 1);
        Ok(())
    }
    
    /// Start health monitoring for all plugins
    pub fn start_health_monitor(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let check_interval = Duration::from_millis(self.config.health_check_interval_ms);
        
        info!("Starting plugin health monitor (interval: {:?})", check_interval);
        
        tokio::spawn(async move {
            let mut interval = interval(check_interval);
            
            loop {
                interval.tick().await;
                
                // Get list of plugins to check
                let plugin_names = self.list_plugins().await;
                
                for plugin_name in plugin_names {
                    // Check if process is still running
                    let is_running = {
                        let mut processes = self.processes.write().await;
                        if let Some(process) = processes.get_mut(&plugin_name) {
                            process.is_running()
                        } else {
                            continue;
                        }
                    };
                    
                    if !is_running {
                        warn!("Plugin '{}' process died, attempting restart", plugin_name);
                        if let Err(e) = self.restart_plugin(&plugin_name).await {
                            error!("Failed to restart plugin '{}': {}", plugin_name, e);
                        }
                        continue;
                    }
                    
                    // Perform health check
                    match self.health_check(&plugin_name).await {
                        Ok(true) => {
                            debug!("Health check OK for plugin: {}", plugin_name);
                        }
                        Ok(false) => {
                            warn!("Health check failed for plugin: {}", plugin_name);
                        }
                        Err(e) => {
                            error!("Health check error for plugin '{}': {}", plugin_name, e);
                        }
                    }
                }
            }
        })
    }
    
    /// Stop all plugins
    pub async fn stop_all(&self) -> Result<(), PluginError> {
        info!("Stopping all plugins");
        
        let plugin_names = self.list_plugins().await;
        
        for plugin_name in plugin_names {
            if let Err(e) = self.stop_plugin(&plugin_name).await {
                error!("Failed to stop plugin '{}': {}", plugin_name, e);
            }
        }
        
        Ok(())
    }
    
    /// Get number of running plugins
    pub async fn count(&self) -> usize {
        let processes = self.processes.read().await;
        processes.len()
    }
    
    /// Get resource usage for a plugin (from cgroups if available, otherwise /proc)
    pub async fn get_resource_usage(&self, plugin_name: &str) -> Result<PluginResourceUsage, PluginError> {
        let processes = self.processes.read().await;
        let process = processes.get(plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;
        
        let pid = process.pid.ok_or_else(|| PluginError::LoadError("No PID available".to_string()))?;
        
        // Try cgroups first (more accurate for sandboxed plugins)
        #[cfg(target_os = "linux")]
        {
            if let Some(memory) = process.get_cgroup_memory_usage() {
                let cpu = process.get_cgroup_cpu_usage().unwrap_or(0);
                return Ok(PluginResourceUsage {
                    memory_bytes: memory,
                    cpu_time_ms: cpu,
                    source: ResourceUsageSource::Cgroups,
                });
            }
        }
        
        // Fallback to /proc
        #[cfg(target_os = "linux")]
        {
            use crate::PluginResourceMonitor;
            let monitor = PluginResourceMonitor::new(
                plugin_name.to_string(),
                pid,
                process.resource_limits.clone(),
            );
            
            let memory = monitor.check_resources()
                .ok()
                .and_then(|v| v.iter().find_map(|viol| {
                    if let crate::limits::ViolationType::HeapMemory { used, .. } = viol {
                        Some(*used)
                    } else {
                        None
                    }
                }))
                .unwrap_or(0);
            
            return Ok(PluginResourceUsage {
                memory_bytes: memory,
                cpu_time_ms: 0,
                source: ResourceUsageSource::Proc,
            });
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Err(PluginError::LoadError("Resource monitoring not available on this platform".to_string()))
        }
    }
}

/// Resource usage data
#[derive(Debug, Clone)]
pub struct PluginResourceUsage {
    pub memory_bytes: usize,
    pub cpu_time_ms: u64,
    pub source: ResourceUsageSource,
}

/// Source of resource usage data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceUsageSource {
    /// From cgroups (most accurate, per-plugin)
    Cgroups,
    /// From /proc filesystem (fallback, may include shared resources)
    Proc,
}

impl Drop for PluginProcessManager {
    fn drop(&mut self) {
        warn!("PluginProcessManager dropping - processes will be killed");
        // Processes will be killed when PluginProcess instances are dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_manager_creation() {
        let config = ProcessConfig::default();
        let ipc_config = IpcConfig::default();
        let manager = PluginProcessManager::new(config, ipc_config);
        
        assert_eq!(manager.count().await, 0);
    }
    
    #[tokio::test]
    async fn test_list_empty() {
        let config = ProcessConfig::default();
        let ipc_config = IpcConfig::default();
        let manager = PluginProcessManager::new(config, ipc_config);
        
        let plugins = manager.list_plugins().await;
        assert!(plugins.is_empty());
    }
}

