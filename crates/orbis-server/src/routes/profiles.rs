//! Profile management routes.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::error::ServerResult;
use crate::extractors::AuthenticatedUser;
use crate::state::AppState;

/// Create profiles router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/profiles", get(list_profiles))
        .route("/profiles", post(create_profile))
        .route("/profiles/{id}", get(get_profile))
        .route("/profiles/{id}", put(update_profile))
        .route("/profiles/{id}", delete(delete_profile))
        .route("/profiles/{id}/default", post(set_default_profile))
}

/// Create profile request.
#[derive(Debug, Deserialize)]
struct CreateProfileRequest {
    name: String,
    server_url: Option<String>,
    use_tls: Option<bool>,
    is_default: Option<bool>,
}

/// List user's profiles.
async fn list_profiles(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    let db = state.db();
    
    let profiles = match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            let rows = sqlx::query(
                "SELECT id, name, server_url, is_default, use_tls, created_at, updated_at 
                 FROM profiles WHERE user_id = $1 ORDER BY name"
            )
            .bind(user.user_id)
            .fetch_all(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;

            rows.into_iter()
                .map(|row| {
                    json!({
                        "id": row.get::<Uuid, _>("id").to_string(),
                        "name": row.get::<String, _>("name"),
                        "server_url": row.get::<Option<String>, _>("server_url"),
                        "is_default": row.get::<bool, _>("is_default"),
                        "use_tls": row.get::<bool, _>("use_tls"),
                        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                        "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339()
                    })
                })
                .collect::<Vec<_>>()
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            let rows = sqlx::query(
                "SELECT id, name, server_url, is_default, use_tls, created_at, updated_at 
                 FROM profiles WHERE user_id = $1 ORDER BY name"
            )
            .bind(user.user_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;

            rows.into_iter()
                .map(|row| {
                    json!({
                        "id": row.get::<String, _>("id"),
                        "name": row.get::<String, _>("name"),
                        "server_url": row.get::<Option<String>, _>("server_url"),
                        "is_default": row.get::<bool, _>("is_default"),
                        "use_tls": row.get::<bool, _>("use_tls"),
                        "created_at": row.get::<String, _>("created_at"),
                        "updated_at": row.get::<String, _>("updated_at")
                    })
                })
                .collect::<Vec<_>>()
        }
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "profiles": profiles,
            "total": profiles.len()
        }
    })))
}

/// Create a new profile.
async fn create_profile(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateProfileRequest>,
) -> ServerResult<Json<Value>> {
    let db = state.db();
    let profile_id = Uuid::new_v4();
    let is_default = req.is_default.unwrap_or(false);
    let use_tls = req.use_tls.unwrap_or(true);

    // If setting as default, unset other defaults first
    if is_default {
        match db.pool() {
            orbis_db::DatabasePool::Postgres(pool) => {
                sqlx::query("UPDATE profiles SET is_default = false WHERE user_id = $1")
                    .bind(user.user_id)
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
            orbis_db::DatabasePool::Sqlite(pool) => {
                sqlx::query("UPDATE profiles SET is_default = false WHERE user_id = $1")
                    .bind(user.user_id.to_string())
                    .execute(pool)
                    .await
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
        }
    }

    match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            sqlx::query(
                "INSERT INTO profiles (id, user_id, name, server_url, is_default, use_tls, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"
            )
            .bind(profile_id)
            .bind(user.user_id)
            .bind(&req.name)
            .bind(&req.server_url)
            .bind(is_default)
            .bind(use_tls)
            .execute(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            sqlx::query(
                "INSERT INTO profiles (id, user_id, name, server_url, is_default, use_tls, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, datetime('now'), datetime('now'))"
            )
            .bind(profile_id.to_string())
            .bind(user.user_id.to_string())
            .bind(&req.name)
            .bind(&req.server_url)
            .bind(is_default)
            .bind(use_tls)
            .execute(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;
        }
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": profile_id.to_string(),
            "name": req.name,
            "server_url": req.server_url,
            "is_default": is_default,
            "use_tls": use_tls
        }
    })))
}

/// Get a profile by ID.
async fn get_profile(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    let db = state.db();
    
    let profile = match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            let row = sqlx::query(
                "SELECT id, name, server_url, is_default, use_tls, created_at, updated_at 
                 FROM profiles WHERE id = $1 AND user_id = $2"
            )
            .bind(id)
            .bind(user.user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;

            row.map(|r| {
                json!({
                    "id": r.get::<Uuid, _>("id").to_string(),
                    "name": r.get::<String, _>("name"),
                    "server_url": r.get::<Option<String>, _>("server_url"),
                    "is_default": r.get::<bool, _>("is_default"),
                    "use_tls": r.get::<bool, _>("use_tls"),
                    "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                    "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339()
                })
            })
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            let row = sqlx::query(
                "SELECT id, name, server_url, is_default, use_tls, created_at, updated_at 
                 FROM profiles WHERE id = $1 AND user_id = $2"
            )
            .bind(id.to_string())
            .bind(user.user_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;

            row.map(|r| {
                json!({
                    "id": r.get::<String, _>("id"),
                    "name": r.get::<String, _>("name"),
                    "server_url": r.get::<Option<String>, _>("server_url"),
                    "is_default": r.get::<bool, _>("is_default"),
                    "use_tls": r.get::<bool, _>("use_tls"),
                    "created_at": r.get::<String, _>("created_at"),
                    "updated_at": r.get::<String, _>("updated_at")
                })
            })
        }
    };

    match profile {
        Some(p) => Ok(Json(json!({
            "success": true,
            "data": p
        }))),
        None => Err(orbis_core::Error::not_found("Profile not found").into())
    }
}

