//! Plugin runtime for executing plugin code.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use wasmtime::{
    AsContextMut, Caller, Engine, Instance, Linker, Memory, Module, Store, StoreLimits,
    StoreLimitsBuilder, TypedFunc, Val,
};

use super::{PluginInfo, PluginSource, SandboxConfig};

/// Maximum size for WASM memory allocations (256MB)
const MAX_ALLOCATION_SIZE: usize = 256 * 1024 * 1024;

/// Context passed to plugin handlers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    /// Request method.
    pub method: String,

    /// Request path.
    pub path: String,

    /// Request headers.
    pub headers: std::collections::HashMap<String, String>,

    /// Query parameters.
    pub query: std::collections::HashMap<String, String>,

    /// Request body (as JSON).
    #[serde(default)]
    pub body: serde_json::Value,

    /// User ID (if authenticated).
    #[serde(default)]
    pub user_id: Option<String>,

    /// User is admin.
    #[serde(default)]
    pub is_admin: bool,
}

/// Plugin state storage - each plugin has its own isolated state
#[derive(Debug, Clone, Default)]
pub struct PluginState {
    /// Key-value state storage (JSON values)
    data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Path to persist state to disk (if set)
    persist_path: Arc<RwLock<Option<std::path::PathBuf>>>,
}

impl PluginState {
    /// Create a new plugin state
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            persist_path: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new plugin state with persistence
    #[must_use]
    pub fn with_persistence(path: std::path::PathBuf) -> Self {
        let state = Self::new();
        *state.persist_path.write() = Some(path.clone());
        
        // Try to load existing state
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&contents) {
                    *state.data.write() = data;
                    tracing::debug!("Loaded plugin state from {:?}", path);
                } else {
                    tracing::warn!("Failed to parse plugin state from {:?}", path);
                }
            }
        }
        
        state
    }

    /// Save state to disk if persistence is enabled
    fn persist(&self) {
        if let Some(ref path) = *self.persist_path.read() {
            let data = self.data.read().clone();
            if let Ok(json) = serde_json::to_string_pretty(&data) {
                // Ensure parent directory exists
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                
                if let Err(e) = std::fs::write(path, json) {
                    tracing::error!("Failed to persist plugin state to {:?}: {}", path, e);
                }
            }
        }
    }

    /// Get a value from the state
    #[must_use]
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.read().get(key).cloned()
    }

    /// Set a value in the state
    pub fn set(&self, key: String, value: serde_json::Value) {
        self.data.write().insert(key, value);
        self.persist();
    }

    /// Remove a value from the state
    pub fn remove(&self, key: &str) -> Option<serde_json::Value> {
        let result = self.data.write().remove(key);
        self.persist();
        result
    }

    /// Clear all state
    pub fn clear(&self) {
        self.data.write().clear();
        self.persist();
    }

    /// Get all keys
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        self.data.read().keys().cloned().collect()
    }
}

/// Plugin configuration storage
#[derive(Debug, Clone, Default)]
pub struct PluginConfig {
    /// Configuration values
    data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl PluginConfig {
    /// Create a new plugin config
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create plugin config from manifest settings
    #[must_use]
    pub fn from_settings(settings: &HashMap<String, serde_json::Value>) -> Self {
        Self {
            data: Arc::new(RwLock::new(settings.clone())),
        }
    }

    /// Get a config value
    #[must_use]
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.read().get(key).cloned()
    }

    /// Set a config value
    pub fn set(&self, key: String, value: serde_json::Value) {
        self.data.write().insert(key, value);
    }
}

/// Store data combining WASM state and host data
pub struct StoreData {
    /// Memory limits for the WASM instance
    limits: StoreLimits,
    /// Plugin state storage
    state: PluginState,
    /// Plugin configuration
    config: PluginConfig,
    /// Plugin name for logging and permissions
    plugin_name: String,
    /// Sandbox configuration
    sandbox: Arc<SandboxConfig>,
    /// Call counter for max_calls enforcement
    call_count: u64,
    /// Execution start time for time limit enforcement
    start_time: Instant,
}

impl StoreData {
    /// Create new store data
    fn new(plugin_name: String, sandbox: Arc<SandboxConfig>, state: PluginState, config: PluginConfig) -> Self {
        let limits = StoreLimitsBuilder::new()
            .memory_size(sandbox.memory_limit)
            .build();

        Self {
            limits,
            state,
            config,
            plugin_name,
            sandbox,
            call_count: 0,
            start_time: Instant::now(),
        }
    }

    /// Check if execution should continue
    fn check_limits(&mut self) -> orbis_core::Result<()> {
        // Check call count
        self.call_count += 1;
        if self.call_count > self.sandbox.max_calls {
            return Err(orbis_core::Error::plugin(format!(
                "Plugin '{}' exceeded maximum calls: {}",
                self.plugin_name, self.sandbox.max_calls
            )));
        }

        // Check execution time
        let elapsed = self.start_time.elapsed();
        if elapsed.as_millis() > u128::from(self.sandbox.time_limit_ms) {
            return Err(orbis_core::Error::plugin(format!(
                "Plugin '{}' exceeded time limit: {}ms",
                self.plugin_name, self.sandbox.time_limit_ms
            )));
        }

        Ok(())
    }
}

/// Plugin runtime instance.
struct PluginInstance {
    engine: Engine,
    module: Module,
    sandbox_config: Arc<SandboxConfig>,
    state: PluginState,
    config: PluginConfig,
}

impl PluginInstance {
    /// Get the sandbox configuration.
    #[must_use]
    pub fn sandbox_config(&self) -> &SandboxConfig {
        &self.sandbox_config
    }

    /// Get the plugin state
    #[must_use]
    pub const fn state(&self) -> &PluginState {
        &self.state
    }

    /// Get the plugin config
    #[must_use]
    pub const fn config(&self) -> &PluginConfig {
        &self.config
    }
}

/// Plugin runtime for executing plugin code.
#[derive(Clone)]
pub struct PluginRuntime {
    instances:   DashMap<String, Arc<PluginInstance>>,
    engine:      Engine,
    plugins_dir: Arc<RwLock<Option<std::path::PathBuf>>>,
}

