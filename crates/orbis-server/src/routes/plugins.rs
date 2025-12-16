//! Plugin route handler.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Method, Request},
    routing::any,
    Json, Router,
};
use serde_json::{json, Value};

use crate::error::ServerResult;
use crate::extractors::OptionalUser;
use crate::state::AppState;

/// Create plugin routes router.
pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        // Dynamic plugin route handler
        .route("/{plugin}/{*path}", any(handle_plugin_route))
        // Plugin pages/UI endpoint
        .route("/{plugin}/pages", axum::routing::get(get_plugin_pages))
}

/// Handle dynamic plugin routes.
async fn handle_plugin_route(
    Path((plugin_name, path)): Path<(String, String)>,
    State(state): State<AppState>,
    user: OptionalUser,
    method: Method,
    request: Request<Body>,
) -> ServerResult<Json<Value>> {
    // Find the plugin
    let info = state.plugins().registry().get(&plugin_name).ok_or_else(|| {
        orbis_core::Error::not_found(format!("Plugin '{}' not found", plugin_name))
    })?;

    // Check if plugin is running
    if info.state != orbis_plugin::PluginState::Running {
        return Err(orbis_core::Error::plugin(format!(
            "Plugin '{}' is not running",
            plugin_name
        )).into());
    }

    // Find matching route
    let route_path = format!("/{}", path);
    let route = info
        .manifest
        .routes
        .iter()
        .find(|r| r.path == route_path && r.method.eq_ignore_ascii_case(method.as_str()))
        .ok_or_else(|| {
            orbis_core::Error::not_found(format!(
                "Route {} {} not found in plugin '{}'",
                method, route_path, plugin_name
            ))
        })?;

    // Check authentication if required
    if route.requires_auth && user.0.is_none() {
        return Err(orbis_core::Error::auth("Authentication required").into());
    }

    // Build plugin context
    let context = orbis_plugin::PluginContext {
        method: method.to_string(),
        path: route_path,
        headers: request
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str().ok().map(|v| (k.to_string(), v.to_string()))
            })
            .collect(),
        query: std::collections::HashMap::new(), // TODO: Parse query params
        body: serde_json::Value::Null,           // TODO: Parse body
        user_id: user.0.as_ref().map(|u| u.user_id.to_string()),
        is_admin: user.0.as_ref().is_some_and(|u| u.is_admin),
    };

    // Execute plugin handler
    let result = state
        .plugins()
        .execute_route(&plugin_name, &route.handler, context)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": result
    })))
}

/// Get plugin pages for UI rendering.
async fn get_plugin_pages(
    Path(plugin_name): Path<String>,
    State(state): State<AppState>,
    user: OptionalUser,
) -> ServerResult<Json<Value>> {
    let info = state.plugins().registry().get(&plugin_name).ok_or_else(|| {
        orbis_core::Error::not_found(format!("Plugin '{}' not found", plugin_name))
    })?;

    // Filter pages based on auth requirements
    let pages: Vec<_> = info
        .manifest
        .pages
        .iter()
        .filter(|page| !page.requires_auth || user.0.is_some())
        .map(|page| {
            json!({
                "route": page.full_route(&plugin_name),
                "title": page.title,
                "icon": page.icon,
                "description": page.description,
                "show_in_menu": page.show_in_menu,
                "menu_order": page.menu_order,
                "layout": page.layout
            })
        })
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": {
            "pages": pages
        }
    })))
}

/// Get all plugin pages for the UI.
pub async fn get_all_plugin_pages(state: &AppState, is_authenticated: bool) -> Vec<Value> {
    state
        .plugins()
        .get_all_pages()
        .iter()
        .filter(|(_, page)| !page.requires_auth || is_authenticated)
        .map(|(plugin_name, page)| {
            json!({
                "plugin": plugin_name,
                "route": page.full_route(plugin_name),
                "title": page.title,
                "icon": page.icon,
                "description": page.description,
                "show_in_menu": page.show_in_menu,
                "menu_order": page.menu_order,
                "layout": page.layout
            })
        })
        .collect()
}
