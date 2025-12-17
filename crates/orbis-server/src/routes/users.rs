//! User management routes.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::error::ServerResult;
use crate::extractors::{AdminUser, AuthenticatedUser};
use crate::state::AppState;

/// Create users router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
        .route("/users/{id}", put(update_user))
        .route("/users/{id}", delete(delete_user))
}

/// Pagination query params.
#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

/// List all users (admin only).
async fn list_users(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> ServerResult<Json<Value>> {
    let db = state.db();
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    let (users, total) = match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            // Get total count
            let count_row = sqlx::query("SELECT COUNT(*) as count FROM users")
                .fetch_one(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            let total: i64 = count_row.get("count");

            // Get paginated users
            let rows = sqlx::query(
                "SELECT id, username, email, display_name, is_active, is_admin, created_at, updated_at 
                 FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;

            let users: Vec<Value> = rows.into_iter()
                .map(|row| {
                    json!({
                        "id": row.get::<Uuid, _>("id").to_string(),
                        "username": row.get::<String, _>("username"),
                        "email": row.get::<String, _>("email"),
                        "display_name": row.get::<Option<String>, _>("display_name"),
                        "is_active": row.get::<bool, _>("is_active"),
                        "is_admin": row.get::<bool, _>("is_admin"),
                        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                        "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339()
                    })
                })
                .collect();
            
            (users, total as u64)
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            // Get total count
            let count_row = sqlx::query("SELECT COUNT(*) as count FROM users")
                .fetch_one(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            let total: i64 = count_row.get("count");

            // Get paginated users
            let rows = sqlx::query(
                "SELECT id, username, email, display_name, is_active, is_admin, created_at, updated_at 
                 FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;

            let users: Vec<Value> = rows.into_iter()
                .map(|row| {
                    json!({
                        "id": row.get::<String, _>("id"),
                        "username": row.get::<String, _>("username"),
                        "email": row.get::<String, _>("email"),
                        "display_name": row.get::<Option<String>, _>("display_name"),
                        "is_active": row.get::<bool, _>("is_active"),
                        "is_admin": row.get::<bool, _>("is_admin"),
                        "created_at": row.get::<String, _>("created_at"),
                        "updated_at": row.get::<String, _>("updated_at")
                    })
                })
                .collect();
            
            (users, total as u64)
        }
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "users": users,
            "total": total,
            "page": page,
            "limit": limit,
            "pages": (total as f64 / limit as f64).ceil() as u64
        }
    })))
}

/// Get a user by ID.
async fn get_user(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // Users can only view themselves unless admin
    if user.user_id != id && !user.is_admin {
        return Err(orbis_core::Error::unauthorized("Cannot view other users").into());
    }

    let auth = state.auth().ok_or_else(|| {
        orbis_core::Error::config("Authentication is not configured")
    })?;

    let found_user = auth.user().find_by_id(id).await?.ok_or_else(|| {
        orbis_core::Error::not_found("User not found")
    })?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": found_user.id.to_string(),
            "username": found_user.username,
            "email": found_user.email,
            "display_name": found_user.display_name,
            "is_active": found_user.is_active,
            "is_admin": found_user.is_admin,
            "created_at": found_user.created_at.to_rfc3339()
        }
    })))
}

/// Update user request.
#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    display_name: Option<String>,
    email: Option<String>,
    is_active: Option<bool>,
    is_admin: Option<bool>,
}

/// Update a user.
async fn update_user(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateUserRequest>,
) -> ServerResult<Json<Value>> {
    // Users can only update themselves unless admin
    if user.user_id != id && !user.is_admin {
        return Err(orbis_core::Error::unauthorized("Cannot update other users").into());
    }

    // Non-admins cannot update is_admin or is_active fields
    if !user.is_admin && (req.is_admin.is_some() || req.is_active.is_some()) {
        return Err(orbis_core::Error::unauthorized("Cannot modify admin or active status").into());
    }

    let db = state.db();

    match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            let mut query = String::from("UPDATE users SET updated_at = NOW()");
            let mut param_idx = 1;
            
            if req.display_name.is_some() {
                query.push_str(&format!(", display_name = ${}", param_idx));
                param_idx += 1;
            }
            if req.email.is_some() {
                query.push_str(&format!(", email = ${}", param_idx));
                param_idx += 1;
            }
            if req.is_active.is_some() {
                query.push_str(&format!(", is_active = ${}", param_idx));
                param_idx += 1;
            }
            if req.is_admin.is_some() {
                query.push_str(&format!(", is_admin = ${}", param_idx));
                param_idx += 1;
            }
            
            query.push_str(&format!(" WHERE id = ${}", param_idx));
            
            let mut q = sqlx::query(&query);
            if let Some(ref dn) = req.display_name { q = q.bind(dn); }
            if let Some(ref email) = req.email { q = q.bind(email); }
            if let Some(active) = req.is_active { q = q.bind(active); }
            if let Some(admin) = req.is_admin { q = q.bind(admin); }
            q = q.bind(id);
            
            q.execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            let mut query = String::from("UPDATE users SET updated_at = datetime('now')");
            let mut param_idx = 1;
            
            if req.display_name.is_some() {
                query.push_str(&format!(", display_name = ${}", param_idx));
                param_idx += 1;
            }
            if req.email.is_some() {
                query.push_str(&format!(", email = ${}", param_idx));
                param_idx += 1;
            }
            if req.is_active.is_some() {
                query.push_str(&format!(", is_active = ${}", param_idx));
                param_idx += 1;
            }
            if req.is_admin.is_some() {
                query.push_str(&format!(", is_admin = ${}", param_idx));
                param_idx += 1;
            }
            
            query.push_str(&format!(" WHERE id = ${}", param_idx));
            
            let mut q = sqlx::query(&query);
            if let Some(ref dn) = req.display_name { q = q.bind(dn); }
            if let Some(ref email) = req.email { q = q.bind(email); }
            if let Some(active) = req.is_active { q = q.bind(active); }
            if let Some(admin) = req.is_admin { q = q.bind(admin); }
            q = q.bind(id.to_string());
            
            q.execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": "User updated"
    })))
}

/// Delete a user (admin only).
async fn delete_user(
    admin: AdminUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // Prevent self-deletion
    if admin.0.user_id == id {
        return Err(orbis_core::Error::validation("Cannot delete your own account").into());
    }

    let db = state.db();

    let rows_affected = match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?
                .rows_affected()
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(id.to_string())
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?
                .rows_affected()
        }
    };

    if rows_affected == 0 {
        return Err(orbis_core::Error::not_found("User not found").into());
    }

    Ok(Json(json!({
        "success": true,
        "message": "User deleted"
    })))
}
