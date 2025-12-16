//! Application state shared across handlers.

use orbis_auth::AuthService;
use orbis_config::Config;
use orbis_db::Database;
use orbis_plugin::PluginManager;
use std::sync::Arc;

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration.
    config: Arc<Config>,

    /// Database connection.
    db: Database,

    /// Authentication service.
    auth: Option<AuthService>,

    /// Plugin manager.
    plugins: Arc<PluginManager>,
}

impl AppState {
    /// Create new application state.
    pub fn new(
        config: Arc<Config>,
        db: Database,
        auth: Option<AuthService>,
        plugins: PluginManager,
    ) -> Self {
        Self {
            config,
            db,
            auth,
            plugins: Arc::new(plugins),
        }
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the database.
    #[must_use]
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Get the auth service.
    #[must_use]
    pub fn auth(&self) -> Option<&AuthService> {
        self.auth.as_ref()
    }

    /// Get the plugin manager.
    #[must_use]
    pub fn plugins(&self) -> &PluginManager {
        &self.plugins
    }

    /// Get the plugin manager Arc.
    #[must_use]
    pub fn plugins_arc(&self) -> Arc<PluginManager> {
        Arc::clone(&self.plugins)
    }

    /// Check if authentication is required.
    #[must_use]
    pub fn is_auth_required(&self) -> bool {
        self.auth.as_ref().is_some_and(|a| a.is_auth_required())
    }
}
