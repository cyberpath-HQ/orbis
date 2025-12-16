//! Database connection abstraction.

use crate::DatabasePool;
use async_trait::async_trait;
use sqlx::{PgPool, SqlitePool};

/// A database connection that can execute queries.
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// Execute a raw SQL query.
    async fn execute_raw(&self, query: &str) -> orbis_core::Result<u64>;

    /// Check if the connection is valid.
    async fn is_valid(&self) -> bool;
}

/// Unified connection wrapper.
pub enum Connection<'a> {
    /// PostgreSQL connection.
    Postgres(&'a PgPool),

    /// SQLite connection.
    Sqlite(&'a SqlitePool),
}

impl<'a> Connection<'a> {
    /// Create a connection from a pool.
    #[must_use]
    pub fn from_pool(pool: &'a DatabasePool) -> Self {
        match pool {
            DatabasePool::Postgres(p) => Self::Postgres(p),
            DatabasePool::Sqlite(p) => Self::Sqlite(p),
        }
    }
}

#[async_trait]
impl DatabaseConnection for Connection<'_> {
    async fn execute_raw(&self, query: &str) -> orbis_core::Result<u64> {
        match self {
            Self::Postgres(pool) => {
                let result = sqlx::query(query)
                    .execute(*pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(result.rows_affected())
            }
            Self::Sqlite(pool) => {
                let result = sqlx::query(query)
                    .execute(*pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(result.rows_affected())
            }
        }
    }

    async fn is_valid(&self) -> bool {
        match self {
            Self::Postgres(pool) => sqlx::query("SELECT 1").execute(*pool).await.is_ok(),
            Self::Sqlite(pool) => sqlx::query("SELECT 1").execute(*pool).await.is_ok(),
        }
    }
}

/// Helper trait for executing queries on either backend.
#[async_trait]
#[allow(dead_code)]
pub trait QueryExecutor {
    /// Execute a query and return the number of affected rows.
    async fn execute(&self, query: &str) -> orbis_core::Result<u64>;
}

#[async_trait]
impl QueryExecutor for DatabasePool {
    async fn execute(&self, query: &str) -> orbis_core::Result<u64> {
        match self {
            Self::Postgres(pool) => {
                let result = sqlx::query(query)
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(result.rows_affected())
            }
            Self::Sqlite(pool) => {
                let result = sqlx::query(query)
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(result.rows_affected())
            }
        }
    }
}
