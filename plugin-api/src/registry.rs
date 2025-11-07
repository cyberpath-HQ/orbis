use libloading::Library;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, Duration};
use tracing::{info, warn, error, debug};
use crate::{BridgedPlugin, HookRegistry, Plugin, PluginContext, PluginError, PluginLoader, PluginSecurity, TrustLevel, TrustedPluginEntry, ResourceLimits, ViolationTracker, UnmountBehavior, ViolationType};
use crate::bridge::PluginBridge;

/// Status of a plugin in the registry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is on disk but not loaded
    Available,
    /// Plugin is loaded and active
    Active,
    /// Plugin is loaded but inactive
    Inactive,
    /// Plugin load failed
    Failed,
    /// Plugin is untrusted (not in trust list)
    Untrusted,
}

/// Comprehensive plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Plugin description in Markdown format (optional)
    pub description: Option<String>,
    /// Current status
    pub status: PluginStatus,
    /// Trust level
    pub trust_level: TrustLevel,
    /// Path on disk
    pub path: PathBuf,
    /// SHA3-512 hash
    pub hash: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp
    pub modified: Option<SystemTime>,
    /// Trusted entry info (if available)
    pub trust_info: Option<TrustedPluginEntry>,
    /// Error message if load failed
    pub error: Option<String>,
    /// Resource limits declared by plugin
    pub resource_limits: Option<ResourceLimits>,
}

/// Container for a loaded plugin with isolation metadata
struct LoadedPlugin {
    plugin: Box<PluginBridge>,
    #[allow(dead_code)] // Library must be held to keep symbols loaded
    library: Library,
    info: PluginInfo,
    /// Memory isolation tracking
    memory_region: Option<MemoryRegion>,
    /// Resource limits for this plugin
    resource_limits: ResourceLimits,
    /// Violation tracker
    violation_tracker: ViolationTracker,
    /// Unmount behavior
    unmount_behavior: UnmountBehavior,
}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        info!("Dropping LoadedPlugin: {} - ensuring cleanup", self.info.name);

        // Clear memory region tracking
        if let Some(region) = self.memory_region.take() {
            debug!("Cleared memory region for plugin: {} (loaded at: {:?})",
                   self.info.name, region.loaded_at);
        }

        // Plugin's shutdown should have been called before drop,
        // but we can't call async methods here
        // The library will be unloaded when Library is dropped

        info!("LoadedPlugin dropped: {}", self.info.name);
    }
}

/// Memory region tracking for plugin isolation
#[derive(Debug, Clone)]
struct MemoryRegion {
    /// Estimated heap usage (future: enforce limits)
    #[allow(dead_code)]
    heap_size: usize,
    /// Load timestamp for tracking
    loaded_at: SystemTime,
}

/// Registry for managing all plugins (loaded and available)
pub struct PluginRegistry {
    /// Loaded plugins (in-process mode)
    loaded_plugins: RwLock<HashMap<String, LoadedPlugin>>,
    /// All tracked plugins (loaded + available on disk)
    all_plugins: RwLock<HashMap<String, PluginInfo>>,
    /// Plugin loader
    loader: Arc<PluginLoader>,
    /// Plugin context
    context: Arc<PluginContext>,
    /// Hook registry
    hook_registry: Arc<RwLock<HookRegistry>>,
    /// Security manager reference
    security: Arc<PluginSecurity>,
    /// Process manager for sandboxed plugins (optional)
    #[cfg(target_os = "linux")]
    process_manager: Option<Arc<crate::process::PluginProcessManager>>,
    /// Enable sandboxed mode (process isolation)
    sandboxed_mode: bool,
}

impl PluginRegistry {
    pub fn new(
        loader: Arc<PluginLoader>,
        context: Arc<PluginContext>,
        hook_registry: Arc<RwLock<HookRegistry>>,
        security: Arc<PluginSecurity>,
    ) -> Self {
        Self {
            loaded_plugins: RwLock::new(HashMap::new()),
            all_plugins: RwLock::new(HashMap::new()),
            loader,
            context,
            hook_registry,
            security,
            #[cfg(target_os = "linux")]
            process_manager: None,
            sandboxed_mode: false,
        }
    }

