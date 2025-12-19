//! Orbis Plugin SDK
//!
//! This module provides a complete, ergonomic SDK for building WASM plugins with minimal boilerplate.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use orbis_plugin_api::prelude::*;
//!
//! // Define your plugin using the macro
//! orbis_plugin! {
//!     name: "my-plugin",
//!     version: "1.0.0",
//!
//!     // Optional initialization
//!     init: || {
//!         log::info!("Plugin initialized!");
//!         Ok(())
//!     },
//!
//!     // Optional cleanup
//!     cleanup: || {
//!         log::info!("Plugin cleaning up!");
//!         Ok(())
//!     },
//!
//!     handlers: {
//!         "my_handler" => my_handler,
//!     }
//! }
//!
//! fn my_handler(ctx: Context) -> Result<Response> {
//!     let count: i64 = state::get("counter")?.unwrap_or(0);
//!     state::set("counter", &(count + 1))?;
//!
//!     Response::json(&json!({
//!         "message": "Hello!",
//!         "count": count + 1
//!     }))
//! }
//! ```
//!
//! # Features
//!
//! - **Zero boilerplate**: No manual memory management, extern declarations, or FFI
//! - **Type-safe state**: Automatic JSON serialization/deserialization
//! - **Ergonomic logging**: Simple `log::info!()` style macros
//! - **Database access**: Query and execute SQL with typed results
//! - **HTTP client**: Make external API calls
//! - **Event system**: Emit and subscribe to events
//! - **Error handling**: Proper Result types with context

pub mod context;
pub mod db;
pub mod error;
pub mod ffi;
pub mod http;
pub mod log;
pub mod response;
pub mod state;

// Re-export everything for convenience
pub use context::Context;
pub use db::{DbRow, DbValue};
pub use error::{Error, Result};
pub use response::Response;

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::context::Context;
    pub use super::db::{self, DbRow, DbValue};
    pub use super::error::{Error, Result};
    pub use super::ffi::*;
    pub use super::http;
    pub use super::log;
    pub use super::response::Response;
    pub use super::state;

    // Re-export serde for convenience
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value as JsonValue};
}
