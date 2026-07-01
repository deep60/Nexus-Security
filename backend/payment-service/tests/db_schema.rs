//! Schema/query integration test for payment-service.
//!
//! Applies the embedded migrations to a real Postgres database and exercises
//! the payments, payment_transactions, escrow_accounts and wallet_balances
//! tables. Runs only when `PAYMENT_SERVICE_DATABASE_URL` (or `DATABASE_URL`) is
//! set; otherwise it is a no-op. All writes run inside a rolled-back transaction.

use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn test_database_url() -> Option<String> {
    std::env::var("PAYMENT_SERVICE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn schema_supports_service_queries() {
    let Some(database_url) = test_database_url() else {
        eprintln!("skipping payment-service schema test; PAYMENT_SERVICE_DATABASE_URL not set");
        return;
    };

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations apply cleanly");

    let mut tx = pool.begin().await.expect("begin tx");

    // Mirrors PaymentService::create_payment; capture the id for the FK below.
    let payment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO payments
            (bounty_id, payer_address, recipient_address, amount,
             token_address, status, payment_type)
        VALUES (gen_random_uuid(), '0xpayer', '0xrecipient', 100.5,
                '0xtoken', 'pending', 'reward')
        RETURNING id
        "#,
    )
    .fetch_one(&mut *tx)
    .await
    .expect("insert into payments");

    sqlx::query(
        r#"
        INSERT INTO payment_transactions
            (payment_id, transaction_hash, from_address, to_address, value, status)
        VALUES ($1, '0xhash', '0xfrom', '0xto', 100.5, 'pending')
        "#,
    )
    .bind(payment_id)
    .execute(&mut *tx)
    .await
    .expect("insert into payment_transactions");

    sqlx::query(
        r#"
        INSERT INTO escrow_accounts (bounty_id, holder_address, amount, token_address)
        VALUES (gen_random_uuid(), '0xholder', 500.0, '0xtoken')
        "#,
    )
    .execute(&mut *tx)
    .await
    .expect("insert into escrow_accounts");

    // Mirrors the balance-reconciliation worker's wallet_balances usage.
    sqlx::query("INSERT INTO wallet_balances (address, balance) VALUES ('0xwallet', '1000')")
        .execute(&mut *tx)
        .await
        .expect("insert into wallet_balances");

    tx.rollback().await.expect("rollback tx");
    pool.close().await;
}
