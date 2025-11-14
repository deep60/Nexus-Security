use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{AuthService, Claims};
use crate::config::JwtConfig;
use crate::AppState;

/// Extract user claims from Authorization header
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    // Extract token from "Bearer {token}" format
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidFormat)?;

    // Create auth service
    let auth_service = AuthService::new(state.config.jwt.clone());

    // Validate token
    let claims = auth_service
        .validate_token(token)
        .map_err(|_| AuthError::InvalidToken)?;

    // Ensure it's an access token
    if claims.token_type != "access" {
        return Err(AuthError::InvalidTokenType);
    }

    // Insert claims into request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Extract user claims from Authorization header and verify admin role
pub async fn admin_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidFormat)?;

    let auth_service = AuthService::new(state.config.jwt.clone());

    let claims = auth_service
        .validate_token(token)
        .map_err(|_| AuthError::InvalidToken)?;

    if claims.token_type != "access" {
        return Err(AuthError::InvalidTokenType);
    }

    // Check if user is admin
    if !claims.is_admin {
        return Err(AuthError::InsufficientPermissions);
    }

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Optionally extract user claims (doesn't fail if no token provided)
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let auth_service = AuthService::new(state.config.jwt.clone());
                if let Ok(claims) = auth_service.validate_token(token) {
                    if claims.token_type == "access" {
                        request.extensions_mut().insert(claims);
                    }
                }
            }
        }
    }

    next.run(request).await
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidFormat,
    InvalidToken,
    InvalidTokenType,
    InsufficientPermissions,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidFormat => (StatusCode::UNAUTHORIZED, "Invalid authorization format"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            AuthError::InvalidTokenType => (StatusCode::UNAUTHORIZED, "Invalid token type"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

/// Extension trait to extract user ID from request
pub trait UserIdExtractor {
    fn user_id(&self) -> Result<Uuid, AuthError>;
    fn claims(&self) -> Result<&Claims, AuthError>;
}

impl UserIdExtractor for Request {
    fn user_id(&self) -> Result<Uuid, AuthError> {
        let claims = self
            .extensions()
            .get::<Claims>()
            .ok_or(AuthError::MissingToken)?;

        Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)
    }

    fn claims(&self) -> Result<&Claims, AuthError> {
        self.extensions()
            .get::<Claims>()
            .ok_or(AuthError::MissingToken)
    }
}
