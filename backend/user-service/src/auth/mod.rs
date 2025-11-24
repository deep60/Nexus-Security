use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use ethers::core::types::Signature;
use ethers::signers::{LocalWallet, Signer};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::models::{UserError, UserResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User ID
    pub email: String,
    pub username: String,
    pub is_admin: bool,
    pub exp: i64,           // Expiry timestamp
    pub iat: i64,           // Issued at timestamp
    pub token_type: String, // "access" or "refresh"
}

pub struct AuthService {
    jwt_config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new(jwt_config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(jwt_config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(jwt_config.secret.as_bytes());

        Self {
            jwt_config,
            encoding_key,
            decoding_key,
        }
    }

    /// Hash a password using Argon2
    pub fn hash_password(&self, password: &str) -> UserResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| UserError::ValidationError(format!("Failed to hash password: {}", e)))?
            .to_string();

        Ok(password_hash)
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, password_hash: &str) -> UserResult<bool> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| UserError::ValidationError(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Generate an access token
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        email: &str,
        username: &str,
        is_admin: bool,
    ) -> UserResult<String> {
        let now = Utc::now();
        let expiry = now + Duration::hours(self.jwt_config.access_token_expiry_hours as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            username: username.to_string(),
            is_admin,
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| UserError::AuthenticationError(format!("Failed to generate token: {}", e)))
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(
        &self,
        user_id: Uuid,
        email: &str,
        username: &str,
        is_admin: bool,
    ) -> UserResult<String> {
        let now = Utc::now();
        let expiry = now + Duration::days(self.jwt_config.refresh_token_expiry_days as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            username: username.to_string(),
            is_admin,
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| UserError::AuthenticationError(format!("Failed to generate refresh token: {}", e)))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> UserResult<Claims> {
        let validation = Validation::default();

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|_e| UserError::InvalidToken)
    }

    /// Verify Ethereum wallet signature
    pub fn verify_wallet_signature(
        &self,
        message: &str,
        signature: &str,
        expected_address: &str,
    ) -> UserResult<bool> {
        // Parse the signature
        let sig = Signature::from_str(signature)
            .map_err(|e| UserError::ValidationError(format!("Invalid signature format: {}", e)))?;

        // Recover the address from the signature
        let recovered_address = sig
            .recover(message)
            .map_err(|e| UserError::ValidationError(format!("Failed to recover address: {}", e)))?;

        // Compare with expected address (case-insensitive)
        let recovered_str = format!("{:?}", recovered_address).to_lowercase();
        let expected_str = expected_address.to_lowercase();

        Ok(recovered_str == expected_str)
    }

    /// Generate a verification token for email/password reset
    pub fn generate_verification_token(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const TOKEN_LEN: usize = 32;
        let mut rng = rand::thread_rng();

        (0..TOKEN_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Generate a 2FA secret
    pub fn generate_2fa_secret(&self) -> String {
        use base32::Alphabet;
        use rand::Rng;

        let secret: Vec<u8> = (0..20).map(|_| rand::thread_rng().gen()).collect();
        base32::encode(Alphabet::RFC4648 { padding: false }, &secret)
    }

    /// Verify a 2FA code
    pub fn verify_2fa_code(&self, secret: &str, code: &str) -> UserResult<bool> {
        use totp_lite::{totp_custom, Sha1};

        let time_step = 30; // 30 seconds
        let current_time = Utc::now().timestamp() as u64;

        // Decode the base32 secret
        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret)
            .ok_or_else(|| UserError::ValidationError("Invalid 2FA secret".to_string()))?;

        // Generate current TOTP code
        let expected_code = totp_custom::<Sha1>(time_step, 6, &secret_bytes, current_time);

        // Also check the previous and next time windows for clock drift
        let prev_code = totp_custom::<Sha1>(time_step, 6, &secret_bytes, current_time - time_step);
        let next_code = totp_custom::<Sha1>(time_step, 6, &secret_bytes, current_time + time_step);

        Ok(code == expected_code || code == prev_code || code == next_code)
    }

    /// Get token expiry time in seconds
    pub fn get_access_token_expiry(&self) -> u64 {
        self.jwt_config.access_token_expiry_hours * 3600
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_jwt_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key_for_testing_only".to_string(),
            access_token_expiry_hours: 24,
            refresh_token_expiry_days: 30,
        }
    }

    #[test]
    fn test_password_hashing() {
        let auth_service = AuthService::new(get_test_jwt_config());
        let password = "SecurePassword123!";

        let hash = auth_service.hash_password(password).unwrap();
        assert!(auth_service.verify_password(password, &hash).unwrap());
        assert!(!auth_service.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_token_generation_and_validation() {
        let auth_service = AuthService::new(get_test_jwt_config());
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let username = "testuser";

        let token = auth_service
            .generate_access_token(user_id, email, username, false)
            .unwrap();

        let claims = auth_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_verification_token_generation() {
        let auth_service = AuthService::new(get_test_jwt_config());
        let token = auth_service.generate_verification_token();

        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
