use sqlx::{PgPool, Row};
use anyhow::{Result, Context};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{
    bounty::{Bounty, BountyStatus, CreateBountyRequest},
    user::{User, UserRole},
    analysis::{AnalysisResult, AnalysisStatus, ThreatVerdict}
};

#[derive(Clone)]
pub struct DatabaseService {
    pool: PgPool,
}

impl DatabaseService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get database connection pool reference
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // User operations
    pub async fn create_user(&self, wallet_address: &str, username: &str, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, wallet_address, username, email, password_hash, reputation_score, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            Uuid::new_v4(),
            wallet_address,
            username,
            email,
            password_hash,
            0i32,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create user")?;

        Ok(user)
    }

    pub async fn get_user_by_wallet(&self, wallet_address: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users 
            WHERE wallet_address = $1
            "#,
            wallet_address
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by wallet address")?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users 
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by ID")?;

        Ok(user)
    }

    pub async fn update_user_reputation(&self, user_id: Uuid, reputation_delta: i32) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET reputation_score = reputation_score + $1, updated_at = $2
            WHERE id = $3
            "#,
            reputation_delta,
            Utc::now(),
            user_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update user reputation")?;

        Ok(())
    }

    // Bounty operations
    pub async fn create_bounty(&self, request: CreateBountyRequest, creator_id: Uuid) -> Result<Bounty> {
        let bounty = sqlx::query_as!(
            Bounty,
            r#"
            INSERT INTO bounties (
                id, title, description, reward_amount, creator_id, status, 
                target_hash, target_url, target_type, expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING 
                id, title, description, reward_amount, creator_id, 
                status as "status: BountyStatus", target_hash, target_url, 
                target_type, expires_at, created_at, updated_at
            "#,
            Uuid::new_v4(),
            request.title,
            request.description,
            request.reward_amount,
            creator_id,
            BountyStatus::Active as BountyStatus,
            request.target_hash,
            request.target_url,
            request.target_type,
            request.expires_at,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create bounty")?;

        Ok(bounty)
    }

    pub async fn get_bounty_by_id(&self, bounty_id: Uuid) -> Result<Option<Bounty>> {
        let bounty = sqlx::query_as!(
            Bounty,
            r#"
            SELECT 
                id, title, description, reward_amount, creator_id, 
                status as "status: BountyStatus", target_hash, target_url, 
                target_type, expires_at, created_at, updated_at
            FROM bounties 
            WHERE id = $1
            "#,
            bounty_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch bounty by ID")?;

        Ok(bounty)
    }

    pub async fn get_active_bounties(&self, limit: i64, offset: i64) -> Result<Vec<Bounty>> {
        let bounties = sqlx::query_as!(
            Bounty,
            r#"
            SELECT 
                id, title, description, reward_amount, creator_id, 
                status as "status: BountyStatus", target_hash, target_url, 
                target_type, expires_at, created_at, updated_at
            FROM bounties 
            WHERE status = $1 AND expires_at > $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            BountyStatus::Active as BountyStatus,
            Utc::now(),
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch active bounties")?;

        Ok(bounties)
    }

    pub async fn get_bounties_by_creator(&self, creator_id: Uuid) -> Result<Vec<Bounty>> {
        let bounties = sqlx::query_as!(
            Bounty,
            r#"
            SELECT 
                id, title, description, reward_amount, creator_id, 
                status as "status: BountyStatus", target_hash, target_url, 
                target_type, expires_at, created_at, updated_at
            FROM bounties 
            WHERE creator_id = $1
            ORDER BY created_at DESC
            "#,
            creator_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch bounties by creator")?;

        Ok(bounties)
    }

    pub async fn update_bounty_status(&self, bounty_id: Uuid, status: BountyStatus) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE bounties 
            SET status = $1, updated_at = $2
            WHERE id = $3
            "#,
            status as BountyStatus,
            Utc::now(),
            bounty_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update bounty status")?;

        Ok(())
    }

    // Analysis operations
    pub async fn create_analysis_result(
        &self, 
        bounty_id: Uuid, 
        analyzer_id: Option<Uuid>,
        engine_name: &str,
        verdict: ThreatVerdict,
        confidence_score: f32,
        metadata: serde_json::Value
    ) -> Result<AnalysisResult> {
        let analysis = sqlx::query_as!(
            AnalysisResult,
            r#"
            INSERT INTO analysis_results (
                id, bounty_id, analyzer_id, engine_name, verdict, 
                confidence_score, metadata, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING 
                id, bounty_id, analyzer_id, engine_name, 
                verdict as "verdict: ThreatVerdict", confidence_score, 
                metadata, status as "status: AnalysisStatus", 
                created_at, updated_at
            "#,
            Uuid::new_v4(),
            bounty_id,
            analyzer_id,
            engine_name,
            verdict as ThreatVerdict,
            confidence_score,
            metadata,
            AnalysisStatus::Completed as AnalysisStatus,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create analysis result")?;

        Ok(analysis)
    }

    pub async fn get_analysis_results_by_bounty(&self, bounty_id: Uuid) -> Result<Vec<AnalysisResult>> {
        let results = sqlx::query_as!(
            AnalysisResult,
            r#"
            SELECT 
                id, bounty_id, analyzer_id, engine_name, 
                verdict as "verdict: ThreatVerdict", confidence_score, 
                metadata, status as "status: AnalysisStatus", 
                created_at, updated_at
            FROM analysis_results 
            WHERE bounty_id = $1
            ORDER BY created_at DESC
            "#,
            bounty_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch analysis results by bounty")?;

        Ok(results)
    }

    pub async fn get_analysis_results_by_analyzer(&self, analyzer_id: Uuid) -> Result<Vec<AnalysisResult>> {
        let results = sqlx::query_as!(
            AnalysisResult,
            r#"
            SELECT 
                id, bounty_id, analyzer_id, engine_name, 
                verdict as "verdict: ThreatVerdict", confidence_score, 
                metadata, status as "status: AnalysisStatus", 
                created_at, updated_at
            FROM analysis_results 
            WHERE analyzer_id = $1
            ORDER BY created_at DESC
            "#,
            analyzer_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch analysis results by analyzer")?;

        Ok(results)
    }

    // Consensus and reputation operations
    pub async fn calculate_consensus_for_bounty(&self, bounty_id: Uuid) -> Result<Option<ConsensusResult>> {
        let consensus = sqlx::query_as!(
            ConsensusResult,
            r#"
            SELECT 
                COUNT(*) as total_analyses,
                AVG(confidence_score) as avg_confidence,
                COUNT(CASE WHEN verdict = 'malicious' THEN 1 END) as malicious_count,
                COUNT(CASE WHEN verdict = 'benign' THEN 1 END) as benign_count,
                COUNT(CASE WHEN verdict = 'suspicious' THEN 1 END) as suspicious_count
            FROM analysis_results 
            WHERE bounty_id = $1 AND status = 'completed'
            "#,
            bounty_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to calculate consensus")?;

        Ok(consensus)
    }

    pub async fn get_user_analysis_stats(&self, user_id: Uuid) -> Result<Option<UserAnalysisStats>> {
        let stats = sqlx::query_as!(
            UserAnalysisStats,
            r#"
            SELECT 
                COUNT(*) as total_analyses,
                AVG(confidence_score) as avg_confidence,
                COUNT(CASE WHEN verdict = 'malicious' THEN 1 END) as malicious_detections,
                COUNT(CASE WHEN verdict = 'benign' THEN 1 END) as benign_detections
            FROM analysis_results 
            WHERE analyzer_id = $1 AND status = 'completed'
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user analysis stats")?;

        Ok(stats)
    }

    // Transaction management
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.pool.begin().await.context("Failed to begin transaction")
    }

    // Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Database health check failed")?;
        
        Ok(())
    }
}

// Helper structs for complex queries
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub total_analyses: Option<i64>,
    pub avg_confidence: Option<f64>,
    pub malicious_count: Option<i64>,
    pub benign_count: Option<i64>,
    pub suspicious_count: Option<i64>,
}

impl ConsensusResult {
    pub fn get_consensus_verdict(&self) -> Option<ThreatVerdict> {
        let malicious = self.malicious_count.unwrap_or(0);
        let benign = self.benign_count.unwrap_or(0);
        let suspicious = self.suspicious_count.unwrap_or(0);
        let total = self.total_analyses.unwrap_or(0);

        if total == 0 {
            return None;
        }

        if malicious > benign && malicious > suspicious {
            Some(ThreatVerdict::Malicious)
        } else if benign > malicious && benign > suspicious {
            Some(ThreatVerdict::Benign)
        } else {
            Some(ThreatVerdict::Suspicious)
        }
    }

    pub fn get_consensus_confidence(&self) -> f32 {
        self.avg_confidence.unwrap_or(0.0) as f32
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAnalysisStats {
    pub total_analyses: Option<i64>,
    pub avg_confidence: Option<f64>,
    pub malicious_detections: Option<i64>,
    pub benign_detections: Option<i64>,
}

// Database connection helper
pub async fn create_connection_pool(database_url: &str) -> Result<PgPool> {
    PgPool::connect(database_url)
        .await
        .context("Failed to create database connection pool")
}

// Migration helper
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Failed to run database migrations")?;
    
    Ok(())
}