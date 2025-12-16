//! Repository trait for database access patterns.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::DatabasePool;

/// Base repository trait for CRUD operations.
#[async_trait]
pub trait Repository<T>: Send + Sync
where
    T: Send + Sync + Serialize + DeserializeOwned,
{
    /// The error type for this repository.
    type Error: std::error::Error + Send + Sync;

    /// Find an entity by its ID.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<T>, Self::Error>;

    /// Find all entities.
    async fn find_all(&self) -> Result<Vec<T>, Self::Error>;

    /// Find entities with pagination.
    async fn find_paginated(&self, offset: u32, limit: u32) -> Result<Vec<T>, Self::Error>;

    /// Count all entities.
    async fn count(&self) -> Result<u64, Self::Error>;

    /// Create a new entity.
    async fn create(&self, entity: &T) -> Result<T, Self::Error>;

    /// Update an existing entity.
    async fn update(&self, entity: &T) -> Result<T, Self::Error>;

    /// Delete an entity by its ID.
    async fn delete(&self, id: Uuid) -> Result<bool, Self::Error>;

    /// Check if an entity exists.
    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error>;
}

/// Base repository implementation helper.
#[allow(dead_code)]
pub struct BaseRepository {
    pool: DatabasePool,
}

impl BaseRepository {
    /// Create a new base repository.
    #[must_use]
    #[allow(dead_code)]
    pub const fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Get a reference to the pool.
    #[must_use]
    #[allow(dead_code)]
    pub const fn pool(&self) -> &DatabasePool {
        &self.pool
    }
}

impl Clone for BaseRepository {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
