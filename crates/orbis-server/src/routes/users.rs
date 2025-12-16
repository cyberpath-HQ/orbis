//! User management routes.

use axum::{
    extract::{Path, State},
    routing::{delete, get, put},
    Json, Router,
};
use serde_json::{json, Value};
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

/// List all users (admin only).
async fn list_users(
    _admin: AdminUser,
    State(_state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // TODO: Implement pagination and user listing
    Ok(Json(json!({
        "success": true,
        "data": {
            "users": [],
            "total": 0
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

/// Update a user.
async fn update_user(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(_state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // Users can only update themselves unless admin
    if user.user_id != id && !user.is_admin {
        return Err(orbis_core::Error::unauthorized("Cannot update other users").into());
    }

    // TODO: Implement user update
    Ok(Json(json!({
        "success": true,
        "message": "User updated"
    })))
}

/// Delete a user (admin only).
async fn delete_user(
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    State(_state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // TODO: Implement user deletion
    Ok(Json(json!({
        "success": true,
        "message": "User deleted"
    })))
}
