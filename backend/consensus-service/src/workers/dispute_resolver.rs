use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::services::consensus_service::ConsensusService;

pub async fn start(_service: Arc<ConsensusService>) -> Result<()> {
    info!("Dispute resolver worker started");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        // Process and resolve disputes
    }
}
