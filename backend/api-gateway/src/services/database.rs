use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::models::{
    analysis::AnalysisResult,
    bounty::{Bounty, BountyStatus, CreateBountyRequest},
    user::{User, UserRole},
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

    /// Get database connection pool for use as executor
    pub fn executor(&self) -> &PgPool {
        &self.pool
    }

    // ═══════════════════════════════════════════════
    // User operations
    // ═══════════════════════════════════════════════

    pub async fn create_user(
        &self,
        wallet_address: &str,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, wallet_address, username, email, password_hash, reputation_score, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(wallet_address)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(0i32)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to create user")?;

        Ok(user)
    }

    pub async fn get_user_by_wallet(&self, wallet_address: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE wallet_address = $1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by wallet address")?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by ID")?;

        Ok(user)
    }

    pub async fn update_user_reputation(&self, user_id: Uuid, reputation_delta: i32) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET reputation_score = reputation_score + $1, updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(reputation_delta)
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context("Failed to update user reputation")?;

        Ok(())
    }

    // ═══════════════════════════════════════════════
    // Bounty operations — matches `bounties` table
    // ═══════════════════════════════════════════════

    pub async fn create_bounty(
        &self,
        request: CreateBountyRequest,
        creator_id: Uuid,
    ) -> Result<Bounty> {
        let now = Utc::now();
        let bounty_id = Uuid::new_v4();
        let deadline = request.deadline_hours.map(|h| now + chrono::Duration::hours(h as i64));
        let min_stake = request.min_stake_amount.unwrap_or_else(|| "0".to_string());
        let consensus = request.consensus_threshold.unwrap_or(0.60);
        let priority = request.priority_level.unwrap_or(1);
        let requires_verification = request.requires_verification.unwrap_or(false);

        let bounty = sqlx::query_as::<_, Bounty>(
            r#"
            INSERT INTO bounties (
                id, creator_id, submission_id, title, description,
                reward_amount, min_stake_amount, max_participants,
                deadline, bounty_status, requires_verification,
                priority_level, consensus_threshold,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8,
                $9, 'active', $10,
                $11, $12,
                $13, $14
            )
            RETURNING *
            "#,
        )
        .bind(bounty_id)
        .bind(creator_id)
        .bind(request.submission_id)
        .bind(&request.title)
        .bind(&request.description)
        .bind(&request.reward_amount)
        .bind(&min_stake)
        .bind(request.max_participants)
        .bind(deadline)
        .bind(requires_verification)
        .bind(priority)
        .bind(consensus)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create bounty")?;

        Ok(bounty)
    }

    /// Store the on-chain bounty ID (incremental counter from BountyManager) and tx hash
    pub async fn update_bounty_on_chain_id(
        &self,
        bounty_id: Uuid,
        tx_hash: &str,
        on_chain_id: i64,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE bounties SET blockchain_tx_hash = $1, on_chain_id = $2, updated_at = $3 WHERE id = $4"
        )
        .bind(tx_hash)
        .bind(on_chain_id)
        .bind(Utc::now())
        .bind(bounty_id)
        .execute(&self.pool)
        .await
        .context("Failed to update bounty on_chain_id")?;

        Ok(())
    }

    /// Look up the on-chain bounty ID for a given DB bounty UUID.
    pub async fn get_bounty_on_chain_id(&self, bounty_id: Uuid) -> Result<Option<i64>> {
        let row: Option<(Option<i64>,)> = sqlx::query_as(
            "SELECT on_chain_id FROM bounties WHERE id = $1"
        )
        .bind(bounty_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch bounty on_chain_id")?;

        Ok(row.and_then(|r| r.0))
    }

    pub async fn get_bounty_by_id(&self, bounty_id: Uuid) -> Result<Option<Bounty>> {
        let bounty = sqlx::query_as::<_, Bounty>(
            r#"
            SELECT * FROM bounties
            WHERE id = $1
            "#,
        )
        .bind(bounty_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch bounty by ID")?;

        Ok(bounty)
    }

    pub async fn get_active_bounties(&self, limit: i64, offset: i64) -> Result<Vec<Bounty>> {
        let bounties = sqlx::query_as::<_, Bounty>(
            r#"
            SELECT * FROM bounties
            WHERE bounty_status = 'active' AND (deadline IS NULL OR deadline > $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(Utc::now())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch active bounties")?;

        Ok(bounties)
    }

    pub async fn get_bounties_by_creator(&self, creator_id: Uuid) -> Result<Vec<Bounty>> {
        let bounties = sqlx::query_as::<_, Bounty>(
            r#"
            SELECT * FROM bounties
            WHERE creator_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(creator_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch bounties by creator")?;

        Ok(bounties)
    }

    pub async fn update_bounty_status(&self, bounty_id: Uuid, status: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE bounties
            SET bounty_status = $1, updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(status)
        .bind(Utc::now())
        .bind(bounty_id)
        .execute(&self.pool)
        .await
        .context("Failed to update bounty status")?;

        Ok(())
    }

    // ═══════════════════════════════════════════════
    // Analysis operations — matches `analysis_results` table
    // ═══════════════════════════════════════════════

    pub async fn create_analysis_result(
        &self,
        engine_id: Uuid,
        submission_id: Uuid,
        bounty_id: Option<Uuid>,
        verdict: &str,
        confidence_score: f64,
        detailed_report: serde_json::Value,
    ) -> Result<AnalysisResult> {
        let analysis = sqlx::query_as::<_, AnalysisResult>(
            r#"
            INSERT INTO analysis_results (
                id, engine_id, submission_id, bounty_id,
                verdict, confidence_score, detailed_report,
                analysis_status, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'completed', $8)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(engine_id)
        .bind(submission_id)
        .bind(bounty_id)
        .bind(verdict)
        .bind(confidence_score)
        .bind(detailed_report)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to create analysis result")?;

        Ok(analysis)
    }

    pub async fn get_analysis_results_by_bounty(
        &self,
        bounty_id: Uuid,
    ) -> Result<Vec<AnalysisResult>> {
        let results = sqlx::query_as::<_, AnalysisResult>(
            r#"
            SELECT * FROM analysis_results
            WHERE bounty_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(bounty_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch analysis results by bounty")?;

        Ok(results)
    }

    pub async fn get_analysis_results_by_engine(
        &self,
        engine_id: Uuid,
    ) -> Result<Vec<AnalysisResult>> {
        let results = sqlx::query_as::<_, AnalysisResult>(
            r#"
            SELECT * FROM analysis_results
            WHERE engine_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(engine_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch analysis results by engine")?;

        Ok(results)
    }

    // ═══════════════════════════════════════════════
    // Consensus and reputation operations
    // ═══════════════════════════════════════════════

    pub async fn calculate_consensus_for_bounty(
        &self,
        bounty_id: Uuid,
    ) -> Result<Option<ConsensusResult>> {
        let consensus = sqlx::query_as::<_, ConsensusResult>(
            r#"
            SELECT
                COUNT(*) as total_analyses,
                AVG(confidence_score::float) as avg_confidence,
                COUNT(CASE WHEN verdict = 'malicious' THEN 1 END) as malicious_count,
                COUNT(CASE WHEN verdict = 'benign' THEN 1 END) as benign_count,
                COUNT(CASE WHEN verdict = 'suspicious' THEN 1 END) as suspicious_count
            FROM analysis_results
            WHERE bounty_id = $1 AND analysis_status = 'completed'
            "#,
        )
        .bind(bounty_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to calculate consensus")?;

        Ok(consensus)
    }

    pub async fn get_user_analysis_stats(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserAnalysisStats>> {
        let stats = sqlx::query_as::<_, UserAnalysisStats>(
            r#"
            SELECT
                COUNT(*) as total_analyses,
                AVG(confidence_score::float) as avg_confidence,
                COUNT(CASE WHEN verdict = 'malicious' THEN 1 END) as malicious_detections,
                COUNT(CASE WHEN verdict = 'benign' THEN 1 END) as benign_detections
            FROM analysis_results
            WHERE analyzer_id = $1 AND analysis_status = 'completed'
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user analysis stats")?;

        Ok(stats)
    }

    // ═══════════════════════════════════════════════
    // Transaction management
    // ═══════════════════════════════════════════════

    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.pool
            .begin()
            .await
            .context("Failed to begin transaction")
    }

    // Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Database health check failed")?;

        Ok(())
    }

    // === Submission-related methods (stubs for compilation) ===

    /// Get analysis result by ID
    pub async fn get_analysis_result(&self, _analysis_id: Uuid) -> Result<AnalysisResult> {
        anyhow::bail!("get_analysis_result not yet implemented")
    }

    /// Store file metadata
    pub async fn store_file_metadata(
        &self,
        _file_id: Uuid,
        _metadata: &crate::models::analysis::FileMetadata,
    ) -> Result<()> {
        anyhow::bail!("store_file_metadata not yet implemented")
    }

    /// Get file info by hash
    pub async fn get_file_info(
        &self,
        _file_hash: &str,
    ) -> Result<Option<crate::handlers::submission::FileInfo>> {
        Ok(None)
    }

    /// Create extended submission
    pub async fn create_extended_submission(
        &self,
        _submission: &crate::models::bounty::BountySubmission,
    ) -> Result<()> {
        anyhow::bail!("create_extended_submission not yet implemented")
    }

    /// Get submissions with filters
    pub async fn get_submissions_with_filters(
        &self,
        _filters: &crate::handlers::submission::SubmissionFilters,
        _page: u32,
        _limit: u32,
    ) -> Result<(Vec<crate::models::bounty::BountySubmission>, u32)> {
        Ok((Vec::new(), 0))
    }

    /// Get extended submission by ID
    pub async fn get_extended_submission_by_id(
        &self,
        _submission_id: Uuid,
    ) -> Result<Option<crate::models::bounty::ExtendedSubmission>> {
        Ok(None)
    }

    /// Update submission
    pub async fn update_submission(
        &self,
        _submission_id: Uuid,
        _updates: &crate::handlers::submission::UpdateSubmissionRequest,
    ) -> Result<crate::models::bounty::ExtendedSubmission> {
        anyhow::bail!("update_submission not yet implemented")
    }

    /// Delete submission
    pub async fn delete_submission(&self, _submission_id: Uuid) -> Result<()> {
        anyhow::bail!("delete_submission not yet implemented")
    }
}

// Helper structs for complex queries
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConsensusResult {
    pub total_analyses: Option<i64>,
    pub avg_confidence: Option<f64>,
    pub malicious_count: Option<i64>,
    pub benign_count: Option<i64>,
    pub suspicious_count: Option<i64>,
}

impl ConsensusResult {
    pub fn get_consensus_verdict(&self) -> Option<String> {
        let malicious = self.malicious_count.unwrap_or(0);
        let benign = self.benign_count.unwrap_or(0);
        let suspicious = self.suspicious_count.unwrap_or(0);
        let total = self.total_analyses.unwrap_or(0);

        if total == 0 {
            return None;
        }

        if malicious > benign && malicious > suspicious {
            Some("malicious".to_string())
        } else if benign > malicious && benign > suspicious {
            Some("benign".to_string())
        } else {
            Some("suspicious".to_string())
        }
    }

    pub fn get_consensus_confidence(&self) -> f32 {
        self.avg_confidence.unwrap_or(0.0) as f32
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
