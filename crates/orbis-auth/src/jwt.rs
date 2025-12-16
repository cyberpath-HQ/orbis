//! JWT token handling.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use orbis_config::Config;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::User;

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,

    /// Username.
    pub username: String,

    /// Email.
    pub email: String,

    /// Is admin.
    pub is_admin: bool,

    /// Token type (access or refresh).
    pub token_type: String,

    /// Issued at timestamp.
    pub iat: i64,

    /// Expiration timestamp.
    pub exp: i64,

    /// Not before timestamp.
    pub nbf: i64,

    /// JWT ID.
    pub jti: String,
}

/// JWT service for token generation and validation.
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl JwtService {
    /// Create a new JWT service.
    ///
    /// # Errors
    ///
    /// Returns an error if the JWT secret is missing.
    pub fn new(config: Arc<Config>) -> orbis_core::Result<Self> {
        let secret = config.jwt_secret.as_ref().ok_or_else(|| {
            orbis_core::Error::config("JWT secret is required. Set ORBIS_JWT_SECRET")
        })?;

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_expiry: config.jwt_expiry_seconds as i64,
            refresh_token_expiry: (config.jwt_expiry_seconds as i64) * 24 * 7, // 7 days
        })
    }

    /// Generate an access token for a user.
    ///
    /// # Errors
    ///
    /// Returns an error if token generation fails.
    pub fn generate_access_token(&self, user: &User) -> orbis_core::Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.access_token_expiry);

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            is_admin: user.is_admin,
            token_type: "access".to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::now_v7().to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| orbis_core::Error::auth(format!("Failed to generate token: {}", e)))
    }

    /// Generate a refresh token for a user.
    ///
    /// # Errors
    ///
    /// Returns an error if token generation fails.
    pub fn generate_refresh_token(&self, user: &User) -> orbis_core::Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.refresh_token_expiry);

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            is_admin: user.is_admin,
            token_type: "refresh".to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::now_v7().to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| orbis_core::Error::auth(format!("Failed to generate token: {}", e)))
    }

    /// Validate a token and return the claims.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid.
    pub fn validate_token(&self, token: &str) -> orbis_core::Result<Claims> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| orbis_core::Error::auth(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Decode a token without validation (for debugging).
    ///
    /// # Errors
    ///
    /// Returns an error if the token cannot be decoded.
    pub fn decode_without_validation(&self, token: &str) -> orbis_core::Result<Claims> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| orbis_core::Error::auth(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }
}
