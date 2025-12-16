//! Profile management routes.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
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
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // TODO: Implement profile listing from database
    Ok(Json(json!({
        "success": true,
        "data": {
            "profiles": [],
            "total": 0
        }
    })))
}

/// Create a new profile.
async fn create_profile(
    user: AuthenticatedUser,
    State(_state): State<AppState>,
    Json(req): Json<CreateProfileRequest>,
) -> ServerResult<Json<Value>> {
    let profile = orbis_core::Profile::new(&req.name)
        .with_default(req.is_default.unwrap_or(false))
        .with_tls(req.use_tls.unwrap_or(true));

    let profile = if let Some(url) = req.server_url {
        profile.with_server_url(url)
    } else {
        profile
    };

    // TODO: Save to database

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": profile.id.to_string(),
            "name": profile.name,
            "server_url": profile.server_url,
            "is_default": profile.is_default,
            "use_tls": profile.use_tls
        }
    })))
}

/// Get a profile by ID.
async fn get_profile(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(_state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // TODO: Fetch from database
    Err(orbis_core::Error::not_found("Profile not found").into())
}

/// Update a profile.
async fn update_profile(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(_state): State<AppState>,
    Json(req): Json<CreateProfileRequest>,
) -> ServerResult<Json<Value>> {
    // TODO: Update in database
    Ok(Json(json!({
        "success": true,
        "message": "Profile updated"
    })))
}

/// Delete a profile.
async fn delete_profile(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(_state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // TODO: Delete from database
    Ok(Json(json!({
        "success": true,
        "message": "Profile deleted"
    })))
}

/// Set a profile as default.
async fn set_default_profile(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(_state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // TODO: Update in database
    Ok(Json(json!({
        "success": true,
        "message": "Default profile updated"
    })))
}
