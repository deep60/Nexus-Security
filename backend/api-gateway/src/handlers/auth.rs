use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

// use crate::models::user::User;
// use crate::services::database::DatabaseService;
// use crate::utils::crypto::{hash_password, verify_password};
// use super::{ApiError, ApiResult, ApiResponse};

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route()
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub database_service: DatabaseService,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // user_id
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
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Serialize)]
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
            total_earnings: user.total_earnings.to_string(),
            created_at: user.created_at,
            is_verified: user.is_verified,
        }
    }
}

pub async fn register(
    State(state): State<Arc<AppState>>, 
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponse>>> {
    // Validate input

    if payload.username.is_empty() || payload.email.is_empty() || payload.password.len() < 8 {
        return Err(ApiError::Validation(
            "Invalid input: username and email required, password must be 8 characters"
            .to_string(),
        ));
    }

    // Check if user already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = $1 OR email = $2",
        payload.username,
        payload.email
    )
    .fetch_optional(&state.db)
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
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO user (id, username, email, password_hash, wallet_address, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        user_id,
        payload.username,
        payload.email,
        password_hash,
        payload.wallet_address,
        Utc::now()
    )
    .fetch_one(&state.db)
    .await?;

    // Generate Tokens
    let (access_token, refresh_token) = generate_tokens(&user, &state.jwt_secret)?;

    let response = AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
        expires_in: 3600      // 1 hour
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn login(
    State(state): State<Arc<AppState>>, 
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponse>>> {
    // Find user by username or email
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = $1 OR email = $1",
        payload.identifier
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::Unauthorized)?;

    // Verify Password
    if !verify_password(&payload.password, &user.password_hash)
        .map_err(|e| ApiError::Internal(format!("Password verification failed: {}", e)))? 
    {
        return Err(ApiError::Unauthorized);
    }

    // Update last login
    sqlx::query!(
        "UPDATE users SET last_login = $1 WHERE id = $2",
        Utc::now(),
        user.id
    )
    .execute(&state.db)
    .await?;

    // Generate Tokens
    let (access_token, refresh_token) = generate_tokens(&user, &state.jwt_secret)?;

    let response = AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
        expires_in: 3600
    };

    Ok(Json(ApiResponse::success(response)))
} 

pub async fn logout(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // Extract token from header
    let token = extract_token_from_header(&headers)?;

    // Add token to blacklist (you might want to implement a Redis-based blacklist)
    // For now, we'll just return success
    
    Ok(Json(ApiResponse::success_with_message(
        (),
        "Successfully logged out".to_string(),
    )))
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponse>>> {
    // Decode refresh token
    let claims = decode_token(&payload.refresh_token, &state.jwt_secret)?;
    
    // Get user from database
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::BadRequest("Invalid user ID in token".to_string()))?;
    
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    // Generate new tokens
    let (access_token, refresh_token) = generate_tokens(&user, &state.jwt_secret)?;

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
    State(state): State<Arc<AppState>>, 
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let token = extract_token_from_header(&header)?;
    let claims = decode_token(&token, &state.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::BadRequest("Invalid user ID in token".to_string()))?;

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    Ok(Json(ApiResponse::success(user.into())))
}

pub async fn get_profile(
    headers: HeaderMap, 
    State(state): State<Arc<AppState>>, 
) -> ApiResponse<Json<ApiResponse<UserResponse>>> {
    let user = authenticate_user(&headers, &state).await?;
    Ok(Json(ApiResponse::success(user.into())))
}

pub async fn collect_wallet(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WalletConnectRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let mut user = authenticate_user(&headers, &state).await?;

    // Verify wallet signature ()
    if !verify_wallet_signature(&payload.wallet_address, &payload.signature, &payload.message) {
        return Err(ApiError::BadRequest("Invalid wallet signature".to_string()));
    }

    // Update user's wallet address
    user.wallet_address = Some(payload.wallet_address);

    sqlx::query!(
        "UPDATE users SET wallet_address = $1 WHERE id = $2",
        user.wallet_address,
        user.id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(ApiResponse::success(user.into())))
}

pub async fn disconnect_wallet(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> ApiResponse<Json<ApiResponse<UserResponse>>> {
    let mut user = authenticate_user(&headers, &state).await?;
    user.wallet_address = None;

    sqlx::query!(
        "UPDATE users SET wallet_address = NULL WHERE id = $1",
        user.id
    )
    .execute(&state.db)
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

async fn authenticate_user(headers: &HeaderMap, state: &Arc<AppState>) -> ApiResult<User> {
    let token = extract_token_from_header(headers)?;
    let claims = decode_token(&token, &state.jwt_secret)?;
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::BadRequest("Invalid user ID in token".to_string()))?;
    
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::Unauthorized)
}

fn verify_wallet_signature(_address: &str, _signature: &str, _message: &str) -> bool {
    // TODO: Implement proper wallet signature verification using ethers or web3
    // This is a placeholder implementation
    true
}