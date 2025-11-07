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
mod storage;
mod scanners;
mod sandbox;

use crate::analyzers::{AnalysisEngine, AnalysisEngineConfig, FileAnalysisRequest, AnalysisOptions, AnalysisPriority};
use crate::analyzers::hash_analyzer::{HashInfo, HashType};
use crate::models::analysis_result::{AnalysisResult, ThreatVerdict, FileMetadata};
use crate::utils::file_handler::FileHandler;
use crate::storage::{StorageManager, StorageConfig};
use crate::scanners::file_scanner::{FileScanner, FileScannerConfig};
use crate::scanners::url_scanner::{UrlScanner, UrlScannerConfig};
use chrono::Utc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    analysis_engine: Arc<Mutex<AnalysisEngine>>,
    file_handler: Arc<FileHandler>,
    storage_manager: Arc<StorageManager>,
    file_scanner: Arc<FileScanner>,
    url_scanner: Arc<UrlScanner>,
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

    // Initialize analyzers via combined engine
    info!("Initializing analysis engines...");
    let mut config = AnalysisEngineConfig::default();
    config.yara_engine.rules_directory = std::path::PathBuf::from(yara_rule_path);
    let analysis_engine = Arc::new(Mutex::new(AnalysisEngine::new(config)?));
    let file_handler = Arc::new(FileHandler::new(&upload_dir)?);

    // Initialize storage manager
    info!("Initializing storage manager...");
    let storage_config = StorageConfig::default();
    let storage_manager = Arc::new(StorageManager::new(storage_config).await?);

    // Initialize scanners
    info!("Initializing scanners...");
    let file_scanner = Arc::new(FileScanner::new(FileScannerConfig::default())?);
    let url_scanner = Arc::new(UrlScanner::new(UrlScannerConfig::default())?);

    // Create application state
    let app_state = AppState {
        analysis_engine,
        file_handler,
        storage_manager,
        file_scanner,
        url_scanner,
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
    let mut _analysis_req: Option<AnalysisRequest> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let name = field.name().map(|s| s.to_string()).unwrap_or_default();
        if name == "file" {
            filename = field.file_name().map(|s| s.to_string()).unwrap_or_default();
            file_data = field.bytes().await.map_err(|e| {
                error!("Failed to read file bytes: {}", e);
                StatusCode::BAD_REQUEST
            })?.to_vec();
        } else if name == "request" {
            let json_str = field.text().await.map_err(|e| {
                error!("Failed to read request json: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            _analysis_req = Some(serde_json::from_str(&json_str).map_err(|e| {
                error!("Invalid request json: {}", e);
                StatusCode::BAD_REQUEST
            })?);
        }
    }

    let request = FileAnalysisRequest {
        filename,
        file_data,
        file_hashes: None,
        analysis_options: AnalysisOptions::default(),
    };

    let mut engine_guard = state.analysis_engine.lock().await;
    let analysis_result = engine_guard.analyze_file(request).await.map_err(|e| {
        error!("Analysis failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Store analysis result in database
    if let Err(e) = state.storage_manager
        .store_analysis_result(&analysis_result.analysis_id, &analysis_result, None)
        .await
    {
        error!("Failed to store analysis result: {}", e);
    }

    info!("Analysis completed: {:?}", analysis_result.analysis_id);

    Ok(Json(AnalysisResponse {
        analysis_id: analysis_result.analysis_id.to_string(),
        status: "completed".to_string(),
        message: "File Analysis completed successfully".to_string(),
    }))
}

async fn analyze_url(
    State(state): State<AppState>, 
    Json(request): Json<serde_json::Value>,
) -> Result<Json<AnalysisResponse>, StatusCode> {
    let url = request.get("url")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?.to_string();

    let analysis_id = Uuid::new_v4().to_string();

    let state_clone = state.clone();
    let analysis_id_clone = analysis_id.clone();

    tokio::spawn(async move {
        if let Err(e) = perform_url_analysis(state_clone, &analysis_id_clone, &url).await {
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
        .ok_or(StatusCode::BAD_REQUEST)?.to_string();

    let state_clone = state.clone();
    let analysis_id_clone = analysis_id.clone();
    let hash_clone = hash;

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
    State(state): State<AppState>,
) -> Result<Json<AnalysisResult>, StatusCode> {
    info!("Fetching analysis result for: {}", id);

    let analysis_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Retrieve from database
    let result = state.storage_manager
        .get_analysis_result(&analysis_id)
        .await
        .map_err(|e| {
            error!("Failed to retrieve analysis result: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match result {
        Some(analysis_result) => Ok(Json(analysis_result)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_detailed_analysis(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Fetching detailed analysis for: {}", id);

    // TODO: Implement detailed analysis retrieval
    Ok(Json(serde_json::json!({
        "analysis_id": id,
        "detailed_result": "Detailed analysis data here"
    })))
}

async fn engines_status(
    State(state): State<AppState>
) -> Json<EngineStatus> {
    // Check storage health
    let storage_health = state.storage_manager.health_check().await;

    Json(EngineStatus {
        static_analyzer: storage_health.database_healthy,
        hash_analyzer: storage_health.s3_healthy,
        yara_engine: storage_health.overall_healthy,
    })
}

async fn perform_file_analysis(
    state: AppState,
    _analysis_id: &str,
    file_path: &str,
    _request: Option<AnalysisRequest>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_data = state.file_handler.get_file(file_path).await?;
    let req = FileAnalysisRequest {
        filename: file_path.to_string(),
        file_data,
        file_hashes: None,
        analysis_options: AnalysisOptions::default(),
    };

    let mut engine_guard = state.analysis_engine.lock().await;
    engine_guard.analyze_file(req).await?;

    Ok(())
}

async fn perform_url_analysis(
    state: AppState,
    analysis_id: &str,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting URL analysis for: {} ({})", analysis_id, url);

    // Use URL scanner to analyze the URL
    let scan_result = state.url_scanner.scan(url.as_bytes(), None).await?;

    info!("URL analysis completed for: {} - Risk Level: {:?}", analysis_id, scan_result.risk_level);

    // TODO: Store URL scan results in database with proper AnalysisResult structure

    Ok(())
}

async fn perform_hash_analysis(
    state: AppState,
    analysis_id: &str,
    hash: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting hash analysis for: {} ({})", analysis_id, hash);

    // Use hash analyzer
    let hash_info = HashInfo {
        hash_type: HashType::SHA256,
        hash_value: hash.to_string(),
        file_size: None,
    };
    let mut engine_guard = state.analysis_engine.lock().await;
    engine_guard.hash_analyzer.analyze_hash(&hash_info, None).await?;

    info!("Hash analysis completed for: {}", analysis_id);
    Ok(())
}