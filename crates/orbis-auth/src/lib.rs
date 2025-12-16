//! # Orbis Auth
//!
//! Authentication and authorization for Orbis.
//! Provides JWT-based authentication, password hashing, and session management.

mod jwt;
mod password;
mod session;
mod user;

pub use jwt::{Claims, JwtService};
pub use password::PasswordService;
pub use session::{Session, SessionService};
pub use user::{CreateUser, User, UserService};

use orbis_config::Config;
use orbis_db::Database;
use std::sync::Arc;

/// Authentication service combining all auth functionality.
#[derive(Clone)]
pub struct AuthService {
    jwt: JwtService,
    password: PasswordService,
    session: SessionService,
    user: UserService,
    config: Arc<Config>,
}

impl AuthService {
    /// Create a new auth service.
    ///
    /// # Errors
    ///
    /// Returns an error if the JWT secret is missing in client-server mode.
    pub fn new(config: Arc<Config>, db: Database) -> orbis_core::Result<Self> {
        let jwt = JwtService::new(config.clone())?;
        let password = PasswordService::new();
        let session = SessionService::new(db.clone());
        let user = UserService::new(db);

        Ok(Self {
            jwt,
            password,
            session,
            user,
            config,
        })
    }

    /// Get the JWT service.
    #[must_use]
    pub const fn jwt(&self) -> &JwtService {
        &self.jwt
    }

    /// Get the password service.
    #[must_use]
    pub const fn password(&self) -> &PasswordService {
        &self.password
    }

    /// Get the session service.
    #[must_use]
    pub const fn session(&self) -> &SessionService {
        &self.session
    }

    /// Get the user service.
    #[must_use]
    pub const fn user(&self) -> &UserService {
        &self.user
    }

    /// Check if authentication is required.
    #[must_use]
    pub fn is_auth_required(&self) -> bool {
        self.config.auth_enabled || self.config.mode.requires_auth()
    }

    /// Authenticate a user with username/email and password.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails.
    pub async fn authenticate(
        &self,
        username_or_email: &str,
        password: &str,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> orbis_core::Result<AuthResult> {
        // Find user
        let user = self
            .user
            .find_by_username_or_email(username_or_email)
            .await?
            .ok_or_else(|| orbis_core::Error::auth("Invalid credentials"))?;

        // Verify password
        if !self.password.verify(password, &user.password_hash)? {
            return Err(orbis_core::Error::auth("Invalid credentials"));
        }

        // Check if user is active
        if !user.is_active {
            return Err(orbis_core::Error::auth("Account is disabled"));
        }

        // Generate tokens
        let access_token = self.jwt.generate_access_token(&user)?;
        let refresh_token = self.jwt.generate_refresh_token(&user)?;

        // Create session
        let session = self
            .session
            .create(
                user.id,
                &refresh_token,
                user_agent,
                ip_address,
                self.config.jwt_expiry_seconds,
            )
            .await?;

        Ok(AuthResult {
            user,
            access_token,
            refresh_token,
            session,
            expires_in: self.config.jwt_expiry_seconds,
        })
    }

    /// Refresh an access token using a refresh token.
    ///
    /// # Errors
    ///
    /// Returns an error if the refresh token is invalid.
    pub async fn refresh(&self, refresh_token: &str) -> orbis_core::Result<AuthResult> {
        // Validate refresh token
        let claims = self.jwt.validate_token(refresh_token)?;

        if claims.token_type != "refresh" {
            return Err(orbis_core::Error::auth("Invalid token type"));
        }

        // Find session
        let session = self
            .session
            .find_by_token(refresh_token)
            .await?
            .ok_or_else(|| orbis_core::Error::auth("Session not found"))?;

        // Check if session is valid
        if session.is_expired() {
            self.session.delete(session.id).await?;
            return Err(orbis_core::Error::auth("Session expired"));
        }

        // Find user
        let user = self
            .user
            .find_by_id(session.user_id)
            .await?
            .ok_or_else(|| orbis_core::Error::auth("User not found"))?;

        if !user.is_active {
            return Err(orbis_core::Error::auth("Account is disabled"));
        }

        // Generate new access token
        let access_token = self.jwt.generate_access_token(&user)?;

        Ok(AuthResult {
            user,
            access_token,
            refresh_token: refresh_token.to_string(),
            session,
            expires_in: self.config.jwt_expiry_seconds,
        })
    }

    /// Logout a user by invalidating their session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be deleted.
    pub async fn logout(&self, refresh_token: &str) -> orbis_core::Result<()> {
        if let Some(session) = self.session.find_by_token(refresh_token).await? {
            self.session.delete(session.id).await?;
        }
        Ok(())
    }

    /// Validate an access token and return the claims.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid.
    pub fn validate_token(&self, token: &str) -> orbis_core::Result<Claims> {
        self.jwt.validate_token(token)
    }
}

/// Authentication result containing user and tokens.
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// The authenticated user.
    pub user: User,

    /// Access token (short-lived).
    pub access_token: String,

    /// Refresh token (long-lived).
    pub refresh_token: String,

    /// Session information.
    pub session: Session,

    /// Token expiry time in seconds.
    pub expires_in: u64,
}
