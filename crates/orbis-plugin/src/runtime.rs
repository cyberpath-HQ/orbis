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
}

impl PluginState {
    /// Create a new plugin state
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
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
    }

    /// Remove a value from the state
    pub fn remove(&self, key: &str) -> Option<serde_json::Value> {
        self.data.write().remove(key)
    }

    /// Clear all state
    pub fn clear(&self) {
        self.data.write().clear();
    }

    /// Get all keys
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        self.data.read().keys().cloned().collect()
    }
}

/// Store data combining WASM state and host data
pub struct StoreData {
    /// Memory limits for the WASM instance
    limits: StoreLimits,
    /// Plugin state storage
    state: PluginState,
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
    fn new(plugin_name: String, sandbox: Arc<SandboxConfig>, state: PluginState) -> Self {
        let limits = StoreLimitsBuilder::new()
            .memory_size(sandbox.memory_limit)
            .build();

        Self {
            limits,
            state,
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
}

/// Plugin runtime for executing plugin code.
pub struct PluginRuntime {
    instances: DashMap<String, Arc<PluginInstance>>,
    engine: Engine,
}

impl PluginRuntime {
    /// Create a new plugin runtime.
    #[must_use]
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true); // Enable fuel consumption for execution limits
        config.epoch_interruption(true); // Enable epoch-based interruption
        config.max_wasm_stack(512 * 1024); // 512KB max stack

        let engine = Engine::new(&config).expect("Failed to create WASM engine");

        Self {
            instances: DashMap::new(),
            engine,
        }
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

        let state = PluginState::new();

        let instance = PluginInstance {
            engine: self.engine.clone(),
            module,
            sandbox_config: Arc::new(SandboxConfig::from_permissions(&info.manifest.permissions)),
            state,
        };

        self.instances
            .insert(info.manifest.name.clone(), Arc::new(instance));

        Ok(())
    }

    /// Start a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be started.
    pub async fn start(&self, name: &str) -> orbis_core::Result<()> {
        let instance = self.instances.get(name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not initialized", name))
        })?;

        // Create store for initialization
        let store_data =
            StoreData::new(name.to_string(), instance.sandbox_config.clone(), instance.state.clone());
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

        // Call init function if it exists
        if let Some(init_func) = wasm_instance.get_func(&mut store, "init") {
            let init_typed: TypedFunc<(), ()> = init_func.typed(&store).map_err(|e| {
                orbis_core::Error::plugin(format!("Init function has wrong signature: {}", e))
            })?;

            init_typed.call(&mut store, ()).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to execute init function: {}", e))
            })?;
        }

        tracing::debug!("Started plugin: {}", name);
        Ok(())
    }

    /// Stop a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be stopped.
    pub async fn stop(&self, name: &str) -> orbis_core::Result<()> {
        let instance = self.instances.get(name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not running", name))
        })?;

        // Create store for cleanup
        let store_data =
            StoreData::new(name.to_string(), instance.sandbox_config.clone(), instance.state.clone());
        let mut store = Store::new(&instance.engine, store_data);
        store.limiter(|data| &mut data.limits);

        // Add fuel for execution
        store
            .set_fuel(u64::from(instance.sandbox_config.time_limit_ms) * 1000)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to set fuel: {}", e)))?;

        // Create linker
        let mut linker = Linker::new(&instance.engine);
        Self::register_host_functions(&mut linker)?;

        // Instantiate the module
        let wasm_instance = linker
            .instantiate(&mut store, &instance.module)
            .map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to instantiate plugin: {}", e))
            })?;

        // Call cleanup function if it exists
        if let Some(cleanup_func) = wasm_instance.get_func(&mut store, "cleanup") {
            let cleanup_typed: TypedFunc<(), ()> = cleanup_func.typed(&store).map_err(|e| {
                orbis_core::Error::plugin(format!("Cleanup function has wrong signature: {}", e))
            })?;

            cleanup_typed.call(&mut store, ()).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to execute cleanup function: {}", e))
            })?;
        }

        // Clear plugin state
        instance.state.clear();

        self.instances.remove(name);
        tracing::debug!("Stopped plugin: {}", name);
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

        let ptr = alloc_typed.call(&mut *store, len as i32).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to allocate memory: {}", e))
        })? as u32;

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
        let len = data.len() as u32;

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

        let mut results = vec![Val::I32(0)];
        alloc_func
            .call(caller.as_context_mut(), &[Val::I32(len as i32)], &mut results)
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

        // Write data
        memory
            .write(caller, ptr as usize, data)
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to write to memory: {}", e)))?;

        Ok((ptr, len))
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
        let mut store_data = StoreData::new("test".to_string(), sandbox, state);

        // Should succeed for first 10 calls
        for _ in 0..10 {
            assert!(store_data.check_limits().is_ok());
        }

        // Should fail on 11th call
        assert!(store_data.check_limits().is_err());
    }
}
