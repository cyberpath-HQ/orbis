//! # Orbis Plugin System
//!
//! Secure runtime plugin system for extending Orbis with WASM plugins.
//!
//! ## Plugin Flavors
//!
//! - **Packed**: ZIP file containing WASM, manifest.json, and assets
//! - **Unpacked**: Folder with WASM, manifest.json, and assets
//! - **Standalone**: Single WASM file with embedded manifest
//!
//! Note: Packed and unpacked plugins can also have manifests embedded in WASM.
//!
//! ## Features
//!
//! - Define custom API routes
//! - Define React pages via JSON GUI schema
//! - Access database through controlled API
//! - Secure WASM sandboxing

mod loader;
mod registry;
mod runtime;
mod sandbox;
mod watcher;

pub use loader::{PluginLoader, PluginSource};
pub use registry::{PluginInfo, PluginRegistry, PluginState};
pub use runtime::{PluginContext, PluginRuntime};
pub use sandbox::SandboxConfig;
pub use watcher::{PluginChangeEvent, PluginChangeKind, PluginWatcher, WatcherConfig};

// Re-export public API types from orbis-plugin-api
pub use orbis_plugin_api::{
    AccordionItem, Action, ArgMapping, BreadcrumbItem, ComponentSchema, CustomValidation,
    DialogDefinition, Error as PluginApiError, EventHandlers, FormField, NavigationConfig,
    NavigationItem, PageDefinition, PageLifecycleHooks, PluginDependency, PluginManifest,
    PluginPermission, PluginRoute, Result as PluginApiResult, SelectOption, StateFieldDefinition,
    StateFieldType, TabItem, TableColumn, ToastLevel, ValidationRule,
};

use orbis_db::Database;
use std::path::PathBuf;
use uuid::Uuid;

/// Plugin manager handling all plugin operations.
pub struct PluginManager {
    registry: PluginRegistry,
    loader: PluginLoader,
    runtime: PluginRuntime,
    plugins_dir: PathBuf,
    db: Database,
}

