//! Tauri commands for IPC.

use crate::{OrbisState, state::AuthSession};
use orbis_core::AppMode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::{Emitter, State};

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub session: Option<AuthSession>,
}

/// Server auth response (for client mode)
#[derive(Debug, Deserialize)]
struct ServerAuthResponse {
    user: ServerUserInfo,
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct ServerUserInfo {
    id: String,
    username: String,
    email: String,
    #[serde(rename = "is_active")]
    _is_active: bool,
    is_admin: bool,
}

/// Login command - authenticates user and creates session
#[tauri::command]
pub async fn login(
    username: String,
    password: String,
    state: State<'_, OrbisState>,
) -> Result<LoginResponse, String> {
    if username.is_empty() || password.is_empty() {
        return Ok(LoginResponse {
            success: false,
            message: "Username and password are required".to_string(),
            session: None,
        });
    }

    if state.is_standalone() || state.is_server() {
        // Use local auth service
        login_standalone(&username, &password, &state).await
    } else {
        // Client mode: authenticate against remote server
        login_client(&username, &password, &state).await
    }
}

/// Authenticate using local auth service (standalone/server mode)
async fn login_standalone(
    username: &str,
    password: &str,
    state: &State<'_, OrbisState>,
) -> Result<LoginResponse, String> {
    // Get auth service
    let auth = match state.auth() {
        Some(auth) => auth,
        None => {
            // Fallback: if no auth service, create a simple session (dev mode)
            tracing::warn!("No auth service available, creating dev session");
            let session = AuthSession {
                user_id: format!("dev_{}", username),
                username: username.to_string(),
                email: format!("{}@localhost", username),
                token: format!("dev_token_{}", chrono::Utc::now().timestamp()),
                refresh_token: None,
                permissions: vec!["admin".to_string()],
                roles: vec!["admin".to_string()],
                is_admin: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                expires_at: None,
            };
            state.set_session(Some(session.clone()));
            return Ok(LoginResponse {
                success: true,
                message: "Login successful (dev mode)".to_string(),
                session: Some(session),
            });
        }
    };

    // Authenticate using orbis-auth
    match auth.authenticate(username, password, None, None).await {
        Ok(result) => {
            // Build roles from user info
            let mut roles = Vec::new();
            if result.user.is_admin {
                roles.push("admin".to_string());
            }
            roles.push("user".to_string());

            let session = AuthSession {
                user_id: result.user.id.to_string(),
                username: result.user.username.clone(),
                email: result.user.email.clone(),
                token: result.access_token.clone(),
                refresh_token: Some(result.refresh_token.clone()),
                permissions: if result.user.is_admin {
                    vec!["admin".to_string(), "read".to_string(), "write".to_string()]
                } else {
                    vec!["read".to_string()]
                },
                roles,
                is_admin: result.user.is_admin,
                created_at: chrono::Utc::now().to_rfc3339(),
                expires_at: Some(
                    (chrono::Utc::now() + chrono::Duration::seconds(result.expires_in as i64))
                        .to_rfc3339(),
                ),
            };

            state.set_session(Some(session.clone()));

            Ok(LoginResponse {
                success: true,
                message: "Login successful".to_string(),
                session: Some(session),
            })
        }
        Err(e) => Ok(LoginResponse {
            success: false,
            message: format!("Authentication failed: {}", e),
            session: None,
        }),
    }
}

/// Authenticate against remote server (client mode)
async fn login_client(
    username: &str,
    password: &str,
    state: &State<'_, OrbisState>,
) -> Result<LoginResponse, String> {
    let server_url = state
        .server_url()
        .ok_or("Server URL not configured")?;

    let client = state.http_client();

    // Make login request to server
    let response = client
        .post(format!("{}/api/auth/login", server_url))
        .json(&json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to server: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(LoginResponse {
            success: false,
            message: format!("Authentication failed ({}): {}", status, body),
            session: None,
        });
    }

    let auth_response: ServerAuthResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse server response: {}", e))?;

    // Build roles from user info
    let mut roles = Vec::new();
    if auth_response.user.is_admin {
        roles.push("admin".to_string());
    }
    roles.push("user".to_string());

    let session = AuthSession {
        user_id: auth_response.user.id.clone(),
        username: auth_response.user.username.clone(),
        email: auth_response.user.email.clone(),
        token: auth_response.access_token.clone(),
        refresh_token: Some(auth_response.refresh_token.clone()),
        permissions: if auth_response.user.is_admin {
            vec!["admin".to_string(), "read".to_string(), "write".to_string()]
        } else {
            vec!["read".to_string()]
        },
        roles,
        is_admin: auth_response.user.is_admin,
        created_at: chrono::Utc::now().to_rfc3339(),
        expires_at: Some(
            (chrono::Utc::now() + chrono::Duration::seconds(auth_response.expires_in as i64))
                .to_rfc3339(),
        ),
    };

    state.set_session(Some(session.clone()));

    Ok(LoginResponse {
        success: true,
        message: "Login successful".to_string(),
        session: Some(session),
    })
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
        "is_default": profile_name == "default",
    }))
}

/// Profile data structure for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredProfile {
    name: String,
    server_url: Option<String>,
    is_default: bool,
    use_tls: bool,
    created_at: String,
}

impl Default for StoredProfile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            server_url: None,
            is_default: true,
            use_tls: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Get profiles file path
