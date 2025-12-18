//! Plugin registry for tracking loaded plugins.

use super::{PluginSource};
use orbis_plugin_api::PluginManifest;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}

impl PluginRegistry {
    /// Create a new plugin registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
        }
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
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
