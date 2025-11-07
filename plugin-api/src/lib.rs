// Core modules (always included)
mod error;
mod context;
mod hook;
mod signature;
mod limits;
mod requirements;  // Plugin requirements declaration
mod network_types;  // Network target types (used by requirements)
pub mod ipc;  // IPC module for process isolation
#[cfg(feature = "macros")]
pub mod macros;

// Plugin-side modules
#[cfg(feature = "context-proxy")]
mod context_proxy;  // Context proxy for IPC-based context access

// Server-side modules
#[cfg(feature = "loader")]
mod loader;

#[cfg(feature = "loader")]
mod bridge;

#[cfg(feature = "security")]
mod security;

#[cfg(feature = "registry")]
mod registry;

#[cfg(feature = "monitoring")]
mod monitoring;  // Per-plugin resource monitoring

#[cfg(feature = "server")]
mod context_manager;  // Server-side context manager with permission checking

#[cfg(feature = "server")]
mod integration;  // Requirements to sandbox config integration

#[cfg(feature = "process-management")]
pub mod process;  // Process management for plugin sandboxing

#[cfg(all(feature = "sandboxing", target_os = "linux"))]
pub mod sandbox;  // Linux sandboxing (namespaces, seccomp, cgroups)

use std::sync::Arc;

// Core exports (always available)
pub use error::*;
pub use context::*;
pub use hook::*;
pub use signature::*;
pub use limits::*;
pub use requirements::*;
pub use network_types::*;

// Plugin-side exports
#[cfg(feature = "context-proxy")]
pub use context_proxy::*;

// Server-side exports
#[cfg(feature = "loader")]
pub use loader::*;

#[cfg(feature = "security")]
pub use security::*;

#[cfg(feature = "registry")]
pub use registry::*;

#[cfg(feature = "monitoring")]
pub use monitoring::PluginResourceMonitor;

#[cfg(feature = "server")]
pub use context_manager::*;

#[cfg(feature = "server")]
pub use integration::{requirements_to_sandbox_config, get_resource_limits};

#[cfg(all(feature = "server", feature = "security"))]
pub use integration::validate_requirements_against_policy;

#[cfg(all(feature = "server", not(feature = "security")))]
pub use integration::validate_requirements_against_policy;


// Re-export paste and async_trait for use in macros
#[cfg(feature = "macros")]
#[doc(hidden)]
pub use paste;

#[doc(hidden)]
pub use async_trait;

/// Main plugin trait that all plugins must implement
/// This trait must be object-safe to be used as a trait object
#[async_trait::async_trait(?Send)]
pub trait Plugin: Send + Sync {
    /// Returns the plugin name
    fn name(&self) -> &str;

    /// Returns the plugin version
    fn version(&self) -> &str;

    /// Returns the plugin author
    fn author(&self) -> &str;

    /// Returns the plugin description in Markdown format (optional)
    /// Return None if no description is provided
    fn description(&self) -> Option<&str> {
        None
    }

    /// Returns the resource limits for this plugin
    /// If None, default limits will be used
    fn resource_limits(&self) -> Option<ResourceLimits> {
        None  // Use defaults
    }

    /// Returns the requirements for this plugin (network, filesystem, etc.)
    /// If None, minimal requirements will be used (isolated plugin)
    fn requirements(&self) -> PluginRequirements {
        PluginRequirements::minimal()
    }

    /// Initialize the plugin with the provided context (opaque pointer)
    /// The context is passed as a raw pointer to avoid exposing internal types
    async fn init(&mut self, context: *const ()) -> Result<(), PluginError>;

    /// Called when the plugin is being unloaded
    async fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Register hooks that this plugin provides (opaque pointer)
    async fn register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError>;
}

/// Bridged plugin trait for internal use
#[async_trait::async_trait(?Send)]
pub trait BridgedPlugin: Send + Sync {
    /// Returns the plugin name
    fn name(&self) -> &str;

    /// Returns the plugin version
    fn version(&self) -> &str;

    /// Returns the plugin author
    fn author(&self) -> &str;

    /// Returns the plugin description in Markdown format (optional)
    fn description(&self) -> Option<&str>;

    /// Returns the resource limits for this plugin
    fn resource_limits(&self) -> Option<ResourceLimits>;

    /// Returns the requirements for this plugin
    fn requirements(&self) -> PluginRequirements;

    /// Initialize the plugin with the provided context (opaque pointer)
    /// The context is passed as a raw pointer to avoid exposing internal types
    async fn init(&mut self, context: Arc<PluginContext>) -> Result<(), PluginError>;

    /// Called when the plugin is being unloaded
    async fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Register hooks that this plugin provides (opaque pointer)
    async fn register_hooks(&self, hook_registry: &mut HookRegistry) -> Result<(), PluginError>;
}

/// Type alias for the plugin constructor function
pub type PluginConstructor = unsafe extern "C" fn() -> *mut dyn Plugin;

/// Symbol name for the plugin constructor
pub const PLUGIN_CONSTRUCTOR_SYMBOL: &str = "create_plugin";

/// Symbol name for the plugin signature
pub const PLUGIN_SIGNATURE_SYMBOL: &str = "plugin_signature";

/// Symbol name for the plugin hash
pub const PLUGIN_HASH_SYMBOL: &str = "plugin_hash";
