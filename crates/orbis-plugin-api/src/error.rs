//! Error types for plugin development.

/// Plugin API error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid plugin configuration.
    #[error("Invalid plugin: {0}")]
    InvalidPlugin(String),

    /// Invalid manifest.
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    /// Invalid UI schema.
    #[error("Invalid UI schema: {0}")]
    InvalidSchema(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Error {
    /// Create a new plugin error.
    #[must_use]
    pub fn plugin<S: Into<String>>(msg: S) -> Self {
        Self::InvalidPlugin(msg.into())
    }

    /// Create a new manifest error.
    #[must_use]
    pub fn manifest<S: Into<String>>(msg: S) -> Self {
        Self::InvalidManifest(msg.into())
    }

    /// Create a new schema error.
    #[must_use]
    pub fn schema<S: Into<String>>(msg: S) -> Self {
        Self::InvalidSchema(msg.into())
    }

    /// Create a new validation error.
    #[must_use]
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }
}

/// Result type for plugin operations.
pub type Result<T> = std::result::Result<T, Error>;