impl PluginManager {
    /// Create a new plugin manager.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub fn new(plugins_dir: PathBuf, db: Database) -> orbis_core::Result<Self> {
        // Ensure plugins directory exists
        if !plugins_dir.exists() {
            std::fs::create_dir_all(&plugins_dir).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to create plugins directory: {}", e))
            })?;
        }
        
        // State file in plugin directory
        let state_file = plugins_dir.join(".plugin_states.json");

        Ok(Self {
            registry: PluginRegistry::with_persistence(state_file),
            loader: PluginLoader::new(),
            runtime: PluginRuntime::new(),
            plugins_dir,
            db,
        })
    }

    /// Get the database instance for plugin operations.
    #[must_use]
    pub const fn database(&self) -> &Database {
        &self.db
    }

    /// Get the plugin registry.
    #[must_use]
    pub const fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// Get the plugin runtime.
    #[must_use]
    pub const fn runtime(&self) -> &PluginRuntime {
        &self.runtime
    }

    /// Load all plugins from the plugins directory.
    ///
    /// Scans for:
    /// - Unpacked: Directories containing manifest.json or plugin.wasm
    /// - Packed: .zip files
    /// - Standalone: .wasm files
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails.
    pub async fn load_all(&self) -> orbis_core::Result<Vec<PluginInfo>> {
        tracing::info!("Loading plugins from {:?}", self.plugins_dir);

        let mut loaded = Vec::new();

        let entries = std::fs::read_dir(&self.plugins_dir).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to read plugins directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            if path.is_dir() {
                // Unpacked plugin: directory containing manifest.json or plugin.wasm
                let has_manifest = path.join("manifest.json").exists();
                let has_wasm = path.join("plugin.wasm").exists();
                
                if has_manifest || has_wasm {
                    match self.load_plugin(&path).await {
                        Ok(info) => {
                            tracing::info!("Loaded unpacked plugin: {} v{}", info.manifest.name, info.manifest.version);
                            loaded.push(info);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load unpacked plugin from {:?}: {}", path, e);
                        }
                    }
                }
            } else if let Some(ext) = path.extension() {
                match ext.to_str() {
                    Some("wasm") => {
                        // Standalone plugin: single WASM file with embedded manifest
                        match self.load_plugin(&path).await {
                            Ok(info) => {
                                tracing::info!("Loaded standalone plugin: {} v{}", info.manifest.name, info.manifest.version);
                                loaded.push(info);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load standalone plugin from {:?}: {}", path, e);
                            }
                        }
                    }
                    Some("zip") => {
                        // Packed plugin: ZIP archive containing WASM, manifest, and assets
                        match self.load_plugin(&path).await {
                            Ok(info) => {
                                tracing::info!("Loaded packed plugin: {} v{}", info.manifest.name, info.manifest.version);
                                loaded.push(info);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load packed plugin from {:?}: {}", path, e);
                            }
                        }
                    }
                    _ => {
                        // Ignore other file types
                    }
                }
            }
        }

        tracing::info!("Loaded {} plugins", loaded.len());
        
        // Restore saved states (enabled/disabled) from previous session
        self.registry.restore_states()?;
        
        // Auto-start plugins that were previously running
        for plugin in &loaded {
            if plugin.state == PluginState::Running {
                tracing::info!("Auto-starting previously running plugin: {}", plugin.manifest.name);
                if let Err(e) = self.runtime.start(&plugin.manifest.name).await {
                    tracing::error!("Failed to auto-start plugin {}: {}", plugin.manifest.name, e);
                }
            }
        }
        
        Ok(loaded)
    }

    /// Load a single plugin from a path.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be loaded.
    pub async fn load_plugin(&self, path: &PathBuf) -> orbis_core::Result<PluginInfo> {
        let source = PluginSource::from_path(path)?;
        let manifest = self.loader.load_manifest(&source)?;

        // Validate manifest
        manifest.validate()?;

        // Check if plugin already exists
        if self.registry.get(&manifest.name).is_some() {
            return Err(orbis_core::Error::plugin(format!(
                "Plugin '{}' is already loaded",
                manifest.name
            )));
        }

        // Create plugin info
        let info = PluginInfo {
            id: Uuid::now_v7(),
            manifest: manifest.clone(),
            source: source.clone(),
            state: PluginState::Loaded,
            loaded_at: chrono::Utc::now(),
        };

        // Register the plugin
        self.registry.register(info.clone());

        // Initialize the plugin in the runtime
        self.runtime.initialize(&info, &source).await?;

        Ok(info)
    }

    /// Unload a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be unloaded.
    pub async fn unload_plugin(&self, name: &str) -> orbis_core::Result<()> {
        let info = self.registry.get(name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not found", name))
        })?;

        // Stop the plugin runtime
        self.runtime.stop(&info.manifest.name).await?;

        // Unregister the plugin
        self.registry.unregister(name);

        tracing::info!("Unloaded plugin: {}", name);
        Ok(())
    }

    /// Enable a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be enabled.
    pub async fn enable_plugin(&self, name: &str) -> orbis_core::Result<()> {
        // Check if plugin is already loaded
        if !self.runtime.is_running(name) {
            return Err(orbis_core::Error::plugin(format!(
                "Plugin '{}' is not loaded. Load the plugin first before enabling.",
                name
            )));
        }
        
        // Update state
        self.registry.set_state(name, PluginState::Running)?;
        
        // Call init function (lightweight - just calls WASM init)
        self.runtime.start(name).await?;
        
        tracing::info!("Enabled plugin: {}", name);
        Ok(())
    }

    /// Disable a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be disabled.
    pub async fn disable_plugin(&self, name: &str) -> orbis_core::Result<()> {
        // Call cleanup function
        self.runtime.stop(name).await?;
        
        // Update state
        self.registry.set_state(name, PluginState::Disabled)?;
        
        tracing::info!("Disabled plugin: {}", name);
        Ok(())
    }

    /// Reload a plugin (hot reload).
    ///
    /// Unloads the current version and reloads from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be reloaded.
    pub async fn reload_plugin(&self, name: &str) -> orbis_core::Result<PluginInfo> {
        // Get current plugin info to find the source path
        let old_info = self.registry.get(name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not found", name))
        })?;

        let source_path = match &old_info.source {
            PluginSource::Unpacked(p) | PluginSource::Standalone(p) | PluginSource::Packed(p) => {
                p.clone()
            }
            PluginSource::Remote(_) => {
                return Err(orbis_core::Error::plugin(
                    "Cannot reload remote plugins",
                ));
            }
        };

        tracing::info!("Hot reloading plugin: {}", name);

        // Stop the plugin runtime
        self.runtime.stop(name).await?;

        // Unregister the old version
        self.registry.unregister(name);

        // Clear runtime cache for this plugin
        self.runtime.clear_cache(name);

        // Load the new version
        let new_info = self.load_plugin(&source_path).await?;

        // Start the new version if it was running before
        if old_info.state == PluginState::Running {
            self.runtime.start(&new_info.manifest.name).await?;
            self.registry
                .set_state(&new_info.manifest.name, PluginState::Running)?;
        }

        tracing::info!(
            "Hot reload complete: {} v{}",
            new_info.manifest.name,
            new_info.manifest.version
        );

        Ok(new_info)
    }

    /// Reload a plugin by path (for file watcher events).
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be reloaded.
    pub async fn reload_plugin_by_path(&self, path: &PathBuf) -> orbis_core::Result<Option<PluginInfo>> {
        // Find plugin by path
        let plugin_name = self.registry.list().iter().find_map(|info| {
            let source_path = match &info.source {
                PluginSource::Unpacked(p) | PluginSource::Standalone(p) | PluginSource::Packed(p) => {
                    Some(p)
                }
                PluginSource::Remote(_) => None,
            };

            if let Some(sp) = source_path {
                // Check if the changed path is within or is the plugin path
                if path.starts_with(sp) || path == sp {
                    return Some(info.manifest.name.clone());
                }
            }
            None
        });

        if let Some(name) = plugin_name {
            let info = self.reload_plugin(&name).await?;
            Ok(Some(info))
        } else {
            // New plugin, try to load it
            if path.exists() {
                // Determine the plugin root path
                let plugin_root = Self::find_plugin_root(path);
                if let Some(root) = plugin_root {
                    match self.load_plugin(&root).await {
                        Ok(info) => {
                            tracing::info!("Loaded new plugin: {} v{}", info.manifest.name, info.manifest.version);
                            Ok(Some(info))
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load plugin from {:?}: {}", root, e);
                            Ok(None)
                        }
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
    }

    /// Find the plugin root directory from a file path.
    fn find_plugin_root(path: &PathBuf) -> Option<PathBuf> {
        // If it's a WASM or ZIP file, that's the root
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if matches!(ext, "wasm" | "zip") {
                return Some(path.clone());
            }
        }

        // For other files, look for parent directory with manifest.json or plugin.wasm
        let mut current = path.clone();
        while let Some(parent) = current.parent() {
            if parent.join("manifest.json").exists() || parent.join("plugin.wasm").exists() {
                return Some(parent.to_path_buf());
            }
            current = parent.to_path_buf();
        }

        None
    }

    /// Create a plugin watcher for hot reload.
    #[must_use]
    pub fn create_watcher(&self) -> PluginWatcher {
        PluginWatcher::new(WatcherConfig {
            watch_dir: self.plugins_dir.clone(),
            debounce_duration: std::time::Duration::from_millis(500),
            recursive: true,
        })
    }

    /// Get all registered routes from plugins.
    #[must_use]
    pub fn get_all_routes(&self) -> Vec<(String, PluginRoute)> {
        self.registry
            .list()
            .iter()
            .filter(|info| info.state == PluginState::Running)
            .flat_map(|info| {
                info.manifest
                    .routes
                    .iter()
                    .map(|route| (info.manifest.name.clone(), route.clone()))
            })
            .collect()
    }

    /// Get all registered pages from plugins.
    #[must_use]
    pub fn get_all_pages(&self) -> Vec<(String, PageDefinition)> {
        self.registry
            .list()
            .iter()
            .filter(|info| info.state == PluginState::Running)
            .flat_map(|info| {
                info.manifest
                    .pages
                    .iter()
                    .map(|page| (info.manifest.name.clone(), page.clone()))
            })
            .collect()
    }

    /// Execute a plugin route handler.
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails.
    pub async fn execute_route(
        &self,
        plugin_name: &str,
        handler: &str,
        context: PluginContext,
    ) -> orbis_core::Result<serde_json::Value> {
        self.runtime.execute(plugin_name, handler, context).await
    }
}
