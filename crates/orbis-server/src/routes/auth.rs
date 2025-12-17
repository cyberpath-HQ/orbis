//! Authentication routes.

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ServerResult;
use crate::extractors::AuthenticatedUser;
use crate::state::AppState;

/// Create auth router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
}

/// Login request.
#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// Login handler.
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ServerResult<Json<Value>> {
    let auth = state.auth().ok_or_else(|| {
        orbis_core::Error::config("Authentication is not configured")
    })?;

    let result = auth
        .authenticate(&req.username, &req.password, None, None)
        .await
        .map_err(|e| orbis_core::Error::auth(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "access_token": result.access_token,
            "refresh_token": result.refresh_token,
            "expires_in": result.expires_in,
            "user": {
                "id": result.user.id.to_string(),
                "username": result.user.username,
                "email": result.user.email,
                "display_name": result.user.display_name,
                "is_admin": result.user.is_admin
            }
        }
    })))
}

/// Register request.
#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
    display_name: Option<String>,
}

/// Register handler.
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ServerResult<Json<Value>> {
    let auth = state.auth().ok_or_else(|| {
        orbis_core::Error::config("Authentication is not configured")
    })?;

    // Check password strength
    let strength = orbis_auth::PasswordService::validate_password_strength(&req.password);
    if !strength.is_valid() {
        return Err(orbis_core::Error::validation(
            "Password must be at least 8 characters long",
        ).into());
    }

    // Check if username exists
    if auth.user().username_exists(&req.username).await? {
        return Err(orbis_core::Error::conflict("Username already exists").into());
    }

    // Check if email exists
    if auth.user().email_exists(&req.email).await? {
        return Err(orbis_core::Error::conflict("Email already exists").into());
    }

    // Hash password
    let password_hash = auth.password().hash(&req.password)?;

    // Create user
    let user = auth
        .user()
        .create(
            orbis_auth::CreateUser {
                username: req.username,
                email: req.email,
                password: req.password,
                display_name: req.display_name,
                is_admin: false,
            },
            password_hash,
        )
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": user.id.to_string(),
            "username": user.username,
            "email": user.email,
            "display_name": user.display_name,
            "is_admin": user.is_admin
        }
    })))
}

/// Refresh request.
#[derive(Debug, Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

/// Refresh handler.
async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> ServerResult<Json<Value>> {
    let auth = state.auth().ok_or_else(|| {
        orbis_core::Error::config("Authentication is not configured")
    })?;

    let result = auth
        .refresh(&req.refresh_token)
        .await
        .map_err(|e| orbis_core::Error::auth(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "access_token": result.access_token,
            "refresh_token": result.refresh_token,
            "expires_in": result.expires_in
        }
    })))
}

/// Logout request.
#[derive(Debug, Deserialize)]
struct LogoutRequest {
    refresh_token: String,
}

/// Logout handler.
async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> ServerResult<Json<Value>> {
    let auth = state.auth().ok_or_else(|| {
        orbis_core::Error::config("Authentication is not configured")
    })?;

    auth.logout(&req.refresh_token).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

/// Get current user.
async fn me(user: AuthenticatedUser) -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "id": user.user_id.to_string(),
            "username": user.username,
            "is_admin": user.is_admin
        }
    }))
}
