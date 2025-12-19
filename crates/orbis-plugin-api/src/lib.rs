//! Orbis Plugin API
//!
//! This crate provides the core types and traits needed to develop plugins for Orbis.
//! It includes:
//! - UI schema types for declarative interface definitions
//! - Plugin manifest structures for metadata and configuration
//! - Runtime host functions for WASM plugins
//! - Error handling specific to plugin development
//!
//! This is the only crate plugin developers need to depend on.

pub mod error;
pub mod manifest;
pub mod runtime;
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
