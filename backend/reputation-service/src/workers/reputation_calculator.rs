use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::services::reputation_service::ReputationService;

/// Periodically recomputes user ranks/percentiles across the table.
pub async fn start(service: Arc<ReputationService>) -> Result<()> {
    info!("Reputation calculator worker started");
    let interval = std::time::Duration::from_secs(60);

    loop {
        tokio::time::sleep(interval).await;
        match service.recompute_ranks().await {
            Ok(n) => {
                if n > 0 {
                    info!("Recomputed ranks for {} users", n);
                }
            }
            Err(e) => error!("Rank recomputation failed: {}", e),
        }
    }
}
