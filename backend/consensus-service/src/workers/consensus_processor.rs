use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::services::consensus_service::ConsensusService;

/// Periodically finalizes bounties whose voting window has elapsed.
pub async fn start(service: Arc<ConsensusService>) -> Result<()> {
    info!("Consensus processor worker started");
    let interval = std::time::Duration::from_secs(60);

    loop {
        tokio::time::sleep(interval).await;

        match service.find_finalizable_bounties().await {
            Ok(bounties) => {
                if !bounties.is_empty() {
                    info!("Auto-finalizing {} bounties", bounties.len());
                }
                for bounty_id in bounties {
                    match service.calculate_and_store(bounty_id, true).await {
                        Ok(resp) => info!(
                            "Finalized bounty {} -> {:?} ({}% agreement)",
                            bounty_id, resp.final_verdict, resp.agreement_score
                        ),
                        Err(e) => error!("Failed to finalize bounty {}: {}", bounty_id, e),
                    }
                }
            }
            Err(e) => error!("Failed to query finalizable bounties: {}", e),
        }
    }
}