/// Update profile request.
#[derive(Debug, Deserialize)]
struct UpdateProfileRequest {
    name: Option<String>,
    server_url: Option<String>,
    use_tls: Option<bool>,
}

/// Update a profile.
async fn update_profile(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateProfileRequest>,
) -> ServerResult<Json<Value>> {
    let db = state.db();

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut has_updates = false;
    
    if req.name.is_some() { 
        updates.push("name"); 
        has_updates = true; 
    }
    if req.server_url.is_some() { 
        updates.push("server_url"); 
        has_updates = true; 
    }
    if req.use_tls.is_some() { 
        updates.push("use_tls"); 
        has_updates = true; 
    }

    if !has_updates {
        return Ok(Json(json!({
            "success": true,
            "message": "No updates provided"
        })));
    }

    match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            let mut query = String::from("UPDATE profiles SET updated_at = NOW()");
            let mut param_idx = 1;
            
            if req.name.is_some() {
                query.push_str(&format!(", name = ${}", param_idx));
                param_idx += 1;
            }
            if req.server_url.is_some() {
                query.push_str(&format!(", server_url = ${}", param_idx));
                param_idx += 1;
            }
            if req.use_tls.is_some() {
                query.push_str(&format!(", use_tls = ${}", param_idx));
                param_idx += 1;
            }
            
            query.push_str(&format!(" WHERE id = ${} AND user_id = ${}", param_idx, param_idx + 1));
            
            let mut q = sqlx::query(&query);
            if let Some(ref name) = req.name { q = q.bind(name); }
            if let Some(ref url) = req.server_url { q = q.bind(url); }
            if let Some(tls) = req.use_tls { q = q.bind(tls); }
            q = q.bind(id).bind(user.user_id);
            
            q.execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            let mut query = String::from("UPDATE profiles SET updated_at = datetime('now')");
            let mut param_idx = 1;
            
            if req.name.is_some() {
                query.push_str(&format!(", name = ${}", param_idx));
                param_idx += 1;
            }
            if req.server_url.is_some() {
                query.push_str(&format!(", server_url = ${}", param_idx));
                param_idx += 1;
            }
            if req.use_tls.is_some() {
                query.push_str(&format!(", use_tls = ${}", param_idx));
                param_idx += 1;
            }
            
            query.push_str(&format!(" WHERE id = ${} AND user_id = ${}", param_idx, param_idx + 1));
            
            let mut q = sqlx::query(&query);
            if let Some(ref name) = req.name { q = q.bind(name); }
            if let Some(ref url) = req.server_url { q = q.bind(url); }
            if let Some(tls) = req.use_tls { q = q.bind(tls); }
            q = q.bind(id.to_string()).bind(user.user_id.to_string());
            
            q.execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": "Profile updated"
    })))
}

/// Delete a profile.
async fn delete_profile(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    let db = state.db();

    let rows_affected = match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            sqlx::query("DELETE FROM profiles WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user.user_id)
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?
                .rows_affected()
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            sqlx::query("DELETE FROM profiles WHERE id = $1 AND user_id = $2")
                .bind(id.to_string())
                .bind(user.user_id.to_string())
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?
                .rows_affected()
        }
    };

    if rows_affected == 0 {
        return Err(orbis_core::Error::not_found("Profile not found").into());
    }

    Ok(Json(json!({
        "success": true,
        "message": "Profile deleted"
    })))
}

/// Set a profile as default.
async fn set_default_profile(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    let db = state.db();

    match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            // Unset all defaults for this user
            sqlx::query("UPDATE profiles SET is_default = false WHERE user_id = $1")
                .bind(user.user_id)
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            
            // Set the new default
            let result = sqlx::query("UPDATE profiles SET is_default = true WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user.user_id)
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            
            if result.rows_affected() == 0 {
                return Err(orbis_core::Error::not_found("Profile not found").into());
            }
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            // Unset all defaults for this user
            sqlx::query("UPDATE profiles SET is_default = false WHERE user_id = $1")
                .bind(user.user_id.to_string())
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            
            // Set the new default
            let result = sqlx::query("UPDATE profiles SET is_default = true WHERE id = $1 AND user_id = $2")
                .bind(id.to_string())
                .bind(user.user_id.to_string())
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            
            if result.rows_affected() == 0 {
                return Err(orbis_core::Error::not_found("Profile not found").into());
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": "Default profile updated"
    })))
}
