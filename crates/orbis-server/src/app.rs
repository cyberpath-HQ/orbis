//! Application router and middleware setup.

use crate::middleware::{cors_layer, compression_layer, logging_layer};
use crate::routes;
use crate::state::AppState;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

/// The main Orbis application.
pub struct OrbisApp {
    router: Router,
}

impl OrbisApp {
    /// Create a new Orbis application.
    #[must_use]
    pub fn new(state: AppState) -> Self {
        let router = create_app(state);
        Self { router }
    }

    /// Get the router.
    #[must_use]
    pub fn router(self) -> Router {
        self.router
    }
}

/// Create the main application router.
#[must_use]
pub fn create_app(state: AppState) -> Router {
    let config = state.config();

    // Build middleware stack
    let middleware = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.server.request_timeout_seconds,
        )));

    // Create the main router
    let mut app = Router::new()
        // Health check
        .merge(routes::health::router())
        // API routes
        .nest("/api", api_routes(state.clone()))
        // Plugin routes
        .nest("/api/plugins", routes::plugins::router(state.clone()))
        // Static files and SPA fallback
        .merge(routes::static_files::router())
        // Apply middleware
        .layer(middleware)
        .with_state(state.clone());

    // Add logging if enabled
    if config.server.request_logging {
        app = app.layer(logging_layer());
    }

    // Add compression if enabled
    if config.server.compression {
        app = app.layer(compression_layer());
    }

    // Add CORS if enabled
    if config.server.cors_enabled {
        app = app.layer(cors_layer(&config.server.cors_origins));
    }

    app
}

/// Create API routes.
fn api_routes(_state: AppState) -> Router<AppState> {
    let router = Router::new()
        // Auth routes
        .merge(routes::auth::router())
        // User routes
        .merge(routes::users::router())
        // Profile routes
        .merge(routes::profiles::router())
        // Settings routes
        .merge(routes::settings::router())
        // Plugin management routes
        .merge(routes::plugin_management::router());

    // TODO: Apply auth middleware when axum 0.8 middleware API is stabilized
    // if state.is_auth_required() {
    //     router = router.layer(...);
    // }

    router
}
