//! Runtime host functions available to plugins.
//!
//! This module provides the interface for WASM plugins to interact with the Orbis runtime.
//! These functions are imported by plugins and implemented by the host runtime.
//!
//! # Memory Management
//!
//! Plugins must implement two functions for memory management:
//! ```rust,no_run
//! #[unsafe(no_mangle)]
//! pub extern "C" fn allocate(size: i32) -> *mut u8 {
//!     let mut buf = Vec::with_capacity(size as usize);
//!     let ptr = buf.as_mut_ptr();
//!     std::mem::forget(buf);
//!     ptr
//! }
//!
//! #[unsafe(no_mangle)]
//! pub extern "C" fn deallocate(ptr: *mut u8, size: i32) {
//!     unsafe {
//!         let _ = Vec::from_raw_parts(ptr, 0, size as usize);
//!     }
//! }
//! ```
//!
//! # Handler Functions
//!
//! Plugin handlers receive a pointer and length to JSON-serialized context data,
//! and must return a pointer to JSON-serialized response data:
//! ```rust,no_run
//! #[unsafe(no_mangle)]
//! pub extern "C" fn my_handler(context_ptr: i32, context_len: i32) -> i32 {
//!     // Read context from memory
//!     // Process request
//!     // Return pointer to response (with length prefix)
//!     0 // placeholder
//! }
//! ```
//!
//! Response format: [4 bytes length (u32 le)] [data bytes]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context passed to plugin handlers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    /// HTTP method.
    pub method: String,

    /// Request path.
    pub path: String,

    /// Request headers.
    pub headers: HashMap<String, String>,

    /// Query parameters.
    pub query: HashMap<String, String>,

    /// Request body (as JSON).
    #[serde(default)]
    pub body: serde_json::Value,

    /// Authenticated user ID.
    #[serde(default)]
    pub user_id: Option<String>,

    /// Whether user is admin.
    #[serde(default)]
    pub is_admin: bool,
}

/// Log levels for plugin logging.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Error level (0).
    Error = 0,
    /// Warning level (1).
    Warn = 1,
    /// Info level (2).
    Info = 2,
    /// Debug level (3).
    Debug = 3,
    /// Trace level (4).
    Trace = 4,
}

/// Host functions that plugins can import and call.
///
/// These functions are implemented by the Orbis runtime and available to all plugins.
/// Plugins should declare these in an `unsafe extern "C"` block:
///
/// ```rust,no_run
/// unsafe extern "C" {
///     fn log(level: i32, ptr: *const u8, len: i32);
///     fn state_get(key_ptr: *const u8, key_len: i32) -> *const u8;
///     fn state_set(key_ptr: *const u8, key_len: i32, value_ptr: *const u8, value_len: i32) -> i32;
///     fn state_remove(key_ptr: *const u8, key_len: i32) -> i32;
/// }
/// ```
#[allow(dead_code)]
pub struct HostFunctions;

impl HostFunctions {
    /// Log a message from the plugin.
    ///
    /// # Parameters
    /// - `level`: Log level (0=ERROR, 1=WARN, 2=INFO, 3=DEBUG, 4=TRACE)
    /// - `ptr`: Pointer to UTF-8 message bytes
    /// - `len`: Length of message in bytes
    ///
    /// # Example
    /// ```rust,no_run
    /// unsafe extern "C" {
    ///     fn log(level: i32, ptr: *const u8, len: i32);
    /// }
    ///
    /// fn log_info(msg: &str) {
    ///     unsafe {
    ///         log(2, msg.as_ptr(), msg.len() as i32);
    ///     }
    /// }
    /// ```
    pub const LOG: &'static str = "log";

    /// Get a value from plugin state.
    ///
    /// # Parameters
    /// - `key_ptr`: Pointer to UTF-8 key bytes
    /// - `key_len`: Length of key in bytes
    ///
    /// # Returns
    /// Pointer to JSON-serialized value (with 4-byte length prefix), or NULL if key not found.
    ///
    /// # Example
    /// ```rust,no_run
    /// unsafe extern "C" {
    ///     fn state_get(key_ptr: *const u8, key_len: i32) -> *const u8;
    /// }
    ///
    /// fn get_counter() -> Option<i64> {
    ///     let key = "counter";
    ///     let ptr = unsafe {
    ///         state_get(key.as_ptr(), key.len() as i32)
    ///     };
    ///     
    ///     if ptr.is_null() {
    ///         return None;
    ///     }
    ///     
    ///     // Read length and data, deserialize JSON
    ///     // ...
    ///     Some(0)
    /// }
    /// ```
    pub const STATE_GET: &'static str = "state_get";

