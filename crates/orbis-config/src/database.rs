//! Database configuration.

use crate::Cli;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Database backend type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    /// PostgreSQL backend.
    Postgres,

    /// SQLite backend.
    #[default]
    Sqlite,
}

impl std::str::FromStr for DatabaseBackend {
    type Err = orbis_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgres" | "postgresql" | "pg" => Ok(Self::Postgres),
            "sqlite" | "sqlite3" => Ok(Self::Sqlite),
            _ => Err(orbis_core::Error::config(format!(
                "Invalid database backend: '{}'. Expected 'postgres' or 'sqlite'",
                s
            ))),
        }
    }
}

impl std::fmt::Display for DatabaseBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postgres => write!(f, "postgres"),
            Self::Sqlite => write!(f, "sqlite"),
        }
    }
}

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database backend.
    pub backend: DatabaseBackend,

    /// Database URL (takes precedence over individual settings).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Database host (for PostgreSQL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Database port (for PostgreSQL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Database user (for PostgreSQL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Database password (for PostgreSQL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Database name (for PostgreSQL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Database schema (for PostgreSQL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Database file path (for SQLite).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    /// Maximum connections.
    pub max_connections: u32,

    /// Minimum connections.
    pub min_connections: u32,

    /// Connection timeout.
    pub connect_timeout_ms: u64,

    /// Acquire timeout.
    pub acquire_timeout_ms: u64,

    /// Idle timeout.
    pub idle_timeout_ms: u64,

    /// Max connection lifetime.
    pub max_lifetime_ms: u64,

    /// Run migrations on startup.
    pub run_migrations: bool,
}

impl DatabaseConfig {
    /// Create database config from CLI arguments.
    pub fn from_cli(cli: &Cli, file_config: Option<&DatabaseConfig>) -> Self {
        let backend = cli.db_backend.parse().unwrap_or_else(|_| {
            file_config
                .map(|c| c.backend)
                .unwrap_or(DatabaseBackend::Sqlite)
        });

        Self {
            backend,
            url: cli.db_url.clone().or_else(|| {
                file_config.and_then(|c| c.url.clone())
            }),
            host: cli.db_host.clone().or_else(|| {
                file_config.and_then(|c| c.host.clone())
            }),
            port: cli.db_port.or_else(|| {
                file_config.and_then(|c| c.port)
            }),
            user: cli.db_user.clone().or_else(|| {
                file_config.and_then(|c| c.user.clone())
            }),
            password: cli.db_password.clone().or_else(|| {
                file_config.and_then(|c| c.password.clone())
            }),
            name: cli.db_name.clone().or_else(|| {
                file_config.and_then(|c| c.name.clone())
            }),
            schema: cli.db_schema.clone().or_else(|| {
                file_config.and_then(|c| c.schema.clone())
            }),
            path: cli.db_path.clone().or_else(|| {
                file_config.and_then(|c| c.path.clone())
            }),
            max_connections: cli.db_max_connections,
            min_connections: cli.db_min_connections,
            connect_timeout_ms: cli.db_connect_timeout_ms,
            acquire_timeout_ms: cli.db_acquire_timeout_ms,
            idle_timeout_ms: cli.db_idle_timeout_ms,
            max_lifetime_ms: cli.db_max_lifetime_ms,
            run_migrations: cli.db_run_migrations,
        }
    }

    /// Get the database URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be constructed.
    pub fn database_url(&self) -> orbis_core::Result<String> {
        // If URL is explicitly provided, use it
        if let Some(url) = &self.url {
            return Ok(url.clone());
        }

        match self.backend {
            DatabaseBackend::Postgres => {
                let host = self.host.as_deref().unwrap_or("localhost");
                let port = self.port.unwrap_or(5432);
                let user = self.user.as_deref().ok_or_else(|| {
                    orbis_core::Error::config("Database user is required for PostgreSQL")
                })?;
                let password = self.password.as_deref().ok_or_else(|| {
                    orbis_core::Error::config("Database password is required for PostgreSQL")
                })?;
                let name = self.name.as_deref().ok_or_else(|| {
                    orbis_core::Error::config("Database name is required for PostgreSQL")
                })?;

                let mut url = format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, name);

                if let Some(schema) = &self.schema {
                    url.push_str(&format!("?options=-c%20search_path%3D{}", schema));
                }

                Ok(url)
            }
            DatabaseBackend::Sqlite => {
                let path = self.path.as_ref().ok_or_else(|| {
                    orbis_core::Error::config("Database path is required for SQLite")
                })?;

                Ok(format!("sqlite:{}?mode=rwc", path.display()))
            }
        }
    }

    /// Get the connect timeout as Duration.
    #[must_use]
    pub const fn connect_timeout(&self) -> Duration {
        Duration::from_millis(self.connect_timeout_ms)
    }

    /// Get the acquire timeout as Duration.
    #[must_use]
    pub const fn acquire_timeout(&self) -> Duration {
        Duration::from_millis(self.acquire_timeout_ms)
    }

    /// Get the idle timeout as Duration.
    #[must_use]
    pub const fn idle_timeout(&self) -> Duration {
        Duration::from_millis(self.idle_timeout_ms)
    }

    /// Get the max lifetime as Duration.
    #[must_use]
    pub const fn max_lifetime(&self) -> Duration {
        Duration::from_millis(self.max_lifetime_ms)
    }

    /// Validate the database configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        // Validate that we can construct a URL
        self.database_url()?;

        // Validate pool settings
        if self.min_connections > self.max_connections {
            return Err(orbis_core::Error::config(
                "min_connections cannot be greater than max_connections",
            ));
        }

        Ok(())
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend: DatabaseBackend::Sqlite,
            url: None,
            host: None,
            port: None,
            user: None,
            password: None,
            name: None,
            schema: None,
            path: Some(PathBuf::from("orbis.db")),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_ms: 5000,
            acquire_timeout_ms: 5000,
            idle_timeout_ms: 10000,
            max_lifetime_ms: 60000,
            run_migrations: true,
        }
    }
}
