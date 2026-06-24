use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::services::reputation_service::ReputationService;

/// Hourly inactivity decay. Users idle for more than `IDLE_DAYS` lose score
/// according to the configured per-day decay rate.
pub async fn start(service: Arc<ReputationService>) -> Result<()> {
    info!("Decay processor worker started");
    let interval = std::time::Duration::from_secs(3600);
    let idle_days: f64 = std::env::var("DECAY_IDLE_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7.0);

    loop {
        tokio::time::sleep(interval).await;
        match service.apply_decay_all(idle_days).await {
            Ok(n) => {
                if n > 0 {
                    info!("Applied decay to {} inactive users", n);
                }
            }
            Err(e) => error!("Decay processing failed: {}", e),
        }
    }
}
