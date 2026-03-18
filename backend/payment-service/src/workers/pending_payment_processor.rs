use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn, error};

use crate::services::payment_service::PaymentService;

/// Pending payment processor: fetches queued payments from the database
/// and processes them by recording status updates.
/// Actual token transfers happen on-chain via BountyManager.resolveBounty().
pub async fn start(service: Arc<PaymentService>) -> Result<()> {
    info!("Pending payment processor worker started");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        // Query queued payments
        let pending = sqlx::query_as::<_, PendingPayment>(
            "SELECT id, bounty_id, recipient_address, amount FROM payments WHERE status = 'queued' LIMIT 20"
        )
        .fetch_all(service.db_pool())
        .await;

        match pending {
            Ok(payments) => {
                for payment in payments {
                    info!(
                        "Processing payment {} for bounty {} -> {} ({})",
                        payment.id, payment.bounty_id, payment.recipient_address, payment.amount
                    );

                    // Mark as processing
                    let _ = sqlx::query(
                        "UPDATE payments SET status = 'processing', updated_at = NOW() WHERE id = $1"
                    )
                    .bind(payment.id)
                    .execute(service.db_pool())
                    .await;

                    // Token transfers are handled by BountyManager.resolveBounty() on-chain.
                    // This processor tracks the DB state and marks payments as completed
                    // once the tx_monitor confirms on-chain settlement.
                    let _ = sqlx::query(
                        "UPDATE payments SET status = 'awaiting_chain', updated_at = NOW() WHERE id = $1"
                    )
                    .bind(payment.id)
                    .execute(service.db_pool())
                    .await;
                }
            }
            Err(e) => {
                // Table may not exist yet — non-fatal
                if !e.to_string().contains("does not exist") {
                    warn!("Failed to query pending payments: {}", e);
                }
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct PendingPayment {
    id: uuid::Uuid,
    bounty_id: uuid::Uuid,
    recipient_address: String,
    amount: String,
}
