//! HTTP client for making external requests.
//!
//! Allows plugins to make HTTP requests to external APIs.
//!
//! # Example
//!
//! ```rust,ignore
//! use orbis_plugin_api::sdk::http;
//!
//! // Simple GET request
//! let response = http::get("https://api.example.com/data")?;
//!
//! // POST with JSON body
//! let response = http::post("https://api.example.com/users")
//!     .json(&CreateUser { name: "John" })?
//!     .send()?;
//!
//! // Parse response as JSON
//! let user: User = response.json()?;
//! ```

use super::error::{Error, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP method
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Patch => write!(f, "PATCH"),
            Self::Delete => write!(f, "DELETE"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
        }
    }
}

/// HTTP request builder
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Request {
    method: Method,
    url: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl Request {
    /// Create a new request
    fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set Content-Type header
    pub fn content_type(self, content_type: &str) -> Self {
        self.header("Content-Type", content_type)
    }

    /// Set Authorization header with Bearer token
    pub fn bearer_token(self, token: &str) -> Self {
        self.header("Authorization", format!("Bearer {}", token))
    }

    /// Set a JSON body
    pub fn json<T: Serialize>(mut self, body: &T) -> Result<Self> {
        let json = serde_json::to_vec(body)?;
        self.body = Some(json);
        Ok(self.content_type("application/json"))
    }

    /// Set a raw body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set form data body
    pub fn form(mut self, data: &HashMap<String, String>) -> Self {
        let encoded = data
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        self.body = Some(encoded.into_bytes());
        self.content_type("application/x-www-form-urlencoded")
    }

    /// Send the request
    #[cfg(target_arch = "wasm32")]
    pub fn send(self) -> Result<Response> {
        let method_str = self.method.to_string();
        let headers_json = serde_json::to_vec(&self.headers)?;
        let body = self.body.unwrap_or_default();

        let result_ptr = unsafe {
            super::ffi::http_request(
                method_str.as_ptr(),
                method_str.len() as i32,
                self.url.as_ptr(),
                self.url.len() as i32,
                headers_json.as_ptr(),
                headers_json.len() as i32,
                body.as_ptr(),
                body.len() as i32,
            )
        };

        if result_ptr.is_null() {
            return Err(Error::http("HTTP request failed"));
        }

        let result_bytes = unsafe { super::ffi::read_length_prefixed(result_ptr) };
        let response: Response = serde_json::from_slice(&result_bytes)?;

        Ok(response)
    }

    /// Send the request (non-WASM stub)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn send(self) -> Result<Response> {
        Err(Error::http("HTTP not available outside WASM"))
    }
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// HTTP status code
    pub status: u16,

    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Response body (as bytes, base64 encoded in JSON)
    #[serde(default)]
    pub body: Vec<u8>,

    /// Error message (if any)
    #[serde(default)]
    pub error: Option<String>,
}

impl Response {
    /// Check if the response was successful (2xx status)
    #[inline]
    pub const fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Check if there was an error
    #[inline]
    pub fn is_error(&self) -> bool {
        self.error.is_some() || self.status >= 400
    }

    /// Get the response body as a string
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone()).map_err(Error::from)
    }

    /// Parse the response body as JSON
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(Error::from)
    }

    /// Get a header value (case-insensitive)
    pub fn header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Ensure the response was successful, or return an error
    pub fn error_for_status(self) -> Result<Self> {
        if self.is_success() {
            Ok(self)
        } else {
            let msg = self.error.clone().unwrap_or_else(|| {
                format!("HTTP {}: {}", self.status, self.text().unwrap_or_default())
            });
            Err(Error::http(msg))
        }
    }
}

/// Create a GET request
#[inline]
pub fn get(url: impl Into<String>) -> Request {
    Request::new(Method::Get, url)
}

/// Create a POST request
#[inline]
pub fn post(url: impl Into<String>) -> Request {
    Request::new(Method::Post, url)
}

/// Create a PUT request
#[inline]
pub fn put(url: impl Into<String>) -> Request {
    Request::new(Method::Put, url)
}

/// Create a PATCH request
#[inline]
pub fn patch(url: impl Into<String>) -> Request {
    Request::new(Method::Patch, url)
}

/// Create a DELETE request
#[inline]
pub fn delete(url: impl Into<String>) -> Request {
    Request::new(Method::Delete, url)
}

/// Simple URL encoding module
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::with_capacity(s.len() * 3);
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                ' ' => result.push('+'),
                _ => {
                    for b in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", b));
                    }
                }
            }
        }
        result
    }
}
