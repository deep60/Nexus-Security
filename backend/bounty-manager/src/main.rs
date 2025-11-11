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

mod config;
mod handlers;
mod models;
mod services;
mod workers;

use handlers::bounty_crud;
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
    let reputation_service = Arc::new(reputation::ReputationService::new());

    // Create application state using BountyManagerState
    let state = bounty_crud::BountyManagerState {
        reputation_service: reputation_service.clone(),
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

fn create_router(state: bounty_crud::BountyManagerState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))

        // Bounty management routes
        .route("/bounties", post(bounty_crud::create_bounty))
        .route("/bounties", get(bounty_crud::list_bounties))
        .route("/bounties/:id", get(bounty_crud::get_bounty))
        .route("/bounties/:id", put(bounty_crud::update_bounty))
        .route("/bounties/:id/cancel", post(bounty_crud::cancel_bounty))

        // Stats route
        .route("/bounties/stats", get(bounty_crud::get_bounty_stats))

        // TODO: Add more routes as handlers are implemented
        // .route("/bounties/:id/submit", post(handlers::submit_analysis))

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
// NOTE: These functions use sqlx::query! macro which requires DATABASE_URL to be set
// They are commented out for now. Use the models in src/models/ instead.
//
// pub async fn get_bounty_by_id(db: &PgPool, bounty_id: Uuid) -> Result<Option<Bounty>, sqlx::Error> {
//     models::bounty::BountyModel::find_by_id(db, bounty_id).await
// }

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