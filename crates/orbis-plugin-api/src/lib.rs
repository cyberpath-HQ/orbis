//! Orbis Plugin API
//!
//! This crate provides the core types and traits needed to develop plugins for Orbis.
//! It includes:
//! - **SDK**: Complete plugin development kit with minimal boilerplate
//! - **UI schema types**: Declarative interface definitions
//! - **Plugin manifest**: Metadata and configuration structures
//! - **Runtime host functions**: WASM plugin host interface
//! - **Error handling**: Plugin-specific error types
//!
//! This is the only crate plugin developers need to depend on.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use orbis_plugin_api::prelude::*;
//!
//! // Define your plugin with zero boilerplate
//! orbis_plugin! {
//!     init: || {
//!         log::info!("Plugin initialized!");
//!         Ok(())
//!     }
//! }
//!
//! // Create a handler
//! #[handler]
//! fn my_handler(ctx: Context) -> Result<Response> {
//!     let count = state::increment("visits")?;
//!     Response::json(&json!({ "visits": count }))
//! }
//! ```

pub mod error;
pub mod manifest;
pub mod runtime;
pub mod sdk;
pub mod ui;

// Re-export key types for convenience
pub use error::{Error, Result};
pub use manifest::{PluginDependency, PluginManifest, PluginPermission, PluginRoute};
pub use runtime::{HostFunctions, LogLevel, PluginContext};
pub use ui::{
    AccordionItem, Action, ArgMapping, BreadcrumbItem, ComponentSchema, CustomValidation,
    DialogDefinition, EventHandlers, FormField, NavigationConfig, NavigationItem, PageDefinition,
    PageLifecycleHooks, SelectOption, StateFieldDefinition, StateFieldType, TabItem, TableColumn,
    ToastLevel, ValidationRule,
};

/// Prelude for convenient imports in plugins
///
/// ```rust,ignore
/// use orbis_plugin_api::prelude::*;
/// ```
pub mod prelude {
    pub use crate::sdk::prelude::*;
    pub use crate::{log_debug, log_error, log_info, log_trace, log_warn, orbis_plugin, wrap_handler};
}
