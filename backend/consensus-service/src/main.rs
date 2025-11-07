use axum::{routing::get, Router};
use std::net::SocketAddr;

mod aggregation;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Consensus Service...");

    let app = Router::new()
        .route("/health", get(health_check));

    let port = std::env::var("CONSENSUS_SERVICE_PORT")
        .unwrap_or_else(|_| "8087".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Consensus Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Consensus Service is healthy"
}
