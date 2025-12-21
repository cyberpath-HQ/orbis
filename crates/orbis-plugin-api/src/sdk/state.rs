//! Type-safe state management for plugins.
//!
//! Provides an ergonomic API for storing and retrieving plugin state.
//!
//! # Example
//!
//! ```rust,ignore
//! use orbis_plugin_api::sdk::state;
//!
//! // Set a value
//! state::set("counter", &42)?;
//!
//! // Get a value
//! let counter: i32 = state::get("counter")?.unwrap_or(0);
//!
//! // Remove a value
//! state::remove("counter")?;
//! ```

#[allow(unused_imports)]
use super::error::{Error, Result};
use serde::{de::DeserializeOwned, Serialize};

/// Get a value from plugin state.
///
/// Returns `None` if the key doesn't exist, or `Some(value)` if it does.
///
/// # Errors
///
/// Returns an error if deserialization fails.
#[cfg(target_arch = "wasm32")]
pub fn get<T: DeserializeOwned>(key: &str) -> Result<Option<T>> {
    let ptr = unsafe {
        super::ffi::state_get(key.as_ptr() as i32, key.len() as i32)
    };

    if ptr == 0 {
        return Ok(None);
    }

    let bytes = unsafe { super::ffi::read_length_prefixed(ptr) };
    let value: T = serde_json::from_slice(&bytes)?;
    Ok(Some(value))
}

/// Get a value from plugin state (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn get<T: DeserializeOwned>(_key: &str) -> Result<Option<T>> {
    Ok(None)
}

/// Get a value or return a default.
///
/// # Example
///
/// ```rust,ignore
/// let count: i32 = state::get_or("counter", 0)?;
/// ```
#[inline]
pub fn get_or<T: DeserializeOwned>(key: &str, default: T) -> Result<T> {
    get(key).map(|opt| opt.unwrap_or(default))
}

/// Get a value or compute a default.
#[inline]
pub fn get_or_else<T: DeserializeOwned, F: FnOnce() -> T>(key: &str, f: F) -> Result<T> {
    get(key).map(|opt| opt.unwrap_or_else(f))
}

/// Set a value in plugin state.
///
/// # Errors
///
/// Returns an error if serialization fails or the host rejects the operation.
#[cfg(target_arch = "wasm32")]
pub fn set<T: Serialize>(key: &str, value: &T) -> Result<()> {
    let value_json = serde_json::to_vec(value)?;

    let result = unsafe {
        super::ffi::state_set(
            key.as_ptr() as i32,
            key.len() as i32,
            value_json.as_ptr() as i32,
            value_json.len() as i32,
        )
    };

    if result == 1 {
        Ok(())
    } else {
        Err(Error::state(format!("Failed to set state key: {}", key)))
    }
}

/// Set a value in plugin state (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn set<T: Serialize>(_key: &str, _value: &T) -> Result<()> {
    Ok(())
}

/// Remove a value from plugin state.
///
/// # Errors
///
/// Returns an error if the host rejects the operation.
#[cfg(target_arch = "wasm32")]
pub fn remove(key: &str) -> Result<()> {
    let result = unsafe {
        super::ffi::state_remove(key.as_ptr() as i32, key.len() as i32)
    };

    if result == 1 {
        Ok(())
    } else {
        Err(Error::state(format!("Failed to remove state key: {}", key)))
    }
}

/// Remove a value from plugin state (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn remove(_key: &str) -> Result<()> {
    Ok(())
}

/// Update a value in state using a function.
///
/// If the key doesn't exist, uses the default value.
///
/// # Example
///
/// ```rust,ignore
/// state::update("counter", 0, |n| n + 1)?;
/// ```
pub fn update<T, F>(key: &str, default: T, f: F) -> Result<T>
where
    T: DeserializeOwned + Serialize + Clone,
    F: FnOnce(T) -> T,
{
    let current = get(key)?.unwrap_or(default);
    let new_value = f(current);
    set(key, &new_value)?;
    Ok(new_value)
}

/// Increment a numeric value in state.
///
/// # Example
///
/// ```rust,ignore
/// let new_count = state::increment("counter")?;
/// ```
pub fn increment(key: &str) -> Result<i64> {
    update(key, 0i64, |n| n + 1)
}

/// Decrement a numeric value in state.
pub fn decrement(key: &str) -> Result<i64> {
    update(key, 0i64, |n| n - 1)
}

/// Append to a list in state.
///
/// # Example
///
/// ```rust,ignore
/// state::push("items", &"new item")?;
/// ```
pub fn push<T>(key: &str, value: &T) -> Result<()>
where
    T: Serialize + DeserializeOwned,
{
    let mut items: Vec<T> = get(key)?.unwrap_or_default();
    // We need to serialize and deserialize to get the value in the right format
    let value_json = serde_json::to_value(value)?;
    let typed_value: T = serde_json::from_value(value_json)?;
    items.push(typed_value);
    set(key, &items)
}

/// Get the length of a list in state.
pub fn len(key: &str) -> Result<usize> {
    let items: Vec<serde_json::Value> = get(key)?.unwrap_or_default();
    Ok(items.len())
}

/// Check if a key exists in state.
#[cfg(target_arch = "wasm32")]
pub fn exists(key: &str) -> bool {
    let ptr = unsafe {
        super::ffi::state_get(key.as_ptr() as i32, key.len() as i32)
    };
    ptr != 0
}

/// Check if a key exists in state (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn exists(_key: &str) -> bool {
    false
}

/// Scoped state access with a prefix.
///
/// Useful for organizing state by feature or entity.
///
/// # Example
///
/// ```rust,ignore
/// let user_state = state::scoped("user:123");
/// user_state.set("name", &"John")?;
/// let name: String = user_state.get("name")?.unwrap();
/// ```
pub struct ScopedState {
    prefix: String,
}

impl ScopedState {
    /// Create a new scoped state accessor
    #[must_use]
    pub fn new(prefix: impl Into<String>) -> Self {
        let mut prefix = prefix.into();
        if !prefix.ends_with(':') {
            prefix.push(':');
        }
        Self { prefix }
    }

    fn key(&self, name: &str) -> String {
        format!("{}{}", self.prefix, name)
    }

    /// Get a value from scoped state
    pub fn get<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {
        get(&self.key(name))
    }

    /// Set a value in scoped state
    pub fn set<T: Serialize>(&self, name: &str, value: &T) -> Result<()> {
        set(&self.key(name), value)
    }

    /// Remove a value from scoped state
    pub fn remove(&self, name: &str) -> Result<()> {
        remove(&self.key(name))
    }

    /// Check if a key exists in scoped state
    pub fn exists(&self, name: &str) -> bool {
        exists(&self.key(name))
    }
}

/// Create a scoped state accessor
#[inline]
#[must_use]
pub fn scoped(prefix: impl Into<String>) -> ScopedState {
    ScopedState::new(prefix)
}
