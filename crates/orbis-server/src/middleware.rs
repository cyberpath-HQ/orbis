//! Server middleware.

use axum::{
    body::Body,
    extract::State,
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::state::AppState;

/// Create logging middleware layer.
pub fn logging_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}

/// Create compression middleware layer.
pub fn compression_layer() -> CompressionLayer {
    CompressionLayer::new()
}

/// Create CORS middleware layer.
pub fn cors_layer(origins: &[String]) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ]);

    if origins.iter().any(|o| o == "*") {
        cors.allow_origin(Any)
    } else {
        // Parse origins - for simplicity, allow any for now
        cors.allow_origin(Any)
    }
}

/// Create auth middleware layer.
pub fn auth_layer(state: AppState) -> impl Clone + Send + Sync + 'static {
    axum::middleware::from_fn_with_state::<_, _, Body>(state, auth_middleware)
}

/// Auth middleware function.
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for public routes
    let path = request.uri().path();
    if is_public_route(path) {
        return Ok(next.run(request).await);
    }

    // Check if auth is required
    if !state.is_auth_required() {
        return Ok(next.run(request).await);
    }

    // Get auth header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let Some(auth_header) = auth_header else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Extract token
    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validate token
    let Some(auth) = state.auth() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if auth.validate_token(token).is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

/// Check if a route is public (no auth required).
fn is_public_route(path: &str) -> bool {
    let public_routes = [
        "/health",
        "/api/auth/login",
        "/api/auth/register",
        "/api/auth/refresh",
        "/api/health",
    ];

    public_routes.iter().any(|r| path.starts_with(r))
}

/// Request timing middleware.
pub async fn timing_middleware(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    let duration = start.elapsed();
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}
