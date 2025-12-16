//! Password hashing and verification.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Password service for hashing and verification.
#[derive(Clone)]
pub struct PasswordService {
    argon2: Argon2<'static>,
}

impl PasswordService {
    /// Create a new password service.
    #[must_use]
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hash a password.
    ///
    /// # Errors
    ///
    /// Returns an error if hashing fails.
    pub fn hash(&self, password: &str) -> orbis_core::Result<String> {
        let salt = SaltString::generate(&mut OsRng);

        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| orbis_core::Error::internal(format!("Failed to hash password: {}", e)))?;

        Ok(hash.to_string())
    }

    /// Verify a password against a hash.
    ///
    /// # Errors
    ///
    /// Returns an error if the hash is invalid.
    pub fn verify(&self, password: &str, hash: &str) -> orbis_core::Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| orbis_core::Error::internal(format!("Invalid password hash: {}", e)))?;

        Ok(self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Check if a password meets minimum requirements.
    #[must_use]
    pub fn validate_password_strength(password: &str) -> PasswordStrength {
        let len = password.len();
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let mut score = 0;

        if len >= 8 {
            score += 1;
        }
        if len >= 12 {
            score += 1;
        }
        if len >= 16 {
            score += 1;
        }
        if has_uppercase {
            score += 1;
        }
        if has_lowercase {
            score += 1;
        }
        if has_digit {
            score += 1;
        }
        if has_special {
            score += 1;
        }

        let strength = match score {
            0..=2 => StrengthLevel::Weak,
            3..=4 => StrengthLevel::Fair,
            5..=6 => StrengthLevel::Good,
            _ => StrengthLevel::Strong,
        };

        PasswordStrength {
            score,
            strength,
            has_minimum_length: len >= 8,
            has_uppercase,
            has_lowercase,
            has_digit,
            has_special,
        }
    }
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

/// Password strength information.
#[derive(Debug, Clone)]
pub struct PasswordStrength {
    /// Numeric score (0-7).
    pub score: u8,

    /// Strength level.
    pub strength: StrengthLevel,

    /// Has minimum length (8 characters).
    pub has_minimum_length: bool,

    /// Contains uppercase letters.
    pub has_uppercase: bool,

    /// Contains lowercase letters.
    pub has_lowercase: bool,

    /// Contains digits.
    pub has_digit: bool,

    /// Contains special characters.
    pub has_special: bool,
}

impl PasswordStrength {
    /// Check if the password meets all requirements.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.has_minimum_length
    }

    /// Check if the password is at least fair strength.
    #[must_use]
    pub const fn is_acceptable(&self) -> bool {
        matches!(
            self.strength,
            StrengthLevel::Fair | StrengthLevel::Good | StrengthLevel::Strong
        )
    }
}

/// Password strength level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrengthLevel {
    /// Weak password.
    Weak,
    /// Fair password.
    Fair,
    /// Good password.
    Good,
    /// Strong password.
    Strong,
}
