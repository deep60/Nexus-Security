use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::services::payment_service::PaymentService;

pub async fn start(_service: Arc<PaymentService>) -> Result<()> {
    info!("Pending payment processor worker started");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
