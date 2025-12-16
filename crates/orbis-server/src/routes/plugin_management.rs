//! Plugin management routes (admin).

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::error::ServerResult;
use crate::extractors::AdminUser;
use crate::state::AppState;

/// Create plugin management router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/plugins", get(list_plugins))
        .route("/plugins/{name}", get(get_plugin))
        .route("/plugins/{name}/enable", post(enable_plugin))
        .route("/plugins/{name}/disable", post(disable_plugin))
        .route("/plugins/{name}", delete(uninstall_plugin))
}

/// List all plugins.
async fn list_plugins(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    let plugins: Vec<_> = state
        .plugins()
        .registry()
        .list()
        .iter()
        .map(|info| {
            json!({
                "id": info.id.to_string(),
                "name": info.manifest.name,
                "version": info.manifest.version,
                "description": info.manifest.description,
                "author": info.manifest.author,
                "state": format!("{:?}", info.state),
                "routes_count": info.manifest.routes.len(),
                "pages_count": info.manifest.pages.len(),
                "loaded_at": info.loaded_at.to_rfc3339()
            })
        })
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": {
            "plugins": plugins,
            "total": plugins.len()
        }
    })))
}

/// Get plugin details.
async fn get_plugin(
    _admin: AdminUser,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    let info = state.plugins().registry().get(&name).ok_or_else(|| {
        orbis_core::Error::not_found(format!("Plugin '{}' not found", name))
    })?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": info.id.to_string(),
            "name": info.manifest.name,
            "version": info.manifest.version,
            "description": info.manifest.description,
            "author": info.manifest.author,
            "homepage": info.manifest.homepage,
            "license": info.manifest.license,
            "state": format!("{:?}", info.state),
            "permissions": info.manifest.permissions,
            "routes": info.manifest.routes,
            "pages": info.manifest.pages,
            "loaded_at": info.loaded_at.to_rfc3339()
        }
    })))
}

/// Enable a plugin.
async fn enable_plugin(
    _admin: AdminUser,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    state.plugins().enable_plugin(&name).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Plugin '{}' enabled", name)
    })))
}

/// Disable a plugin.
async fn disable_plugin(
    _admin: AdminUser,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    state.plugins().disable_plugin(&name).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Plugin '{}' disabled", name)
    })))
}

/// Uninstall a plugin.
async fn uninstall_plugin(
    _admin: AdminUser,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    state.plugins().unload_plugin(&name).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Plugin '{}' uninstalled", name)
    })))
}
