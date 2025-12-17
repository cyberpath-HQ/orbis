//! Application state for Tauri commands.

use orbis_auth::AuthService;
use orbis_config::Config;
use orbis_core::AppMode;
use orbis_db::Database;
use orbis_plugin::{PluginManager, PluginWatcher, WatcherConfig};
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Authentication session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub token: String,
    pub refresh_token: Option<String>,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
    pub is_admin: bool,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// Orbis application state.
pub struct OrbisState {
    /// Application mode.
    mode: AppMode,

    /// Database connection (standalone/server only).
    db: Option<Database>,

    /// Auth service (standalone/server only).
    auth: Option<AuthService>,

    /// Plugin manager (standalone/server only).
    plugins: Option<Arc<PluginManager>>,

    /// Plugin watcher for hot reload (standalone/server only).
    plugin_watcher: Arc<RwLock<Option<PluginWatcher>>>,

    /// Plugins directory path.
    plugins_dir: Option<PathBuf>,

    /// Remote server URL (client only).
    server_url: Option<String>,

    /// Application configuration.
    config: Config,

    /// Current authentication session.
    session: Arc<RwLock<Option<AuthSession>>>,

    /// HTTP client for client mode.
    http_client: reqwest::Client,
}

impl OrbisState {
    /// Create state for standalone mode.
    pub fn new_standalone(
        db: Database,
        auth: Option<AuthService>,
        plugins: Arc<PluginManager>,
        config: Config,
    ) -> Self {
        let plugins_dir = config.plugins_dir.clone();
        Self {
            mode: AppMode::Standalone,
            db: Some(db),
            auth,
            plugins: Some(plugins),
            plugin_watcher: Arc::new(RwLock::new(None)),
            plugins_dir,
            server_url: None,
            config,
            session: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::new(),
        }
    }

    /// Create state for standalone mode with plugins directory.
    pub fn new_standalone_with_dir(
        db: Database,
        auth: Option<AuthService>,
        plugins: Arc<PluginManager>,
        plugins_dir: PathBuf,
        config: Config,
    ) -> Self {
        Self {
            mode: AppMode::Standalone,
            db: Some(db),
            auth,
            plugins: Some(plugins),
            plugin_watcher: Arc::new(RwLock::new(None)),
            plugins_dir: Some(plugins_dir),
            server_url: None,
            config,
            session: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::new(),
        }
    }

    /// Create state for client mode.
    pub fn new_client(server_url: String, config: Config) -> Self {
        Self {
            mode: AppMode::ClientServer,
            db: None,
            auth: None,
            plugins: None,
            plugin_watcher: Arc::new(RwLock::new(None)),
            plugins_dir: None,
            server_url: Some(server_url),
            config,
            session: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get the application mode.
    #[must_use]
    pub const fn mode(&self) -> AppMode {
        self.mode
    }

    /// Get the database (if available).
    #[must_use]
    pub fn db(&self) -> Option<&Database> {
        self.db.as_ref()
    }

    /// Get the auth service (if available).
    #[must_use]
    pub fn auth(&self) -> Option<&AuthService> {
        self.auth.as_ref()
    }

    /// Get the plugin manager (if available).
    #[must_use]
    pub fn plugins(&self) -> Option<&Arc<PluginManager>> {
        self.plugins.as_ref()
    }

    /// Get the plugins directory path.
    #[must_use]
    pub fn plugins_dir(&self) -> Option<&PathBuf> {
        self.plugins_dir.as_ref()
    }

    /// Get the plugin watcher.
    #[must_use]
    pub fn plugin_watcher(&self) -> &Arc<RwLock<Option<PluginWatcher>>> {
        &self.plugin_watcher
    }

    /// Start the plugin watcher.
    /// Returns true if the watcher was started successfully.
    pub fn start_plugin_watcher(&self) -> Result<(), String> {
        let Some(plugins_dir) = self.plugins_dir.as_ref() else {
            return Err("Plugins directory not configured".to_string());
        };

        let mut watcher_guard = self.plugin_watcher.write()
            .map_err(|_| "Failed to acquire watcher lock")?;

        if watcher_guard.is_some() {
            return Ok(()); // Already running
        }

        let config = WatcherConfig {
            watch_dir: plugins_dir.clone(),
            debounce_duration: std::time::Duration::from_millis(500),
            recursive: true,
        };

        let watcher = PluginWatcher::new(config);
        *watcher_guard = Some(watcher);

        Ok(())
    }

    /// Stop the plugin watcher.
    pub fn stop_plugin_watcher(&self) {
        if let Ok(mut watcher_guard) = self.plugin_watcher.write() {
            if let Some(mut watcher) = watcher_guard.take() {
                watcher.stop();
            }
        }
    }

    /// Get the server URL (for client mode).
    #[must_use]
    pub fn server_url(&self) -> Option<&str> {
        self.server_url.as_deref()
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Get the HTTP client.
    #[must_use]
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Check if running in standalone mode.
    #[must_use]
    pub const fn is_standalone(&self) -> bool {
        matches!(self.mode, AppMode::Standalone)
    }

    /// Check if running in client mode.
    #[must_use]
    pub fn is_client(&self) -> bool {
        matches!(self.mode, AppMode::ClientServer) && self.server_url.is_some()
    }

    /// Check if running in server mode.
    #[must_use]
    pub fn is_server(&self) -> bool {
        matches!(self.mode, AppMode::ClientServer) && self.db.is_some()
    }

    /// Get current session (read-only).
    pub fn get_session(&self) -> Option<AuthSession> {
        self.session.read().ok()?.clone()
    }

    /// Set session.
    pub fn set_session(&self, session: Option<AuthSession>) {
        if let Ok(mut s) = self.session.write() {
            *s = session;
        }
    }

    /// Check if user is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.session
            .read()
            .ok()
            .map(|s| s.is_some())
            .unwrap_or(false)
    }

    /// Get current auth token (if authenticated).
    pub fn get_token(&self) -> Option<String> {
        self.session
            .read()
            .ok()
            .and_then(|s| s.as_ref().map(|session| session.token.clone()))
    }
}
