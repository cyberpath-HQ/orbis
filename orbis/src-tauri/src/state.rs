//! Application state for Tauri commands.

use orbis_auth::AuthService;
use orbis_config::Config;
use orbis_core::AppMode;
use orbis_db::Database;
use orbis_plugin::PluginManager;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

/// Authentication session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub permissions: Vec<String>,
    pub created_at: String,
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

    /// Remote server URL (client only).
    server_url: Option<String>,

    /// Application configuration.
    config: Config,

    /// Current authentication session.
    session: Arc<RwLock<Option<AuthSession>>>,
}

impl OrbisState {
    /// Create state for standalone mode.
    pub fn new_standalone(
        db: Database,
        auth: Option<AuthService>,
        plugins: Arc<PluginManager>,
        config: Config,
    ) -> Self {
        Self {
            mode: AppMode::Standalone,
            db: Some(db),
            auth,
            plugins: Some(plugins),
            server_url: None,
            config,
            session: Arc::new(RwLock::new(None)),
        }
    }

    /// Create state for client mode.
    pub fn new_client(server_url: String, config: Config) -> Self {
        Self {
            mode: AppMode::ClientServer,
            db: None,
            auth: None,
            plugins: None,
            server_url: Some(server_url),
            config,
            session: Arc::new(RwLock::new(None)),
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
}