fn get_profiles_path() -> PathBuf {
    // Use platform-specific data directory
    let data_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
            .unwrap_or_else(|_| PathBuf::from("."))
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .unwrap_or_else(|_| PathBuf::from("."))
    };
    
    let orbis_dir = data_dir.join("orbis");
    std::fs::create_dir_all(&orbis_dir).ok();
    orbis_dir.join("profiles.json")
}

/// Load profiles from file
fn load_profiles() -> Vec<StoredProfile> {
    let path = get_profiles_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_else(|| vec![StoredProfile::default()])
    } else {
        vec![StoredProfile::default()]
    }
}

/// Save profiles to file
fn save_profiles(profiles: &[StoredProfile]) -> Result<(), String> {
    let path = get_profiles_path();
    let content = serde_json::to_string_pretty(profiles)
        .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to save profiles: {}", e))?;
    Ok(())
}

/// List all profiles.
#[tauri::command]
pub async fn list_profiles(state: State<'_, OrbisState>) -> Result<Value, String> {
    let profiles = load_profiles();
    let active = state.config().active_profile.as_deref().unwrap_or("default");
    
    let profile_values: Vec<Value> = profiles
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "server_url": p.server_url,
                "is_active": p.name == active,
                "is_default": p.is_default,
                "use_tls": p.use_tls,
                "created_at": p.created_at,
            })
        })
        .collect();

    Ok(json!({
        "profiles": profile_values,
        "active": active
    }))
}

/// Create a new profile
#[tauri::command]
pub async fn create_profile(
    name: String,
    server_url: Option<String>,
    use_tls: Option<bool>,
) -> Result<Value, String> {
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let mut profiles = load_profiles();
    
    // Check if profile already exists
    if profiles.iter().any(|p| p.name == name) {
        return Err(format!("Profile '{}' already exists", name));
    }

    let new_profile = StoredProfile {
        name: name.clone(),
        server_url,
        is_default: false,
        use_tls: use_tls.unwrap_or(true),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    profiles.push(new_profile.clone());
    save_profiles(&profiles)?;

    Ok(json!({
        "success": true,
        "message": format!("Profile '{}' created", name),
        "profile": {
            "name": new_profile.name,
            "server_url": new_profile.server_url,
            "is_default": new_profile.is_default,
            "use_tls": new_profile.use_tls,
        }
    }))
}

/// Delete a profile
#[tauri::command]
pub async fn delete_profile(name: String) -> Result<Value, String> {
    if name == "default" {
        return Err("Cannot delete the default profile".to_string());
    }

    let mut profiles = load_profiles();
    let initial_len = profiles.len();
    profiles.retain(|p| p.name != name);

    if profiles.len() == initial_len {
        return Err(format!("Profile '{}' not found", name));
    }

    save_profiles(&profiles)?;

    Ok(json!({
        "success": true,
        "message": format!("Profile '{}' deleted", name)
    }))
}

/// Switch to a different profile.
#[tauri::command]
pub async fn switch_profile(
    name: String,
    _state: State<'_, OrbisState>,
) -> Result<Value, String> {
    let profiles = load_profiles();
    
    // Check if profile exists
    let profile = profiles
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Profile '{}' not found", name))?;

    // Note: Actually switching the profile would require app restart
    // or dynamic reconfiguration which isn't implemented yet.
    // For now, we return success and the app should restart to apply changes.
    
    Ok(json!({
        "success": true,
        "message": format!("Switched to profile: {}. Restart the app to apply changes.", name),
        "profile": {
            "name": profile.name,
            "server_url": profile.server_url,
            "use_tls": profile.use_tls,
        },
        "requires_restart": true
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

/// Start watching plugins directory for changes.
#[tauri::command]
pub async fn start_plugin_watcher(
    app_handle: tauri::AppHandle,
    state: State<'_, OrbisState>,
) -> Result<Value, String> {
    use orbis_plugin::PluginChangeKind;

    // Start the watcher in state
    state.start_plugin_watcher()?;

    // Get a reference to the watcher and start it
    let watcher_arc = state.plugin_watcher().clone();
    let mut watcher_guard = watcher_arc.write()
        .map_err(|_| "Failed to acquire watcher lock")?;

    let Some(watcher) = watcher_guard.as_mut() else {
        return Err("Watcher not initialized".to_string());
    };

    let rx = watcher.start()
        .map_err(|e| format!("Failed to start watcher: {}", e))?;

    // Spawn a task to process watcher events and emit to frontend
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut rx = rx;
        while let Some(event) = rx.recv().await {
            let kind = match event.kind {
                PluginChangeKind::Added => "Added",
                PluginChangeKind::Modified => "Modified",
                PluginChangeKind::Removed => "Removed",
            };

            let _ = app_handle_clone.emit("plugin-changed", json!({
                "kind": kind,
                "path": event.path.to_string_lossy(),
                "plugin_id": event.plugin_id,
            }));
        }
    });

    Ok(json!({
        "success": true,
        "message": "Plugin watcher started"
    }))
}

/// Stop watching plugins directory.
#[tauri::command]
pub async fn stop_plugin_watcher(state: State<'_, OrbisState>) -> Result<Value, String> {
    state.stop_plugin_watcher();

    Ok(json!({
        "success": true,
        "message": "Plugin watcher stopped"
    }))
}
