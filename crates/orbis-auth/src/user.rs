//! User management.

use chrono::{DateTime, Utc};
use orbis_db::{Database, DatabasePool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID.
    pub id: Uuid,

    /// Username.
    pub username: String,

    /// Email address.
    pub email: String,

    /// Password hash (not serialized).
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Display name.
    pub display_name: Option<String>,

    /// Whether the user is active.
    pub is_active: bool,

    /// Whether the user is an admin.
    pub is_admin: bool,

    /// Creation time.
    pub created_at: DateTime<Utc>,

    /// Last update time.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new user.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUser {
    /// Username.
    pub username: String,

    /// Email address.
    pub email: String,

    /// Password (plain text, will be hashed).
    pub password: String,

    /// Display name.
    pub display_name: Option<String>,

    /// Whether the user is an admin.
    #[serde(default)]
    pub is_admin: bool,
}

/// User service for managing users.
#[derive(Clone)]
pub struct UserService {
    db: Database,
}

impl UserService {
    /// Create a new user service.
    #[must_use]
    pub const fn new(db: Database) -> Self {
        Self { db }
    }

    /// Find a user by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn find_by_id(&self, id: Uuid) -> orbis_core::Result<Option<User>> {
        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                let row: Option<(
                    Uuid,
                    String,
                    String,
                    String,
                    Option<String>,
                    bool,
                    bool,
                    DateTime<Utc>,
                    DateTime<Utc>,
                )> = sqlx::query_as(
                    "SELECT id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at 
                    FROM users WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;

                Ok(row.map(
                    |(id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at)| {
                        User {
                            id,
                            username,
                            email,
                            password_hash,
                            display_name,
                            is_active,
                            is_admin,
                            created_at,
                            updated_at,
                        }
                    },
                ))
            }
            DatabasePool::Sqlite(pool) => {
                let row: Option<(
                    String,
                    String,
                    String,
                    String,
                    Option<String>,
                    i32,
                    i32,
                    String,
                    String,
                )> = sqlx::query_as(
                    "SELECT id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at 
                    FROM users WHERE id = $1",
                )
                .bind(id.to_string())
                .fetch_optional(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;

                Ok(row.map(
                    |(id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at)| {
                        User {
                            id: id.parse().unwrap_or_default(),
                            username,
                            email,
                            password_hash,
                            display_name,
                            is_active: is_active != 0,
                            is_admin: is_admin != 0,
                            created_at: DateTime::parse_from_rfc3339(&created_at)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now()),
                            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now()),
                        }
                    },
                ))
            }
        }
    }

    /// Find a user by username or email.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn find_by_username_or_email(
        &self,
        username_or_email: &str,
    ) -> orbis_core::Result<Option<User>> {
        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                let row: Option<(
                    Uuid,
                    String,
                    String,
                    String,
                    Option<String>,
                    bool,
                    bool,
                    DateTime<Utc>,
                    DateTime<Utc>,
                )> = sqlx::query_as(
                    "SELECT id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at 
                    FROM users WHERE username = $1 OR email = $1",
                )
                .bind(username_or_email)
                .fetch_optional(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;

                Ok(row.map(
                    |(id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at)| {
                        User {
                            id,
                            username,
                            email,
                            password_hash,
                            display_name,
                            is_active,
                            is_admin,
                            created_at,
                            updated_at,
                        }
                    },
                ))
            }
            DatabasePool::Sqlite(pool) => {
                let row: Option<(
                    String,
                    String,
                    String,
                    String,
                    Option<String>,
                    i32,
                    i32,
                    String,
                    String,
                )> = sqlx::query_as(
                    "SELECT id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at 
                    FROM users WHERE username = $1 OR email = $1",
                )
                .bind(username_or_email)
                .fetch_optional(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;

                Ok(row.map(
                    |(id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at)| {
                        User {
                            id: id.parse().unwrap_or_default(),
                            username,
                            email,
                            password_hash,
                            display_name,
                            is_active: is_active != 0,
                            is_admin: is_admin != 0,
                            created_at: DateTime::parse_from_rfc3339(&created_at)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now()),
                            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now()),
                        }
                    },
                ))
            }
        }
    }

    /// Create a new user.
    ///
    /// # Errors
    ///
    /// Returns an error if the user cannot be created.
    pub async fn create(&self, data: CreateUser, password_hash: String) -> orbis_core::Result<User> {
        let id = Uuid::now_v7();
        let now = Utc::now();

        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                sqlx::query(
                    r"
                    INSERT INTO users (id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, TRUE, $6, $7, $7)
                    ",
                )
                .bind(id)
                .bind(&data.username)
                .bind(&data.email)
                .bind(&password_hash)
                .bind(&data.display_name)
                .bind(data.is_admin)
                .bind(now)
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
            DatabasePool::Sqlite(pool) => {
                sqlx::query(
                    r"
                    INSERT INTO users (id, username, email, password_hash, display_name, is_active, is_admin, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, 1, $6, $7, $7)
                    ",
                )
                .bind(id.to_string())
                .bind(&data.username)
                .bind(&data.email)
                .bind(&password_hash)
                .bind(&data.display_name)
                .bind(if data.is_admin { 1 } else { 0 })
                .bind(now.to_rfc3339())
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
        }

        Ok(User {
            id,
            username: data.username,
            email: data.email,
            password_hash,
            display_name: data.display_name,
            is_active: true,
            is_admin: data.is_admin,
            created_at: now,
            updated_at: now,
        })
    }

    /// Check if a username exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn username_exists(&self, username: &str) -> orbis_core::Result<bool> {
        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                let count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = $1")
                        .bind(username)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(count.0 > 0)
            }
            DatabasePool::Sqlite(pool) => {
                let count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = $1")
                        .bind(username)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(count.0 > 0)
            }
        }
    }

    /// Check if an email exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn email_exists(&self, email: &str) -> orbis_core::Result<bool> {
        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
                    .bind(email)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(count.0 > 0)
            }
            DatabasePool::Sqlite(pool) => {
                let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
                    .bind(email)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(count.0 > 0)
            }
        }
    }

    /// Count total users.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn count(&self) -> orbis_core::Result<u64> {
        match self.db.pool() {
            DatabasePool::Postgres(pool) => {
                let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(count.0 as u64)
            }
            DatabasePool::Sqlite(pool) => {
                let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                Ok(count.0 as u64)
            }
        }
    }
}
