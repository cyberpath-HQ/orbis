//! Database connection pool management.

use orbis_config::{DatabaseBackend, DatabaseConfig};
use sqlx::{PgPool, Sqlite, SqlitePool, migrate::MigrateDatabase as _, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};

/// Unified database pool supporting multiple backends.
#[derive(Clone)]
pub enum DatabasePool {
    /// PostgreSQL connection pool.
    Postgres(PgPool),

    /// SQLite connection pool.
    Sqlite(SqlitePool),
}

impl DatabasePool {
    /// Get the backend type.
    #[must_use]
    pub const fn backend(&self) -> DatabaseBackend {
        match self {
            Self::Postgres(_) => DatabaseBackend::Postgres,
            Self::Sqlite(_) => DatabaseBackend::Sqlite,
        }
    }

    /// Check if this is a PostgreSQL pool.
    #[must_use]
    pub const fn is_postgres(&self) -> bool {
        matches!(self, Self::Postgres(_))
    }

    /// Check if this is a SQLite pool.
    #[must_use]
    pub const fn is_sqlite(&self) -> bool {
        matches!(self, Self::Sqlite(_))
    }

    /// Get the PostgreSQL pool, if this is a PostgreSQL connection.
    #[must_use]
    pub const fn as_postgres(&self) -> Option<&PgPool> {
        match self {
            Self::Postgres(pool) => Some(pool),
            Self::Sqlite(_) => None,
        }
    }

    /// Get the SQLite pool, if this is a SQLite connection.
    #[must_use]
    pub const fn as_sqlite(&self) -> Option<&SqlitePool> {
        match self {
            Self::Postgres(_) => None,
            Self::Sqlite(pool) => Some(pool),
        }
    }
}

/// Create a database connection pool based on configuration.
///
/// # Errors
///
/// Returns an error if the pool cannot be created.
pub async fn create_pool(config: &DatabaseConfig) -> orbis_core::Result<DatabasePool> {
    let url = config.database_url()?;

    match config.backend {
        DatabaseBackend::Postgres => {
            tracing::info!("Connecting to PostgreSQL database...");

            let pool = PgPoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(config.acquire_timeout())
                .idle_timeout(Some(config.idle_timeout()))
                .max_lifetime(Some(config.max_lifetime()))
                .connect(&url)
                .await
                .map_err(|e| {
                    orbis_core::Error::database(format!("Failed to connect to PostgreSQL: {}", e))
                })?;

            tracing::info!("Connected to PostgreSQL database");
            Ok(DatabasePool::Postgres(pool))
        }
        DatabaseBackend::Sqlite => {
            tracing::info!("Connecting to SQLite database...");

            // Ensure the database file's parent directory exists
            if let Some(path) = &config.path {
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            orbis_core::Error::database(format!(
                                "Failed to create database directory: {}",
                                e
                            ))
                        })?;
                    }
                }
            }

            if !Sqlite::database_exists(&url).await.unwrap_or(false) {
                tracing::info!("SQLite database does not exist, creating new database...");
                Sqlite::create_database(&url).await.map_err(|e| {
                    orbis_core::Error::database(format!(
                        "Failed to create SQLite database: {}",
                        e
                    ))
                })?;
            }

            let pool = SqlitePoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(config.acquire_timeout())
                .idle_timeout(Some(config.idle_timeout()))
                .max_lifetime(Some(config.max_lifetime()))
                .connect(&url)
                .await
                .map_err(|e| {
                    orbis_core::Error::database(format!("Failed to connect to SQLite: {}", e))
                })?;

            tracing::info!("Connected to SQLite database");
            Ok(DatabasePool::Sqlite(pool))
        }
    }
}
