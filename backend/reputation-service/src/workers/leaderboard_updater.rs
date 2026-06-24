use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::services::reputation_service::ReputationService;

/// Refreshes the cached leaderboard in Redis every few minutes.
pub async fn start(service: Arc<ReputationService>) -> Result<()> {
    info!("Leaderboard updater worker started");
    let interval = std::time::Duration::from_secs(300);

    loop {
        tokio::time::sleep(interval).await;
        if let Err(e) = service.cache_leaderboard().await {
            error!("Leaderboard cache refresh failed: {}", e);
        }
    }
}
