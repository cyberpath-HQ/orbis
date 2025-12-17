//! Request extractors.

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts, StatusCode},
    Json,
    response::{IntoResponse, Response},
};
use orbis_auth::Claims;

use crate::state::AppState;

/// Authenticated user extractor.
pub struct AuthenticatedUser {
    /// User claims from JWT.
    claims: Claims,

    /// User ID.
    pub user_id: uuid::Uuid,

    /// Username.
    pub username: String,

    /// Is admin.
    pub is_admin: bool,
}

impl AuthenticatedUser {
    /// Get the JWT claims.
    #[must_use]
    pub const fn claims(&self) -> &Claims {
        &self.claims
    }

    /// Get the token expiration time.
    #[must_use]
    pub fn expires_at(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(self.claims.exp as i64, 0)
            .unwrap_or_default()
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let app_state = AppState::from_ref(state);

            // Get auth service
            let auth = app_state.auth().ok_or(AuthError::AuthNotConfigured)?;

            // Extract token from Authorization header
            let auth_header = parts
                .headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .ok_or(AuthError::MissingToken)?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or(AuthError::InvalidHeader)?;

            // Validate token
            let claims = auth.validate_token(token).map_err(|_| AuthError::InvalidToken)?;

            // Parse user ID
            let user_id = claims
                .sub
                .parse()
                .map_err(|_| AuthError::InvalidToken)?;

            Ok(Self {
                username: claims.username.clone(),
                is_admin: claims.is_admin,
                claims,
                user_id,
            })
        }
    }
}

/// Optional authenticated user extractor.
pub struct OptionalUser(pub Option<AuthenticatedUser>);

impl<S> FromRequestParts<S> for OptionalUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            Ok(Self(
                AuthenticatedUser::from_request_parts(parts, state)
                    .await
                    .ok(),
            ))
        }
    }
}

/// Admin user extractor (requires admin role).
pub struct AdminUser(pub AuthenticatedUser);

impl<S> FromRequestParts<S> for AdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let user = AuthenticatedUser::from_request_parts(parts, state).await?;

            if !user.is_admin {
                return Err(AuthError::NotAdmin);
            }

            Ok(Self(user))
        }
    }
}

/// Authentication error.
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidHeader,
    InvalidToken,
    NotAdmin,
    AuthNotConfigured,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Authorization token is required",
            ),
            Self::InvalidHeader => (
                StatusCode::UNAUTHORIZED,
                "INVALID_HEADER",
                "Invalid Authorization header format",
            ),
            Self::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid or expired token",
            ),
            Self::NotAdmin => (
                StatusCode::FORBIDDEN,
                "NOT_ADMIN",
                "Admin privileges required",
            ),
            Self::AuthNotConfigured => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "AUTH_NOT_CONFIGURED",
                "Authentication is not configured",
            ),
        };

        let body = Json(serde_json::json!({
            "success": false,
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}
