//! # Orbis Server
//!
//! Axum-based HTTP/HTTPS server for Orbis supporting authentication,
//! plugin routes, and the REST API.

mod app;
mod error;
mod extractors;
mod middleware;
mod routes;
mod state;
mod tls;

pub use app::{create_app, OrbisApp};
pub use error::ServerError;
pub use state::AppState;

use orbis_auth::AuthService;
use orbis_config::Config;
use orbis_db::Database;
use orbis_plugin::PluginManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::Service;

/// Server instance.
pub struct Server {
    config: Arc<Config>,
    state: AppState,
}

impl Server {
    /// Create a new server instance.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub async fn new(config: Config) -> orbis_core::Result<Self> {
        let config = Arc::new(config);

        // Initialize database
        let db = Database::new(config.database.clone()).await?;

        // Run migrations if configured
        if config.database.run_migrations {
            db.migrate().await?;
        }

        // Initialize auth service
        let auth = if config.auth_enabled || config.mode.requires_auth() {
            Some(AuthService::new(config.clone(), db.clone())?)
        } else {
            None
        };

        // Initialize plugin manager
        let plugins_dir = config
            .plugins_dir
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("./plugins"));
        let plugins = PluginManager::new(plugins_dir, db.clone())?;

        // Load plugins
        plugins.load_all().await?;

        // Create app state
        let state = AppState::new(config.clone(), db, auth, plugins);

        Ok(Self { config, state })
    }

    /// Run the server.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start.
    pub async fn run(self) -> orbis_core::Result<()> {
        let addr = self.config.server.socket_addr()?;
        let app = create_app(self.state.clone());

        tracing::info!("Starting server on {}", addr);

        if self.config.is_tls_enabled() {
            self.run_https(app, addr).await
        } else {
            self.run_http(app, addr).await
        }
    }

    /// Run HTTP server.
    async fn run_http(
        self,
        app: axum::Router,
        addr: SocketAddr,
    ) -> orbis_core::Result<()> {
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            orbis_core::Error::server(format!("Failed to bind to {}: {}", addr, e))
        })?;

        tracing::info!("HTTP server listening on http://{}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| orbis_core::Error::server(format!("Server error: {}", e)))
    }

    /// Run HTTPS server.
    async fn run_https(
        self,
        app: axum::Router,
        addr: SocketAddr,
    ) -> orbis_core::Result<()> {
        let tls_config = tls::create_tls_config(&self.config.tls)?;
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_config));

        let listener = TcpListener::bind(addr).await.map_err(|e| {
            orbis_core::Error::server(format!("Failed to bind to {}: {}", addr, e))
        })?;

        tracing::info!("HTTPS server listening on https://{}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await.map_err(|e| {
                orbis_core::Error::server(format!("Failed to accept connection: {}", e))
            })?;

            let acceptor = acceptor.clone();
            let app = app.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let io = tokio_rustls::server::TlsStream::from(tls_stream);
                        let tower_service = app.clone();
                        
                        if let Err(e) = hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                            .serve_connection(hyper_util::rt::TokioIo::new(io), hyper::service::service_fn(move |req| {
                                tower_service.clone().call(req)
                            }))
                            .await
                        {
                            tracing::error!("Error serving connection from {}: {}", peer_addr, e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("TLS handshake failed for {}: {}", peer_addr, e);
                    }
                }
            });
        }
    }

    /// Get the app state.
    #[must_use]
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}
