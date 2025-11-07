use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::{BridgedPlugin, Plugin, PluginConstructor, PluginError, PluginSecurity, TrustLevel, PLUGIN_CONSTRUCTOR_SYMBOL};
use crate::bridge::PluginBridge;

/// Plugin loader responsible for loading dynamic libraries
pub struct PluginLoader {
    security: Arc<PluginSecurity>,
}

impl PluginLoader {
    pub fn new(security: Arc<PluginSecurity>) -> Self {
        Self { security }
    }

    /// Load a plugin from a dynamic library
    pub fn load<P: AsRef<Path>>(
        &self,
        path: P,
        _trust_level: TrustLevel,
    ) -> Result<(Box<PluginBridge>, Library), PluginError> {
        let path = path.as_ref();

        // Convert to absolute path
        let abs_path = path.canonicalize().map_err(|e| {
            PluginError::LoadError(format!("Failed to resolve plugin path: {}", e))
        })?;

        // Load the library
        let library = unsafe {
            Library::new(&abs_path).map_err(|e| {
                PluginError::LoadError(format!("Failed to load library: {}", e))
            })?
        };

        // Get the constructor function
        let constructor: Symbol<PluginConstructor> = unsafe {
            library
                .get(PLUGIN_CONSTRUCTOR_SYMBOL.as_bytes())
                .map_err(|e| {
                    PluginError::SymbolNotFound(format!(
                        "Constructor symbol '{}' not found: {}",
                        PLUGIN_CONSTRUCTOR_SYMBOL, e
                    ))
                })?
        };

        // Create the plugin instance
        let plugin_ptr = unsafe { constructor() };
        let api_plugin = unsafe { Box::from_raw(plugin_ptr) };

        // Wrap in bridge
        let plugin = Box::new(PluginBridge::new(api_plugin));

        // Validate the plugin with security manager (trust_level is ignored, we use hash-based trust)
        self.security.validate_plugin(
            plugin.name(),
            &abs_path,
            TrustLevel::Trusted,  // Only trusted plugins can be loaded
        )?;

        Ok((plugin, library))
    }

    /// Load a plugin from a path string
    pub fn load_from_path(
        &self,
        path: &str,
        trust_level: TrustLevel,
    ) -> Result<(Box<PluginBridge>, Library), PluginError> {
        self.load(PathBuf::from(path), trust_level)
    }
}

#[cfg(test)]
mod tests {
    use crate::SecurityPolicy;
    use super::*;

    #[test]
    fn test_loader_creation() {
        let security = Arc::new(PluginSecurity::new(SecurityPolicy::default(), vec![], vec![]));
        let _loader = PluginLoader::new(security);
    }
}

