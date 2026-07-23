pub mod file_upload;
pub mod url_submission;
pub mod validation;

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;

/// Extracts the submitter's user ID from the `X-User-Id` header.
/// The api-gateway sets this header after validating the JWT.
/// Returns `None` (via `Option<SubmitterId>`) for anonymous submissions.
pub struct SubmitterId(pub Uuid);

impl<S: Send + Sync> FromRequestParts<S> for SubmitterId {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("X-User-Id")
            .and_then(|v| v.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing X-User-Id header".to_string(),
            ))?;

        let user_id = Uuid::parse_str(header).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid X-User-Id format".to_string(),
            )
        })?;

        Ok(SubmitterId(user_id))
    }
}

// axum 0.8 routes `Option<SubmitterId>` through `OptionalFromRequestParts`
// (no longer the plain `FromRequestParts`): a missing header yields `None`
// (anonymous submission), while a present-but-malformed header still rejects.
impl<S: Send + Sync> OptionalFromRequestParts<S> for SubmitterId {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let Some(header) = parts.headers.get("X-User-Id").and_then(|v| v.to_str().ok()) else {
            return Ok(None);
        };

        let user_id = Uuid::parse_str(header).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid X-User-Id format".to_string(),
            )
        })?;

        Ok(Some(SubmitterId(user_id)))
    }
}
