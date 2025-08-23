use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// User role enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Engine,
    Admin,
    Moderator,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub wallet_address: Option<String>,
    pub reputation_score: i32,
    pub total_stakes: i64, // in wei or smallest token unit
    pub successful_analyses: i32,
    pub failed_analyses: i32,
    pub is_verified: bool,
    pub is_active: bool,
    pub is_engine: bool, // true if user operates an analysis engine
    pub api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub is_engine: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub wallet_address: Option<String>,
    pub reputation_score: i32,
    pub total_stakes: i64,
    pub successful_analyses: i32,
    pub failed_analyses: i32,
    pub success_rate: f64,
    pub is_verified: bool,
    pub is_engine: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub total_bounties_participated: i32,
    pub total_earnings: i64, // in wei
    pub average_accuracy: f64,
    pub rank: i32,
    pub recent_analyses: Vec<RecentAnalysis>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentAnalysis {
    pub bounty_id: Uuid,
    pub verdict: String, // "malicious", "benign", "suspicious"
    pub confidence: f64,
    pub stake_amount: i64,
    pub was_correct: Option<bool>,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationUpdate {
    pub user_id: Uuid,
    pub change: i32, // positive for gain, negative for loss
    pub reason: String,
    pub bounty_id: Option<Uuid>,
}

impl User {
    pub fn new(
        username: String,
        email: String,
        password_hash: String,
        wallet_address: Option<String>,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            wallet_address,
            reputation_score: 100, // Starting reputation
            total_stakes: 0,
            successful_analyses: 0,
            failed_analyses: 0,
            is_verified: false,
            is_active: true,
            is_engine: false,
            api_key: None,
            created_at: now,
            updated_at: now,
            last_login: None,
        }
    }

    pub fn to_response(&self) -> UserResponse {
        let success_rate = if self.successful_analyses + self.failed_analyses > 0 {
            self.successful_analyses as f64 / (self.successful_analyses + self.failed_analyses) as f64
        } else {
            0.0
        };

        UserResponse {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            wallet_address: self.wallet_address.clone(),
            reputation_score: self.reputation_score,
            total_stakes: self.total_stakes,
            successful_analyses: self.successful_analyses,
            failed_analyses: self.failed_analyses,
            success_rate,
            is_verified: self.is_verified,
            is_engine: self.is_engine,
            created_at: self.created_at,
            last_login: self.last_login,
        }
    }

    pub fn update_reputation(&mut self, change: i32) {
        self.reputation_score = (self.reputation_score + change).max(0); // Prevent negative reputation
        self.updated_at = Utc::now();
    }

    pub fn record_analysis_result(&mut self, was_successful: bool, stake_amount: i64) {
        if was_successful {
            self.successful_analyses += 1;
            self.reputation_score += 10; // Bonus reputation for correct analysis
        } else {
            self.failed_analyses += 1;
            self.reputation_score = (self.reputation_score - 5).max(0); // Penalty for incorrect analysis
        }
        
        self.total_stakes += stake_amount;
        self.updated_at = Utc::now();
    }

    pub fn can_participate_in_bounty(&self, required_reputation: i32) -> bool {
        self.is_active && self.reputation_score >= required_reputation
    }

    pub fn generate_api_key(&mut self) -> String {
        let api_key = format!("nxs_{}", Uuid::new_v4().to_string().replace("-", ""));
        self.api_key = Some(api_key.clone());
        self.updated_at = Utc::now();
        api_key
    }

    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn verify_account(&mut self) {
        self.is_verified = true;
        self.reputation_score += 50; // Verification bonus
        self.updated_at = Utc::now();
    }

    pub fn suspend_account(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate_account(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
}

// Database-related implementations
impl User {
    pub async fn find_by_id(
        pool: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_email(
        pool: &sqlx::PgPool,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_username(
        pool: &sqlx::PgPool,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE username = $1",
            username
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_api_key(
        pool: &sqlx::PgPool,
        api_key: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE api_key = $1 AND is_active = true",
            api_key
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        user: &User,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (
                id, username, email, password_hash, wallet_address,
                reputation_score, total_stakes, successful_analyses, failed_analyses,
                is_verified, is_active, is_engine, api_key, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.wallet_address,
            user.reputation_score,
            user.total_stakes,
            user.successful_analyses,
            user.failed_analyses,
            user.is_verified,
            user.is_active,
            user.is_engine,
            user.api_key,
            user.created_at,
            user.updated_at
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &sqlx::PgPool,
        user: &User,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            UPDATE users SET
                username = $2,
                email = $3,
                password_hash = $4,
                wallet_address = $5,
                reputation_score = $6,
                total_stakes = $7,
                successful_analyses = $8,
                failed_analyses = $9,
                is_verified = $10,
                is_active = $11,
                is_engine = $12,
                api_key = $13,
                updated_at = $14,
                last_login = $15
            WHERE id = $1
            RETURNING *
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.wallet_address,
            user.reputation_score,
            user.total_stakes,
            user.successful_analyses,
            user.failed_analyses,
            user.is_verified,
            user.is_active,
            user.is_engine,
            user.api_key,
            user.updated_at,
            user.last_login
        )
        .fetch_one(pool)
        .await
    }

    pub async fn get_leaderboard(
        pool: &sqlx::PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserResponse>, sqlx::Error> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users 
            WHERE is_active = true 
            ORDER BY reputation_score DESC, successful_analyses DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(users.into_iter().map(|u| u.to_response()).collect())
    }
}