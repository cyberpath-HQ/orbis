/// Individual plugin process management
use crate::ipc::{IpcChannel, IpcMessage, IpcServer};
use crate::{PluginError, ResourceLimits};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

#[cfg(target_os = "linux")]
use crate::sandbox::{SandboxConfig, linux::LinuxSandbox};

/// Status of a plugin process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is starting up
    Starting,
    /// Process is running and healthy
    Running,
    /// Process is shutting down
    ShuttingDown,
    /// Process has stopped
    Stopped,
    /// Process has crashed
    Crashed,
    /// Process failed to start
    Failed,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Starting => write!(f, "Starting"),
            ProcessStatus::Running => write!(f, "Running"),
            ProcessStatus::ShuttingDown => write!(f, "ShuttingDown"),
            ProcessStatus::Stopped => write!(f, "Stopped"),
            ProcessStatus::Crashed => write!(f, "Crashed"),
            ProcessStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// Represents a plugin running in a separate process
pub struct PluginProcess {
    /// Plugin name
    pub plugin_name: String,

    /// Path to the plugin library
    pub plugin_path: PathBuf,

    /// Child process handle
    child: Option<Child>,

    /// Process ID
    pub pid: Option<u32>,

    /// IPC server for this plugin
    ipc_server: IpcServer,

    /// IPC channel for communication
    ipc_channel: Option<IpcChannel>,

    /// Current status
    pub status: ProcessStatus,

    /// Resource limits
    pub resource_limits: ResourceLimits,

    /// Start time
    pub start_time: Option<Instant>,

    /// Restart attempt count
    restart_attempts: u32,

    /// Last health check time
    last_health_check: Option<Instant>,

    /// Linux sandbox (if enabled)
    #[cfg(target_os = "linux")]
    sandbox: Option<LinuxSandbox>,
}

impl PluginProcess {
    /// Create a new plugin process (not yet started)
    pub async fn new(
        plugin_name: String,
        plugin_path: PathBuf,
        resource_limits: ResourceLimits,
        ipc_config: crate::ipc::IpcConfig,
    ) -> Result<Self, PluginError> {
        // Create IPC server for this plugin
        let ipc_server = IpcServer::new(&plugin_name, ipc_config).await?;

        // Create Linux sandbox if on Linux
        #[cfg(target_os = "linux")]
        let sandbox = {
            let sandbox_config = SandboxConfig::default();
            match LinuxSandbox::new(&plugin_name, resource_limits.clone(), sandbox_config) {
                Ok(s) => Some(s),
                Err(e) => {
                    warn!("Failed to create sandbox: {} (continuing without sandbox)", e);
                    None
                }
            }
        };

        Ok(Self {
            plugin_name,
            plugin_path,
            child: None,
            pid: None,
            ipc_server,
            ipc_channel: None,
            status: ProcessStatus::Starting,
            resource_limits,
            start_time: None,
            restart_attempts: 0,
            last_health_check: None,
            #[cfg(target_os = "linux")]
            sandbox,
        })
    }

    /// Start the plugin worker process
    pub async fn start(&mut self, worker_binary: &PathBuf, startup_timeout: Duration) -> Result<(), PluginError> {
        info!("Starting plugin process: {}", self.plugin_name);

        self.status = ProcessStatus::Starting;
        self.start_time = Some(Instant::now());

        // Prepare sandbox (create cgroups, etc.)
        #[cfg(target_os = "linux")]
        if let Some(sandbox) = &mut self.sandbox {
            sandbox.prepare()?;
        }

        // Get IPC endpoint
        let endpoint = self.ipc_server.endpoint();

        // Spawn worker process
        let child = Command::new(worker_binary)
            .arg("--plugin").arg(&self.plugin_path)
            .arg("--endpoint").arg(&endpoint)
            .arg("--name").arg(&self.plugin_name)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| PluginError::LoadError(format!("Failed to spawn worker process: {}", e)))?;

        let pid = child.id();
        info!("Spawned worker process for plugin '{}' with PID: {}", self.plugin_name, pid);

