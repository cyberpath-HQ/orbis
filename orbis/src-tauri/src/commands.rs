//! Tauri commands for IPC.

use crate::{OrbisState, state::AuthSession};
use orbis_core::AppMode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::State;

/// Authentication session data (re-export for easier access)
pub use crate::state::AuthSession as SessionData;

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub session: Option<AuthSession>,
}

/// Login command - authenticates user and creates session
#[tauri::command]
pub async fn login(
    username: String,
    password: String,
    state: State<'_, OrbisState>,
) -> Result<LoginResponse, String> {
    // In standalone mode with no auth, allow any credentials
    if state.is_standalone() {
        // TODO: Integrate with orbis-auth crate for proper password verification
        // For now, simple validation
        if username.is_empty() || password.is_empty() {
            return Ok(LoginResponse {
                success: false,
                message: "Username and password are required".to_string(),
                session: None,
            });
        }

        // Create session
        let session = AuthSession {
            user_id: format!("user_{}", username),
            username: username.clone(),
            token: format!("token_{}", chrono::Utc::now().timestamp()),
            permissions: vec!["admin".to_string()], // Default admin in standalone
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Store session in state
        state.set_session(Some(session.clone()));

        Ok(LoginResponse {
            success: true,
            message: "Login successful".to_string(),
            session: Some(session),
        })
    } else {
        // In client mode, forward to server
        // TODO: Implement HTTP auth call to server
        Err("Client mode authentication not yet implemented".to_string())
    }
}

/// Logout command - destroys current session
#[tauri::command]
pub async fn logout(state: State<'_, OrbisState>) -> Result<Value, String> {
    // Clear session
    state.set_session(None);

    Ok(json!({
        "success": true,
        "message": "Logged out successfully"
    }))
}

/// Get current session
#[tauri::command]
pub async fn get_session(state: State<'_, OrbisState>) -> Result<Option<AuthSession>, String> {
    Ok(state.get_session())
}

/// Verify current session is valid
#[tauri::command]
pub async fn verify_session(state: State<'_, OrbisState>) -> Result<bool, String> {
    Ok(state.is_authenticated())
}

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
                    "sections": page.sections,
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

/// Reload a specific plugin (hot reload).
#[tauri::command]
pub async fn reload_plugin(name: String, state: State<'_, OrbisState>) -> Result<Value, String> {
    let pm = state.plugins().ok_or("Plugins not available in client mode")?;

    let info = pm.reload_plugin(&name).await.map_err(|e| e.to_string())?;

    Ok(json!({
        "success": true,
        "message": format!("Plugin '{}' reloaded successfully", name),
        "plugin": {
            "id": info.id.to_string(),
            "name": info.manifest.name,
            "version": info.manifest.version,
            "state": format!("{:?}", info.state),
        }
    }))
}

/// Enable a disabled plugin.
#[tauri::command]
pub async fn enable_plugin(name: String, state: State<'_, OrbisState>) -> Result<Value, String> {
    let pm = state.plugins().ok_or("Plugins not available in client mode")?;

    pm.enable_plugin(&name).await.map_err(|e| e.to_string())?;

    Ok(json!({
        "success": true,
        "message": format!("Plugin '{}' enabled", name)
    }))
}

/// Disable a running plugin.
#[tauri::command]
pub async fn disable_plugin(name: String, state: State<'_, OrbisState>) -> Result<Value, String> {
    let pm = state.plugins().ok_or("Plugins not available in client mode")?;

    pm.disable_plugin(&name).await.map_err(|e| e.to_string())?;

    Ok(json!({
        "success": true,
        "message": format!("Plugin '{}' disabled", name)
    }))
}

/// Uninstall a plugin.
#[tauri::command]
pub async fn uninstall_plugin(name: String, state: State<'_, OrbisState>) -> Result<Value, String> {
    let pm = state.plugins().ok_or("Plugins not available in client mode")?;

    pm.unload_plugin(&name).await.map_err(|e| e.to_string())?;

    Ok(json!({
        "success": true,
        "message": format!("Plugin '{}' uninstalled", name)
    }))
}

/// Install a plugin from a local path.
#[tauri::command]
pub async fn install_plugin(path: String, state: State<'_, OrbisState>) -> Result<Value, String> {
    let pm = state.plugins().ok_or("Plugins not available in client mode")?;

    let plugin_path = PathBuf::from(&path);
    if !plugin_path.exists() {
        return Err(format!("Plugin path does not exist: {}", path));
    }

    let info = pm.load_plugin(&plugin_path).await.map_err(|e| e.to_string())?;

    Ok(json!({
        "success": true,
        "message": format!("Plugin '{}' installed successfully", info.manifest.name),
        "plugin": {
            "id": info.id.to_string(),
            "name": info.manifest.name,
            "version": info.manifest.version,
            "description": info.manifest.description,
            "state": format!("{:?}", info.state),
        }
    }))
}

/// Get detailed information about a specific plugin.
#[tauri::command]
pub fn get_plugin_info(name: String, state: State<'_, OrbisState>) -> Result<Value, String> {
    let pm = state.plugins().ok_or("Plugins not available in client mode")?;

    let info = pm.registry().get(&name).ok_or_else(|| format!("Plugin '{}' not found", name))?;

    Ok(json!({
        "id": info.id.to_string(),
        "name": info.manifest.name,
        "version": info.manifest.version,
        "description": info.manifest.description,
        "author": info.manifest.author,
        "license": info.manifest.license,
        "state": format!("{:?}", info.state),
        "loaded_at": info.loaded_at.to_rfc3339(),
        "permissions": info.manifest.permissions.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
        "routes_count": info.manifest.routes.len(),
        "pages_count": info.manifest.pages.len(),
    }))
}
