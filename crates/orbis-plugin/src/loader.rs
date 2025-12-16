//! Plugin loader for loading plugins from various sources.

use super::PluginManifest;
use std::path::PathBuf;

/// Plugin source location.
#[derive(Debug, Clone)]
pub enum PluginSource {
    /// Local directory containing manifest and plugin files.
    Directory(PathBuf),

    /// Single WASM file.
    WasmFile(PathBuf),

    /// Single native library file.
    NativeFile(PathBuf),

    /// Remote URL (for future use).
    Remote(String),
}

impl PluginSource {
    /// Create a plugin source from a path.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid.
    pub fn from_path(path: &PathBuf) -> orbis_core::Result<Self> {
        if !path.exists() {
            return Err(orbis_core::Error::plugin(format!(
                "Plugin path does not exist: {:?}",
                path
            )));
        }

        if path.is_dir() {
            Ok(Self::Directory(path.clone()))
        } else if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("wasm") => Ok(Self::WasmFile(path.clone())),
                Some("so") | Some("dll") | Some("dylib") => Ok(Self::NativeFile(path.clone())),
                _ => Err(orbis_core::Error::plugin(format!(
                    "Unsupported plugin file type: {:?}",
                    ext
                ))),
            }
        } else {
            Err(orbis_core::Error::plugin(
                "Cannot determine plugin type from path",
            ))
        }
    }

    /// Get the manifest path for this source.
    #[must_use]
    pub fn manifest_path(&self) -> Option<PathBuf> {
        match self {
            Self::Directory(dir) => Some(dir.join("manifest.json")),
            Self::WasmFile(path) => {
                // Look for manifest.json next to the wasm file
                path.parent().map(|p| p.join("manifest.json"))
            }
            Self::NativeFile(path) => {
                path.parent().map(|p| p.join("manifest.json"))
            }
            Self::Remote(_) => None,
        }
    }

    /// Get the plugin entry point.
    #[must_use]
    pub fn entry_point(&self, manifest: &PluginManifest) -> Option<PathBuf> {
        match self {
            Self::Directory(dir) => {
                if let Some(wasm_entry) = &manifest.wasm_entry {
                    Some(dir.join(wasm_entry))
                } else if let Some(native_entry) = &manifest.native_entry {
                    Some(dir.join(native_entry))
                } else {
                    // Default to plugin.wasm
                    Some(dir.join("plugin.wasm"))
                }
            }
            Self::WasmFile(path) => Some(path.clone()),
            Self::NativeFile(path) => Some(path.clone()),
            Self::Remote(_) => None,
        }
    }
}

impl Default for PluginSource {
    fn default() -> Self {
        Self::Directory(PathBuf::new())
    }
}

/// Plugin loader for loading plugin manifests and code.
pub struct PluginLoader;

impl PluginLoader {
    /// Create a new plugin loader.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Load a plugin manifest from a source.
    ///
    /// # Errors
    ///
    /// Returns an error if the manifest cannot be loaded.
    pub fn load_manifest(&self, source: &PluginSource) -> orbis_core::Result<PluginManifest> {
        let manifest_path = source.manifest_path().ok_or_else(|| {
            orbis_core::Error::plugin("Cannot determine manifest path for this source")
        })?;

        if !manifest_path.exists() {
            return Err(orbis_core::Error::plugin(format!(
                "Manifest file not found: {:?}",
                manifest_path
            )));
        }

        let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to read manifest: {}", e))
        })?;

        let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to parse manifest: {}", e))
        })?;

        Ok(manifest)
    }

    /// Load plugin binary/wasm code.
    ///
    /// # Errors
    ///
    /// Returns an error if the code cannot be loaded.
    pub fn load_code(&self, source: &PluginSource, manifest: &PluginManifest) -> orbis_core::Result<Vec<u8>> {
        let entry_point = source.entry_point(manifest).ok_or_else(|| {
            orbis_core::Error::plugin("Cannot determine entry point for this source")
        })?;

        if !entry_point.exists() {
            return Err(orbis_core::Error::plugin(format!(
                "Plugin entry point not found: {:?}",
                entry_point
            )));
        }

        std::fs::read(&entry_point).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to read plugin code: {}", e))
        })
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
