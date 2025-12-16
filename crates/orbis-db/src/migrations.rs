//! Database migrations management.

use crate::DatabasePool;

/// Run embedded migrations.
///
/// # Errors
///
/// Returns an error if migrations fail.
pub async fn run_migrations(pool: &DatabasePool) -> orbis_core::Result<()> {
    tracing::info!("Running database migrations...");

    match pool {
        DatabasePool::Postgres(pool) => {
            sqlx::migrate!("./migrations/postgres")
                .run(pool)
                .await
                .map_err(|e| orbis_core::Error::database(format!("Migration failed: {}", e)))?;
        }
        DatabasePool::Sqlite(pool) => {
            sqlx::migrate!("./migrations/sqlite")
                .run(pool)
                .await
                .map_err(|e| orbis_core::Error::database(format!("Migration failed: {}", e)))?;
        }
    }

    tracing::info!("Database migrations completed");
    Ok(())
}

/// Migration runner for manual migration management.
pub struct MigrationRunner<'a> {
    pool: &'a DatabasePool,
}

impl<'a> MigrationRunner<'a> {
    /// Create a new migration runner.
    #[must_use]
    pub const fn new(pool: &'a DatabasePool) -> Self {
        Self { pool }
    }

    /// Run all pending migrations.
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail.
    pub async fn run(&self) -> orbis_core::Result<()> {
        run_migrations(self.pool).await
    }

    /// Get the current migration version.
    ///
    /// # Errors
    ///
    /// Returns an error if the version cannot be retrieved.
    pub async fn current_version(&self) -> orbis_core::Result<Option<i64>> {
        let query = "SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1";

        match self.pool {
            DatabasePool::Postgres(pool) => {
                let result: Option<(i64,)> = sqlx::query_as(query)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(result.map(|(v,)| v))
            }
            DatabasePool::Sqlite(pool) => {
                let result: Option<(i64,)> = sqlx::query_as(query)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(result.map(|(v,)| v))
            }
        }
    }

    /// List all applied migrations.
    ///
    /// # Errors
    ///
    /// Returns an error if migrations cannot be listed.
    pub async fn list_applied(&self) -> orbis_core::Result<Vec<AppliedMigration>> {
        let query = "SELECT version, description, installed_on, success, checksum FROM _sqlx_migrations ORDER BY version";

        match self.pool {
            DatabasePool::Postgres(pool) => {
                let rows: Vec<(i64, String, chrono::DateTime<chrono::Utc>, bool, Vec<u8>)> =
                    sqlx::query_as(query)
                        .fetch_all(pool)
                        .await
                        .map_err(|e| orbis_core::Error::database(e.to_string()))?;

                Ok(rows
                    .into_iter()
                    .map(|(version, description, installed_on, success, checksum)| {
                        AppliedMigration {
                            version,
                            description,
                            installed_on,
                            success,
                            checksum,
                        }
                    })
                    .collect())
            }
            DatabasePool::Sqlite(pool) => {
                let rows: Vec<(i64, String, chrono::DateTime<chrono::Utc>, bool, Vec<u8>)> =
                    sqlx::query_as(query)
                        .fetch_all(pool)
                        .await
                        .map_err(|e| orbis_core::Error::database(e.to_string()))?;

                Ok(rows
                    .into_iter()
                    .map(|(version, description, installed_on, success, checksum)| {
                        AppliedMigration {
                            version,
                            description,
                            installed_on,
                            success,
                            checksum,
                        }
                    })
                    .collect())
            }
        }
    }
}

/// An applied migration record.
#[derive(Debug, Clone)]
pub struct AppliedMigration {
    /// Migration version (timestamp).
    pub version: i64,

    /// Migration description.
    pub description: String,

    /// When the migration was applied.
    pub installed_on: chrono::DateTime<chrono::Utc>,

    /// Whether the migration succeeded.
    pub success: bool,

    /// Migration checksum.
    pub checksum: Vec<u8>,
}
