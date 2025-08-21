use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

mod handlers;
mod services;

use handlers::bounty_handler;
use services::reputation;

// Application State 
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub reputation_server: Arc<reputation::ReputationService>,
}

// Bounty Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounty {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub reward_amount: Decimal,
    pub token_address: Option<String>,
    pub creator_address: String,
    pub artifact_hash: String,
    pub artifact_type: ArtifactType,
    pub status: BountyStatus,
    pub min_reputation: i32,
    pub max_participants: i32,
    pub current_participants: i32, 
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    File,
    Url,
    Hash,
    Domain,
    IpAddress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BountyStatus {
    Open,
    Active,
    Pending,
    Completed,
    Cancelled,
    Expired,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBountyRequest {
    pub title: String,
    pub description: String,
    pub reward_amount: Decimal,
    pub token_address: Option<String>,
    pub creator_address: String,
    pub artifact_hash: String,
    pub artifact_type: ArtifactType,
    pub min_reputation: Option<i32>,
    pub max_participants: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BountyFilters {
    pub status: Option<BountyStatus>,
    pub artifact_type: Option<ArtifactType>,
    pub min_reward: Option<Decimal>,
    pub max_reward: Option<Decimal>,
    pub creator_address: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BountyParticipation {
    pub bounty_id: Uuid,
    pub participant_address: String,
    pub stake_amount: Decimal,
    pub verdict: Option<ThreatVerdict>,
    pub confidence_score: Option<f64>,
    pub analysis_data: Option<serde_json::Value>,
    pub submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ThreatVerdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
            timestamp: Utc::now(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://nexus:password@localhost/nexus_security".to_string());
    
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3002".to_string())
        .parse::<u16>()?;

    // Initialize database connection
    info!("Connecting to database: {}", database_url);
    let db = PgPool::connect(&database_url).await?;

    // Run database migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&db).await?;

    // Initialize reputation service
    let reputation_service = Arc::new(reputation::ReputationService::new(db.clone()).await?);

    // Create application state
    let state = AppState {
        db,
        reputation_service,
    };

    // Build router
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Bounty Manager service starting on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Bounty management routes
        .route("/bounties", post(bounty_handler::create_bounty))
        .route("/bounties", get(bounty_handler::list_bounties))
        .route("/bounties/:id", get(bounty_handler::get_bounty))
        .route("/bounties/:id", put(bounty_handler::update_bounty))
        .route("/bounties/:id/cancel", post(bounty_handler::cancel_bounty))
        
        // Participation routes
        .route("/bounties/:id/participate", post(bounty_handler::join_bounty))
        .route("/bounties/:id/submit", post(bounty_handler::submit_analysis))
        .route("/bounties/:id/participants", get(bounty_handler::get_participants))
        
        // Analytics routes
        .route("/bounties/:id/consensus", get(bounty_handler::get_consensus))
        .route("/stats/creator/:address", get(bounty_handler::get_creator_stats))
        .route("/stats/participant/:address", get(bounty_handler::get_participant_stats))
        
        // State management
        .with_state(state)
        
        // Middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
}

async fn health_check() -> Json<ApiResponse<HashMap<String, String>>> {
    let mut status = HashMap::new();
    status.insert("service".to_string(), "bounty-manager".to_string());
    status.insert("status".to_string(), "healthy".to_string());
    status.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    
    Json(ApiResponse::success(status))
}

// Database helper functions
pub async fn get_bounty_by_id(
    db: &PgPool, 
    bounty_id: Uuid
) -> Result<Option<Bounty>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id, title, description, reward_amount, token_address,
            creator_address, artifact_hash, artifact_type as "artifact_type: ArtifactType",
            status as "status: BountyStatus", min_reputation, max_participants,
            current_participants, created_at, expires_at, metadata
        FROM bounties 
        WHERE id = $1
        "#,
        bounty_id
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(Bounty {
            id: row.id,
            title: row.title,
            description: row.description,
            reward_amount: row.reward_amount,
            token_address: row.token_address,
            creator_address: row.creator_address,
            artifact_hash: row.artifact_hash,
            artifact_type: row.artifact_type,
            status: row.status,
            min_reputation: row.min_reputation,
            max_participants: row.max_participants,
            current_participants: row.current_participants,
            created_at: row.created_at,
            expires_at: row.expires_at,
            metadata: row.metadata,
        })),
        None => Ok(None),
    }
}

pub async fn create_bounty_in_db(
    db: &PgPool,
    request: &CreateBountyRequest,
) -> Result<Bounty, sqlx::Error> {
    let bounty_id = Uuid::new_v4();
    let now = Utc::now();
    
    sqlx::query!(
        r#"
        INSERT INTO bounties (
            id, title, description, reward_amount, token_address, creator_address,
            artifact_hash, artifact_type, status, min_reputation, max_participants,
            current_participants, created_at, expires_at, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#,
        bounty_id,
        request.title,
        request.description,
        request.reward_amount,
        request.token_address,
        request.creator_address,
        request.artifact_hash,
        request.artifact_type as ArtifactType,
        BountyStatus::Open as BountyStatus,
        request.min_reputation.unwrap_or(0),
        request.max_participants.unwrap_or(10),
        0, // current_participants starts at 0
        now,
        request.expires_at,
        request.metadata
    )
    .execute(db)
    .await?;

    Ok(Bounty {
        id: bounty_id,
        title: request.title.clone(),
        description: request.description.clone(),
        reward_amount: request.reward_amount,
        token_address: request.token_address.clone(),
        creator_address: request.creator_address.clone(),
        artifact_hash: request.artifact_hash.clone(),
        artifact_type: request.artifact_type.clone(),
        status: BountyStatus::Open,
        min_reputation: request.min_reputation.unwrap_or(0),
        max_participants: request.max_participants.unwrap_or(10),
        current_participants: 0,
        created_at: now,
        expires_at: request.expires_at,
        metadata: request.metadata.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert!(response.0.success);
    }

    #[test]
    fn test_bounty_serialization() {
        let bounty = Bounty {
            id: Uuid::new_v4(),
            title: "Test Bounty".to_string(),
            description: "Test Description".to_string(),
            reward_amount: Decimal::new(100, 0),
            token_address: None,
            creator_address: "0x123".to_string(),
            artifact_hash: "hash123".to_string(),
            artifact_type: ArtifactType::File,
            status: BountyStatus::Open,
            min_reputation: 0,
            max_participants: 10,
            current_participants: 0,
            created_at: Utc::now(),
            expires_at: None,
            metadata: None,
        };

        let json = serde_json::to_string(&bounty).unwrap();
        assert!(json.contains("Test Bounty"));
    }
}