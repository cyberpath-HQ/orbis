//! # Orbis Config
//!
//! Configuration management for Orbis supporting CLI arguments and environment variables.
//!
//! All configuration options can be set via:
//! - Command line arguments (highest priority)
//! - Environment variables (prefixed with `ORBIS_`)
//! - Configuration file (lowest priority)

mod cli;
mod database;
mod logging;
mod server;
mod tls;

pub use cli::{Cli, Commands};
pub use database::{DatabaseConfig, DatabaseBackend};
pub use logging::{LogConfig, LogFormat};
pub use server::ServerConfig;
pub use tls::TlsConfig;

use orbis_core::{AppMode, RunMode};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Global configuration instance.
static CONFIG: once_cell::sync::OnceCell<Arc<RwLock<Config>>> = once_cell::sync::OnceCell::new();

/// Get the global configuration instance.
///
/// # Panics
///
/// Panics if configuration has not been initialized.
#[must_use]
pub fn get_config() -> Arc<RwLock<Config>> {
    CONFIG
        .get()
        .expect("Configuration not initialized. Call Config::init() first.")
        .clone()
}

/// Initialize the global configuration from CLI args.
///
/// # Errors
///
/// Returns an error if configuration is invalid.
pub fn init_config() -> orbis_core::Result<Arc<RwLock<Config>>> {
    use clap::Parser;

    // Load .env file if present
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();
    let config = Config::from_cli(&cli)?;

    let config = Arc::new(RwLock::new(config));
    CONFIG
        .set(config.clone())
        .map_err(|_| orbis_core::Error::config("Configuration already initialized"))?;

    Ok(config)
}

/// Complete application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application mode (standalone or client-server).
    pub mode: AppMode,

    /// Run mode (server or client) - only relevant in client-server mode.
    pub run_mode: RunMode,

    /// Server configuration.
    pub server: ServerConfig,

    /// Database configuration.
    pub database: DatabaseConfig,

    /// TLS configuration.
    pub tls: TlsConfig,

    /// Logging configuration.
    pub log: LogConfig,

    /// Path to configuration file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<PathBuf>,

    /// Path to profiles directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles_dir: Option<PathBuf>,

    /// Path to plugins directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins_dir: Option<PathBuf>,

    /// Path to data directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,

    /// Active profile name (for client mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_profile: Option<String>,

    /// Whether authentication is enabled (mandatory in client-server mode).
    pub auth_enabled: bool,

    /// JWT secret for token signing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_secret: Option<String>,

    /// JWT token expiry in seconds.
    pub jwt_expiry_seconds: u64,
}

impl Config {
    /// Create configuration from CLI arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration is invalid.
    pub fn from_cli(cli: &Cli) -> orbis_core::Result<Self> {
        // Load config file if specified
        let file_config = if let Some(path) = &cli.config {
            Some(Self::load_from_file(path)?)
        } else {
            None
        };

        // Determine app mode
        let mode = cli.mode.parse().unwrap_or_else(|_| {
            file_config
                .as_ref()
                .map(|c| c.mode)
                .unwrap_or(AppMode::Standalone)
        });

        // Determine run mode
        let run_mode = cli.run_mode.parse().unwrap_or_else(|_| {
            file_config
                .as_ref()
                .map(|c| c.run_mode)
                .unwrap_or(RunMode::Client)
        });

        // Auth is mandatory in client-server mode
        let auth_enabled = if mode.requires_auth() {
            true
        } else {
            cli.auth_enabled
                || file_config.as_ref().is_some_and(|c| c.auth_enabled)
        };

        Ok(Self {
            mode,
            run_mode,
            server: ServerConfig::from_cli(cli, file_config.as_ref().map(|c| &c.server)),
            database: DatabaseConfig::from_cli(cli, file_config.as_ref().map(|c| &c.database)),
            tls: TlsConfig::from_cli(cli, file_config.as_ref().map(|c| &c.tls)),
            log: LogConfig::from_cli(cli, file_config.as_ref().map(|c| &c.log)),
            config_file: cli.config.clone(),
            profiles_dir: cli.profiles_dir.clone().or_else(|| {
                file_config
                    .as_ref()
                    .and_then(|c| c.profiles_dir.clone())
            }),
            plugins_dir: cli.plugins_dir.clone().or_else(|| {
                file_config
                    .as_ref()
                    .and_then(|c| c.plugins_dir.clone())
            }),
            data_dir: cli.data_dir.clone().or_else(|| {
                file_config.as_ref().and_then(|c| c.data_dir.clone())
            }),
            active_profile: cli.profile.clone().or_else(|| {
                file_config
                    .as_ref()
                    .and_then(|c| c.active_profile.clone())
            }),
            auth_enabled,
            jwt_secret: cli.jwt_secret.clone().or_else(|| {
                file_config
                    .as_ref()
                    .and_then(|c| c.jwt_secret.clone())
            }),
            jwt_expiry_seconds: cli.jwt_expiry_seconds.unwrap_or_else(|| {
                file_config
                    .as_ref()
                    .map(|c| c.jwt_expiry_seconds)
                    .unwrap_or(3600)
            }),
        })
    }

    /// Load configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_from_file(path: &PathBuf) -> orbis_core::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            orbis_core::Error::config(format!("Failed to read config file: {}", e))
        })?;

        toml::from_str(&content)
            .map_err(|e| orbis_core::Error::config(format!("Failed to parse config file: {}", e)))
    }

    /// Save configuration to a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save_to_file(&self, path: &PathBuf) -> orbis_core::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| orbis_core::Error::config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| orbis_core::Error::config(format!("Failed to write config file: {}", e)))
    }

    /// Validate the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        // In client-server mode, JWT secret is required
        if self.mode.is_client_server() && self.jwt_secret.is_none() {
            return Err(orbis_core::Error::config(
                "JWT secret is required in client-server mode. Set ORBIS_JWT_SECRET or --jwt-secret",
            ));
        }

        // Validate server config
        self.server.validate()?;

        // Validate database config
        self.database.validate()?;

        // Validate TLS config
        self.tls.validate()?;

        Ok(())
    }

    /// Check if TLS is enabled.
    #[must_use]
    pub const fn is_tls_enabled(&self) -> bool {
        self.tls.enabled
    }

    /// Check if running as server.
    #[must_use]
    pub fn is_server(&self) -> bool {
        self.mode.is_standalone() || self.run_mode.is_server()
    }

    /// Check if running as client.
    #[must_use]
    pub fn is_client(&self) -> bool {
        self.mode.is_client_server() && self.run_mode.is_client()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: AppMode::Standalone,
            run_mode: RunMode::Client,
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            tls: TlsConfig::default(),
            log: LogConfig::default(),
            config_file: None,
            profiles_dir: None,
            plugins_dir: None,
            data_dir: None,
            active_profile: None,
            auth_enabled: false,
            jwt_secret: None,
            jwt_expiry_seconds: 3600,
        }
    }
}
