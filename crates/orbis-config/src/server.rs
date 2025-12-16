//! Server configuration.

use crate::Cli;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host address.
    pub host: String,

    /// Port number.
    pub port: u16,

    /// Server URL (for client mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Request timeout in seconds.
    pub request_timeout_seconds: u64,

    /// Maximum request body size in bytes.
    pub max_body_size: usize,

    /// Enable request logging.
    pub request_logging: bool,

    /// Enable CORS.
    pub cors_enabled: bool,

    /// CORS allowed origins.
    #[serde(default)]
    pub cors_origins: Vec<String>,

    /// Enable compression.
    pub compression: bool,
}

impl ServerConfig {
    /// Create server config from CLI arguments.
    pub fn from_cli(cli: &Cli, file_config: Option<&ServerConfig>) -> Self {
        Self {
            host: cli.server_host.clone(),
            port: cli.server_port,
            url: cli.server_url.clone().or_else(|| {
                file_config.and_then(|c| c.url.clone())
            }),
            request_timeout_seconds: cli.request_timeout,
            max_body_size: file_config
                .map(|c| c.max_body_size)
                .unwrap_or(10 * 1024 * 1024), // 10MB
            request_logging: file_config.is_some_and(|c| c.request_logging),
            cors_enabled: file_config.is_some_and(|c| c.cors_enabled),
            cors_origins: file_config
                .map(|c| c.cors_origins.clone())
                .unwrap_or_default(),
            compression: file_config.map(|c| c.compression).unwrap_or(true),
        }
    }

    /// Get the socket address.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn socket_addr(&self) -> orbis_core::Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e| orbis_core::Error::config(format!("Invalid server address: {}", e)))
    }

    /// Validate the server configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        // Validate socket address
        self.socket_addr()?;

        // Validate timeout
        if self.request_timeout_seconds == 0 {
            return Err(orbis_core::Error::config(
                "Request timeout must be greater than 0",
            ));
        }

        Ok(())
    }

    /// Get the base URL for this server.
    #[must_use]
    pub fn base_url(&self, use_tls: bool) -> String {
        let scheme = if use_tls { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8000,
            url: None,
            request_timeout_seconds: 30,
            max_body_size: 10 * 1024 * 1024, // 10MB
            request_logging: true,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
            compression: true,
        }
    }
}