impl PluginRuntime {
    /// Create a new plugin runtime.
    #[must_use]
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true); // Enable fuel consumption for execution limits
        // config.epoch_interruption(true); // Enable epoch-based interruption
        config.max_wasm_stack(512 * 1024); // 512KB max stack

        let engine = Engine::new(&config).expect("Failed to create WASM engine");

        Self {
            instances:   DashMap::new(),
            engine,
            plugins_dir: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the plugins directory for state persistence.
    pub fn set_plugins_dir(&self, plugins_dir: std::path::PathBuf) {
        *self.plugins_dir.write() = Some(plugins_dir);
    }

    /// Check if a plugin has a specific permission.
    #[must_use]
    pub fn has_permission(&self, plugin_name: &str, permission: &str) -> bool {
        self.instances
            .get(plugin_name)
            .map(|instance| instance.sandbox_config().has_permission(permission))
            .unwrap_or(false)
    }

    /// Check if a plugin is allowed to access a network host.
    #[must_use]
    pub fn can_access_network(&self, plugin_name: &str, host: &str) -> bool {
        self.instances
            .get(plugin_name)
            .map(|instance| instance.sandbox_config().can_access_network(host))
            .unwrap_or(false)
    }

    /// Check if a plugin is allowed to access a file path.
    #[must_use]
    pub fn can_access_path(&self, plugin_name: &str, path: &str) -> bool {
        self.instances
            .get(plugin_name)
            .map(|instance| instance.sandbox_config().can_access_path(path))
            .unwrap_or(false)
    }

    /// Initialize a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub async fn initialize(
        &self,
        info: &PluginInfo,
        source: &PluginSource,
    ) -> orbis_core::Result<()> {
        let loader = super::PluginLoader::new();
        let code = loader.load_code(source, &info.manifest)?;

        let module = Module::new(&self.engine, &code).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to compile WASM module: {}", e))
        })?;

        // Create state with persistence if plugins directory is set
        let state = if let Some(ref plugins_dir) = *self.plugins_dir.read() {
            let state_dir = plugins_dir.join(".plugin_data");
            let state_file = state_dir.join(format!("{}.json", info.manifest.name));
            PluginState::with_persistence(state_file)
        } else {
            PluginState::new()
        };
        
        // Extract config from manifest
        let config = if let Some(obj) = info.manifest.config.as_object() {
            PluginConfig::from_settings(&obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        } else {
            PluginConfig::new()
        };

        let instance = PluginInstance {
            engine: self.engine.clone(),
            module,
            sandbox_config: Arc::new(SandboxConfig::from_permissions(&info.manifest.permissions)),
            state,
            config,
        };

        self.instances
            .insert(info.manifest.name.clone(), Arc::new(instance));

        Ok(())
    }

    /// Start a plugin.
    ///
    /// This is called when a plugin is enabled. The heavy initialization
    /// (WASM compilation and module instantiation) happens in `initialize()`.
    /// This method just verifies the plugin is ready to run.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be started.
    pub async fn start(&self, name: &str) -> orbis_core::Result<()> {
        // Just verify the plugin is initialized
        let _instance = self.instances.get(name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not initialized", name))
        })?;

        tracing::debug!("Started plugin: {}", name);
        Ok(())
    }

    /// Stop a plugin.
    ///
    /// This is called when a plugin is disabled. It clears runtime state
    /// but keeps the compiled WASM module cached for fast re-enable.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be stopped.
    pub async fn stop(&self, name: &str) -> orbis_core::Result<()> {
        // Just verify plugin exists, don't remove it (keep module cached)
        if let Some(instance) = self.instances.get(name) {
            // Only clear runtime state, not the instance itself
            instance.state.clear();
            tracing::debug!("Stopped plugin: {}", name);
        }
        Ok(())
    }

    /// Execute a plugin handler.
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails.
    pub async fn execute(
        &self,
        plugin_name: &str,
        handler: &str,
        context: PluginContext,
    ) -> orbis_core::Result<serde_json::Value> {
        let instance = self.instances.get(plugin_name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not running", plugin_name))
        })?;

        // Create store for execution
        let store_data = StoreData::new(
            plugin_name.to_string(),
            instance.sandbox_config.clone(),
            instance.state.clone(),
            instance.config.clone(),
        );
        let mut store = Store::new(&instance.engine, store_data);
        store.limiter(|data| &mut data.limits);

        // Add fuel for execution
        store
            .set_fuel(u64::from(instance.sandbox_config.time_limit_ms) * 1000)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to set fuel: {}", e)))?;

        // Create linker with host functions
        let mut linker = Linker::new(&instance.engine);
        Self::register_host_functions(&mut linker)?;

        // Instantiate the module
        let wasm_instance = linker
            .instantiate(&mut store, &instance.module)
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to instantiate plugin: {}", e))
            })?;

        // Get memory for data transfer
        let memory = wasm_instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| orbis_core::Error::plugin("Plugin memory not found"))?;

        // Serialize context to JSON
        let context_json = serde_json::to_vec(&context).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to serialize context: {}", e))
        })?;

        // Allocate memory in WASM for the context
        let (context_ptr, context_len) =
            Self::allocate_and_write(&mut store, &memory, &wasm_instance, &context_json)?;

        // Get the handler function
        let handler_func = wasm_instance
            .get_func(&mut store, handler)
            .ok_or_else(|| {
                orbis_core::Error::plugin(format!("Handler '{}' not found", handler))
            })?;

        // Call the handler with (ptr: i32, len: i32) -> i32 signature
        // The return value is a pointer to the result JSON
        let handler_typed: TypedFunc<(i32, i32), i32> = handler_func.typed(&store).map_err(|e| {
            orbis_core::Error::plugin(format!("Handler '{}' has wrong signature: {}", handler, e))
        })?;

        let result_ptr = handler_typed
            .call(&mut store, (context_ptr as i32, context_len as i32))
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to execute handler '{}': {}", handler, e))
            })?;

        // Read the result from WASM memory
        let result = Self::read_result(&mut store, &memory, result_ptr as u32)?;

        // Deallocate the context memory
        Self::deallocate(&mut store, &wasm_instance, context_ptr, context_len)?;

        Ok(result)
    }

    /// Check if a plugin is running.
    #[must_use]
    pub fn is_running(&self, name: &str) -> bool {
        self.instances.contains_key(name)
    }

    /// Clear cached data for a plugin.
    ///
    /// This is used during hot reload to ensure fresh module compilation.
    pub fn clear_cache(&self, name: &str) {
        if let Some((_, instance)) = self.instances.remove(name) {
            instance.state.clear();
        }
        tracing::debug!("Cleared cache for plugin: {}", name);
    }

    /// Get plugin state (for inspection/debugging)
    #[must_use]
    pub fn get_state(&self, name: &str) -> Option<PluginState> {
        self.instances.get(name).map(|i| i.state.clone())
    }

    /// Register host functions that plugins can call
    fn register_host_functions(linker: &mut Linker<StoreData>) -> orbis_core::Result<()> {
        // State management functions
        linker
            .func_wrap(
                "env",
                "state_get",
                |mut caller: Caller<'_, StoreData>, key_ptr: i32, key_len: i32| -> i32 {
                    match Self::host_state_get(&mut caller, key_ptr as u32, key_len as u32) {
                        Ok(ptr) => ptr as i32,
                        Err(e) => {
                            tracing::error!("state_get error: {}", e);
                            0 // Return null pointer on error
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register state_get: {}", e))
            })?;

        linker
            .func_wrap(
                "env",
                "state_set",
                |mut caller: Caller<'_, StoreData>,
                 key_ptr: i32,
                 key_len: i32,
                 value_ptr: i32,
                 value_len: i32|
                 -> i32 {
                    match Self::host_state_set(
                        &mut caller,
                        key_ptr as u32,
                        key_len as u32,
                        value_ptr as u32,
                        value_len as u32,
                    ) {
                        Ok(()) => 1, // Success
                        Err(e) => {
                            tracing::error!("state_set error: {}", e);
                            0 // Failure
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register state_set: {}", e))
            })?;

        linker
            .func_wrap(
                "env",
                "state_remove",
                |mut caller: Caller<'_, StoreData>, key_ptr: i32, key_len: i32| -> i32 {
                    match Self::host_state_remove(&mut caller, key_ptr as u32, key_len as u32) {
                        Ok(()) => 1, // Success
                        Err(e) => {
                            tracing::error!("state_remove error: {}", e);
                            0 // Failure
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register state_remove: {}", e))
            })?;

        // Logging functions
        linker
            .func_wrap(
                "env",
                "log",
                |mut caller: Caller<'_, StoreData>, level: i32, ptr: i32, len: i32| {
                    if let Err(e) = Self::host_log(&mut caller, level, ptr as u32, len as u32) {
                        tracing::error!("log error: {}", e);
                    }
                },
            )
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to register log: {}", e)))?;

        // Memory management functions
        linker
            .func_wrap("env", "allocate", |_caller: Caller<'_, StoreData>, size: i32| -> i32 {
                // This is a placeholder - actual allocation happens in WASM
                size
            })
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register allocate: {}", e))
            })?;

        linker
            .func_wrap(
                "env",
                "deallocate",
                |_caller: Caller<'_, StoreData>, _ptr: i32, _size: i32| {
                    // Placeholder for WASM-side deallocation
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register deallocate: {}", e))
            })?;

        // Database functions
        linker
            .func_wrap(
                "env",
                "db_query",
                |mut caller: Caller<'_, StoreData>,
                 query_ptr: i32,
                 query_len: i32,
                 params_ptr: i32,
                 params_len: i32|
                 -> i32 {
                    match Self::host_db_query(
                        &mut caller,
                        query_ptr as u32,
                        query_len as u32,
                        params_ptr as u32,
                        params_len as u32,
                    ) {
                        Ok(ptr) => ptr as i32,
                        Err(e) => {
                            tracing::error!("db_query error: {}", e);
                            0
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register db_query: {}", e))
            })?;

        linker
            .func_wrap(
                "env",
                "db_execute",
                |mut caller: Caller<'_, StoreData>,
                 query_ptr: i32,
                 query_len: i32,
                 params_ptr: i32,
                 params_len: i32|
                 -> i32 {
                    match Self::host_db_execute(
                        &mut caller,
                        query_ptr as u32,
                        query_len as u32,
                        params_ptr as u32,
                        params_len as u32,
                    ) {
                        Ok(rows) => rows as i32,
                        Err(e) => {
                            tracing::error!("db_execute error: {}", e);
                            -1
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register db_execute: {}", e))
            })?;

        // HTTP functions
        linker
            .func_wrap(
                "env",
                "http_request",
                |mut caller: Caller<'_, StoreData>,
                 method_ptr: i32,
                 method_len: i32,
                 url_ptr: i32,
                 url_len: i32,
                 headers_ptr: i32,
                 headers_len: i32,
                 body_ptr: i32,
                 body_len: i32|
                 -> i32 {
                    match Self::host_http_request(
                        &mut caller,
                        method_ptr as u32,
                        method_len as u32,
                        url_ptr as u32,
                        url_len as u32,
                        headers_ptr as u32,
                        headers_len as u32,
                        body_ptr as u32,
                        body_len as u32,
                    ) {
                        Ok(ptr) => ptr as i32,
                        Err(e) => {
                            tracing::error!("http_request error: {}", e);
                            0
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register http_request: {}", e))
            })?;

        // Event functions
        linker
            .func_wrap(
                "env",
                "emit_event",
                |mut caller: Caller<'_, StoreData>,
                 event_ptr: i32,
                 event_len: i32,
                 payload_ptr: i32,
                 payload_len: i32|
                 -> i32 {
                    match Self::host_emit_event(
                        &mut caller,
                        event_ptr as u32,
                        event_len as u32,
                        payload_ptr as u32,
                        payload_len as u32,
                    ) {
                        Ok(()) => 1,
                        Err(e) => {
                            tracing::error!("emit_event error: {}", e);
                            0
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register emit_event: {}", e))
            })?;

        // Config functions
        linker
            .func_wrap(
                "env",
                "get_config",
                |mut caller: Caller<'_, StoreData>, key_ptr: i32, key_len: i32| -> i32 {
                    match Self::host_get_config(&mut caller, key_ptr as u32, key_len as u32) {
                        Ok(ptr) => ptr as i32,
                        Err(e) => {
                            tracing::error!("get_config error: {}", e);
                            0
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register get_config: {}", e))
            })?;

        // Crypto functions
        linker
            .func_wrap(
                "env",
                "crypto_hash",
                |mut caller: Caller<'_, StoreData>,
                 algorithm: i32,
                 data_ptr: i32,
                 data_len: i32|
                 -> i32 {
                    match Self::host_crypto_hash(
                        &mut caller,
                        algorithm,
                        data_ptr as u32,
                        data_len as u32,
                    ) {
                        Ok(ptr) => ptr as i32,
                        Err(e) => {
                            tracing::error!("crypto_hash error: {}", e);
                            0
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register crypto_hash: {}", e))
            })?;

        linker
            .func_wrap(
                "env",
                "crypto_random",
                |mut caller: Caller<'_, StoreData>, len: i32| -> i32 {
                    match Self::host_crypto_random(&mut caller, len as u32) {
                        Ok(ptr) => ptr as i32,
                        Err(e) => {
                            tracing::error!("crypto_random error: {}", e);
                            0
                        }
                    }
                },
            )
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to register crypto_random: {}", e))
            })?;

        Ok(())
    }

    /// Host function: Get state value
    fn host_state_get(
        caller: &mut Caller<'_, StoreData>,
        key_ptr: u32,
        key_len: u32,
    ) -> orbis_core::Result<u32> {
        caller.data_mut().check_limits()?;

        let memory = Self::get_memory(caller)?;
        let key_bytes = Self::read_memory(caller, &memory, key_ptr, key_len)?;
        let key = String::from_utf8(key_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in state key: {}", e))
        })?;

        let value = caller.data().state.get(&key);

        if let Some(val) = value {
            let val_bytes = serde_json::to_vec(&val).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to serialize state value: {}", e))
            })?;

            let (ptr, _) = Self::allocate_and_write_bytes(caller, &val_bytes)?;
            Ok(ptr)
        } else {
            Ok(0) // Null pointer for missing key
        }
    }

    /// Host function: Set state value
    fn host_state_set(
        caller: &mut Caller<'_, StoreData>,
        key_ptr: u32,
        key_len: u32,
        value_ptr: u32,
        value_len: u32,
    ) -> orbis_core::Result<()> {
        caller.data_mut().check_limits()?;

        let memory = Self::get_memory(caller)?;
        let key_bytes = Self::read_memory(caller, &memory, key_ptr, key_len)?;
        let value_bytes = Self::read_memory(caller, &memory, value_ptr, value_len)?;

        let key = String::from_utf8(key_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in state key: {}", e))
        })?;

        let value: serde_json::Value = serde_json::from_slice(&value_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to parse state value: {}", e))
        })?;

        caller.data().state.set(key, value);
        Ok(())
    }

    /// Host function: Remove state value
    fn host_state_remove(
        caller: &mut Caller<'_, StoreData>,
        key_ptr: u32,
        key_len: u32,
    ) -> orbis_core::Result<()> {
        caller.data_mut().check_limits()?;

        let memory = Self::get_memory(caller)?;
        let key_bytes = Self::read_memory(caller, &memory, key_ptr, key_len)?;
        let key = String::from_utf8(key_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in state key: {}", e))
        })?;

        caller.data().state.remove(&key);
        Ok(())
    }

    /// Host function: Log message
    fn host_log(
        caller: &mut Caller<'_, StoreData>,
        level: i32,
        ptr: u32,
        len: u32,
    ) -> orbis_core::Result<()> {
        caller.data_mut().check_limits()?;

        let memory = Self::get_memory(caller)?;
        let msg_bytes = Self::read_memory(caller, &memory, ptr, len)?;
        let msg = String::from_utf8_lossy(&msg_bytes);

        let plugin_name = &caller.data().plugin_name;

        match level {
            0 => tracing::error!("[Plugin: {}] {}", plugin_name, msg),
            1 => tracing::warn!("[Plugin: {}] {}", plugin_name, msg),
            2 => tracing::info!("[Plugin: {}] {}", plugin_name, msg),
            3 => tracing::debug!("[Plugin: {}] {}", plugin_name, msg),
            _ => tracing::trace!("[Plugin: {}] {}", plugin_name, msg),
        }

        Ok(())
    }

    /// Host function: Query database
    fn host_db_query(
        caller: &mut Caller<'_, StoreData>,
        query_ptr: u32,
        query_len: u32,
        params_ptr: u32,
        params_len: u32,
    ) -> orbis_core::Result<u32> {
        caller.data_mut().check_limits()?;

        // Check permission
        if !caller.data().sandbox.has_permission("database:read") {
            return Err(orbis_core::Error::plugin(
                "Plugin does not have database:read permission",
            ));
        }

        let memory = Self::get_memory(caller)?;
        let query_bytes = Self::read_memory(caller, &memory, query_ptr, query_len)?;
        let _query = String::from_utf8(query_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in query: {}", e))
        })?;

        let params_bytes = Self::read_memory(caller, &memory, params_ptr, params_len)?;
        let _params: Vec<serde_json::Value> = serde_json::from_slice(&params_bytes)
            .map_err(|e| orbis_core::Error::plugin(format!("Invalid params JSON: {}", e)))?;

        // TODO: Actually execute query against database
        // For now, return empty result set as placeholder
        let result: Vec<serde_json::Value> = vec![];
        let result_bytes = serde_json::to_vec(&result).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to serialize result: {}", e))
        })?;

        let (ptr, _) = Self::allocate_and_write_bytes(caller, &result_bytes)?;
        Ok(ptr)
    }

    /// Host function: Execute database statement
    fn host_db_execute(
        caller: &mut Caller<'_, StoreData>,
        query_ptr: u32,
        query_len: u32,
        params_ptr: u32,
        params_len: u32,
    ) -> orbis_core::Result<u64> {
        caller.data_mut().check_limits()?;

        // Check permission
        if !caller.data().sandbox.has_permission("database:write") {
            return Err(orbis_core::Error::plugin(
                "Plugin does not have database:write permission",
            ));
        }

        let memory = Self::get_memory(caller)?;
        let query_bytes = Self::read_memory(caller, &memory, query_ptr, query_len)?;
        let _query = String::from_utf8(query_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in query: {}", e))
        })?;

        let params_bytes = Self::read_memory(caller, &memory, params_ptr, params_len)?;
        let _params: Vec<serde_json::Value> = serde_json::from_slice(&params_bytes)
            .map_err(|e| orbis_core::Error::plugin(format!("Invalid params JSON: {}", e)))?;

        // TODO: Actually execute statement against database
        // For now, return 0 rows affected as placeholder
        Ok(0)
    }

    /// Host function: Make HTTP request
    fn host_http_request(
        caller: &mut Caller<'_, StoreData>,
        method_ptr: u32,
        method_len: u32,
        url_ptr: u32,
        url_len: u32,
        headers_ptr: u32,
        headers_len: u32,
        body_ptr: u32,
        body_len: u32,
    ) -> orbis_core::Result<u32> {
        caller.data_mut().check_limits()?;

        // Check permission
        if !caller.data().sandbox.has_permission("network:http") {
            return Err(orbis_core::Error::plugin(
                "Plugin does not have network:http permission",
            ));
        }

        let memory = Self::get_memory(caller)?;

        let method_bytes = Self::read_memory(caller, &memory, method_ptr, method_len)?;
        let _method = String::from_utf8(method_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in method: {}", e))
        })?;

        let url_bytes = Self::read_memory(caller, &memory, url_ptr, url_len)?;
        let url = String::from_utf8(url_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in URL: {}", e))
        })?;

        // Check if URL host is allowed
        if let Ok(parsed_url) = url::Url::parse(&url) {
            if let Some(host) = parsed_url.host_str() {
                if !caller.data().sandbox.can_access_network(host) {
                    return Err(orbis_core::Error::plugin(format!(
                        "Plugin is not allowed to access host: {}",
                        host
                    )));
                }
            }
        }

        let headers_bytes = Self::read_memory(caller, &memory, headers_ptr, headers_len)?;
        let _headers: HashMap<String, String> = serde_json::from_slice(&headers_bytes)
            .map_err(|e| orbis_core::Error::plugin(format!("Invalid headers JSON: {}", e)))?;

        let _body_bytes = Self::read_memory(caller, &memory, body_ptr, body_len)?;

        // TODO: Actually make HTTP request
        // For now, return mock response
        let response = serde_json::json!({
            "status": 501,
            "headers": {},
            "body": "HTTP requests not yet implemented"
        });
        let response_bytes = serde_json::to_vec(&response).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to serialize response: {}", e))
        })?;

        let (ptr, _) = Self::allocate_and_write_bytes(caller, &response_bytes)?;
        Ok(ptr)
    }

    /// Host function: Emit event
    fn host_emit_event(
        caller: &mut Caller<'_, StoreData>,
        event_ptr: u32,
        event_len: u32,
        payload_ptr: u32,
        payload_len: u32,
    ) -> orbis_core::Result<()> {
        caller.data_mut().check_limits()?;

        // Check permission
        if !caller.data().sandbox.has_permission("events:emit") {
            return Err(orbis_core::Error::plugin(
                "Plugin does not have events:emit permission",
            ));
        }

        let memory = Self::get_memory(caller)?;

        let event_bytes = Self::read_memory(caller, &memory, event_ptr, event_len)?;
        let event_name = String::from_utf8(event_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in event name: {}", e))
        })?;

        let payload_bytes = Self::read_memory(caller, &memory, payload_ptr, payload_len)?;
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
            .map_err(|e| orbis_core::Error::plugin(format!("Invalid payload JSON: {}", e)))?;

        let plugin_name = caller.data().plugin_name.clone();
        tracing::info!(
            "[Plugin: {}] Emitting event '{}' with payload: {:?}",
            plugin_name,
            event_name,
            payload
        );

        // TODO: Actually emit event to event system
        Ok(())
    }

    /// Host function: Get config value
    fn host_get_config(
        caller: &mut Caller<'_, StoreData>,
        key_ptr: u32,
        key_len: u32,
    ) -> orbis_core::Result<u32> {
        caller.data_mut().check_limits()?;

        let memory = Self::get_memory(caller)?;
        let key_bytes = Self::read_memory(caller, &memory, key_ptr, key_len)?;
        let key = String::from_utf8(key_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid UTF-8 in config key: {}", e))
        })?;

        let value = caller.data().config.get(&key);

        if let Some(val) = value {
            let val_bytes = serde_json::to_vec(&val).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to serialize config value: {}", e))
            })?;

            let (ptr, _) = Self::allocate_and_write_bytes(caller, &val_bytes)?;
            Ok(ptr)
        } else {
            Ok(0) // Null pointer for missing key
        }
    }

    /// Host function: Hash data
    fn host_crypto_hash(
        caller: &mut Caller<'_, StoreData>,
        algorithm: i32,
        data_ptr: u32,
        data_len: u32,
    ) -> orbis_core::Result<u32> {
        caller.data_mut().check_limits()?;

        let memory = Self::get_memory(caller)?;
        let data = Self::read_memory(caller, &memory, data_ptr, data_len)?;

        use sha2::{Digest, Sha256, Sha512};

        let hash: Vec<u8> = match algorithm {
            0 => {
                // SHA-256
                let mut hasher = Sha256::new();
                hasher.update(&data);
                hasher.finalize().to_vec()
            }
            1 => {
                // SHA-512
                let mut hasher = Sha512::new();
                hasher.update(&data);
                hasher.finalize().to_vec()
            }
            _ => {
                return Err(orbis_core::Error::plugin(format!(
                    "Unknown hash algorithm: {}",
                    algorithm
                )));
            }
        };

        let (ptr, _) = Self::allocate_and_write_bytes(caller, &hash)?;
        Ok(ptr)
    }

    /// Host function: Generate random bytes
    fn host_crypto_random(caller: &mut Caller<'_, StoreData>, len: u32) -> orbis_core::Result<u32> {
        caller.data_mut().check_limits()?;

        if len > 1024 * 1024 {
            return Err(orbis_core::Error::plugin(format!(
                "Random bytes request too large: {} bytes",
                len
            )));
        }

        use rand::RngCore;
        let mut bytes = vec![0u8; len as usize];
        rand::rng().fill_bytes(&mut bytes);

        let (ptr, _) = Self::allocate_and_write_bytes(caller, &bytes)?;
        Ok(ptr)
    }

    /// Get memory from caller
    fn get_memory(caller: &mut Caller<'_, StoreData>) -> orbis_core::Result<Memory> {
        caller
            .get_export("memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| orbis_core::Error::plugin("Memory export not found"))
    }

    /// Read memory from WASM
    fn read_memory(
        caller: &mut Caller<'_, StoreData>,
        memory: &Memory,
        ptr: u32,
        len: u32,
    ) -> orbis_core::Result<Vec<u8>> {
        if len as usize > MAX_ALLOCATION_SIZE {
            return Err(orbis_core::Error::plugin(format!(
                "Memory read too large: {} bytes",
                len
            )));
        }

        let mut buffer = vec![0u8; len as usize];
        memory
            .read(caller, ptr as usize, &mut buffer)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to read memory: {}", e)))?;

        Ok(buffer)
    }

    /// Allocate memory in WASM and write data
    fn allocate_and_write(
        store: &mut Store<StoreData>,
        memory: &Memory,
        instance: &Instance,
        data: &[u8],
    ) -> orbis_core::Result<(u32, u32)> {
        let len = data.len() as u32;

        if data.len() > MAX_ALLOCATION_SIZE {
            return Err(orbis_core::Error::plugin(format!(
                "Allocation too large: {} bytes",
                data.len()
            )));
        }

        // Try to get allocate function from WASM
        let alloc_func = instance.get_func(&mut *store, "allocate").ok_or_else(|| {
            orbis_core::Error::plugin("allocate function not found in WASM module")
        })?;

        let alloc_typed: TypedFunc<i32, i32> = alloc_func.typed(&*store).map_err(|e| {
            orbis_core::Error::plugin(format!("allocate function has wrong signature: {}", e))
        })?;

        // Attempt allocation and capture any traps with additional diagnostics
        let ptr = match alloc_typed.call(&mut *store, len as i32) {
            Ok(p) => p as u32,
            Err(e) => {
                // Gather memory/sandbox diagnostics
                let pages = memory.size(&*store);
                let mem_bytes = (pages as usize) * 65536usize;
                let sandbox_limit = store.data().sandbox.memory_limit;
                let plugin_name = store.data().plugin_name.clone();

                tracing::error!(
                    plugin = %plugin_name,
                    requested = %len,
                    memory_pages = %pages,
                    memory_bytes = %mem_bytes,
                    sandbox_limit = %sandbox_limit,
                    "WASM allocation trap: {}",
                    e
                );

                // If requested allocation exceeds sandbox limit, return a clearer error
                if (len as usize) > sandbox_limit {
                    return Err(orbis_core::Error::plugin(format!(
                        "Failed to allocate memory: requested {} bytes exceeds sandbox limit {}",
                        len, sandbox_limit
                    )));
                }

                // Try to grow memory as a fallback and retry allocation several times
                let mut pages_needed = ((len as usize) + 65535) / 65536; // round up
                if pages_needed == 0 {
                    pages_needed = 1;
                }

                tracing::debug!(plugin = %plugin_name, requested = %len, pages_needed = %pages_needed, "Attempting to grow WASM memory as fallback");

                // Compute how many pages we can still add under sandbox limit
                let mut attempts = 0u8;
                let mut pages_to_add = pages_needed;
                loop {
                    if attempts >= 4 {
                        tracing::warn!(plugin = %plugin_name, "Reached maximum memory grow attempts (4)");
                        break;
                    }

                    // Check sandbox limit in pages
                    let mem_pages = memory.size(&*store) as usize;
                    let mem_bytes = mem_pages * 65536usize;
                    let remaining = if sandbox_limit > mem_bytes { sandbox_limit - mem_bytes } else { 0 };
                    let max_addable = if remaining == 0 { 0 } else { (remaining + 65535) / 65536 };

                    if max_addable == 0 {
                        tracing::warn!(plugin = %plugin_name, "Cannot grow memory, sandbox limit reached ({} bytes)", sandbox_limit);
                        break;
                    }

                    let add = std::cmp::min(pages_to_add, max_addable);

                    match memory.grow(&mut *store, add as u64) {
                        Ok(old_pages) => {
                            tracing::info!(plugin = %plugin_name, old_pages = %old_pages, pages_added = %add, "Memory grow succeeded, retrying allocation");
                            // Retry allocation
                            match alloc_typed.call(&mut *store, len as i32) {
                                Ok(p) => return Ok((p as u32, len)),
                                Err(e2) => {
                                    tracing::error!(plugin = %plugin_name, "Allocation after grow attempt failed: {}", e2);

                                    // Dump useful globals for debugging
                                    let heap_base = instance
                                        .get_global(&mut *store, "__heap_base")
                                        .and_then(|g: wasmtime::Global| g.get(&mut *store).i32())
                                        .unwrap_or(-1);
                                    let data_end = instance
                                        .get_global(&mut *store, "__data_end")
                                        .and_then(|g: wasmtime::Global| g.get(&mut *store).i32())
                                        .unwrap_or(-1);
                                    let current_pages = memory.size(&*store);

                                    tracing::error!(
                                        plugin = %plugin_name,
                                        requested = %len,
                                        heap_base = %heap_base,
                                        data_end = %data_end,
                                        memory_pages = %current_pages,
                                        memory_bytes = %((current_pages as usize) * 65536usize),
                                        "Allocation failed after grow with diagnostics"
                                    );

                                    // Prepare for next attempt: try to grow more (exponential)
                                    attempts += 1;
                                    pages_to_add = pages_to_add.saturating_mul(2);
                                    continue;
                                }
                            }
                        }
                        Err(grow_err) => {
                            tracing::warn!(plugin = %plugin_name, "Memory grow failed: {}", grow_err);
                            break;
                        }
                    }
                }

                // If we get here, all grow+retry attempts failed
                return Err(orbis_core::Error::plugin(format!(
                    "Failed to allocate memory after grow attempts: {}",
                    e
                )));
            }
        };

        // Write data to allocated memory
        memory
            .write(&mut *store, ptr as usize, data)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to write to memory: {}", e)))?;

        Ok((ptr, len))
    }

    /// Allocate and write bytes (used by host functions)
    fn allocate_and_write_bytes(
        caller: &mut Caller<'_, StoreData>,
        data: &[u8],
    ) -> orbis_core::Result<(u32, u32)> {
        let data_len = data.len() as u32;

        if data.len() > MAX_ALLOCATION_SIZE {
            return Err(orbis_core::Error::plugin(format!(
                "Allocation too large: {} bytes",
                data.len()
            )));
        }

        let memory = Self::get_memory(caller)?;

        // Get allocate function
        let alloc_func = caller
            .get_export("allocate")
            .and_then(|e| e.into_func())
            .ok_or_else(|| {
                orbis_core::Error::plugin("allocate function not found in WASM module")
            })?;

        // Allocate space for length prefix (4 bytes) + data
        let total_size = 4 + data_len;
        let mut results = vec![Val::I32(0)];
        alloc_func
            .call(caller.as_context_mut(), &[Val::I32(total_size as i32)], &mut results)
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to call allocate: {}", e))
            })?;

        let ptr = match results[0] {
            Val::I32(p) => p as u32,
            _ => {
                return Err(orbis_core::Error::plugin(
                    "allocate returned wrong type".to_string(),
                ))
            }
        };

        // Write length prefix (little-endian u32)
        let len_bytes = data_len.to_le_bytes();
        memory
            .write( caller.as_context_mut(), ptr as usize, &len_bytes)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to write length prefix: {}", e)))?;

        // Write data after the length prefix
        memory
            .write(caller.as_context_mut(), (ptr + 4) as usize, data)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to write data: {}", e)))?;

        Ok((ptr, total_size))
    }

    /// Deallocate memory in WASM
    fn deallocate(
        store: &mut Store<StoreData>,
        instance: &Instance,
        ptr: u32,
        len: u32,
    ) -> orbis_core::Result<()> {
        // Try to get deallocate function from WASM
        if let Some(dealloc_func) = instance.get_func(&mut *store, "deallocate") {
            let dealloc_typed: TypedFunc<(i32, i32), ()> = dealloc_func.typed(&*store).map_err(|e| {
                orbis_core::Error::plugin(format!("deallocate function has wrong signature: {}", e))
            })?;

            dealloc_typed
                .call(&mut *store, (ptr as i32, len as i32))
                .map_err(|e| {
                    orbis_core::Error::plugin(format!("Failed to deallocate memory: {}", e))
                })?;
        }

        Ok(())
    }

    /// Read result from WASM memory
    fn read_result(
        store: &mut Store<StoreData>,
        memory: &Memory,
        ptr: u32,
    ) -> orbis_core::Result<serde_json::Value> {
        if ptr == 0 {
            return Ok(serde_json::Value::Null);
        }

        // First 4 bytes are the length
        let mut len_bytes = [0u8; 4];
        memory
            .read(&mut *store, ptr as usize, &mut len_bytes)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to read result length: {}", e)))?;

        let len = u32::from_le_bytes(len_bytes);

        if len as usize > MAX_ALLOCATION_SIZE {
            return Err(orbis_core::Error::plugin(format!(
                "Result too large: {} bytes",
                len
            )));
        }

        // Read the actual data
        let mut data = vec![0u8; len as usize];
        memory
            .read(&mut *store, (ptr + 4) as usize, &mut data)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to read result data: {}", e)))?;

        let result: serde_json::Value = serde_json::from_slice(&data)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to parse result: {}", e)))?;

        Ok(result)
    }
}

