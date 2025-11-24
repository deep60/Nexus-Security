use chrono::Utc;
use redis::AsyncCommands;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthService;
use crate::config::Config;
use crate::models::*;

pub struct UserService {
    config: Config,
    db_pool: PgPool,
    redis_conn: redis::aio::ConnectionManager,
    auth_service: Arc<AuthService>,
}

impl UserService {
    pub async fn new(
        config: Config,
        db_pool: PgPool,
        redis_conn: redis::aio::ConnectionManager,
    ) -> UserResult<Self> {
        let auth_service = Arc::new(AuthService::new(config.jwt.clone()));

        Ok(Self {
            config,
            db_pool,
            redis_conn,
            auth_service,
        })
    }

    // ============= Authentication Methods =============

    /// Register a new user
    pub async fn register(&self, req: RegisterRequest) -> UserResult<AuthResponse> {
        req.validate()
            .map_err(|e| UserError::ValidationError(format!("{}", e)))?;

        // Check if user already exists
        let existing = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 OR username = $2"
        )
        .bind(&req.email)
        .bind(&req.username)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(UserError::AlreadyExists);
        }

        // Hash password
        let password_hash = self.auth_service.hash_password(&req.password)?;

        // Create user
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, ethereum_address, email_verified,
                              is_active, is_admin, two_factor_enabled, kyc_status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, false, true, false, false, 'not_submitted', NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&req.username)
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.ethereum_address)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Create default profile
        sqlx::query(
            r#"
            INSERT INTO user_profiles (user_id, created_at, updated_at)
            VALUES ($1, NOW(), NOW())
            "#
        )
        .bind(user.id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Create default settings
        sqlx::query(
            r#"
            INSERT INTO user_settings (user_id, email_notifications, push_notifications,
                                       webhook_notifications, privacy_public_profile,
                                       privacy_show_email, privacy_show_stats,
                                       language, timezone, updated_at)
            VALUES ($1, true, true, false, true, false, true, 'en', 'UTC', NOW())
            "#
        )
        .bind(user.id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Generate verification token
        let verification_token = self.auth_service.generate_verification_token();
        let token_key = format!("email_verification:{}", user.id);
        let mut conn = self.redis_conn.clone();
        conn.set_ex::<_, _, ()>(&token_key, &verification_token, 3600 * 24) // 24 hours
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // TODO: Send verification email via notification service

        // Generate tokens
        let access_token = self.auth_service.generate_access_token(
            user.id,
            &user.email,
            &user.username,
            user.is_admin,
        )?;

        let refresh_token = self.auth_service.generate_refresh_token(
            user.id,
            &user.email,
            &user.username,
            user.is_admin,
        )?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.into(),
            expires_in: self.auth_service.get_access_token_expiry(),
        })
    }

    /// Login user
    pub async fn login(&self, req: LoginRequest) -> UserResult<AuthResponse> {
        req.validate()
            .map_err(|e| UserError::ValidationError(format!("{}", e)))?;

        // Find user by email
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&req.email)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
            .ok_or(UserError::AuthenticationError("Invalid credentials".to_string()))?;

        // Check if user is active
        if !user.is_active {
            return Err(UserError::Unauthorized("Account is suspended".to_string()));
        }

        // Verify password
        if !self.auth_service.verify_password(&req.password, &user.password_hash)? {
            return Err(UserError::AuthenticationError("Invalid credentials".to_string()));
        }

        // Check 2FA if enabled
        if user.two_factor_enabled {
            let code = req.two_factor_code.ok_or(
                UserError::AuthenticationError("2FA code required".to_string())
            )?;

            let secret = user.two_factor_secret.as_ref().ok_or(
                UserError::DatabaseError("2FA secret not found".to_string())
            )?;

            if !self.auth_service.verify_2fa_code(secret, &code)? {
                return Err(UserError::AuthenticationError("Invalid 2FA code".to_string()));
            }
        }

        // Update last login
        sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Generate tokens
        let access_token = self.auth_service.generate_access_token(
            user.id,
            &user.email,
            &user.username,
            user.is_admin,
        )?;

        let refresh_token = self.auth_service.generate_refresh_token(
            user.id,
            &user.email,
            &user.username,
            user.is_admin,
        )?;

        // Store refresh token in Redis
        let session_key = format!("session:{}", user.id);
        let mut conn = self.redis_conn.clone();
        conn.set_ex::<_, _, ()>(&session_key, &refresh_token, self.config.redis.session_ttl_seconds)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.into(),
            expires_in: self.auth_service.get_access_token_expiry(),
        })
    }

    /// Logout user
    pub async fn logout(&self, user_id: Uuid) -> UserResult<()> {
        let session_key = format!("session:{}", user_id);
        let mut conn = self.redis_conn.clone();

        conn.del::<_, ()>(&session_key)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> UserResult<AuthResponse> {
        // Validate refresh token
        let claims = self.auth_service.validate_token(refresh_token)?;

        if claims.token_type != "refresh" {
            return Err(UserError::InvalidToken);
        }

        // Verify session exists in Redis
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| UserError::InvalidToken)?;

        let session_key = format!("session:{}", user_id);
        let mut conn = self.redis_conn.clone();

        let stored_token: Option<String> = conn.get(&session_key)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if stored_token.as_deref() != Some(refresh_token) {
            return Err(UserError::InvalidToken);
        }

        // Get user from database
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
            .ok_or(UserError::NotFound)?;

        // Generate new tokens
        let new_access_token = self.auth_service.generate_access_token(
            user.id,
            &user.email,
            &user.username,
            user.is_admin,
        )?;

        let new_refresh_token = self.auth_service.generate_refresh_token(
            user.id,
            &user.email,
            &user.username,
            user.is_admin,
        )?;

        // Update session in Redis
        conn.set_ex::<_, _, ()>(&session_key, &new_refresh_token, self.config.redis.session_ttl_seconds)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(AuthResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            user: user.into(),
            expires_in: self.auth_service.get_access_token_expiry(),
        })
    }

    /// Verify email
    pub async fn verify_email(&self, user_id: Uuid, token: &str) -> UserResult<()> {
        let token_key = format!("email_verification:{}", user_id);
        let mut conn = self.redis_conn.clone();

        let stored_token: Option<String> = conn.get(&token_key)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if stored_token.as_deref() != Some(token) {
            return Err(UserError::ValidationError("Invalid or expired verification token".to_string()));
        }

        // Update user
        sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Delete token
        conn.del::<_, ()>(&token_key)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ============= Profile Methods =============

    /// Get user profile
    pub async fn get_profile(&self, user_id: Uuid) -> UserResult<UserProfile> {
        sqlx::query_as::<_, UserProfile>("SELECT * FROM user_profiles WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
            .ok_or(UserError::NotFound)
    }

    /// Update user profile
    pub async fn update_profile(&self, user_id: Uuid, req: UpdateProfileRequest) -> UserResult<UserProfile> {
        let profile = sqlx::query_as::<_, UserProfile>(
            r#"
            UPDATE user_profiles
            SET display_name = COALESCE($1, display_name),
                bio = COALESCE($2, bio),
                location = COALESCE($3, location),
                website = COALESCE($4, website),
                twitter = COALESCE($5, twitter),
                github = COALESCE($6, github),
                specializations = COALESCE($7, specializations),
                updated_at = NOW()
            WHERE user_id = $8
            RETURNING *
            "#
        )
        .bind(&req.display_name)
        .bind(&req.bio)
        .bind(&req.location)
        .bind(&req.website)
        .bind(&req.twitter)
        .bind(&req.github)
        .bind(&req.specializations)
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(profile)
    }

    // ============= Settings Methods =============

    /// Get user settings
    pub async fn get_settings(&self, user_id: Uuid) -> UserResult<UserSettings> {
        sqlx::query_as::<_, UserSettings>("SELECT * FROM user_settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
            .ok_or(UserError::NotFound)
    }

    /// Change password
    pub async fn change_password(&self, user_id: Uuid, req: ChangePasswordRequest) -> UserResult<()> {
        // Get user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
            .ok_or(UserError::NotFound)?;

        // Verify current password
        if !self.auth_service.verify_password(&req.current_password, &user.password_hash)? {
            return Err(UserError::AuthenticationError("Invalid current password".to_string()));
        }

        // Hash new password
        let new_hash = self.auth_service.hash_password(&req.new_password)?;

        // Update password
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(&new_hash)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Enable 2FA
    pub async fn enable_2fa(&self, user_id: Uuid) -> UserResult<String> {
        // Generate 2FA secret
        let secret = self.auth_service.generate_2fa_secret();

        // Store in database
        sqlx::query("UPDATE users SET two_factor_secret = $1 WHERE id = $2")
            .bind(&secret)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(secret)
    }

    /// Verify and activate 2FA
    pub async fn verify_2fa(&self, user_id: Uuid, code: &str) -> UserResult<()> {
        // Get user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
            .ok_or(UserError::NotFound)?;

        let secret = user.two_factor_secret
            .ok_or(UserError::ValidationError("2FA not set up".to_string()))?;

        // Verify code
        if !self.auth_service.verify_2fa_code(&secret, code)? {
            return Err(UserError::AuthenticationError("Invalid 2FA code".to_string()));
        }

        // Enable 2FA
        sqlx::query("UPDATE users SET two_factor_enabled = true WHERE id = $1")
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Disable 2FA
    pub async fn disable_2fa(&self, user_id: Uuid, code: &str) -> UserResult<()> {
        // Get user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
            .ok_or(UserError::NotFound)?;

        if !user.two_factor_enabled {
            return Err(UserError::ValidationError("2FA is not enabled".to_string()));
        }

        let secret = user.two_factor_secret
            .ok_or(UserError::ValidationError("2FA secret not found".to_string()))?;

        // Verify code before disabling
        if !self.auth_service.verify_2fa_code(&secret, code)? {
            return Err(UserError::AuthenticationError("Invalid 2FA code".to_string()));
        }

        // Disable 2FA
        sqlx::query("UPDATE users SET two_factor_enabled = false, two_factor_secret = NULL WHERE id = $1")
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ============= KYC Methods =============

    /// Submit KYC
    pub async fn submit_kyc(&self, user_id: Uuid, req: SubmitKycRequest) -> UserResult<Uuid> {
        // Parse date
        let date_of_birth = chrono::NaiveDate::parse_from_str(&req.date_of_birth, "%Y-%m-%d")
            .map_err(|_| UserError::ValidationError("Invalid date format, use YYYY-MM-DD".to_string()))?;

        // Create KYC verification (document URLs will be added via upload endpoint)
        let kyc_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO kyc_verifications
            (id, user_id, full_name, date_of_birth, country, document_type, document_number,
             document_front_url, selfie_url, status, submitted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, '', '', 'pending', NOW())
            "#
        )
        .bind(kyc_id)
        .bind(user_id)
        .bind(&req.full_name)
        .bind(date_of_birth)
        .bind(&req.country)
        .bind(&req.document_type)
        .bind(&req.document_number)
        .execute(&self.db_pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Update user KYC status
        sqlx::query("UPDATE users SET kyc_status = 'pending' WHERE id = $1")
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(kyc_id)
    }

    /// Get KYC status
    pub async fn get_kyc_status(&self, user_id: Uuid) -> UserResult<Option<KycVerification>> {
        sqlx::query_as::<_, KycVerification>(
            "SELECT * FROM kyc_verifications WHERE user_id = $1 ORDER BY submitted_at DESC LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))
    }

    // ============= Wallet Methods =============

    /// Link Ethereum wallet
    pub async fn link_wallet(&self, user_id: Uuid, address: &str, signature: &str, message: &str) -> UserResult<()> {
        // Verify signature
        if !self.auth_service.verify_wallet_signature(message, signature, address)? {
            return Err(UserError::ValidationError("Invalid wallet signature".to_string()));
        }

        // Update user
        sqlx::query("UPDATE users SET ethereum_address = $1 WHERE id = $2")
            .bind(address)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: Uuid) -> UserResult<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?
            .ok_or(UserError::NotFound)
    }
}
