//! Response builder for plugin handlers.

use super::error::{Error, Result};
use serde::Serialize;
use std::collections::HashMap;

/// HTTP response returned by plugin handlers.
#[derive(Debug, Clone, Serialize)]
pub struct Response {
    /// HTTP status code
    pub status: u16,

    /// Response headers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    /// Response body
    pub body: serde_json::Value,
}

impl Response {
    /// Create a new response with status and body
    #[inline]
    pub fn new(status: u16, body: serde_json::Value) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body,
        }
    }

    /// Create a 200 OK response with JSON body
    #[inline]
    pub fn json<T: Serialize>(data: &T) -> Result<Self> {
        let body = serde_json::to_value(data)?;
        Ok(Self::new(200, body))
    }

    /// Create a 200 OK response with raw JSON value
    #[inline]
    pub fn ok(body: serde_json::Value) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body,
        }
    }

    /// Create a 201 Created response
    #[inline]
    pub fn created<T: Serialize>(data: &T) -> Result<Self> {
        let body = serde_json::to_value(data)?;
        Ok(Self::new(201, body))
    }

    /// Create a 204 No Content response
    #[inline]
    pub fn no_content() -> Self {
        Self {
            status: 204,
            headers: HashMap::new(),
            body: serde_json::Value::Null,
        }
    }

    /// Create an error response
    #[inline]
    pub fn error(status: u16, message: &str) -> Self {
        Self::new(
            status,
            serde_json::json!({
                "error": true,
                "message": message
            }),
        )
    }

    /// Create a 400 Bad Request response
    #[inline]
    pub fn bad_request(message: &str) -> Self {
        Self::error(400, message)
    }

    /// Create a 401 Unauthorized response
    #[inline]
    pub fn unauthorized(message: &str) -> Self {
        Self::error(401, message)
    }

    /// Create a 403 Forbidden response
    #[inline]
    pub fn forbidden(message: &str) -> Self {
        Self::error(403, message)
    }

    /// Create a 404 Not Found response
    #[inline]
    pub fn not_found(message: &str) -> Self {
        Self::error(404, message)
    }

    /// Create a 500 Internal Server Error response
    #[inline]
    pub fn internal_error(message: &str) -> Self {
        Self::error(500, message)
    }

    /// Create a response from an SDK Error
    #[inline]
    pub fn from_error(err: &Error) -> Self {
        Self::error(err.status_code(), &err.to_string())
    }

    /// Add a header to the response
    #[inline]
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set Content-Type header
    #[inline]
    pub fn content_type(self, content_type: &str) -> Self {
        self.with_header("Content-Type", content_type)
    }

    /// Set Cache-Control header
    #[inline]
    pub fn cache_control(self, value: &str) -> Self {
        self.with_header("Cache-Control", value)
    }

    /// Set no-cache headers
    #[inline]
    pub fn no_cache(self) -> Self {
        self.cache_control("no-store, no-cache, must-revalidate")
    }

    /// Serialize response to raw FFI pointer for returning to host
    #[cfg(target_arch = "wasm32")]
    pub fn to_raw(&self) -> Result<i32> {
        let json = serde_json::to_vec(self)?;
        Ok(super::ffi::return_bytes(&json))
    }

    /// Serialize response (non-WASM stub)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_raw(&self) -> Result<i32> {
        Err(Error::internal("to_raw only available in WASM"))
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        Self::from_error(&err)
    }
}

/// Builder for paginated responses
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    /// The data items
    pub data: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    /// Current page (1-indexed)
    pub page: u32,

    /// Items per page
    pub per_page: u32,

    /// Total number of items
    pub total: u64,

    /// Total number of pages
    pub total_pages: u32,

    /// Whether there is a next page
    pub has_next: bool,

    /// Whether there is a previous page
    pub has_prev: bool,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Self {
            data,
            pagination: PaginationMeta {
                page,
                per_page,
                total,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
        }
    }

    /// Convert to Response
    pub fn into_response(self) -> Result<Response> {
        Response::json(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_json() {
        let data = serde_json::json!({"name": "Test"});
        let resp = Response::json(&data).unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["name"], "Test");
    }

    #[test]
    fn test_response_error() {
        let resp = Response::not_found("User not found");

        assert_eq!(resp.status, 404);
        assert_eq!(resp.body["error"], true);
        assert_eq!(resp.body["message"], "User not found");
    }

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3];
        let paginated = PaginatedResponse::new(items, 2, 10, 35);

        assert_eq!(paginated.pagination.page, 2);
        assert_eq!(paginated.pagination.total_pages, 4);
        assert!(paginated.pagination.has_next);
        assert!(paginated.pagination.has_prev);
    }
}
