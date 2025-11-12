use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::services::consensus_service::ConsensusService;

pub async fn start(_service: Arc<ConsensusService>) -> Result<()> {
    info!("Consensus processor worker started");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        // Process pending consensus calculations
    }
}
