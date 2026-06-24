use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::services::consensus_service::ConsensusService;

/// Moves stale open disputes into the under-review queue so they surface to
/// human moderators / admin endpoints.
pub async fn start(service: Arc<ConsensusService>) -> Result<()> {
    info!("Dispute resolver worker started");
    let interval = std::time::Duration::from_secs(300);

    loop {
        tokio::time::sleep(interval).await;

        match service.find_open_disputes().await {
            Ok(disputes) => {
                for dispute_id in disputes {
                    if let Err(e) = service.mark_dispute_under_review(dispute_id).await {
                        error!("Failed to mark dispute {} under review: {}", dispute_id, e);
                    }
                }
            }
            Err(e) => error!("Failed to query open disputes: {}", e),
        }
    }
}
