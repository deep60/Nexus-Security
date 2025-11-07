use axum::{routing::get, Router};
use std::net::SocketAddr;

mod blockchain;
mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Payment Service...");

    let app = Router::new()
        .route("/health", get(health_check));

    let port = std::env::var("PAYMENT_SERVICE_PORT")
        .unwrap_or_else(|_| "8088".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Payment Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Payment Service is healthy"
}
