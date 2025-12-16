//! Session management.

use chrono::{DateTime, Duration, Utc};
use orbis_db::{Database, DatabasePool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User session information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID.
    pub id: Uuid,

    /// User ID.
    pub user_id: Uuid,

    /// Token hash (for refresh token lookup).
    pub token_hash: String,

    /// User agent.
    pub user_agent: Option<String>,

    /// IP address.
    pub ip_address: Option<String>,

    /// Expiration time.
    pub expires_at: DateTime<Utc>,

    /// Creation time.
    pub created_at: DateTime<Utc>,
}

impl Session {
    /// Check if the session has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    /// Get remaining time until expiration.
    #[must_use]
    pub fn remaining_time(&self) -> Duration {
        self.expires_at - Utc::now()
    }
}

/// Session service for managing user sessions.
#[derive(Clone)]
pub struct SessionService {
    db: Database,
}

impl SessionService {
    /// Create a new session service.
    #[must_use]
    pub const fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a new session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be created.
    pub async fn create(
        &self,
        user_id: Uuid,
        token: &str,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
        expiry_seconds: u64,
    ) -> orbis_core::Result<Session> {
        let id = Uuid::now_v7();
        let token_hash = Self::hash_token(token);
        let now = Utc::now();
        let expires_at = now + Duration::seconds(expiry_seconds as i64 * 24 * 7); // 7 days for refresh token

        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                sqlx::query(
                    r"
                    INSERT INTO sessions (id, user_id, token_hash, user_agent, ip_address, expires_at, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ",
                )
                .bind(id)
                .bind(user_id)
                .bind(&token_hash)
                .bind(user_agent)
                .bind(ip_address)
                .bind(expires_at)
                .bind(now)
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
            DatabasePool::Sqlite(pool) => {
                sqlx::query(
                    r"
                    INSERT INTO sessions (id, user_id, token_hash, user_agent, ip_address, expires_at, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ",
                )
                .bind(id.to_string())
                .bind(user_id.to_string())
                .bind(&token_hash)
                .bind(user_agent)
                .bind(ip_address)
                .bind(expires_at.to_rfc3339())
                .bind(now.to_rfc3339())
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
        }

        Ok(Session {
            id,
            user_id,
            token_hash,
            user_agent: user_agent.map(String::from),
            ip_address: ip_address.map(String::from),
            expires_at,
            created_at: now,
        })
    }

    /// Find a session by token.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn find_by_token(&self, token: &str) -> orbis_core::Result<Option<Session>> {
        let token_hash = Self::hash_token(token);

        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                let row: Option<(Uuid, Uuid, String, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>)> =
                    sqlx::query_as(
                        "SELECT id, user_id, token_hash, user_agent, ip_address, expires_at, created_at 
                        FROM sessions WHERE token_hash = $1",
                    )
                    .bind(&token_hash)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;

                Ok(row.map(|(id, user_id, token_hash, user_agent, ip_address, expires_at, created_at)| {
                    Session {
                        id,
                        user_id,
                        token_hash,
                        user_agent,
                        ip_address,
                        expires_at,
                        created_at,
                    }
                }))
            }
            DatabasePool::Sqlite(pool) => {
                let row: Option<(String, String, String, Option<String>, Option<String>, String, String)> =
                    sqlx::query_as(
                        "SELECT id, user_id, token_hash, user_agent, ip_address, expires_at, created_at 
                        FROM sessions WHERE token_hash = $1",
                    )
                    .bind(&token_hash)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;

                Ok(row.map(|(id, user_id, token_hash, user_agent, ip_address, expires_at, created_at)| {
                    Session {
                        id: id.parse().unwrap_or_default(),
                        user_id: user_id.parse().unwrap_or_default(),
                        token_hash,
                        user_agent,
                        ip_address,
                        expires_at: DateTime::parse_from_rfc3339(&expires_at)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                        created_at: DateTime::parse_from_rfc3339(&created_at)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    }
                }))
            }
        }
    }

    /// Delete a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the deletion fails.
    pub async fn delete(&self, id: Uuid) -> orbis_core::Result<()> {
        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                sqlx::query("DELETE FROM sessions WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
            DatabasePool::Sqlite(pool) => {
                sqlx::query("DELETE FROM sessions WHERE id = $1")
                    .bind(id.to_string())
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Delete all sessions for a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the deletion fails.
    pub async fn delete_all_for_user(&self, user_id: Uuid) -> orbis_core::Result<()> {
        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                sqlx::query("DELETE FROM sessions WHERE user_id = $1")
                    .bind(user_id)
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
            DatabasePool::Sqlite(pool) => {
                sqlx::query("DELETE FROM sessions WHERE user_id = $1")
                    .bind(user_id.to_string())
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Clean up expired sessions.
    ///
    /// # Errors
    ///
    /// Returns an error if the cleanup fails.
    pub async fn cleanup_expired(&self) -> orbis_core::Result<u64> {
        let now = Utc::now();

        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                let result = sqlx::query("DELETE FROM sessions WHERE expires_at < $1")
                    .bind(now)
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(result.rows_affected())
            }
            DatabasePool::Sqlite(pool) => {
                let result = sqlx::query("DELETE FROM sessions WHERE expires_at < $1")
                    .bind(now.to_rfc3339())
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(result.rows_affected())
            }
        }
    }

    /// Hash a token for storage.
    fn hash_token(token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
