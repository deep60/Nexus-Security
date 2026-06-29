use anyhow::Result;
use chrono::Utc;
use redis::aio::ConnectionManager;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::aggregation::ConsensusAggregator;
use crate::config::Config;
use crate::models::*;

/// Core consensus service: owns DB access, the aggregator, and the cache.
pub struct ConsensusService {
    config: Config,
    db_pool: PgPool,
    redis_conn: ConnectionManager,
    aggregator: ConsensusAggregator,
}

impl ConsensusService {
    pub async fn new(
        config: Config,
        db_pool: PgPool,
        redis_conn: ConnectionManager,
    ) -> Result<Self> {
        let aggregator = ConsensusAggregator::new(config.consensus.clone());

        Ok(Self {
            config,
            db_pool,
            redis_conn,
            aggregator,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Load all votes recorded for a bounty.
    pub async fn load_votes(&self, bounty_id: Uuid) -> ConsensusResult<Vec<SubmissionVote>> {
        let rows = sqlx::query_as::<_, VoteRow>(
            r#"
            SELECT id, bounty_id, engine_id, verdict, confidence, reputation_score, submitted_at
            FROM consensus_submissions
            WHERE bounty_id = $1
            ORDER BY submitted_at ASC
            "#,
        )
        .bind(bounty_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubmissionVote {
                submission_id: r.id,
                user_id: r.id, // engine submissions are keyed by engine; reuse id as placeholder
                engine_id: r.engine_id,
                verdict: parse_verdict(&r.verdict),
                confidence: r.confidence,
                reputation_score: r.reputation_score,
                submitted_at: r.submitted_at,
            })
            .collect())
    }

    /// Record (or update) a single engine's vote for a bounty.
    pub async fn record_vote(
        &self,
        bounty_id: Uuid,
        engine_id: &str,
        verdict: &Verdict,
        confidence: Decimal,
        reputation_score: i32,
    ) -> ConsensusResult<()> {
        sqlx::query(
            r#"
            INSERT INTO consensus_submissions
                (bounty_id, engine_id, verdict, confidence, reputation_score)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (bounty_id, engine_id)
            DO UPDATE SET verdict = EXCLUDED.verdict,
                          confidence = EXCLUDED.confidence,
                          reputation_score = EXCLUDED.reputation_score,
                          submitted_at = NOW()
            "#,
        )
        .bind(bounty_id)
        .bind(engine_id)
        .bind(verdict.to_string())
        .bind(confidence)
        .bind(reputation_score)
        .execute(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Compute consensus for a bounty and persist the result.
    pub async fn calculate_and_store(
        &self,
        bounty_id: Uuid,
        finalize: bool,
    ) -> ConsensusResult<ConsensusResponse> {
        let votes = self.load_votes(bounty_id).await?;

        if votes.len() < self.config.consensus.min_submissions {
            return Err(ConsensusError::InsufficientSubmissions {
                required: self.config.consensus.min_submissions,
                actual: votes.len(),
            });
        }

        let (verdict, confidence, distribution) = self.aggregator.calculate_consensus(&votes);
        let agreement = self.aggregator.calculate_agreement_score(&distribution);
        let can_dispute = self.aggregator.can_be_disputed(agreement);

        let engines: Vec<String> = votes.iter().map(|v| v.engine_id.clone()).collect();
        let distribution_json = serde_json::to_value(&distribution)
            .map_err(|e| ConsensusError::ConsensusFailed(e.to_string()))?;

        let counts = count_verdicts(&votes);

        sqlx::query(
            r#"
            INSERT INTO consensus_results
                (bounty_id, final_verdict, confidence, total_submissions,
                 malicious_count, benign_count, suspicious_count, unknown_count,
                 weighted_voting, agreement_score, is_disputed, is_finalized,
                 finalized_at, participating_engines, verdict_distribution, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW())
            ON CONFLICT (bounty_id) DO UPDATE SET
                final_verdict = EXCLUDED.final_verdict,
                confidence = EXCLUDED.confidence,
                total_submissions = EXCLUDED.total_submissions,
                malicious_count = EXCLUDED.malicious_count,
                benign_count = EXCLUDED.benign_count,
                suspicious_count = EXCLUDED.suspicious_count,
                unknown_count = EXCLUDED.unknown_count,
                agreement_score = EXCLUDED.agreement_score,
                is_disputed = EXCLUDED.is_disputed,
                is_finalized = EXCLUDED.is_finalized,
                finalized_at = EXCLUDED.finalized_at,
                participating_engines = EXCLUDED.participating_engines,
                verdict_distribution = EXCLUDED.verdict_distribution,
                updated_at = NOW()
            "#,
        )
        .bind(bounty_id)
        .bind(verdict.to_string())
        .bind(confidence)
        .bind(votes.len() as i32)
        .bind(counts.malicious)
        .bind(counts.benign)
        .bind(counts.suspicious)
        .bind(counts.unknown)
        .bind(self.config.consensus.weighted_voting)
        .bind(agreement)
        .bind(can_dispute)
        .bind(finalize)
        .bind(if finalize { Some(Utc::now()) } else { None })
        .bind(&engines)
        .bind(&distribution_json)
        .execute(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;

        // Best-effort cache invalidation of the cached response.
        let mut conn = self.redis_conn.clone();
        let _: Result<(), _> = redis::cmd("DEL")
            .arg(format!("consensus:{bounty_id}"))
            .query_async(&mut conn)
            .await;

        Ok(ConsensusResponse {
            bounty_id,
            final_verdict: verdict,
            confidence_score: confidence,
            agreement_score: agreement,
            verdict_distribution: distribution,
            total_submissions: votes.len(),
            is_finalized: finalize,
            can_be_disputed: can_dispute,
        })
    }

    /// Read a previously stored consensus result for a bounty.
    pub async fn get_stored(&self, bounty_id: Uuid) -> ConsensusResult<Option<ConsensusResponse>> {
        let row = sqlx::query_as::<_, ResultRow>(
            r#"
            SELECT bounty_id, final_verdict, confidence, total_submissions,
                   agreement_score, is_finalized, verdict_distribution
            FROM consensus_results
            WHERE bounty_id = $1
            "#,
        )
        .bind(bounty_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let distribution: VerdictDistribution =
            serde_json::from_value(row.verdict_distribution).unwrap_or_default();
        let agreement = row.agreement_score;
        let can_dispute = self.aggregator.can_be_disputed(agreement);

        Ok(Some(ConsensusResponse {
            bounty_id: row.bounty_id,
            final_verdict: parse_verdict(&row.final_verdict),
            confidence_score: row.confidence,
            agreement_score: agreement,
            verdict_distribution: distribution,
            total_submissions: row.total_submissions.to_usize().unwrap_or(0),
            is_finalized: row.is_finalized,
            can_be_disputed: can_dispute,
        }))
    }

    /// Bounties that have enough votes but no finalized result yet,
    /// past the auto-finalize window. Used by the background worker.
    pub async fn find_finalizable_bounties(&self) -> ConsensusResult<Vec<Uuid>> {
        let window_hours = self.config.consensus.auto_finalize_hours as i64;
        let min = self.config.consensus.min_submissions as i64;

        let rows = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT s.bounty_id
            FROM consensus_submissions s
            LEFT JOIN consensus_results r ON r.bounty_id = s.bounty_id
            WHERE (r.is_finalized IS NULL OR r.is_finalized = false)
            GROUP BY s.bounty_id
            HAVING COUNT(*) >= $1
               AND MIN(s.submitted_at) <= NOW() - ($2 || ' hours')::interval
            "#,
        )
        .bind(min)
        .bind(window_hours.to_string())
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;

        Ok(rows)
    }

    // -- Disputes ---------------------------------------------------------

    pub async fn create_dispute(
        &self,
        req: &CreateDisputeRequest,
        initiator_id: Uuid,
    ) -> ConsensusResult<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO consensus_disputes
                (bounty_id, submission_id, initiator_id, disputed_verdict,
                 claimed_verdict, reason, evidence, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'open')
            RETURNING id
            "#,
        )
        .bind(req.bounty_id)
        .bind(req.submission_id)
        .bind(initiator_id)
        .bind(req.disputed_verdict.to_string())
        .bind(req.claimed_verdict.to_string())
        .bind(&req.reason)
        .bind(&req.evidence)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;

        // Flag the bounty's consensus as disputed.
        let _ = sqlx::query("UPDATE consensus_results SET is_disputed = true WHERE bounty_id = $1")
            .bind(req.bounty_id)
            .execute(&self.db_pool)
            .await;

        Ok(id)
    }

    pub async fn get_dispute(&self, dispute_id: Uuid) -> ConsensusResult<Option<Dispute>> {
        sqlx::query_as::<_, Dispute>(
            r#"
            SELECT id, bounty_id, submission_id, initiator_id, disputed_verdict,
                   claimed_verdict, reason, evidence, status, resolution,
                   resolved_by, resolved_at, created_at
            FROM consensus_disputes
            WHERE id = $1
            "#,
        )
        .bind(dispute_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))
    }

    pub async fn get_bounty_disputes(&self, bounty_id: Uuid) -> ConsensusResult<Vec<Dispute>> {
        sqlx::query_as::<_, Dispute>(
            r#"
            SELECT id, bounty_id, submission_id, initiator_id, disputed_verdict,
                   claimed_verdict, reason, evidence, status, resolution,
                   resolved_by, resolved_at, created_at
            FROM consensus_disputes
            WHERE bounty_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(bounty_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))
    }

    pub async fn resolve_dispute(
        &self,
        dispute_id: Uuid,
        req: &ResolveDisputeRequest,
        resolver_id: Uuid,
    ) -> ConsensusResult<()> {
        let dispute = self
            .get_dispute(dispute_id)
            .await?
            .ok_or_else(|| ConsensusError::NotFound(format!("dispute {dispute_id}")))?;

        sqlx::query(
            r#"
            UPDATE consensus_disputes
            SET status = 'resolved', resolution = $2, resolved_by = $3, resolved_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(dispute_id)
        .bind(&req.resolution)
        .bind(resolver_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;

        // Apply the resolved verdict to the consensus result and re-finalize.
        sqlx::query(
            r#"
            UPDATE consensus_results
            SET final_verdict = $2, is_disputed = false, is_finalized = true, finalized_at = NOW()
            WHERE bounty_id = $1
            "#,
        )
        .bind(dispute.bounty_id)
        .bind(req.final_verdict.to_string())
        .execute(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Open disputes that have been waiting long enough to auto-escalate.
    pub async fn find_open_disputes(&self) -> ConsensusResult<Vec<Uuid>> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM consensus_disputes WHERE status = 'open' ORDER BY created_at ASC",
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))
    }

    pub async fn mark_dispute_under_review(&self, dispute_id: Uuid) -> ConsensusResult<()> {
        sqlx::query(
            "UPDATE consensus_disputes SET status = 'under_review' WHERE id = $1 AND status = 'open'",
        )
        .bind(dispute_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

fn parse_verdict(s: &str) -> Verdict {
    match s.to_lowercase().as_str() {
        "malicious" => Verdict::Malicious,
        "benign" => Verdict::Benign,
        "suspicious" => Verdict::Suspicious,
        _ => Verdict::Unknown,
    }
}

struct VerdictCounts {
    malicious: i32,
    benign: i32,
    suspicious: i32,
    unknown: i32,
}

fn count_verdicts(votes: &[SubmissionVote]) -> VerdictCounts {
    let mut c = VerdictCounts {
        malicious: 0,
        benign: 0,
        suspicious: 0,
        unknown: 0,
    };
    for v in votes {
        match v.verdict {
            Verdict::Malicious => c.malicious += 1,
            Verdict::Benign => c.benign += 1,
            Verdict::Suspicious => c.suspicious += 1,
            Verdict::Unknown => c.unknown += 1,
        }
    }
    c
}

#[derive(sqlx::FromRow)]
struct VoteRow {
    id: Uuid,
    #[allow(dead_code)]
    bounty_id: Uuid,
    engine_id: String,
    verdict: String,
    confidence: Decimal,
    reputation_score: i32,
    submitted_at: chrono::DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct ResultRow {
    bounty_id: Uuid,
    final_verdict: String,
    confidence: Decimal,
    total_submissions: i32,
    agreement_score: Decimal,
    is_finalized: bool,
    verdict_distribution: serde_json::Value,
}
