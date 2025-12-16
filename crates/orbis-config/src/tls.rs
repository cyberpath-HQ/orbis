//! TLS configuration.

use crate::Cli;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TLS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS.
    pub enabled: bool,

    /// Path to TLS certificate file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cert_path: Option<PathBuf>,

    /// Path to TLS private key file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_path: Option<PathBuf>,

    /// Path to CA certificate file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_path: Option<PathBuf>,

    /// Verify TLS certificates.
    pub verify: bool,

    /// Minimum TLS version.
    #[serde(default = "default_min_version")]
    pub min_version: String,
}

fn default_min_version() -> String {
    "1.2".to_string()
}

impl TlsConfig {
    /// Create TLS config from CLI arguments.
    pub fn from_cli(cli: &Cli, file_config: Option<&TlsConfig>) -> Self {
        let enabled = cli.tls_enabled
            || cli.tls_cert_path.is_some()
            || file_config.is_some_and(|c| c.enabled);

        Self {
            enabled,
            cert_path: cli.tls_cert_path.clone().or_else(|| {
                file_config.and_then(|c| c.cert_path.clone())
            }),
            key_path: cli.tls_key_path.clone().or_else(|| {
                file_config.and_then(|c| c.key_path.clone())
            }),
            ca_path: cli.tls_ca_path.clone().or_else(|| {
                file_config.and_then(|c| c.ca_path.clone())
            }),
            verify: cli.tls_verify,
            min_version: file_config
                .map(|c| c.min_version.clone())
                .unwrap_or_else(default_min_version),
        }
    }

    /// Validate the TLS configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        if self.enabled {
            // Certificate and key are required when TLS is enabled
            if self.cert_path.is_none() {
                return Err(orbis_core::Error::config(
                    "TLS certificate path is required when TLS is enabled",
                ));
            }

            if self.key_path.is_none() {
                return Err(orbis_core::Error::config(
                    "TLS key path is required when TLS is enabled",
                ));
            }

            // Check that files exist
            if let Some(cert_path) = &self.cert_path {
                if !cert_path.exists() {
                    return Err(orbis_core::Error::config(format!(
                        "TLS certificate file not found: {}",
                        cert_path.display()
                    )));
                }
            }

            if let Some(key_path) = &self.key_path {
                if !key_path.exists() {
                    return Err(orbis_core::Error::config(format!(
                        "TLS key file not found: {}",
                        key_path.display()
                    )));
                }
            }

            if let Some(ca_path) = &self.ca_path {
                if !ca_path.exists() {
                    return Err(orbis_core::Error::config(format!(
                        "TLS CA certificate file not found: {}",
                        ca_path.display()
                    )));
                }
            }

            // Validate min version
            match self.min_version.as_str() {
                "1.2" | "1.3" => {}
                _ => {
                    return Err(orbis_core::Error::config(format!(
                        "Invalid TLS min version: '{}'. Expected '1.2' or '1.3'",
                        self.min_version
                    )));
                }
            }
        }

        Ok(())
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            ca_path: None,
            verify: true,
            min_version: default_min_version(),
        }
    }
}