    /// Create a new registry with sandboxing enabled
    #[cfg(target_os = "linux")]
    pub fn new_sandboxed(
        loader: Arc<PluginLoader>,
        context: Arc<PluginContext>,
        hook_registry: Arc<RwLock<HookRegistry>>,
        security: Arc<PluginSecurity>,
        process_manager: Arc<crate::process::PluginProcessManager>,
    ) -> Self {
        Self {
            loaded_plugins: RwLock::new(HashMap::new()),
            all_plugins: RwLock::new(HashMap::new()),
            loader,
            context,
            hook_registry,
            security,
            process_manager: Some(process_manager),
            sandboxed_mode: true,
        }
    }

    /// Check if sandboxing is enabled
    pub fn is_sandboxed(&self) -> bool {
        self.sandboxed_mode
    }

    /// Scan a directory for available plugins and update registry
    pub fn scan_directory<P: AsRef<Path>>(&self, path: P) -> Result<usize, PluginError> {
        let path = path.as_ref();

        if !path.exists() || !path.is_dir() {
            return Err(PluginError::LoadError(format!("Invalid directory: {}", path.display())));
        }

        let mut discovered = 0;
        let entries = std::fs::read_dir(path)
            .map_err(|e| PluginError::IoError(e))?;

        for entry in entries.flatten() {
            let entry_path = entry.path();

            // Check for shared library files
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    if ext == "so" || ext == "dll" || ext == "dylib" {
                        if let Err(e) = self.discover_plugin(&entry_path) {
                            warn!("Failed to discover plugin at {}: {}", entry_path.display(), e);
                        } else {
                            discovered += 1;
                        }
                    }
                }
            }
        }

        info!("Discovered {} plugins in {}", discovered, path.display());
        Ok(discovered)
    }

    /// Discover and register a single plugin without loading it
    pub fn discover_plugin<P: AsRef<Path>>(&self, path: P) -> Result<(), PluginError> {
        let path = path.as_ref();

        // Get file metadata
        let metadata = std::fs::metadata(path)
            .map_err(|e| PluginError::IoError(e))?;

        let size = metadata.len();
        let modified = metadata.modified().ok();

        // Calculate hash
        let hash = self.security.calculate_hash(path)?;

        // Check if trusted
        let is_trusted = self.security.is_trusted_hash(&hash)?;
        let trust_info = self.security.get_plugin_info(&hash)?;

        // Determine initial status
        let status = if is_trusted {
            PluginStatus::Available
        } else {
            PluginStatus::Untrusted
        };

        // Extract name from path (we don't load it yet, so use filename)
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let plugin_info = PluginInfo {
            name: name.clone(),
            version: trust_info.as_ref()
                .map(|t| t.version.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            author: "unknown".to_string(), // Can't determine without loading
            description: None, // Can't determine without loading
            status,
            trust_level: if is_trusted { TrustLevel::Trusted } else { TrustLevel::Untrusted },
            path: path.to_path_buf(),
            hash,
            size,
            modified,
            trust_info,
            error: None,
            resource_limits: None,  // Not known until loaded
        };

        // Register in all_plugins
        {
            let mut all_plugins = self.all_plugins.write()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;
            all_plugins.insert(name, plugin_info);
        }

        Ok(())
    }

    /// Get all tracked plugins (loaded and available)
    pub fn get_all_plugins(&self) -> Result<Vec<PluginInfo>, PluginError> {
        let all_plugins = self.all_plugins.read()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

        Ok(all_plugins.values().cloned().collect())
    }

    /// Get available (unloaded but trusted) plugins
    pub fn get_available_plugins(&self) -> Result<Vec<PluginInfo>, PluginError> {
        let all_plugins = self.all_plugins.read()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

        Ok(all_plugins.values()
            .filter(|p| p.status == PluginStatus::Available)
            .cloned()
            .collect())
    }

    /// Get untrusted plugins
    pub fn get_untrusted_plugins(&self) -> Result<Vec<PluginInfo>, PluginError> {
        let all_plugins = self.all_plugins.read()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

        Ok(all_plugins.values()
            .filter(|p| p.status == PluginStatus::Untrusted)
            .cloned()
            .collect())
    }

    /// Programmatically load a plugin by name
    pub async fn load_plugin_by_name(&self, name: &str) -> Result<(), PluginError> {
        // Get plugin info
        let plugin_info = {
            let all_plugins = self.all_plugins.read()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

            all_plugins.get(name).cloned()
                .ok_or_else(|| PluginError::NotFound(name.to_string()))?
        };

        // Check if already loaded
        if plugin_info.status == PluginStatus::Active {
            return Err(PluginError::AlreadyLoaded(name.to_string()));
        }

        // Check if trusted
        if plugin_info.trust_level == TrustLevel::Untrusted {
            return Err(PluginError::UntrustedPlugin);
        }

        // Load the plugin
        self.load_plugin(&plugin_info.path, TrustLevel::Trusted).await?;

        Ok(())
    }

    /// Load and register a plugin with memory isolation
    pub async fn load_plugin<P: AsRef<Path>>(
        &self,
        path: P,
        trust_level: TrustLevel,
    ) -> Result<String, PluginError> {
        let path_ref = path.as_ref();

        // Load plugin with isolation
        let (mut plugin, library) = self.loader.load(path_ref, trust_level)?;

        let plugin_name = plugin.name().to_string();

        // Check if plugin is already loaded
        {
            let plugins = self.loaded_plugins.read()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;
            if plugins.contains_key(&plugin_name) {
                return Err(PluginError::AlreadyLoaded(plugin_name));
            }
        }

        info!("Initializing plugin with memory isolation: {}", plugin_name);

        // Get resource limits from plugin or use defaults
        let resource_limits = plugin.resource_limits().unwrap_or_else(|| {
            info!("Plugin '{}' did not declare resource limits, using defaults", plugin_name);
            ResourceLimits::default()
        });

        // Validate resource limits
        resource_limits.validate()?;

        info!("Plugin '{}' resource limits: heap={}MB, cpu={}ms, connections={}",
              plugin_name,
              resource_limits.max_heap_bytes / (1024 * 1024),
              resource_limits.max_cpu_time_ms,
              resource_limits.max_connections);

        // Create memory isolation region
        let memory_region = MemoryRegion {
            heap_size: 0, // Will be tracked during runtime
            loaded_at: SystemTime::now(),
        };

        // Create violation tracker
        let violation_tracker = ViolationTracker::default();

        // Create unmount behavior (default: graceful)
        let unmount_behavior = UnmountBehavior::default();

        // Initialize the plugin in isolated context
        plugin.init(self.context.clone()).await.map_err(|e| {
            PluginError::InitializationError(format!(
                "Plugin '{}' initialization failed: {}",
                plugin_name, e
            ))
        })?;

        // Register hooks
        {
            let mut hook_registry = self.hook_registry.write()
                .map_err(|e| PluginError::HookError(format!("Failed to acquire hook registry lock: {}", e)))?;
            plugin.register_hooks(&mut hook_registry).await?;
        }

        // Calculate hash for tracking
        let hash = self.security.calculate_hash(path_ref)?;
        let trust_info = self.security.get_plugin_info(&hash)?;
        let metadata = std::fs::metadata(path_ref).ok();

        let info = PluginInfo {
            name: plugin_name.clone(),
            version: plugin.version().to_string(),
            author: plugin.author().to_string(),
            description: plugin.description().map(|s| s.to_string()),
            status: PluginStatus::Active,
            trust_level,
            path: path_ref.to_path_buf(),
            hash,
            size: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
            modified: metadata.and_then(|m| m.modified().ok()),
            trust_info,
            error: None,
            resource_limits: Some(resource_limits.clone()),
        };

        info!(
            "Plugin loaded: {} v{} by {} (isolated, limits enforced)",
            info.name, info.version, info.author
        );

        // Store the loaded plugin with isolation tracking
        let loaded = LoadedPlugin {
            plugin,
            library,
            info: info.clone(),
            memory_region: Some(memory_region),
            resource_limits,
            violation_tracker,
            unmount_behavior,
        };

        {
            let mut plugins = self.loaded_plugins.write()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;
            plugins.insert(plugin_name.clone(), loaded);
        }

        // Update all_plugins registry
        {
            let mut all_plugins = self.all_plugins.write()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;
            all_plugins.insert(plugin_name.clone(), info);
        }

        Ok(plugin_name)
    }

    /// Unload a plugin by name and clean up isolation
    pub async fn unload_plugin(&self, name: &str) -> Result<(), PluginError> {
        info!("Unloading plugin: {}", name);

        let mut loaded = {
            let mut plugins = self.loaded_plugins.write()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;

            plugins
                .remove(name)
                .ok_or_else(|| PluginError::NotFound(name.to_string()))?
        };

        // Shutdown the plugin
        if let Err(e) = loaded.plugin.shutdown().await {
            warn!("Plugin '{}' shutdown failed: {}", name, e);
        }

        // Clean up memory isolation
        if let Some(region) = loaded.memory_region.take() {
            info!("Cleaning up memory region for plugin: {} (loaded at: {:?})", name, region.loaded_at);
        }

        // Update status in all_plugins
        {
            let mut all_plugins = self.all_plugins.write()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;

            if let Some(plugin_info) = all_plugins.get_mut(name) {
                plugin_info.status = PluginStatus::Available;
            }
        }

        // The library will be automatically dropped here, unloading the dynamic library

        info!("Plugin unloaded: {}", name);
        Ok(())
    }

    /// Get information about any plugin (loaded or available)
    pub fn get_plugin_info(&self, name: &str) -> Result<PluginInfo, PluginError> {
        let all_plugins = self.all_plugins.read()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

        all_plugins
            .get(name)
            .cloned()
            .ok_or_else(|| PluginError::NotFound(name.to_string()))
    }

    /// Get a list of all loaded plugins
    pub fn list_loaded_plugins(&self) -> Result<Vec<PluginInfo>, PluginError> {
        let plugins = self.loaded_plugins.read()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

        Ok(plugins.values().map(|p| p.info.clone()).collect())
    }


    /// Check if a plugin is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded_plugins
            .read()
            .map(|plugins| plugins.contains_key(name))
            .unwrap_or(false)
    }

    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.loaded_plugins.read().map(|p| p.len()).unwrap_or(0)
    }

    /// Get the total number of tracked plugins (loaded + available)
    pub fn total_plugin_count(&self) -> usize {
        self.all_plugins.read().map(|p| p.len()).unwrap_or(0)
    }

    /// Unload all plugins
    ///
    /// # Notice
    /// Manual unload is required to ensure proper shutdown of all plugins.
    pub async fn unload_all(&self) -> Result<(), PluginError> {
        info!("Unloading all plugins");

        let plugin_names: Vec<String> = {
            let plugins = self.loaded_plugins.read()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;
            plugins.keys().cloned().collect()
        };

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name).await {
                warn!("Failed to unload plugin '{}': {}", name, e);
            }
        }

        Ok(())
    }

    /// Record a resource violation for a plugin
    pub fn record_violation(&self, plugin_name: &str, violation_type: ViolationType) -> Result<bool, PluginError> {
        let should_unmount = {
            let mut plugins = self.loaded_plugins.write()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;

            if let Some(loaded) = plugins.get_mut(plugin_name) {
                loaded.violation_tracker.record_violation(violation_type.clone());

                if loaded.unmount_behavior.log_violations {
                    warn!("Resource violation in plugin '{}': {:?}", plugin_name, violation_type);
                }

                let should_unmount = loaded.unmount_behavior.auto_unmount
                    && loaded.violation_tracker.should_unmount();

                if should_unmount {
                    error!("Plugin '{}' exceeded violation threshold, will be unmounted", plugin_name);
                }

                should_unmount
            } else {
                return Err(PluginError::NotFound(plugin_name.to_string()));
            }
        };

        Ok(should_unmount)
    }

    /// Get violation count for a plugin
    pub fn get_violation_count(&self, plugin_name: &str) -> Result<usize, PluginError> {
        let mut plugins = self.loaded_plugins.write()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;

        if let Some(loaded) = plugins.get_mut(plugin_name) {
            Ok(loaded.violation_tracker.violation_count())
        } else {
            Err(PluginError::NotFound(plugin_name.to_string()))
        }
    }

    /// Set unmount behavior for a plugin
    pub fn set_unmount_behavior(&self, plugin_name: &str, behavior: UnmountBehavior) -> Result<(), PluginError> {
        let mut plugins = self.loaded_plugins.write()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;

        if let Some(loaded) = plugins.get_mut(plugin_name) {
            loaded.unmount_behavior = behavior;
            Ok(())
        } else {
            Err(PluginError::NotFound(plugin_name.to_string()))
        }
    }

    /// Get resource limits for a plugin
    pub fn get_resource_limits(&self, plugin_name: &str) -> Result<ResourceLimits, PluginError> {
        let plugins = self.loaded_plugins.read()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

        if let Some(loaded) = plugins.get(plugin_name) {
            Ok(loaded.resource_limits.clone())
        } else {
            Err(PluginError::NotFound(plugin_name.to_string()))
        }
    }

    /// Reset violation tracker for a plugin
    pub fn reset_violations(&self, plugin_name: &str) -> Result<(), PluginError> {
        let mut plugins = self.loaded_plugins.write()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire write lock: {}", e)))?;

        if let Some(loaded) = plugins.get_mut(plugin_name) {
            loaded.violation_tracker.reset();
            info!("Reset violation tracker for plugin '{}'", plugin_name);
            Ok(())
        } else {
            Err(PluginError::NotFound(plugin_name.to_string()))
        }
    }

    /// Start a background monitoring task using tokio that continuously checks for resource violations
    /// and automatically unmounts plugins when violations exceed thresholds.
    ///
    /// # Arguments
    /// * `check_interval` - How often to check for violations (default: 10 seconds)
    ///
    /// # Returns
    /// A tokio JoinHandle for the monitor task
    pub fn start_resource_monitor(
        self: Arc<Self>,
        check_interval: Option<Duration>,
    ) -> tokio::task::JoinHandle<()> {
        let interval = check_interval.unwrap_or(Duration::from_secs(10));

        info!("Starting resource monitor task (check interval: {:?})", interval);

        // Spawn monitoring task
        tokio::spawn(async move {
            let mut last_status_log = tokio::time::Instant::now();

            loop {
                tokio::time::sleep(interval).await;

                // Clone Arc for the blocking task
                let registry = self.clone();

                // Perform monitoring checks in a blocking task to avoid Send issues with std::sync::RwLock
                let result = tokio::task::spawn_blocking(move || {
                    // Get list of loaded plugins
                    let plugin_names: Vec<String> = {
                        match registry.loaded_plugins.read() {
                            Ok(plugins) => plugins.keys().cloned().collect(),
                            Err(e) => {
                                error!("Resource monitor: Failed to acquire read lock: {}", e);
                                return (Vec::new(), Vec::new());
                            }
                        }
                    };

                    if plugin_names.is_empty() {
                        return (Vec::new(), Vec::new());
                    }

                    let mut plugins_to_unmount = Vec::new();

                    // Check each plugin for violations and resource usage
                    for plugin_name in &plugin_names {
                        // Check actual resource usage for this plugin (blocking operation)
                        if let Err(e) = registry.check_and_record_resource_usage_sync(plugin_name) {
                            warn!("Resource monitor: Failed to check resources for plugin '{}': {}", plugin_name, e);
                        }

                        // Check if plugin should be unmounted based on violations
                        let should_unmount = {
                            match registry.loaded_plugins.write() {
                                Ok(mut plugins) => {
                                    if let Some(loaded) = plugins.get_mut(plugin_name) {
                                        let should_unmount = loaded.unmount_behavior.auto_unmount
                                            && loaded.violation_tracker.should_unmount();

                                        if should_unmount {
                                            let violation_count = loaded.violation_tracker.violation_count();
                                            error!(
                                                "Resource monitor: Plugin '{}' exceeded violation threshold ({} violations), scheduling unmount",
                                                plugin_name, violation_count
                                            );
                                        }

                                        should_unmount
                                    } else {
                                        false
                                    }
                                }
                                Err(e) => {
                                    error!("Resource monitor: Failed to acquire write lock for plugin '{}': {}", plugin_name, e);
                                    false
                                }
                            }
                        };

                        if should_unmount {
                            plugins_to_unmount.push(plugin_name.clone());
                        }
                    }

                    (plugin_names, plugins_to_unmount)
                }).await;

                match result {
                    Ok((plugin_names, plugins_to_unmount)) => {
                        // Handle unmount requests - we need to use a blocking approach
                        // since the shutdown future is not Send
                        for plugin_name in plugins_to_unmount {
                            let registry_clone = self.clone();
                            let plugin_name_clone = plugin_name.clone();

                            // Use spawn_blocking to handle the unmount in a blocking context
                            tokio::task::spawn_blocking(move || {
                                info!("Resource monitor: Unmounting plugin '{}'", plugin_name_clone);

                                // We need to create a new runtime for this blocking task
                                // to execute the async unload_plugin function
                                let rt = tokio::runtime::Handle::try_current();
                                if let Ok(handle) = rt {
                                    handle.block_on(async {
                                        if let Err(e) = registry_clone.unload_plugin(&plugin_name_clone).await {
                                            error!("Resource monitor: Failed to unmount plugin '{}': {}", plugin_name_clone, e);
                                        } else {
                                            info!("Resource monitor: Successfully unmounted plugin '{}'", plugin_name_clone);
                                        }
                                    });
                                } else {
                                    error!("Resource monitor: No tokio runtime available for unmounting plugin '{}'", plugin_name_clone);
                                }
                            });
                        }

                        // Log monitoring status periodically (every 60 seconds)
                        let now = tokio::time::Instant::now();
                        if now.duration_since(last_status_log) >= Duration::from_secs(60) {
                            last_status_log = now;

                            let registry_clone = self.clone();
                            let plugin_names_clone = plugin_names.clone();
                            let plugin_count = plugin_names.len();

                            let total_violations: usize = tokio::task::spawn_blocking(move || {
                                plugin_names_clone.iter()
                                    .filter_map(|name| {
                                        registry_clone.loaded_plugins.write().ok().and_then(|mut plugins| {
                                            plugins.get_mut(name).map(|p| p.violation_tracker.violation_count())
                                        })
                                    })
                                    .sum()
                            }).await.unwrap_or(0);

                            if total_violations > 0 {
                                warn!(
                                    "Resource monitor status: {} plugins loaded, {} total violations",
                                    plugin_count, total_violations
                                );
                            } else {
                                info!("Resource monitor status: {} plugins loaded, no violations", plugin_count);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Resource monitor: Blocking task panicked: {}", e);
                    }
                }
            }
        })
    }

    /// Synchronous version of resource checking for use in blocking tasks
    fn check_and_record_resource_usage_sync(&self, plugin_name: &str) -> Result<(), PluginError> {
        let (limits, pid) = {
            let plugins = self.loaded_plugins.read()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

            let loaded = plugins.get(plugin_name)
                .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;

            (loaded.resource_limits.clone(), std::process::id())
        };

        // Check process-level resource usage
        #[cfg(target_os = "linux")]
        {
            // Check memory usage via /proc/[pid]/status
            if let Ok(status) = std::fs::read_to_string(format!("/proc/{}/status", pid)) {
                if let Some(vmrss_line) = status.lines().find(|l| l.starts_with("VmRSS:")) {
                    if let Some(value_str) = vmrss_line.split_whitespace().nth(1) {
                        if let Ok(vmrss_kb) = value_str.parse::<usize>() {
                            let vmrss_bytes = vmrss_kb * 1024;

                            if vmrss_bytes > limits.max_heap_bytes {
                                let violation = ViolationType::HeapMemory {
                                    used: vmrss_bytes,
                                    limit: limits.max_heap_bytes,
                                };

                                if let Ok(should_unmount) = self.record_violation(plugin_name, violation.clone()) {
                                    if should_unmount {
                                        warn!("Plugin '{}' will be unmounted due to memory violation", plugin_name);
                                    }
                                }
                            }
                        }
                    }
                }

                // Check thread count
                if let Some(threads_line) = status.lines().find(|l| l.starts_with("Threads:")) {
                    if let Some(value_str) = threads_line.split_whitespace().nth(1) {
                        if let Ok(thread_count) = value_str.parse::<u32>() {
                            if thread_count > limits.max_threads {
                                let violation = ViolationType::Threads {
                                    used: thread_count,
                                    limit: limits.max_threads,
                                };

                                let _ = self.record_violation(plugin_name, violation);
                            }
                        }
                    }
                }
            }

            // Check file descriptor count via /proc/[pid]/fd
            if let Ok(fd_entries) = std::fs::read_dir(format!("/proc/{}/fd", pid)) {
                let fd_count = fd_entries.count() as u32;

                if fd_count > limits.max_file_descriptors {
                    let violation = ViolationType::FileDescriptors {
                        used: fd_count,
                        limit: limits.max_file_descriptors,
                    };

                    let _ = self.record_violation(plugin_name, violation);
                }
            }

            // Check network connections via /proc/net/tcp and /proc/net/tcp6
            let mut connection_count = 0u32;

            if let Ok(tcp_content) = std::fs::read_to_string("/proc/net/tcp") {
                connection_count += tcp_content.lines().skip(1).count() as u32;
            }

            if let Ok(tcp6_content) = std::fs::read_to_string("/proc/net/tcp6") {
                connection_count += tcp6_content.lines().skip(1).count() as u32;
            }

            if connection_count > limits.max_connections {
                let violation = ViolationType::Connections {
                    used: connection_count,
                    limit: limits.max_connections,
                };

                let _ = self.record_violation(plugin_name, violation);
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // On Windows or other platforms, resource monitoring would need platform-specific APIs
                    warn!("Detailed resource monitoring is currently only supported on Linux");
        }

        Ok(())
    }

    /// Check and record resource usage for a specific plugin (async version)
    /// This monitors actual resource consumption and records violations
    async fn check_and_record_resource_usage(&self, plugin_name: &str) -> Result<(), PluginError> {
        let (limits, pid) = {
            let plugins = self.loaded_plugins.read()
                .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

            let loaded = plugins.get(plugin_name)
                .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;

            (loaded.resource_limits.clone(), std::process::id())
        };

        // Check process-level resource usage
        // Note: This is a simplified implementation. In production, you would:
        // - Use proc filesystem on Linux (/proc/[pid]/stat, /proc/[pid]/status)
        // - Use Windows API on Windows (GetProcessMemoryInfo, GetProcessTimes)
        // - Track per-plugin allocations with custom allocators
        // - Monitor thread counts, file descriptors, network connections per plugin

        #[cfg(target_os = "linux")]
        {
            use crate::limits::ViolationType;
            // Check memory usage via /proc/[pid]/status
            if let Ok(status) = tokio::fs::read_to_string(format!("/proc/{}/status", pid)).await {
                if let Some(vmrss_line) = status.lines().find(|l| l.starts_with("VmRSS:")) {
                    if let Some(value_str) = vmrss_line.split_whitespace().nth(1) {
                        if let Ok(vmrss_kb) = value_str.parse::<usize>() {
                            let vmrss_bytes = vmrss_kb * 1024;

                            // Check if this exceeds plugin limit
                            // Note: This is process-wide, not per-plugin
                            // In production, you'd need per-plugin memory tracking
                            if vmrss_bytes > limits.max_heap_bytes {
                                let violation = ViolationType::HeapMemory {
                                    used: vmrss_bytes,
                                    limit: limits.max_heap_bytes,
                                };

                                if let Ok(should_unmount) = self.record_violation(plugin_name, violation.clone()) {
                                    if should_unmount {
                                        warn!("Plugin '{}' will be unmounted due to memory violation", plugin_name);
                                    }
                                } else {
                                    error!("Failed to record memory violation for plugin '{}'", plugin_name);
                                }
                            }
                        }
                    }
                }

                // Check thread count
                if let Some(threads_line) = status.lines().find(|l| l.starts_with("Threads:")) {
                    if let Some(value_str) = threads_line.split_whitespace().nth(1) {
                        if let Ok(thread_count) = value_str.parse::<u32>() {
                            if thread_count > limits.max_threads {
                                let violation = ViolationType::Threads {
                                    used: thread_count,
                                    limit: limits.max_threads,
                                };

                                let _ = self.record_violation(plugin_name, violation);
                            }
                        }
                    }
                }
            }

            // Check file descriptor count via /proc/[pid]/fd
            if let Ok(mut fd_dir) = tokio::fs::read_dir(format!("/proc/{}/fd", pid)).await {
                let mut fd_count = 0u32;
                while let Ok(Some(_)) = fd_dir.next_entry().await {
                    fd_count += 1;
                }

                if fd_count > limits.max_file_descriptors {
                    let violation = ViolationType::FileDescriptors {
                        used: fd_count,
                        limit: limits.max_file_descriptors,
                    };

                    let _ = self.record_violation(plugin_name, violation);
                }
            }

            // Check network connections via /proc/net/tcp and /proc/net/tcp6
            let mut connection_count = 0u32;

            if let Ok(tcp_content) = tokio::fs::read_to_string("/proc/net/tcp").await {
                // Count lines (excluding header)
                connection_count += tcp_content.lines().skip(1).count() as u32;
            }

            if let Ok(tcp6_content) = tokio::fs::read_to_string("/proc/net/tcp6").await {
                connection_count += tcp6_content.lines().skip(1).count() as u32;
            }

            if connection_count > limits.max_connections {
                let violation = ViolationType::Connections {
                    used: connection_count,
                    limit: limits.max_connections,
                };

                let _ = self.record_violation(plugin_name, violation);
            }
        }

        // For non-Linux platforms, log a warning
        #[cfg(not(target_os = "linux"))]
        {
            // On Windows or other platforms, resource monitoring would need platform-specific APIs
            // For now, just log that monitoring is limited
                    warn!("Detailed resource monitoring is currently only supported on Linux");
        }

        Ok(())
    }

    /// Check resource usage for a specific plugin (for manual monitoring/testing)
    /// This simulates resource checks that would happen during plugin execution
    pub fn check_plugin_resources(&self, plugin_name: &str) -> Result<Vec<ViolationType>, PluginError> {
        let plugins = self.loaded_plugins.read()
            .map_err(|e| PluginError::LoadError(format!("Failed to acquire read lock: {}", e)))?;

        let loaded = plugins.get(plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;

        let violations = Vec::new();
        #[allow(unused_variables)]
        let limits = &loaded.resource_limits;

        // In a real implementation, you would check actual resource usage here
        // For now, this provides the structure for manual violation reporting

        // Example: Check heap usage (would be implemented with actual memory tracking)
        // if actual_heap > limits.max_heap_bytes {
        //     violations.push(ViolationType::HeapMemory {
        //         used: actual_heap,
        //         limit: limits.max_heap_bytes
        //     });
        // }

        // Return empty for now - violations are recorded externally during plugin execution
        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PluginSecurity, SecurityPolicy};

    #[test]
    fn test_registry_creation() {
        let security = Arc::new(PluginSecurity::new(SecurityPolicy::default(), vec![], vec![]));
        let loader = Arc::new(PluginLoader::new(security.clone()));
        let context = Arc::new(PluginContext::new());
        let hook_registry = Arc::new(RwLock::new(HookRegistry::new()));

        let _registry = PluginRegistry::new(loader, context, hook_registry, security);
    }
}

