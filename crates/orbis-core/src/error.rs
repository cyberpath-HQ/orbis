//! Error types for Orbis.

use thiserror::Error;

/// Result type alias using the Orbis error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Core error types for Orbis.
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(String),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Authorization error.
    #[error("Authorization error: {0}")]
    Unauthorized(String),

    /// Plugin error.
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// Server error.
    #[error("Server error: {0}")]
    Server(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found error.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Conflict error.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Create a new configuration error.
    #[must_use]
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new database error.
    #[must_use]
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    /// Create a new authentication error.
    #[must_use]
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Create a new unauthorized error.
    #[must_use]
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// Create a new plugin error.
    #[must_use]
    pub fn plugin(msg: impl Into<String>) -> Self {
        Self::Plugin(msg.into())
    }

    /// Create a new server error.
    #[must_use]
    pub fn server(msg: impl Into<String>) -> Self {
        Self::Server(msg.into())
    }

    /// Create a new serialization error.
    #[must_use]
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a new validation error.
    #[must_use]
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a new not found error.
    #[must_use]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a new conflict error.
    #[must_use]
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Create a new internal error.
    #[must_use]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

#[cfg(feature = "orbis-plugin-api")]
impl From<orbis_plugin_api::Error> for Error {
    fn from(e: orbis_plugin_api::Error) -> Self {
        Self::Plugin(e.to_string())
    }
}
