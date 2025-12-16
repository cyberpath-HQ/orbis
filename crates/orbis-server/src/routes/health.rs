//! Health check routes.

use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

/// Create health check router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/health", get(api_health_check))
}

/// Basic health check.
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Detailed API health check.
async fn api_health_check(State(state): State<AppState>) -> Json<Value> {
    let db_healthy = state.db().health_check().await.is_ok();

    let plugins_count = state.plugins().registry().count();
    let plugins_running = state.plugins().registry().running_count();

    Json(json!({
        "status": if db_healthy { "ok" } else { "degraded" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "database": {
                "status": if db_healthy { "ok" } else { "error" }
            },
            "plugins": {
                "total": plugins_count,
                "running": plugins_running
            },
            "auth": {
                "enabled": state.is_auth_required()
            }
        },
        "version": env!("CARGO_PKG_VERSION")
    }))
}
