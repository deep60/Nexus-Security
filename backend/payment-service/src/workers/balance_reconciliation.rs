use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::services::payment_service::PaymentService;

pub async fn start(_service: Arc<PaymentService>) -> Result<()> {
    info!("Balance reconciliation worker started");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    }
}
