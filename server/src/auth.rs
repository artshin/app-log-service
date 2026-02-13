//! JWT authentication for protected log server endpoints.
//!
//! Validates JWTs using RSA public key shared with the backend.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at time (Unix timestamp)
    pub iat: i64,
}

/// JWT validator that loads and caches the public key
#[derive(Clone)]
pub struct JwtValidator {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtValidator {
    /// Create a new JWT validator from a public key file
    pub fn from_pem_file(path: &str) -> Result<Self, JwtError> {
        let pem_contents = fs::read_to_string(path)
            .map_err(|e| JwtError::KeyLoadError(format!("Failed to read key file: {}", e)))?;

        let decoding_key = DecodingKey::from_rsa_pem(pem_contents.as_bytes())
            .map_err(|e| JwtError::KeyLoadError(format!("Failed to parse RSA key: {}", e)))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_aud = false; // Backend doesn't set audience
        validation.validate_nbf = false; // Backend doesn't use "not before"

        Ok(Self {
            decoding_key,
            validation,
        })
    }

    /// Validate a JWT token and extract claims
    pub fn validate(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| JwtError::InvalidToken(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Extract user ID from validated token
    pub fn extract_user_id(&self, token: &str) -> Result<Uuid, JwtError> {
        let claims = self.validate(token)?;
        Uuid::parse_str(&claims.sub)
            .map_err(|e| JwtError::InvalidUserId(format!("Invalid user ID in token: {}", e)))
    }
}

/// Authenticated user extractor for Axum handlers
///
/// Usage in handlers:
/// ```
/// async fn protected_handler(auth: AuthUser) {
///     // auth.user_id contains the authenticated user's ID
/// }
/// ```
pub struct AuthUser {
    pub user_id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

        // Check for "Bearer " prefix
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization header format".to_string()))?;

        // Get validator from extensions (set by middleware)
        let validator = parts
            .extensions
            .get::<JwtValidator>()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "JWT validator not configured".to_string()))?;

        // Validate token and extract user ID
        let user_id = validator
            .extract_user_id(token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        Ok(AuthUser { user_id })
    }
}

/// JWT authentication errors
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Failed to load key: {0}")]
    KeyLoadError(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Invalid user ID: {0}")]
    InvalidUserId(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_deserialization() {
        let json = r#"{
            "sub": "550e8400-e29b-41d4-a716-446655440000",
            "exp": 1735516800,
            "iat": 1735430400
        }"#;

        let claims: Claims = serde_json::from_str(json).unwrap();
        assert_eq!(claims.sub, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(claims.exp, 1735516800);
        assert_eq!(claims.iat, 1735430400);
    }
}
