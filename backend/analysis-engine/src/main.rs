use std::env;
use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    response::Json,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::error;
use uuid::Uuid;
use tokio::net::TcpListener;
use tracing::info;

mod analyzers;
mod models;
mod utils;

use analyzers::{
    static_analyzer::{StaticAnalyzer, StaticAnalyzerConfig},
    hash_analyzer::{HashAnalyzer, HashAnalyzerConfig},
    yara_engine::{YaraEngine, YaraEngineConfig},
};
use models::analysis_result::{AnalysisResult, ThreatVerdict, FileMetadata};
use utils::file_handler::FileHandler;
use chrono::Utc;

#[derive(Clone)]
pub struct AppState {
    static_analyzer: Arc<StaticAnalyzer>,
    hash_analyzer: Arc<HashAnalyzer>,
    yara_engine: Arc<YaraEngine>,
    file_handler: Arc<FileHandler>,
    database_url: String,
    redis_url: String,
}

#[derive(Deserialize)]
struct AnalysisRequest {
    artifact_type: String,    // "file", "url", "hash"
    priority: Option<u8>,
    bounty_id: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct AnalysisResponse {
    analysis_id: String,
    status: String,
    message: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    engines: EngineStatus,
}

#[derive(Serialize)]
struct EngineStatus {
    static_analyzer: bool,
    hash_analyzer: bool,
    yara_engine: bool,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    info!("Starting Nexus-Security Analysis Engine");

    // Load configuration
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8002".to_string()).parse::<u16>().unwrap_or(8002);
    let yara_rule_path = env::var("YARA_RULE_PATH").unwrap_or_else(|_| "./rules".to_string());
    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./temp/nexus-uploads".to_string());

    // Initialize analyzers
    info!("Initializing analysis engines...");

    let static_analyzer = Arc::new(StaticAnalyzer::new(Default::default()));
    let hash_analyzer = Arc::new(HashAnalyzer::new(Default::default()));
    let yara_engine = Arc::new(YaraEngine::new(YaraEngineConfig {
        rules_directory: std::path::PathBuf::from(yara_rule_path),
        ..Default::default()
    })?);  
    let file_handler = Arc::new(FileHandler::new(&upload_dir)?);

    // Create application state
    let app_state = AppState {
        static_analyzer,
        hash_analyzer,
        yara_engine,
        file_handler,
        database_url,
        redis_url,
    };

    info!("Analysis engines initialized successfully");

    // Build the application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/analyze/file", post(analyze_file))
        .route("/analyze/url", post(analyze_url))
        .route("/analyze/hash", post(analyze_hash))
        .route("/analysis/:id", get(get_analysis_result))
        .route("/analysis/:id/detailed", get(get_detailed_analysis))
        .route("/engines/status", get(engines_status))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(TraceLayer::new_for_http())
        );

        // Start the server
        let addr = format!("0.0.0.0:{}", port);
        info!("Analysis Engine listening on {}", addr);

        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse { 
        status: "healthy".to_string(), 
        service: "analysis-engine".to_string(), 
        version: env!("CARGO_PKG_VERSION").to_string(), 
        engines: EngineStatus { 
            static_analyzer: true, 
            hash_analyzer: true, 
            yara_engine: true, 
        }, 
    })
}