impl Default for PluginRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_state() {
        let state = PluginState::new();

        state.set("key1".to_string(), serde_json::json!("value1"));
        state.set("key2".to_string(), serde_json::json!(42));
        state.set(
            "key3".to_string(),
            serde_json::json!({"nested": "object"}),
        );

        assert_eq!(state.get("key1"), Some(serde_json::json!("value1")));
        assert_eq!(state.get("key2"), Some(serde_json::json!(42)));
        assert_eq!(
            state.get("key3"),
            Some(serde_json::json!({"nested": "object"}))
        );
        assert_eq!(state.get("nonexistent"), None);

        state.remove("key2");
        assert_eq!(state.get("key2"), None);

        let keys = state.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key3".to_string()));

        state.clear();
        assert_eq!(state.keys().len(), 0);
    }

    #[test]
    fn test_store_data_limits() {
        let sandbox = Arc::new(SandboxConfig {
            max_calls: 10,
            time_limit_ms: 100,
            ..SandboxConfig::minimal()
        });

        let state = PluginState::new();
        let config = PluginConfig::new();
        let mut store_data = StoreData::new("test".to_string(), sandbox, state, config);

        // Should succeed for first 10 calls
        for _ in 0..10 {
            assert!(store_data.check_limits().is_ok());
        }

        // Should fail on 11th call
        assert!(store_data.check_limits().is_err());
    }

    #[test]
    fn test_allocate_via_runtime() {
        // Load wasm
        let bytes = std::fs::read("../../plugins/my-first-plugin/plugin.wasm").expect("wasm not found");

        // Create engine and module
        let engine = Engine::default();
        let module = Module::new(&engine, &bytes).expect("compile module");

        // Create store with StoreData
        let sandbox = Arc::new(SandboxConfig::minimal());
        let state = PluginState::new();
        let config = PluginConfig::new();
        let store_data = StoreData::new("my-first-plugin".to_string(), sandbox, state, config);

        let mut store = Store::new(&engine, store_data);
        store.limiter(|data| &mut data.limits);
        // Set fuel (use set_fuel same as runtime.execute)
        let _ = store.set_fuel(1_000_000).ok();

        // Create linker and register host functions
        let mut linker = Linker::new(&engine);
        PluginRuntime::register_host_functions(&mut linker).expect("register hosts");

        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module).expect("instantiate");

        let memory = instance
            .get_memory(&mut store, "memory")
            .expect("memory not found");

        // Prepare context JSON (small)
        let context = PluginContext {
            method: "POST".to_string(),
            path: "/greeting".to_string(),
            headers: std::collections::HashMap::new(),
            query: std::collections::HashMap::new(),
            body: serde_json::json!({"name": "Test"}),
            user_id: None,
            is_admin: false,
        };

        let data = serde_json::to_vec(&context).expect("serialize");

        // Call allocate_and_write which mirrors runtime flow
        let res = PluginRuntime::allocate_and_write(&mut store, &memory, &instance, &data);

        match res {
            Ok((ptr, len)) => {
                assert!(ptr != 0);
                assert_eq!(len as usize, data.len());
            }
            Err(e) => panic!("allocate_and_write failed: {}", e),
        }
    }

    #[test]
    fn test_allocate_minimal() {
        // Test with minimal setup - no host functions, no fuel, no limits
        let bytes = std::fs::read("../../plugins/test-plugin/test_plugin.wasm")
            .or_else(|_| std::fs::read("../../target/wasm32-unknown-unknown/release/test_plugin.wasm"))
            .expect("test_plugin.wasm not found");

        // Create minimal engine config
        let mut config = wasmtime::Config::new();
        config.consume_fuel(false); // Disable fuel consumption
        let engine = Engine::new(&config).expect("create engine");
        let module = Module::new(&engine, &bytes).expect("compile module");

        // Create simple store with unit type (no StoreData)
        let mut store: Store<()> = Store::new(&engine, ());

        // Create linker and add minimal required imports
        let mut linker: Linker<()> = Linker::new(&engine);

        // Add required imports based on WASM module
        linker.func_wrap("env", "state_get", |_caller: wasmtime::Caller<'_, ()>, _key_ptr: i32, _key_len: i32| -> i32 {
            // Return 0 = no value found
            0i32
        }).expect("wrap state_get");
        
        linker.func_wrap("env", "state_set", |_caller: wasmtime::Caller<'_, ()>, _key_ptr: i32, _key_len: i32, _val_ptr: i32, _val_len: i32| -> i32 {
            // Return 0 = success
            0i32
        }).expect("wrap state_set");

        // Instantiate
        let instance = linker.instantiate(&mut store, &module).expect("instantiate");

        // Check memory info
        let memory = instance.get_memory(&mut store, "memory").expect("get memory");
        let pages = memory.size(&store);
        println!("Memory: {} pages = {} bytes", pages, pages * 65536);

        // Check heap_base
        if let Some(g) = instance.get_global(&mut store, "__heap_base") {
            let hb = g.get(&mut store).i32().unwrap_or(-1);
            println!("__heap_base = {}", hb);
            let available = (pages as i32 * 65536) - hb;
            println!("Available heap: {} bytes", available);
        }

        // Get allocate function
        let allocate: wasmtime::TypedFunc<i32, i32> = instance
            .get_typed_func(&mut store, "allocate")
            .expect("get allocate");

        // Try allocation with small size
        println!("Calling allocate(16)...");
        match allocate.call(&mut store, 16) {
            Ok(ptr) => {
                println!("SUCCESS! Allocated 16 bytes at ptr = {}", ptr);
                assert!(ptr > 0, "Pointer should be non-zero");
            }
            Err(e) => {
                panic!("allocate(16) FAILED: {}", e);
            }
        }
    }

    #[test]
    fn test_allocate_with_fuel() {
        // Same test but with fuel enabled
        let bytes = std::fs::read("../../plugins/test-plugin/test_plugin.wasm")
            .or_else(|_| std::fs::read("../../target/wasm32-unknown-unknown/release/test_plugin.wasm"))
            .expect("test_plugin.wasm not found");

        let mut config = wasmtime::Config::new();
        config.consume_fuel(true); // Enable fuel
        let engine = Engine::new(&config).expect("create engine");
        let module = Module::new(&engine, &bytes).expect("compile module");

        let mut store: Store<()> = Store::new(&engine, ());
        store.set_fuel(10_000_000).expect("set fuel"); // Lots of fuel

        let mut linker: Linker<()> = Linker::new(&engine);

        linker.func_wrap("env", "state_get", |_caller: wasmtime::Caller<'_, ()>, _key_ptr: i32, _key_len: i32| -> i32 {
            0i32
        }).expect("wrap state_get");
        
        linker.func_wrap("env", "state_set", |_caller: wasmtime::Caller<'_, ()>, _key_ptr: i32, _key_len: i32, _val_ptr: i32, _val_len: i32| -> i32 {
            0i32
        }).expect("wrap state_set");

        let instance = linker.instantiate(&mut store, &module).expect("instantiate");

        let allocate: wasmtime::TypedFunc<i32, i32> = instance
            .get_typed_func(&mut store, "allocate")
            .expect("get allocate");

        println!("[WITH FUEL] Calling allocate(16)...");
        match allocate.call(&mut store, 16) {
            Ok(ptr) => {
                println!("[WITH FUEL] SUCCESS! Allocated 16 bytes at ptr = {}", ptr);
                assert!(ptr > 0);
            }
            Err(e) => {
                panic!("[WITH FUEL] allocate(16) FAILED: {}", e);
            }
        }
    }

    #[test]
    fn test_allocate_with_limits() {
        // Same test but with ResourceLimiter enabled
        let bytes = std::fs::read("../../plugins/test-plugin/test_plugin.wasm")
            .or_else(|_| std::fs::read("../../target/wasm32-unknown-unknown/release/test_plugin.wasm"))
            .expect("test_plugin.wasm not found");

        let mut config = wasmtime::Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config).expect("create engine");
        let module = Module::new(&engine, &bytes).expect("compile module");

        // Use a limiter that limits memory to 16MB
        let limits = wasmtime::StoreLimitsBuilder::new()
            .memory_size(16 * 1024 * 1024) // 16MB limit like sandbox
            .build();

        let mut store: Store<wasmtime::StoreLimits> = Store::new(&engine, limits);
        store.limiter(|s| s);
        store.set_fuel(10_000_000).expect("set fuel");

        let mut linker: Linker<wasmtime::StoreLimits> = Linker::new(&engine);

        linker.func_wrap("env", "state_get", |_caller: wasmtime::Caller<'_, wasmtime::StoreLimits>, _key_ptr: i32, _key_len: i32| -> i32 {
            0i32
        }).expect("wrap state_get");
        
        linker.func_wrap("env", "state_set", |_caller: wasmtime::Caller<'_, wasmtime::StoreLimits>, _key_ptr: i32, _key_len: i32, _val_ptr: i32, _val_len: i32| -> i32 {
            0i32
        }).expect("wrap state_set");

        let instance = linker.instantiate(&mut store, &module).expect("instantiate");

        let allocate: wasmtime::TypedFunc<i32, i32> = instance
            .get_typed_func(&mut store, "allocate")
            .expect("get allocate");

        // Test with 16 bytes (small)
        println!("[WITH LIMITS] Calling allocate(16)...");
        match allocate.call(&mut store, 16) {
            Ok(ptr) => {
                println!("[WITH LIMITS] SUCCESS! Allocated 16 bytes at ptr = {}", ptr);
                assert!(ptr > 0);
            }
            Err(e) => {
                panic!("[WITH LIMITS] allocate(16) FAILED: {}", e);
            }
        }

        // Test with 116 bytes (same as integration test context)
        println!("[WITH LIMITS] Calling allocate(116)...");
        match allocate.call(&mut store, 116) {
            Ok(ptr) => {
                println!("[WITH LIMITS] SUCCESS! Allocated 116 bytes at ptr = {}", ptr);
                assert!(ptr > 0);
            }
            Err(e) => {
                panic!("[WITH LIMITS] allocate(116) FAILED: {}", e);
            }
        }
    }

    #[test]
    fn test_allocate_exact_runtime_config() {
        // Test with EXACT PluginRuntime configuration
        let bytes = std::fs::read("../../plugins/test-plugin/test_plugin.wasm")
            .or_else(|_| std::fs::read("../../target/wasm32-unknown-unknown/release/test_plugin.wasm"))
            .expect("test_plugin.wasm not found");

        // Exactly match PluginRuntime::new() config
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true);
        // TEST: Enable epoch_interruption only
        config.epoch_interruption(true);
        // TEST: Comment out max_wasm_stack to see if that's the issue
        // config.max_wasm_stack(512 * 1024);
        let engine = Engine::new(&config).expect("create engine");
        let module = Module::new(&engine, &bytes).expect("compile module");

        // Match StoreData creation
        let sandbox = Arc::new(SandboxConfig::minimal());
        let state = PluginState::new();
        let plugin_config = PluginConfig::new();
        let store_data = StoreData::new("test-plugin".to_string(), sandbox.clone(), state, plugin_config);

        let mut store = Store::new(&engine, store_data);
        store.limiter(|data| &mut data.limits);
        
        // Set fuel exactly like execute() does
        let fuel = u64::from(sandbox.time_limit_ms) * 1000;
        println!("[EXACT CONFIG] Setting fuel to {}", fuel);
        store.set_fuel(fuel).expect("set fuel");

        let mut linker: Linker<StoreData> = Linker::new(&engine);
        PluginRuntime::register_host_functions(&mut linker).expect("register hosts");

        let instance = linker.instantiate(&mut store, &module).expect("instantiate");

        let allocate: wasmtime::TypedFunc<i32, i32> = instance
            .get_typed_func(&mut store, "allocate")
            .expect("get allocate");

        println!("[EXACT CONFIG] Calling allocate(116)...");
        match allocate.call(&mut store, 116) {
            Ok(ptr) => {
                println!("[EXACT CONFIG] SUCCESS! Allocated 116 bytes at ptr = {}", ptr);
                assert!(ptr > 0);
            }
            Err(e) => {
                panic!("[EXACT CONFIG] allocate(116) FAILED: {}", e);
            }
        }
    }
}
