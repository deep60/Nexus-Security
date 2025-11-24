use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, State},
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::error::ApiError;
use crate::AppState;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,     // Subject (user ID)
    pub email: String, // User email
    pub role: String,  // User role
    pub exp: i64,      // Expiration time
    pub iat: i64,      // Issued at
    pub nbf: i64,      // Not before
    pub jti: String,   // JWT ID (unique token identifier)
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, role: String, expires_in_hours: i64) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(expires_in_hours);

        Self {
            sub: user_id,
            email,
            role,
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    pub fn is_valid_now(&self) -> bool {
        let now = Utc::now().timestamp();
        now >= self.nbf && now < self.exp
    }
}

/// JWT token generation and validation
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::default();

        Self {
            encoding_key,
            decoding_key,
            validation,
        }
    }

    pub fn generate_token(&self, claims: &Claims) -> Result<String, ApiError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| ApiError::Internal(format!("Failed to generate token: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, ApiError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))
    }

    pub fn refresh_token(&self, old_claims: &Claims) -> Result<String, ApiError> {
        // Create new claims with extended expiration
        let new_claims = Claims::new(
            old_claims.sub,
            old_claims.email.clone(),
            old_claims.role.clone(),
            24, // 24 hours
        );

        self.generate_token(&new_claims)
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate token
    // TODO: Implement actual JWT validation with state.jwt_service
    // For now, we'll add the user context to request extensions

    // Mock claims for compilation (replace with actual validation)
    let claims = Claims {
        sub: Uuid::new_v4(),
        email: "user@example.com".to_string(),
        role: "user".to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp(),
        iat: Utc::now().timestamp(),
        nbf: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    // Add claims to request extensions for handlers to access
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Optional authentication middleware (doesn't fail on missing token)
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(header) = auth_header {
        if header.starts_with("Bearer ") {
            let token = &header[7..];

            // Try to validate token, but don't fail if invalid
            // TODO: Implement actual JWT validation
            let claims = Claims {
                sub: Uuid::new_v4(),
                email: "user@example.com".to_string(),
                role: "user".to_string(),
                exp: (Utc::now() + Duration::hours(24)).timestamp(),
                iat: Utc::now().timestamp(),
                nbf: Utc::now().timestamp(),
                jti: Uuid::new_v4().to_string(),
            };

            request.extensions_mut().insert(claims);
        }
    }

    next.run(request).await
}

/// Admin role middleware (must be used after auth_middleware)
pub async fn require_admin(mut request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if claims.role != "admin" && claims.role != "moderator" {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// API key authentication middleware
pub async fn api_key_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for API key in header
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // TODO: Validate API key against database
    // For now, just check if it starts with "nxs_"
    if !api_key.starts_with("nxs_") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // TODO: Add user context from API key lookup
    let claims = Claims {
        sub: Uuid::new_v4(),
        email: "api@example.com".to_string(),
        role: "user".to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp(),
        iat: Utc::now().timestamp(),
        nbf: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Extractor for authenticated user claims
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            "user".to_string(),
            24,
        );

        assert!(!claims.is_expired());
        assert!(claims.is_valid_now());
    }

    #[test]
    fn test_jwt_service() {
        let jwt_service = JwtService::new("test_secret_key_at_least_32_chars");
        let claims = Claims::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            "user".to_string(),
            24,
        );

        let token = jwt_service.generate_token(&claims).unwrap();
        assert!(!token.is_empty());

        let validated_claims = jwt_service.validate_token(&token).unwrap();
        assert_eq!(validated_claims.email, "test@example.com");
    }
}