async fn analyze_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<AnalysisResponse>, StatusCode> {
    info!("Received file analysis request");

    let analysis_id = Uuid::new_v4().to_string();

    // Process multipart data
    let mut file_data = Vec::new();
    let mut filename = String::new();
    let mut analysis_req: Option<AnalysisRequest> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        match field.name() {
            Some("file") => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                file_data = field.bytes().await.map_err(|e| {
                    error!("Failed to read file data: {}", e);
                    StatusCode::BAD_REQUEST
                })?.to_vec();
            },
            Some("metadata") => {
                let metadata_str = field.text().await.map_err(|e| {
                    error!("Failed to read metadata: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                analysis_req = serde_json::from_str(&metadata_str).ok();
            },
            _ => continue,
        } 
    }

    if file_data.is_empty() {
        error!("No file data provided");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Save file and start analysis
    let file_path = match state.file_handler.store_file(&file_data, &filename, None, None).await {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to save file: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Start asynchronous analysis
    let state_clone = state.clone();
    let analysis_id_clone = analysis_id.clone();
    let file_path_clone = file_path.clone();

    tokio::spawn(async move {
        if let Err(e) = perform_file_analysis(
            state_clone,
            &analysis_id_clone,
            &file_path_clone.path().to_string_lossy(),
            analysis_req,
        ).await {
            error!("Analysis failed for {}: {}", analysis_id_clone, e);
        } 
    });

    Ok(Json(AnalysisResponse {
        analysis_id,
        status: "submitted".to_string(),
        message: "File Analysis started successfully".to_string()
    }))
}

async fn analyze_url(State(state): 
    State<AppState>, 
    Json(request): Json<serde_json::Value>,
) -> Result<Json<AnalysisResponse>, StatusCode> {
    info!("Received URL analysis request");

    let analysis_id = Uuid::new_v4().to_string();

    let url = request.get("url")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Start asynchronous URL analysis
    let start_clone = state.clone();
    let analysis_id_clone = analysis_id.clone();
    let url_clone = url.to_string();

    tokio::spawn(async move {
        if let Err(e) = perform_url_analysis(start_clone, &analysis_id_clone, &url_clone).await {
            error!("URL analysis failed for {}: {}", analysis_id_clone, e);
        }
    });

    Ok(Json(AnalysisResponse {
        analysis_id,
        status: "submitted".to_string(),
        message: "URL Analysis started successfully".to_string(),
    }))
}

async fn analyze_hash(
    State(state): State<AppState>, 
    Json(request): Json<serde_json::Value>,
) -> Result<Json<AnalysisResponse>, StatusCode> {
    info!("Received hash analysis request");

    let analysis_id = Uuid::new_v4().to_string();

    let hash = request.get("hash")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Start asynchronous hash analysis
    let state_clone = state.clone();
    let analysis_id_clone = analysis_id.clone();
    let hash_clone = hash.to_string();

    tokio::spawn(async move {
        if let Err(e) = perform_hash_analysis(state_clone, &analysis_id_clone, &hash_clone).await {
            error!("Hash analysis failed for {}: {}", analysis_id_clone, e);
        }
    });

    Ok(Json(AnalysisResponse {
        analysis_id,
        status: "submitted".to_string(),
        message: "Hash Analysis started successfully".to_string(),
    }))
}

async fn get_analysis_result(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<AnalysisResult>, StatusCode> {
    info!("Fetching analysis result for: {}", id);

    // TODO: Implement database lookup
    // For now, return a mock response with the correct structure
    let mock_file_metadata = FileMetadata {
        filename: Some("mock_file.exe".to_string()),
        file_size: 1024,
        mime_type: "application/octet-stream".to_string(),
        md5: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
        sha1: "da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string(),
        sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        sha512: None,
        entropy: Some(0.5),
        magic_bytes: Some("4d5a".to_string()),
        executable_info: None,
    };

    Ok(Json(AnalysisResult {
        analysis_id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
        submission_id: Uuid::new_v4(),
        bounty_id: None,
        file_metadata: mock_file_metadata,
        consensus_verdict: ThreatVerdict::Benign,
        consensus_confidence: 0.85,
        consensus_severity: models::analysis_result::SeverityLevel::Low,
        detections: vec![],
        yara_matches: vec![],
        network_indicators: None,
        behavioral_analysis: None,
        tags: vec![],
        notes: None,
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
        total_processing_time_ms: Some(1000),
        status: models::analysis_result::AnalysisStatus::Completed,
        error_message: None,
        analysis_cost: None,
        engine_reputations: std::collections::HashMap::new(),
    }))
}

async fn get_detailed_analysis(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Fetching detailed analysis for: {}", id);

    // TODO: Implement detailed analysis retrieval
    Ok(Json(serde_json::json!({
        "analysis_id": id,
        "detailed_result": "Not implemented yet"
    })))
}

async fn engines_status(
    State(state): State<AppState>
) -> Json<EngineStatus> {
    // TODO: Implement actual engine health checks
    Json(EngineStatus {
        static_analyzer: true,
        hash_analyzer: true,
        yara_engine: true,
    })
}

async fn perform_file_analysis(
    state: AppState,
    analysis_id: &str,
    file_path: &str,
    request: Option<AnalysisRequest>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_data = state.file_handler.get_file(file_path).await?;
    Ok(())
}

async fn perform_url_analysis(
    _state: AppState,
    analysis_id: &str,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting URL analysis for: {} ({})", analysis_id, url);

    // TODO: Implement URL analysis logic once analyzer interfaces are fixed
    // This could include:
    // - DNS resolution checks
    // - Reputation lookups
    // - Content analysis
    // - Phishing detection
    tokio::time::sleep(tokio::time::Duration::from_millis(75)).await;
    
    info!("URL analysis completed for: {}", analysis_id);
    Ok(())
}

async fn perform_hash_analysis(
    _state: AppState,
    analysis_id: &str,
    hash: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting hash analysis for: {} ({})", analysis_id, hash);

    // TODO: Implement actual hash analysis once analyzer interfaces are fixed
    // For now, just simulate the analysis
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    info!("Hash analysis completed for: {}", analysis_id);
    Ok(())
}
