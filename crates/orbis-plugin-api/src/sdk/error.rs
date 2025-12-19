//! Plugin SDK error types.

use std::fmt;

/// Result type for plugin operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for plugin operations
#[derive(Debug)]
pub enum Error {
    /// JSON serialization/deserialization error
    Json(serde_json::Error),

    /// State operation error
    State(String),

    /// Database operation error
    Database(String),

    /// HTTP request error
    Http(String),

    /// Permission denied
    PermissionDenied(String),

    /// Invalid input
    InvalidInput(String),

    /// Not found
    NotFound(String),

    /// Internal plugin error
    Internal(String),

    /// Validation error
    Validation(String),

    /// Timeout error
    Timeout(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "JSON error: {}", e),
            Self::State(msg) => write!(f, "State error: {}", msg),
            Self::Database(msg) => write!(f, "Database error: {}", msg),
            Self::Http(msg) => write!(f, "HTTP error: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::InvalidInput(format!("Invalid UTF-8: {}", e))
    }
}

impl Error {
    /// Create a state error
    #[inline]
    pub fn state<S: Into<String>>(msg: S) -> Self {
        Self::State(msg.into())
    }

    /// Create a database error
    #[inline]
    pub fn database<S: Into<String>>(msg: S) -> Self {
        Self::Database(msg.into())
    }

    /// Create an HTTP error
    #[inline]
    pub fn http<S: Into<String>>(msg: S) -> Self {
        Self::Http(msg.into())
    }

    /// Create a permission denied error
    #[inline]
    pub fn permission_denied<S: Into<String>>(msg: S) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create an invalid input error
    #[inline]
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create a not found error
    #[inline]
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an internal error
    #[inline]
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a validation error
    #[inline]
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Get HTTP status code for this error
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::Json(_) | Self::InvalidInput(_) | Self::Validation(_) => 400,
            Self::PermissionDenied(_) => 403,
            Self::NotFound(_) => 404,
            Self::Timeout(_) => 408,
            Self::State(_) | Self::Database(_) | Self::Http(_) | Self::Internal(_) => 500,
        }
    }
}
