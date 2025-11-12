use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::services::reputation_service::ReputationService;

pub async fn start(_service: Arc<ReputationService>) -> Result<()> {
    info!("Decay processor worker started");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        // Apply time decay to inactive users
    }
}
