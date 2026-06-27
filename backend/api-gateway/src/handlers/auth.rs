use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use ethers::core::types::Signature;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::user::User;
use crate::services::database::DatabaseService;
use crate::utils::crypto::{hash_password, verify_password};
use crate::utils::{ApiError, ApiResult};
use crate::{ApiResponse, AppState};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/verify", get(verify_token))
        .route("/profile", get(get_profile))
        .route("/wallet/connect", post(collect_wallet))
        .route("/wallet/disconnect", post(disconnect_wallet))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub wallet_address: Option<String>,
    pub exp: usize,
    pub iat: usize,
    pub role: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub wallet_address: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub identifier: String, // username or email
    pub password: String,
}

#[derive(Deserialize)]
pub struct WalletConnectRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub wallet_address: Option<String>,
    pub reputation_score: i32,
    pub total_earnings: String, // BigDecimal as string
    pub created_at: DateTime<Utc>,
    pub is_verified: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            wallet_address: user.wallet_address,
            reputation_score: user.reputation_score,
            total_earnings: user.total_stakes.to_string(),
            created_at: user.created_at,
            is_verified: user.is_verified,
        }
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponse>>> {
    // Validate input
    if payload.username.is_empty() || payload.email.is_empty() {
        return Err(ApiError::Validation(
            "Username and email are required".to_string(),
        ));
    }

    // Validate password strength
    if payload.password.len() < 8 {
        return Err(ApiError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Check for at least one uppercase letter
    if !payload.password.chars().any(|c| c.is_uppercase()) {
        return Err(ApiError::Validation(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    // Check for at least one lowercase letter
    if !payload.password.chars().any(|c| c.is_lowercase()) {
        return Err(ApiError::Validation(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    // Check for at least one digit
    if !payload.password.chars().any(|c| c.is_numeric()) {
        return Err(ApiError::Validation(
            "Password must contain at least one number".to_string(),
        ));
    }

    // Check for at least one special character
    if !payload.password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ApiError::Validation(
            "Password must contain at least one special character".to_string(),
        ));
    }

    // Validate username format (alphanumeric and underscores only)
    if !payload
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(ApiError::Validation(
            "Username can only contain letters, numbers, and underscores".to_string(),
        ));
    }

    // Validate username length
    if payload.username.len() < 3 || payload.username.len() > 30 {
        return Err(ApiError::Validation(
            "Username must be between 3 and 30 characters".to_string(),
        ));
    }

    // Check if user already exists
    let existing_user = sqlx::query("SELECT id FROM users WHERE username = $1 OR email = $2")
        .bind(&payload.username)
        .bind(&payload.email)
        .fetch_optional(state.db.pool())
        .await?;

    if existing_user.is_some() {
        return Err(ApiError::BadRequest(
            "User with this username or email already exists".to_string(),
        ));
    }

    // Hash Password
    let password_hash = hash_password(&payload.password)
        .map_err(|e| ApiError::Internal(format!("Password hashing failed: {}", e)))?;

    // Create User
    let user_id = Uuid::new_v4();
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, email, password_hash, wallet_address, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.wallet_address)
    .bind(Utc::now())
    .fetch_one(state.db.pool())
    .await?;

    // Generate Tokens
    let (access_token, refresh_token) = generate_tokens(&user, &state.config.security.jwt_secret)?;

    let response = AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
        expires_in: 3600, // 1 hour
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponse>>> {
    // Find user by username or email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1 OR email = $1")
        .bind(&payload.identifier)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or(ApiError::Unauthorized)?;

    // Verify Password
    if !verify_password(&payload.password, &user.password_hash)
        .map_err(|e| ApiError::Internal(format!("Password verification failed: {}", e)))?
    {
        return Err(ApiError::Unauthorized);
    }

    // Update last login
    sqlx::query("UPDATE users SET last_login = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(user.id)
        .execute(state.db.pool())
        .await?;

    // Generate Tokens
    let (access_token, refresh_token) = generate_tokens(&user, &state.config.security.jwt_secret)?;

    let response = AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
        expires_in: 3600,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn logout(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // Extract token from header
    let token = extract_token_from_header(&headers)?;

    // Decode token to get expiration
    match decode_token(&token, &state.config.security.jwt_secret) {
        Ok(claims) => {
            // Calculate remaining TTL for the token
            let exp = claims.exp;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| ApiError::Internal("System time error".to_string()))?
                .as_secs() as i64;

            let ttl = (exp as i64 - now).max(0) as usize;

            if ttl > 0 {
                // Store token in Redis blacklist
                let blacklist_key = format!("jwt_blacklist:{}", token);

                // Use Redis SETEX to store token with TTL
                let mut conn = state.redis.connection_pool.clone();

                let _: () = redis::cmd("SETEX")
                    .arg(&blacklist_key)
                    .arg(ttl)
                    .arg("blacklisted")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| ApiError::Internal(format!("Failed to blacklist token: {}", e)))?;

                // Also remove from active sessions
                let mut sessions = state.active_sessions.write().await;
                sessions.remove(&claims.sub);
            }

            Ok(Json(ApiResponse::success_with_message(
                (),
                "Successfully logged out".to_string(),
            )))
        }
        Err(e) => {
            // Token invalid or expired - still consider logout success for UX
            Ok(Json(ApiResponse::success_with_message(
                (),
                "Successfully logged out".to_string(),
            )))
        }
    }
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponse>>> {
    // Decode refresh token
    let claims = decode_token(&payload.refresh_token, &state.config.security.jwt_secret)?;

    // Get user from database
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::BadRequest("Invalid user ID in token".to_string()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or(ApiError::Unauthorized)?;

    // Generate new tokens
    let (access_token, refresh_token) = generate_tokens(&user, &state.config.security.jwt_secret)?;

    let response = AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
        expires_in: 3600,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn verify_token(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let token = extract_token_from_header(&headers)?;
    let claims = decode_token(&token, &state.config.security.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::BadRequest("Invalid user ID in token".to_string()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or(ApiError::Unauthorized)?;

    Ok(Json(ApiResponse::success(user.into())))
}

pub async fn get_profile(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let user = authenticate_user(&headers, &state).await?;
    Ok(Json(ApiResponse::success(user.into())))
}

pub async fn collect_wallet(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<WalletConnectRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let mut user = authenticate_user(&headers, &state).await?;

    // Verify wallet signature ()
    if !verify_wallet_signature(
        &payload.wallet_address,
        &payload.signature,
        &payload.message,
    ) {
        return Err(ApiError::BadRequest("Invalid wallet signature".to_string()));
    }

    // Update user's wallet address
    user.wallet_address = Some(payload.wallet_address);

    sqlx::query("UPDATE users SET wallet_address = $1 WHERE id = $2")
        .bind(&user.wallet_address)
        .bind(user.id)
        .execute(state.db.pool())
        .await?;

    Ok(Json(ApiResponse::success(user.into())))
}

pub async fn disconnect_wallet(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let mut user = authenticate_user(&headers, &state).await?;
    user.wallet_address = None;

    sqlx::query("UPDATE users SET wallet_address = NULL WHERE id = $1")
        .bind(user.id)
        .execute(state.db.pool())
        .await?;

    Ok(Json(ApiResponse::success(user.into())))
}

// Helper function
fn generate_tokens(user: &User, secret: &str) -> ApiResult<(String, String)> {
    let now = Utc::now();
    let exp_access = (now + Duration::hours(1)).timestamp() as usize;
    let exp_refresh = (now + Duration::days(30)).timestamp() as usize;

    let claims_access = Claims {
        sub: user.id.to_string(),
        wallet_address: user.wallet_address.clone(),
        exp: exp_access,
        iat: now.timestamp() as usize,
        role: "user".to_string(),
    };

    let claims_refresh = Claims {
        sub: user.id.to_string(),
        wallet_address: user.wallet_address.clone(),
        exp: exp_refresh,
        iat: now.timestamp() as usize,
        role: "refresh".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    let access_token = encode(&Header::default(), &claims_access, &encoding_key)
        .map_err(|e| ApiError::Internal(format!("Token generation failed: {}", e)))?;

    let refresh_token = encode(&Header::default(), &claims_refresh, &encoding_key)
        .map_err(|e| ApiError::Internal(format!("Token generation failed: {}", e)))?;

    Ok((access_token, refresh_token))
}

fn decode_token(token: &str, secret: &str) -> ApiResult<Claims> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|_| ApiError::Unauthorized)
}

fn extract_token_from_header(headers: &HeaderMap) -> ApiResult<String> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(ApiError::Unauthorized)?
        .to_str()
        .map_err(|_| ApiError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::Unauthorized);
    }

    Ok(auth_header[7..].to_string())
}

async fn authenticate_user(headers: &HeaderMap, state: &AppState) -> ApiResult<User> {
    let token = extract_token_from_header(headers)?;
    let claims = decode_token(&token, &state.config.security.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::BadRequest("Invalid user ID in token".to_string()))?;

    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or(ApiError::Unauthorized)
}

fn verify_wallet_signature(address: &str, signature: &str, message: &str) -> bool {
    // Parse the hex signature
    let sig = match signature.parse::<Signature>() {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Recover the signer address from the signature
    // This uses EIP-191 personal_sign recovery (prefix "\x19Ethereum Signed Message:\n")
    let recovered = match sig.recover(message) {
        Ok(addr) => addr,
        Err(_) => return false,
    };

    // Compare recovered address to claimed address (case-insensitive hex comparison)
    let recovered_hex = format!("{:?}", recovered); // "0x..." lowercase
    recovered_hex.eq_ignore_ascii_case(address)
}

/// Verify email address
pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = payload
        .get("token")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Look up verification token in Redis
    let user_id_str = state
        .redis
        .get_raw(format!("email_verify:{}", token))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let user_id = Uuid::parse_str(&user_id_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Mark user as verified
    sqlx::query("UPDATE users SET is_verified = true WHERE id = $1")
        .bind(user_id)
        .execute(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Clean up token
    let _ = state
        .redis
        .delete_raw(format!("email_verify:{}", token))
        .await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Email verified successfully"
    })))
}

/// Forgot password — send reset link
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let email = payload
        .get("email")
        .and_then(|e| e.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Find user by email (always return success to prevent user enumeration)
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(user) = user {
        // Generate reset token and store in Redis with 1-hour expiry
        let reset_token = Uuid::new_v4().to_string();
        let _ = state
            .redis
            .set_raw_with_ttl(
                format!("password_reset:{}", reset_token),
                user.id.to_string(),
                3600,
            )
            .await;

        // Email delivery not yet connected. Log the reset link for development use.\n        // In production, integrate with an email provider (SendGrid, SES, etc.).\n        tracing::info!(\n            user_id = %user.id,\n            \"Password reset link: /reset-password?token={}\",\n            reset_token\n        );
    }

    // Always return success to prevent email enumeration
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "If an account with that email exists, a password reset link has been sent"
    })))
}

/// Reset password
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = payload
        .get("token")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let new_password = payload
        .get("password")
        .and_then(|p| p.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if new_password.len() < 8 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate reset token
    let user_id_str = state
        .redis
        .get_raw(format!("password_reset:{}", token))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let user_id = Uuid::parse_str(&user_id_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Hash the new password
    let password_hash =
        hash_password(new_password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update password
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&password_hash)
        .execute(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Invalidate the reset token
    let _ = state
        .redis
        .delete_raw(format!("password_reset:{}", token))
        .await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password reset successfully"
    })))
}

/// Generate API key
pub async fn generate_api_key(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = authenticate_user(&headers, &state)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Generate a random API key
    let raw_key = format!("nxs_{}", Uuid::new_v4().to_string().replace("-", ""));
    let key_hash = hash_password(&raw_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let key_id = Uuid::new_v4();

    // Store hashed key in database
    sqlx::query(
        "INSERT INTO api_keys (id, user_id, key_hash, name, created_at) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(key_id)
    .bind(user.id)
    .bind(&key_hash)
    .bind("Default API Key")
    .bind(Utc::now())
    .execute(state.db.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return the raw key (only shown once)
    Ok(Json(serde_json::json!({
        "success": true,
        "api_key": raw_key,
        "key_id": key_id,
        "message": "Store this key securely — it will not be shown again"
    })))
}
