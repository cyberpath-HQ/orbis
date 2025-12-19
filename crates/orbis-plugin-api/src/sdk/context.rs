//! Request context passed to plugin handlers.

use super::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context passed to plugin handlers.
///
/// Contains all information about the incoming request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Request path
    pub path: String,

    /// Path parameters (extracted from route pattern)
    #[serde(default)]
    pub params: HashMap<String, String>,

    /// Request headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Query string parameters
    #[serde(default)]
    pub query: HashMap<String, String>,

    /// Request body (parsed as JSON)
    #[serde(default)]
    pub body: serde_json::Value,

    /// Authenticated user ID (if any)
    #[serde(default)]
    pub user_id: Option<String>,

    /// Whether the user is an admin
    #[serde(default)]
    pub is_admin: bool,

    /// Request ID for tracing
    #[serde(default)]
    pub request_id: Option<String>,
}

impl Context {
    /// Parse context from raw FFI pointer
    ///
    /// # Safety
    /// This function reads from raw pointers passed from the host
    #[cfg(target_arch = "wasm32")]
    pub fn from_raw(ptr: i32, len: i32) -> Result<Self> {
        let bytes = unsafe {
            std::slice::from_raw_parts(ptr as *const u8, len as usize)
        };
        serde_json::from_slice(bytes).map_err(Error::from)
    }

    /// Parse context from raw FFI pointer (non-WASM stub)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_raw(_ptr: i32, _len: i32) -> Result<Self> {
        Err(Error::internal("from_raw only available in WASM"))
    }

    /// Get a path parameter by name
    #[inline]
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(String::as_str)
    }

    /// Get a required path parameter, or return an error
    #[inline]
    pub fn param_required(&self, name: &str) -> Result<&str> {
        self.params
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| Error::invalid_input(format!("Missing path parameter: {}", name)))
    }

    /// Get a query parameter by name
    #[inline]
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(String::as_str)
    }

    /// Get a query parameter with a default value
    #[inline]
    pub fn query_param_or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.query.get(name).map(String::as_str).unwrap_or(default)
    }

    /// Get a query parameter parsed as a specific type
    pub fn query_param_as<T: std::str::FromStr>(&self, name: &str) -> Option<T> {
        self.query.get(name).and_then(|v| v.parse().ok())
    }

    /// Get a header by name (case-insensitive)
    #[inline]
    pub fn header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Parse the request body as a specific type
    #[inline]
    pub fn body_as<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_value(self.body.clone()).map_err(Error::from)
    }

    /// Get a field from the body
    #[inline]
    pub fn body_field(&self, name: &str) -> Option<&serde_json::Value> {
        self.body.get(name)
    }

    /// Get a field from the body as a specific type
    pub fn body_field_as<T: for<'de> Deserialize<'de>>(&self, name: &str) -> Result<Option<T>> {
        match self.body.get(name) {
            Some(v) => serde_json::from_value(v.clone()).map(Some).map_err(Error::from),
            None => Ok(None),
        }
    }

    /// Check if the request is authenticated
    #[inline]
    pub const fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Require authentication, return error if not authenticated
    #[inline]
    pub fn require_auth(&self) -> Result<&str> {
        self.user_id
            .as_deref()
            .ok_or_else(|| Error::permission_denied("Authentication required"))
    }

    /// Require admin access
    #[inline]
    pub fn require_admin(&self) -> Result<()> {
        if self.is_admin {
            Ok(())
        } else {
            Err(Error::permission_denied("Admin access required"))
        }
    }

    /// Check if the request method matches
    #[inline]
    pub fn is_method(&self, method: &str) -> bool {
        self.method.eq_ignore_ascii_case(method)
    }

    /// Get pagination parameters from query string
    ///
    /// Returns (page, per_page) with defaults of (1, 20)
    pub fn pagination(&self) -> (u32, u32) {
        let page = self.query_param_as("page").unwrap_or(1).max(1);
        let per_page = self.query_param_as("per_page").unwrap_or(20).clamp(1, 100);
        (page, per_page)
    }

    /// Get offset/limit for database queries from pagination
    ///
    /// Returns (offset, limit)
    pub fn pagination_offset(&self) -> (u32, u32) {
        let (page, per_page) = self.pagination();
        let offset = (page - 1) * per_page;
        (offset, per_page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_parsing() {
        let json = r#"{
            "method": "GET",
            "path": "/users/123",
            "params": {"id": "123"},
            "query": {"page": "2", "per_page": "10"},
            "headers": {"Content-Type": "application/json"},
            "body": {"name": "Test"},
            "user_id": "user123",
            "is_admin": false
        }"#;

        let ctx: Context = serde_json::from_str(json).unwrap();

        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.path, "/users/123");
        assert_eq!(ctx.param("id"), Some("123"));
        assert_eq!(ctx.query_param("page"), Some("2"));
        assert_eq!(ctx.header("content-type"), Some("application/json"));
        assert!(ctx.is_authenticated());
        assert!(!ctx.is_admin);
    }

    #[test]
    fn test_pagination() {
        let ctx = Context {
            method: "GET".into(),
            path: "/".into(),
            params: HashMap::new(),
            headers: HashMap::new(),
            query: [("page".into(), "3".into()), ("per_page".into(), "50".into())].into(),
            body: serde_json::Value::Null,
            user_id: None,
            is_admin: false,
            request_id: None,
        };

        assert_eq!(ctx.pagination(), (3, 50));
        assert_eq!(ctx.pagination_offset(), (100, 50));
    }
}
