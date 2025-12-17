//! Plugin runtime for executing plugin code.

use super::{PluginInfo, PluginSource, SandboxConfig};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(feature = "wasm")]
use wasmtime::{Engine, Linker, Module, Store};

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

/// Plugin runtime instance.
struct PluginInstance {
    #[cfg(feature = "wasm")]
    engine: Engine,
    #[cfg(feature = "wasm")]
    module: Module,
    #[allow(dead_code)]
    sandbox_config: SandboxConfig,
}

/// Plugin runtime for executing plugin code.
pub struct PluginRuntime {
    instances: DashMap<String, Arc<PluginInstance>>,
    #[cfg(feature = "wasm")]
    engine: Engine,
}

impl PluginRuntime {
    /// Create a new plugin runtime.
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: DashMap::new(),
            #[cfg(feature = "wasm")]
            engine: Engine::default(),
        }
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

        #[cfg(feature = "wasm")]
        {
            let module = Module::new(&self.engine, &code).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to compile WASM module: {}", e))
            })?;

            let instance = PluginInstance {
                engine: self.engine.clone(),
                module,
                sandbox_config: SandboxConfig::from_permissions(&info.manifest.permissions),
            };

            self.instances.insert(info.manifest.name.clone(), Arc::new(instance));
        }

        #[cfg(not(feature = "wasm"))]
        {
            let _ = code; // Suppress unused variable warning
            return Err(orbis_core::Error::plugin(
                "WASM runtime not enabled. Enable the 'wasm' feature.",
            ));
        }

        Ok(())
    }

    /// Start a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be started.
    pub async fn start(&self, name: &str) -> orbis_core::Result<()> {
        let _instance = self.instances.get(name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not initialized", name))
        })?;

        // Call plugin's init/start function if it exists
        tracing::debug!("Started plugin: {}", name);
        Ok(())
    }

    /// Stop a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be stopped.
    pub async fn stop(&self, name: &str) -> orbis_core::Result<()> {
        // Call plugin's cleanup function if it exists
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
        let _instance = self.instances.get(plugin_name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not running", plugin_name))
        })?;

        #[cfg(feature = "wasm")]
        {
            // Create a store for this execution
            let mut store = Store::new(&_instance.engine, ());

            // Create linker with host functions
            let linker = Linker::new(&_instance.engine);

            // Instantiate the module
            let instance = linker.instantiate(&mut store, &_instance.module).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to instantiate plugin: {}", e))
            })?;

            // Serialize context to JSON for passing to WASM
            let context_json = serde_json::to_string(&context).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to serialize context: {}", e))
            })?;

            // Get the handler function
            #[allow(unused_variables)]
            let func = instance
                .get_func(&mut store, handler)
                .ok_or_else(|| orbis_core::Error::plugin(format!("Handler '{}' not found", handler)))?;

            // For now, return a placeholder response
            // Full implementation would involve proper memory management for WASM
            tracing::debug!(
                "Executing handler '{}' in plugin '{}' with context: {}",
                handler,
                plugin_name,
                context_json
            );

            // Placeholder response
            Ok(serde_json::json!({
                "status": "ok",
                "plugin": plugin_name,
                "handler": handler
            }))
        }

        #[cfg(not(feature = "wasm"))]
        {
            let _ = handler;
            let _ = context;
            Err(orbis_core::Error::plugin(
                "WASM runtime not enabled. Enable the 'wasm' feature.",
            ))
        }
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
        // Remove any cached instance
        self.instances.remove(name);
        tracing::debug!("Cleared cache for plugin: {}", name);
    }
}

impl Default for PluginRuntime {
    fn default() -> Self {
        Self::new()
    }
}
