//! Schema/query integration test for bounty-manager.
//!
//! Applies the embedded migrations to a real Postgres database and exercises
//! the bounties, submissions, payouts and reputations tables (including the
//! FKs from submissions/payouts to bounties). Runs only when
//! `BOUNTY_MANAGER_DATABASE_URL` (or `DATABASE_URL`) is set; otherwise it is a
//! no-op. All writes run inside a rolled-back transaction.

use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn test_database_url() -> Option<String> {
    std::env::var("BOUNTY_MANAGER_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn schema_supports_model_queries() {
    let Some(database_url) = test_database_url() else {
        eprintln!("skipping bounty-manager schema test; BOUNTY_MANAGER_DATABASE_URL not set");
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

    let bounty_id = Uuid::new_v4();

    // Mirrors BountyModel::create column list.
    sqlx::query(
        r#"
        INSERT INTO bounties
            (id, creator, title, description, artifact_type, reward_amount,
             currency, min_stake, deadline)
        VALUES ($1, 'creator-addr', 'Test bounty', 'desc', 'file', 1000,
                'THREAT', 100, NOW() + INTERVAL '1 day')
        "#,
    )
    .bind(bounty_id)
    .execute(&mut *tx)
    .await
    .expect("insert into bounties");

    // Mirrors SubmissionModel::create (FK to bounties, analysis_details JSONB NOT NULL).
    sqlx::query(
        r#"
        INSERT INTO submissions
            (id, bounty_id, engine_id, engine_type, verdict, confidence,
             stake_amount, analysis_details)
        VALUES (gen_random_uuid(), $1, 'engine-1', 'automated', 'malicious', 0.9, 100, '{}'::jsonb)
        "#,
    )
    .bind(bounty_id)
    .execute(&mut *tx)
    .await
    .expect("insert into submissions");

    // Mirrors PayoutModel::create (FK to bounties).
    sqlx::query(
        r#"
        INSERT INTO payouts
            (id, bounty_id, recipient, amount, currency, payout_type)
        VALUES (gen_random_uuid(), $1, 'recipient-addr', 100, 'THREAT', 'reward')
        "#,
    )
    .bind(bounty_id)
    .execute(&mut *tx)
    .await
    .expect("insert into payouts");

    // Mirrors ReputationModel::create.
    sqlx::query("INSERT INTO reputations (engine_id) VALUES ('engine-1')")
        .execute(&mut *tx)
        .await
        .expect("insert into reputations");

    tx.rollback().await.expect("rollback tx");
    pool.close().await;
}
