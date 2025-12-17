//! # Orbis Database
//!
//! Database layer for Orbis using SQLx with support for PostgreSQL and SQLite.
//! Provides migration management and a unified interface for both backends.

mod connection;
mod migrations;
mod pool;
mod repository;

pub use connection::{Connection, DatabaseConnection, QueryExecutor};
pub use migrations::{run_migrations, MigrationRunner};
pub use pool::{create_pool, DatabasePool};
pub use repository::{BaseRepository, Repository};

use orbis_config::DatabaseConfig;
use std::sync::Arc;

/// Database context holding the connection pool and configuration.
#[derive(Clone)]
pub struct Database {
    pool: DatabasePool,
    config: Arc<DatabaseConfig>,
}

impl Database {
    /// Create a new database instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection pool cannot be created.
    pub async fn new(config: DatabaseConfig) -> orbis_core::Result<Self> {
        let pool = create_pool(&config).await?;
        Ok(Self {
            pool,
            config: Arc::new(config),
        })
    }

    /// Get a reference to the connection pool.
    #[must_use]
    pub const fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    /// Get a reference to the configuration.
    #[must_use]
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Run pending migrations.
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail.
    pub async fn migrate(&self) -> orbis_core::Result<()> {
        run_migrations(&self.pool).await
    }

    /// Check database connectivity.
    ///
    /// # Errors
    ///
    /// Returns an error if the database is not reachable.
    pub async fn health_check(&self) -> orbis_core::Result<()> {
        match &self.pool {
            DatabasePool::Postgres(pool) => {
                sqlx::query("SELECT 1")
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(format!("Health check failed: {}", e)))?;
            }
            DatabasePool::Sqlite(pool) => {
                sqlx::query("SELECT 1")
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(format!("Health check failed: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Close the database connection pool.
    pub async fn close(&self) {
        match &self.pool {
            DatabasePool::Postgres(pool) => pool.close().await,
            DatabasePool::Sqlite(pool) => pool.close().await,
        }
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("backend", &self.config.backend)
            .finish()
    }
}
