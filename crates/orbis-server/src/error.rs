//! Server error types.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Server error wrapper.
#[derive(Debug)]
pub struct ServerError(pub orbis_core::Error);

impl From<orbis_core::Error> for ServerError {
    fn from(err: orbis_core::Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self.0 {
            orbis_core::Error::Config(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", msg.clone())
            }
            orbis_core::Error::Database(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", msg.clone())
            }
            orbis_core::Error::Auth(msg) => {
                (StatusCode::UNAUTHORIZED, "AUTH_ERROR", msg.clone())
            }
            orbis_core::Error::Unauthorized(msg) => {
                (StatusCode::FORBIDDEN, "UNAUTHORIZED", msg.clone())
            }
            orbis_core::Error::Plugin(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "PLUGIN_ERROR", msg.clone())
            }
            orbis_core::Error::Server(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "SERVER_ERROR", msg.clone())
            }
            orbis_core::Error::Io(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", err.to_string())
            }
            orbis_core::Error::Serialization(msg) => {
                (StatusCode::BAD_REQUEST, "SERIALIZATION_ERROR", msg.clone())
            }
            orbis_core::Error::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            orbis_core::Error::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone())
            }
            orbis_core::Error::Conflict(msg) => {
                (StatusCode::CONFLICT, "CONFLICT", msg.clone())
            }
            orbis_core::Error::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

/// Result type alias for server handlers.
pub type ServerResult<T> = Result<T, ServerError>;
