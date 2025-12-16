//! Settings routes.

use axum::{
    extract::State,
    routing::{get, put},
    Json, Router,
};
use serde_json::{json, Value};

use crate::error::ServerResult;
use crate::extractors::AdminUser;
use crate::state::AppState;

/// Create settings router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings", get(get_settings))
        .route("/settings", put(update_settings))
}

/// Get application settings (admin only).
async fn get_settings(
    _admin: AdminUser,
    State(_state): State<AppState>,
) -> ServerResult<Json<Value>> {
    // TODO: Fetch settings from database
    Ok(Json(json!({
        "success": true,
        "data": {
            "settings": {}
        }
    })))
}

/// Update application settings (admin only).
async fn update_settings(
    _admin: AdminUser,
    State(_state): State<AppState>,
    Json(settings): Json<Value>,
) -> ServerResult<Json<Value>> {
    // TODO: Update settings in database
    Ok(Json(json!({
        "success": true,
        "message": "Settings updated"
    })))
}
