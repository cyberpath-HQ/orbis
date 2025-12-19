//! Plugin registry for tracking loaded plugins.

use super::{PluginSource};
use orbis_plugin_api::PluginManifest;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::{Path, PathBuf};

/// Plugin state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginState {
    /// Plugin is loaded but not running.
    Loaded,

    /// Plugin is running.
    Running,

    /// Plugin is disabled.
    Disabled,

    /// Plugin encountered an error.
    Error,
}

/// Information about a loaded plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID.
    pub id: Uuid,

    /// Plugin manifest.
    pub manifest: PluginManifest,

    /// Plugin source.
    #[serde(skip)]
    pub source: PluginSource,

    /// Current state.
    pub state: PluginState,

    /// When the plugin was loaded.
    pub loaded_at: DateTime<Utc>,
}

/// Registry for tracking loaded plugins.
pub struct PluginRegistry {
    plugins: DashMap<String, PluginInfo>,
    state_file: Option<PathBuf>,
}

impl PluginRegistry {
    /// Create a new plugin registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
            state_file: None,
        }
    }
    
    /// Create a plugin registry with persistence.
    #[must_use]
    pub fn with_persistence(state_file: PathBuf) -> Self {
        let mut registry = Self {
            plugins: DashMap::new(),
            state_file: Some(state_file),
        };
        
        // Load existing state
        let _ = registry.load_state();
        
        registry
    }

    /// Register a plugin.
    pub fn register(&self, info: PluginInfo) {
        self.plugins.insert(info.manifest.name.clone(), info);
    }

    /// Unregister a plugin.
    pub fn unregister(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.remove(name).map(|(_, info)| info)
    }

    /// Get a plugin by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.get(name).map(|r| r.value().clone())
    }

    /// List all plugins.
    #[must_use]
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|r| r.value().clone()).collect()
    }

    /// List plugins by state.
    #[must_use]
    pub fn list_by_state(&self, state: PluginState) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .filter(|r| r.value().state == state)
            .map(|r| r.value().clone())
            .collect()
    }

    /// Set plugin state.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is not found.
    pub fn set_state(&self, name: &str, state: PluginState) -> orbis_core::Result<()> {
        let mut entry = self.plugins.get_mut(name).ok_or_else(|| {
            orbis_core::Error::plugin(format!("Plugin '{}' not found", name))
        })?;

        entry.value_mut().state = state;
        
        // Persist state after change
        let _ = self.save_state();
        
        Ok(())
    }

    /// Check if a plugin exists.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get the number of registered plugins.
    #[must_use]
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Get the number of running plugins.
    #[must_use]
    pub fn running_count(&self) -> usize {
        self.plugins
            .iter()
            .filter(|r| r.value().state == PluginState::Running)
            .count()
    }
    
    /// Save plugin states to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    fn save_state(&self) -> orbis_core::Result<()> {
        if let Some(ref state_file) = self.state_file {
            #[derive(Serialize)]
            struct PluginStateRecord {
                name: String,
                state: PluginState,
            }
            
            let states: Vec<PluginStateRecord> = self.plugins
                .iter()
                .map(|entry| PluginStateRecord {
                    name: entry.key().clone(),
                    state: entry.value().state,
                })
                .collect();
            
            let json = serde_json::to_string_pretty(&states)
                .map_err(|e| orbis_core::Error::plugin(format!("Failed to serialize state: {}", e)))?;
            
            // Ensure parent directory exists
            if let Some(parent) = state_file.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| orbis_core::Error::plugin(format!("Failed to create state directory: {}", e)))?;
            }
            
            std::fs::write(state_file, json)
                .map_err(|e| orbis_core::Error::plugin(format!("Failed to write state file: {}", e)))?;
            
            tracing::debug!("Saved plugin states to {:?}", state_file);
        }
        
        Ok(())
    }
    
    /// Load plugin states from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails.
    fn load_state(&mut self) -> orbis_core::Result<()> {
        if let Some(ref state_file) = self.state_file {
            if !state_file.exists() {
                tracing::debug!("State file does not exist: {:?}", state_file);
                return Ok(());
            }
            
            #[derive(Deserialize)]
            struct PluginStateRecord {
                name: String,
                state: PluginState,
            }
            
            let contents = std::fs::read_to_string(state_file)
                .map_err(|e| orbis_core::Error::plugin(format!("Failed to read state file: {}", e)))?;
            
            let states: Vec<PluginStateRecord> = serde_json::from_str(&contents)
                .map_err(|e| orbis_core::Error::plugin(format!("Failed to parse state file: {}", e)))?;
            
            // Apply saved states to matching plugins
            for record in states {
                if let Some(mut entry) = self.plugins.get_mut(&record.name) {
                    entry.value_mut().state = record.state;
                }
            }
            
            tracing::info!("Loaded plugin states from {:?}", state_file);
        }
        
        Ok(())
    }
    
    /// Restore states from saved state file for newly loaded plugins.
    ///
    /// This is called after loading plugins to restore their previous states.
    pub fn restore_states(&self) -> orbis_core::Result<()> {
        if let Some(ref state_file) = self.state_file {
            if !state_file.exists() {
                return Ok(());
            }
            
            #[derive(Deserialize)]
            struct PluginStateRecord {
                name: String,
                state: PluginState,
            }
            
            let contents = std::fs::read_to_string(state_file)
                .map_err(|e| orbis_core::Error::plugin(format!("Failed to read state file: {}", e)))?;
            
            let states: Vec<PluginStateRecord> = serde_json::from_str(&contents)
                .map_err(|e| orbis_core::Error::plugin(format!("Failed to parse state file: {}", e)))?;
            
            // Apply saved states to matching plugins
            for record in states {
                if let Some(mut entry) = self.plugins.get_mut(&record.name) {
                    entry.value_mut().state = record.state;
                    tracing::info!("Restored state for plugin '{}': {:?}", record.name, record.state);
                }
            }
        }
        
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
