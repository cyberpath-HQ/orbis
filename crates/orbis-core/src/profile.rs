//! Connection profiles for Orbis.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A connection profile stores settings for connecting to a server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Unique identifier for the profile.
    pub id: Uuid,

    /// Human-readable name for the profile.
    pub name: String,

    /// Server URL (for client mode).
    #[serde(default)]
    pub server_url: Option<String>,

    /// Whether this profile is the default.
    #[serde(default)]
    pub is_default: bool,

    /// Whether to use TLS.
    #[serde(default = "default_true")]
    pub use_tls: bool,

    /// Whether to verify TLS certificates.
    #[serde(default = "default_true")]
    pub verify_tls: bool,

    /// Custom CA certificate path.
    #[serde(default)]
    pub ca_cert_path: Option<String>,

    /// Stored authentication token (encrypted).
    #[serde(default)]
    pub auth_token: Option<String>,

    /// Additional custom settings.
    #[serde(default)]
    pub custom: serde_json::Value,

    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Helper for serde default.
const fn default_true() -> bool {
    true
}

impl Profile {
    /// Create a new profile with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            server_url: None,
            is_default: false,
            use_tls: true,
            verify_tls: true,
            ca_cert_path: None,
            auth_token: None,
            custom: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the server URL.
    #[must_use]
    pub fn with_server_url(mut self, url: impl Into<String>) -> Self {
        self.server_url = Some(url.into());
        self
    }

    /// Set as default profile.
    #[must_use]
    pub const fn with_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    /// Set TLS usage.
    #[must_use]
    pub const fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::new("default")
    }
}
