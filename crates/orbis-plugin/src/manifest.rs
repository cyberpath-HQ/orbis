//! Plugin manifest definition.

use serde::{Deserialize, Serialize};
use semver::Version;

/// Plugin manifest describing the plugin's metadata, routes, and pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name (unique identifier).
    pub name: String,

    /// Plugin version (semver).
    pub version: String,

    /// Human-readable description.
    #[serde(default)]
    pub description: String,

    /// Plugin author.
    #[serde(default)]
    pub author: Option<String>,

    /// Plugin homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,

    /// Plugin license.
    #[serde(default)]
    pub license: Option<String>,

    /// Minimum Orbis version required.
    #[serde(default)]
    pub min_orbis_version: Option<String>,

    /// Plugin dependencies.
    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,

    /// Required permissions.
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,

    /// API routes defined by the plugin.
    #[serde(default)]
    pub routes: Vec<PluginRoute>,

    /// UI pages defined by the plugin.
    #[serde(default)]
    pub pages: Vec<super::PageDefinition>,

    /// Entry point for WASM plugins.
    #[serde(default)]
    pub wasm_entry: Option<String>,

    /// Entry point for native plugins.
    #[serde(default)]
    pub native_entry: Option<String>,

    /// Additional custom configuration.
    #[serde(default)]
    pub config: serde_json::Value,
}

impl PluginManifest {
    /// Validate the manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if the manifest is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        // Validate name
        if self.name.is_empty() {
            return Err(orbis_core::Error::plugin("Plugin name is required"));
        }

        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(orbis_core::Error::plugin(
                "Plugin name must contain only alphanumeric characters, hyphens, and underscores",
            ));
        }

        // Validate version
        if self.version.is_empty() {
            return Err(orbis_core::Error::plugin("Plugin version is required"));
        }

        Version::parse(&self.version).map_err(|e| {
            orbis_core::Error::plugin(format!("Invalid plugin version '{}': {}", self.version, e))
        })?;

        // Validate routes
        for route in &self.routes {
            route.validate()?;
        }

        // Validate pages
        for page in &self.pages {
            page.validate()?;
        }

        Ok(())
    }

    /// Get the parsed semver version.
    ///
    /// # Errors
    ///
    /// Returns an error if the version is invalid.
    pub fn parsed_version(&self) -> orbis_core::Result<Version> {
        Version::parse(&self.version)
            .map_err(|e| orbis_core::Error::plugin(format!("Invalid version: {}", e)))
    }
}

/// Plugin dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency name.
    pub name: String,

    /// Version requirement (semver).
    pub version: String,

    /// Whether the dependency is optional.
    #[serde(default)]
    pub optional: bool,
}

/// Plugin permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Read from database.
    DatabaseRead,

    /// Write to database.
    DatabaseWrite,

    /// Read files.
    FileRead,

    /// Write files.
    FileWrite,

    /// Make network requests.
    Network,

    /// Access system information.
    System,

    /// Execute shell commands (dangerous).
    Shell,

    /// Access environment variables.
    Environment,

    /// Custom permission.
    Custom(String),
}

/// API route definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRoute {
    /// HTTP method.
    pub method: String,

    /// Route path (relative to plugin prefix).
    pub path: String,

    /// Handler function name.
    pub handler: String,

    /// Route description.
    #[serde(default)]
    pub description: Option<String>,

    /// Whether authentication is required.
    #[serde(default = "default_true")]
    pub requires_auth: bool,

    /// Required permissions.
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Rate limit (requests per minute).
    #[serde(default)]
    pub rate_limit: Option<u32>,
}

fn default_true() -> bool {
    true
}

impl PluginRoute {
    /// Validate the route.
    ///
    /// # Errors
    ///
    /// Returns an error if the route is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        // Validate method
        let valid_methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
        if !valid_methods.contains(&self.method.to_uppercase().as_str()) {
            return Err(orbis_core::Error::plugin(format!(
                "Invalid HTTP method: {}",
                self.method
            )));
        }

        // Validate path
        if self.path.is_empty() {
            return Err(orbis_core::Error::plugin("Route path is required"));
        }

        if !self.path.starts_with('/') {
            return Err(orbis_core::Error::plugin("Route path must start with '/'"));
        }

        // Validate handler
        if self.handler.is_empty() {
            return Err(orbis_core::Error::plugin("Route handler is required"));
        }

        Ok(())
    }

    /// Get the full route path with plugin prefix.
    #[must_use]
    pub fn full_path(&self, plugin_name: &str) -> String {
        format!("/api/plugins/{}{}", plugin_name, self.path)
    }
}