        // Apply sandbox to the spawned process
        #[cfg(target_os = "linux")]
        if let Some(sandbox) = &self.sandbox {
            if let Err(e) = sandbox.apply_to_process(pid) {
                warn!("Failed to apply sandbox to process: {}", e);
            }
        }

        self.child = Some(child);
        self.pid = Some(pid);

        // Wait for worker to connect (with timeout)
        match timeout(startup_timeout, self.ipc_server.accept()).await {
            Ok(Ok(channel)) => {
                info!("Plugin '{}' connected via IPC", self.plugin_name);
                self.ipc_channel = Some(channel);

                // Initialize plugin
                self.initialize_plugin().await?;

                self.status = ProcessStatus::Running;
                Ok(())
            }
            Ok(Err(e)) => {
                error!("IPC connection failed for plugin '{}': {}", self.plugin_name, e);
                self.status = ProcessStatus::Failed;
                self.kill();
                Err(PluginError::LoadError(format!("IPC connection failed: {}", e)))
            }
            Err(_) => {
                error!("Plugin '{}' startup timeout", self.plugin_name);
                self.status = ProcessStatus::Failed;
                self.kill();
                Err(PluginError::LoadError("Startup timeout".to_string()))
            }
        }
    }

    /// Initialize the plugin via IPC
    async fn initialize_plugin(&mut self) -> Result<(), PluginError> {
        if let Some(channel) = &mut self.ipc_channel {
            debug!("Initializing plugin '{}'", self.plugin_name);

            // Send Initialize message
            channel.send(&IpcMessage::Initialize {
                context_data: Vec::new(), // TODO: Serialize actual context
            }).await?;

            // Wait for response
            match channel.recv().await? {
                IpcMessage::InitializeResponse { success, error } => {
                    if success {
                        info!("Plugin '{}' initialized successfully", self.plugin_name);
                        Ok(())
                    } else {
                        let err_msg = error.unwrap_or_else(|| "Unknown error".to_string());
                        error!("Plugin '{}' initialization failed: {}", self.plugin_name, err_msg);
                        Err(PluginError::InitializationError(err_msg))
                    }
                }
                _ => {
                    Err(PluginError::Protocol("Unexpected response to Initialize".to_string()))
                }
            }
        } else {
            Err(PluginError::Protocol("No IPC channel".to_string()))
        }
    }

    /// Execute a hook in the plugin
    pub async fn execute_hook(
        &mut self,
        hook_name: &str,
        data: Vec<u8>,
        timeout_ms: u64,
    ) -> Result<Vec<u8>, PluginError> {
        if let Some(channel) = &mut self.ipc_channel {
            // Send ExecuteHook message
            channel.send(&IpcMessage::ExecuteHook {
                hook_name: hook_name.to_string(),
                data,
                timeout_ms,
            }).await?;

            // Wait for response with timeout
            let response_timeout = Duration::from_millis(timeout_ms + 1000); // Add 1s buffer
            match timeout(response_timeout, channel.recv()).await {
                Ok(Ok(IpcMessage::HookResponse { result, error })) => {
                    if let Some(err) = error {
                        Err(PluginError::HookError(format!("Hook execution failed: {}", err)))
                    } else {
                        Ok(result)
                    }
                }
                Ok(Ok(_)) => {
                    Err(PluginError::Protocol("Unexpected response to ExecuteHook".to_string()))
                }
                Ok(Err(e)) => {
                    Err(PluginError::from(e))
                }
                Err(_) => {
                    warn!("Hook execution timeout for plugin '{}'", self.plugin_name);
                    Err(PluginError::Timeout(timeout_ms))
                }
            }
        } else {
            Err(PluginError::Protocol("No IPC channel".to_string()))
        }
    }

    /// Perform health check (ping/pong)
    pub async fn health_check(&mut self) -> Result<bool, PluginError> {
        if let Some(channel) = &mut self.ipc_channel {
            // Send Ping
            channel.send(&IpcMessage::Ping).await?;

            // Wait for Pong with timeout
            match timeout(Duration::from_secs(5), channel.recv()).await {
                Ok(Ok(IpcMessage::Pong)) => {
                    self.last_health_check = Some(Instant::now());
                    Ok(true)
                }
                Ok(Ok(_)) => {
                    warn!("Unexpected response to Ping from plugin '{}'", self.plugin_name);
                    Ok(false)
                }
                Ok(Err(e)) => {
                    warn!("Health check failed for plugin '{}': {}", self.plugin_name, e);
                    Ok(false)
                }
                Err(_) => {
                    warn!("Health check timeout for plugin '{}'", self.plugin_name);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    /// Shutdown the plugin gracefully
    pub async fn shutdown(&mut self, grace_period: Duration) -> Result<(), PluginError> {
        info!("Shutting down plugin process: {}", self.plugin_name);
        self.status = ProcessStatus::ShuttingDown;

        // Send shutdown message via IPC
        if let Some(channel) = &mut self.ipc_channel {
            let _ = channel.send(&IpcMessage::Shutdown {
                grace_period_ms: grace_period.as_millis() as u64,
            }).await;

            // Wait for acknowledgment with timeout
            let _ = timeout(grace_period, channel.recv()).await;
        }

        // Wait for process to exit
        if let Some(child) = &mut self.child {
            match timeout(grace_period, async {
                child.wait()
            }).await {
                Ok(Ok(status)) => {
                    info!("Plugin '{}' exited with status: {}", self.plugin_name, status);
                }
                Ok(Err(e)) => {
                    warn!("Error waiting for plugin '{}': {}", self.plugin_name, e);
                }
                Err(_) => {
                    warn!("Plugin '{}' shutdown timeout, killing", self.plugin_name);
                    self.kill();
                }
            }
        }

        self.status = ProcessStatus::Stopped;
        self.child = None;
        self.ipc_channel = None;

        Ok(())
    }

    /// Kill the process immediately
    pub fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            info!("Killing plugin process: {}", self.plugin_name);
            let _ = child.kill();
            let _ = child.wait();
        }
        self.status = ProcessStatus::Stopped;
        self.ipc_channel = None;
    }

    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.status = ProcessStatus::Crashed;
                    self.child = None;
                    self.ipc_channel = None;
                    false
                }
                Ok(None) => {
                    // Process still running
                    true
                }
                Err(_) => {
                    // Error checking status
                    false
                }
            }
        } else {
            false
        }
    }

    /// Get process uptime
    pub fn uptime(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// Increment restart attempt counter
    pub fn increment_restart_attempts(&mut self) {
        self.restart_attempts += 1;
    }

    /// Get restart attempt count
    pub fn restart_attempts(&self) -> u32 {
        self.restart_attempts
    }

    /// Reset restart attempt counter
    pub fn reset_restart_attempts(&mut self) {
        self.restart_attempts = 0;
    }

    /// Get memory usage from cgroups (Linux only)
    #[cfg(target_os = "linux")]
    pub fn get_cgroup_memory_usage(&self) -> Option<usize> {
        self.sandbox.as_ref()
            .and_then(|s| s.get_memory_usage().ok())
    }

    /// Get CPU usage from cgroups (Linux only)
    #[cfg(target_os = "linux")]
    pub fn get_cgroup_cpu_usage(&self) -> Option<u64> {
        self.sandbox.as_ref()
            .and_then(|s| s.get_cpu_usage().ok())
    }

    /// Check if sandbox is enabled
    #[cfg(target_os = "linux")]
    pub fn has_sandbox(&self) -> bool {
        self.sandbox.is_some()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn has_sandbox(&self) -> bool {
        false
    }
}

impl Drop for PluginProcess {
    fn drop(&mut self) {
        // Ensure process is killed when PluginProcess is dropped
        if self.child.is_some() {
            warn!("PluginProcess dropped while still running, killing: {}", self.plugin_name);
            self.kill();
        }
    }
}

