//! Tauri commands for IPC.

use crate::OrbisState;
use orbis_core::AppMode;
use serde_json::{json, Value};
use tauri::State;

/// Health check command.
#[tauri::command]
pub async fn health_check(state: State<'_, OrbisState>) -> Result<Value, String> {
    if let Some(db) = state.db() {
        db.health_check()
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(json!({
        "status": "ok",
        "mode": format!("{:?}", state.mode()),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get current application mode.
#[tauri::command]
pub fn get_mode(state: State<'_, OrbisState>) -> Value {
    json!({
        "mode": match state.mode() {
            AppMode::Standalone => "standalone",
            AppMode::ClientServer => if state.is_client() { "client" } else { "server" }
        },
        "is_standalone": state.is_standalone(),
        "is_client": state.is_client(),
        "is_server": state.is_server(),
    })
}

/// Get active profile.
#[tauri::command]
pub fn get_profile(state: State<'_, OrbisState>) -> Result<Value, String> {
    let profile_name = state.config().active_profile.as_deref().unwrap_or("default");
    
    Ok(json!({
        "name": profile_name,
        "server_url": state.server_url(),
    }))
}

/// List all profiles.
#[tauri::command]
pub async fn list_profiles(state: State<'_, OrbisState>) -> Result<Value, String> {
    // TODO: Load profiles from database or file system
    let profiles: Vec<Value> = vec![
        json!({
            "name": "default",
            "is_active": true,
            "is_default": true,
        })
    ];

    Ok(json!({
        "profiles": profiles,
        "active": state.config().active_profile.as_deref().unwrap_or("default")
    }))
}

/// Switch to a different profile.
#[tauri::command]
pub async fn switch_profile(
    name: String,
    _state: State<'_, OrbisState>,
) -> Result<Value, String> {
    // TODO: Implement profile switching
    // This would update the active profile and potentially reconnect to a different server
    
    Ok(json!({
        "success": true,
        "message": format!("Switched to profile: {}", name)
    }))
}

/// Get list of loaded plugins.
#[tauri::command]
pub fn get_plugins(state: State<'_, OrbisState>) -> Result<Value, String> {
    let plugins = if let Some(pm) = state.plugins() {
        pm.registry()
            .list()
            .iter()
            .map(|info| {
                json!({
                    "id": info.id.to_string(),
                    "name": info.manifest.name,
                    "version": info.manifest.version,
                    "description": info.manifest.description,
                    "state": format!("{:?}", info.state),
                })
            })
            .collect::<Vec<_>>()
    } else {
        // In client mode, fetch from server
        vec![]
    };

    Ok(json!({
        "plugins": plugins,
        "count": plugins.len()
    }))
}

/// Get plugin pages for UI rendering.
#[tauri::command]
pub fn get_plugin_pages(state: State<'_, OrbisState>) -> Result<Value, String> {
    let pages = if let Some(pm) = state.plugins() {
        pm.get_all_pages()
            .iter()
            .map(|(plugin, page)| {
                json!({
                    "plugin": plugin,
                    "route": page.full_route(plugin),
                    "title": page.title,
                    "icon": page.icon,
                    "description": page.description,
                    "show_in_menu": page.show_in_menu,
                    "menu_order": page.menu_order,
                    "layout": page.layout,
                })
            })
            .collect::<Vec<_>>()
    } else {
        // In client mode, fetch from server
        vec![]
    };

    Ok(json!({
        "pages": pages,
        "count": pages.len()
    }))
}
