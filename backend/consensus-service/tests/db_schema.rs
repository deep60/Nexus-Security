//! Schema/query integration test for consensus-service.
//!
//! Applies the embedded migrations to a real Postgres database and exercises
//! the tables the service reads/writes (consensus_submissions, consensus_results
//! including the finalization columns, consensus_disputes). Guards against the
//! "relation/column does not exist" class of runtime failures that migrations
//! applying successfully does not by itself rule out.
//!
//! Runs only when `CONSENSUS_SERVICE_DATABASE_URL` (or `DATABASE_URL`) is set —
//! CI provides it via scripts/ci/setup-test-databases.sh. Otherwise it is a
//! no-op so local `cargo test` stays green without a database. All writes run
//! inside a transaction that is rolled back.

use sqlx::postgres::PgPoolOptions;

fn test_database_url() -> Option<String> {
    std::env::var("CONSENSUS_SERVICE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn schema_supports_service_queries() {
    let Some(database_url) = test_database_url() else {
        eprintln!("skipping consensus-service schema test; CONSENSUS_SERVICE_DATABASE_URL not set");
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

    // Mirrors ConsensusService submission insert.
    sqlx::query(
        r#"
        INSERT INTO consensus_submissions
            (bounty_id, engine_id, verdict, confidence, reputation_score)
        VALUES (gen_random_uuid(), 'engine-1', 'malicious', 0.95, 100)
        "#,
    )
    .execute(&mut *tx)
    .await
    .expect("insert into consensus_submissions");

    // Mirrors ConsensusService result insert; validates base columns.
    sqlx::query(
        r#"
        INSERT INTO consensus_results
            (bounty_id, final_verdict, confidence, total_submissions,
             malicious_count, benign_count, suspicious_count, unknown_count)
        VALUES (gen_random_uuid(), 'malicious', 0.9, 3, 2, 1, 0, 0)
        "#,
    )
    .execute(&mut *tx)
    .await
    .expect("insert into consensus_results");

    // Validates the finalization/agreement columns added in migration 2.
    sqlx::query(
        "SELECT agreement_score, is_disputed, is_finalized, finalized_at, \
         participating_engines, verdict_distribution FROM consensus_results LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("select finalization columns from consensus_results");

    // Mirrors dispute insert.
    sqlx::query(
        r#"
        INSERT INTO consensus_disputes
            (bounty_id, initiator_id, disputed_verdict, claimed_verdict, reason)
        VALUES (gen_random_uuid(), gen_random_uuid(), 'malicious', 'benign', 'test dispute')
        "#,
    )
    .execute(&mut *tx)
    .await
    .expect("insert into consensus_disputes");

    tx.rollback().await.expect("rollback tx");
    pool.close().await;
}
