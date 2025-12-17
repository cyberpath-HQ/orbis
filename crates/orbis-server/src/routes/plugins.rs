//! Plugin route handler.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Method, Request, Uri},
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

/// Parse query string into HashMap.
fn parse_query_string(uri: &Uri) -> std::collections::HashMap<String, String> {
    uri.query()
        .map(|q| {
            q.split('&')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    match (parts.next(), parts.next()) {
                        (Some(key), Some(value)) => {
                            // Simple percent decoding
                            let key = percent_decode(key);
                            let value = percent_decode(value);
                            Some((key, value))
                        }
                        (Some(key), None) => {
                            let key = percent_decode(key);
                            Some((key, String::new()))
                        }
                        _ => None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Simple percent decoding for URL query parameters.
fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '%' {
            // Try to read two hex digits
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            // Invalid percent encoding, keep as-is
            result.push('%');
            result.push_str(&hex);
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    
    result
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

    // Parse query parameters
    let query_params = parse_query_string(request.uri());

    // Collect headers before consuming request
    let headers: std::collections::HashMap<String, String> = request
        .headers()
        .iter()
        .filter_map(|(k, v)| {
            v.to_str().ok().map(|v| (k.to_string(), v.to_string()))
        })
        .collect();

    // Parse body for POST/PUT/PATCH requests
    let body = if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
        // Try to parse body as JSON
        let (_parts, body) = request.into_parts();
        let bytes = axum::body::to_bytes(body, 1024 * 1024) // 1MB limit
            .await
            .map_err(|e| orbis_core::Error::plugin(format!("Failed to read body: {}", e)))?;
        
        if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| {
                    // If not JSON, wrap as string
                    serde_json::Value::String(String::from_utf8_lossy(&bytes).into_owned())
                })
        }
    } else {
        serde_json::Value::Null
    };

    // Build plugin context
    let context = orbis_plugin::PluginContext {
        method: method.to_string(),
        path: route_path,
        headers,
        query: query_params,
        body,
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
                "sections": page.sections,
                "state": page.state,
                "computed": page.computed,
                "actions": page.actions,
                "hooks": page.hooks,
                "dialogs": page.dialogs,
                "requires_auth": page.requires_auth,
                "permissions": page.permissions,
                "roles": page.roles
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