    /// Set a value in plugin state.
    ///
    /// # Parameters
    /// - `key_ptr`: Pointer to UTF-8 key bytes
    /// - `key_len`: Length of key in bytes
    /// - `value_ptr`: Pointer to JSON-serialized value bytes
    /// - `value_len`: Length of value in bytes
    ///
    /// # Returns
    /// 1 on success, 0 on failure.
    ///
    /// # Example
    /// ```rust,no_run
    /// unsafe extern "C" {
    ///     fn state_set(key_ptr: *const u8, key_len: i32, value_ptr: *const u8, value_len: i32) -> i32;
    /// }
    ///
    /// fn set_counter(value: i64) -> bool {
    ///     let key = "counter";
    ///     let value_json = serde_json::to_string(&value).unwrap();
    ///     
    ///     let result = unsafe {
    ///         state_set(
    ///             key.as_ptr(),
    ///             key.len() as i32,
    ///             value_json.as_ptr(),
    ///             value_json.len() as i32,
    ///         )
    ///     };
    ///     
    ///     result == 1
    /// }
    /// ```
    pub const STATE_SET: &'static str = "state_set";

    /// Remove a value from plugin state.
    ///
    /// # Parameters
    /// - `key_ptr`: Pointer to UTF-8 key bytes
    /// - `key_len`: Length of key in bytes
    ///
    /// # Returns
    /// 1 on success, 0 on failure.
    ///
    /// # Example
    /// ```rust,no_run
    /// unsafe extern "C" {
    ///     fn state_remove(key_ptr: *const u8, key_len: i32) -> i32;
    /// }
    ///
    /// fn clear_counter() -> bool {
    ///     let key = "counter";
    ///     let result = unsafe {
    ///         state_remove(key.as_ptr(), key.len() as i32)
    ///     };
    ///     
    ///     result == 1
    /// }
    /// ```
    pub const STATE_REMOVE: &'static str = "state_remove";
}

/// Helper utilities for plugin development.
pub mod helpers {
    use super::*;

    /// Read bytes from a pointer.
    ///
    /// # Safety
    /// The pointer must be valid and the length must be correct.
    pub unsafe fn read_bytes(ptr: *const u8, len: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; len];
        // SAFETY: Caller guarantees ptr is valid and len is correct
        unsafe {
            std::ptr::copy_nonoverlapping(ptr, buffer.as_mut_ptr(), len);
        }
        buffer
    }

    /// Read a length-prefixed value from a pointer.
    ///
    /// # Safety
    /// The pointer must point to a valid length-prefixed value.
    pub unsafe fn read_length_prefixed(ptr: *const u8) -> Vec<u8> {
        if ptr.is_null() {
            return Vec::new();
        }

        // SAFETY: Caller guarantees ptr points to valid length-prefixed data
        unsafe {
            let len = *(ptr as *const u32);
            let data_ptr = ptr.add(4);
            read_bytes(data_ptr, len as usize)
        }
    }

    /// Write a length-prefixed value.
    ///
    /// Returns a pointer to the allocated memory (caller must deallocate).
    ///
    /// # Safety
    /// The returned pointer must be deallocated by the plugin.
    pub unsafe fn write_length_prefixed(data: &[u8], allocate_fn: extern "C" fn(i32) -> *mut u8) -> *mut u8 {
        let len = data.len() as u32;
        let total_size = 4 + data.len();
        let ptr = allocate_fn(total_size as i32);

        // SAFETY: allocate_fn returned a valid pointer with sufficient capacity
        unsafe {
            // Write length
            *(ptr as *mut u32) = len;

            // Write data
            let data_ptr = ptr.add(4);
            std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, data.len());
        }

        ptr
    }

    /// Deserialize context from memory.
    pub fn deserialize_context(ptr: *const u8, len: usize) -> Result<PluginContext, serde_json::Error> {
        let bytes = unsafe { read_bytes(ptr, len) };
        serde_json::from_slice(&bytes)
    }

    /// Serialize response to memory.
    ///
    /// # Safety
    /// The allocate function must be valid.
    pub unsafe fn serialize_response<T: Serialize>(
        value: &T,
        allocate_fn: extern "C" fn(i32) -> *mut u8,
    ) -> Result<*mut u8, serde_json::Error> {
        let json = serde_json::to_vec(value)?;
        // SAFETY: Caller guarantees allocate_fn is valid
        unsafe {
            Ok(write_length_prefixed(&json, allocate_fn))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_values() {
        assert_eq!(LogLevel::Error as i32, 0);
        assert_eq!(LogLevel::Warn as i32, 1);
        assert_eq!(LogLevel::Info as i32, 2);
        assert_eq!(LogLevel::Debug as i32, 3);
        assert_eq!(LogLevel::Trace as i32, 4);
    }

    #[test]
    fn test_plugin_context_serialization() {
        let context = PluginContext {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: serde_json::json!({}),
            user_id: Some("user123".to_string()),
            is_admin: false,
        };

        let json = serde_json::to_string(&context).unwrap();
        let deserialized: PluginContext = serde_json::from_str(&json).unwrap();

        assert_eq!(context.method, deserialized.method);
        assert_eq!(context.path, deserialized.path);
        assert_eq!(context.user_id, deserialized.user_id);
    }
}
