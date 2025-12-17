//! Tauri commands for IPC.

use crate::{OrbisState, state::AuthSession};
use orbis_core::AppMode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
