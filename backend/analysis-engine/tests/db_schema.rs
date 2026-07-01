//! Schema/query integration test for analysis-engine.
//!
//! Applies the embedded migrations to a real Postgres database and exercises
//! the engine_analysis_results table (insert + upsert + read), mirroring the
//! queries in main.rs. Runs only when `ANALYSIS_ENGINE_DATABASE_URL` (or
//! `DATABASE_URL`) is set; otherwise it is a no-op. All writes run inside a
//! rolled-back transaction.

use sqlx::postgres::PgPoolOptions;

fn test_database_url() -> Option<String> {
    std::env::var("ANALYSIS_ENGINE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn schema_supports_result_queries() {
    let Some(database_url) = test_database_url() else {
        eprintln!("skipping analysis-engine schema test; ANALYSIS_ENGINE_DATABASE_URL not set");
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

    // Mirrors the pending-record insert in main.rs.
    sqlx::query(
        "INSERT INTO engine_analysis_results (analysis_id, status, created_at, result_data) \
         VALUES (gen_random_uuid(), 'InProgress', NOW(), '{}'::jsonb)",
    )
    .execute(&mut *tx)
    .await
    .expect("insert pending engine_analysis_results");

    // Mirrors the completed upsert in main.rs (full column list + ON CONFLICT).
    sqlx::query(
        r#"
        INSERT INTO engine_analysis_results
            (analysis_id, status, verdict, confidence, created_at, completed_at, result_data)
        VALUES (gen_random_uuid(), 'Completed', 'Unknown', 0.5, NOW(), NOW(), '{}'::jsonb)
        ON CONFLICT (analysis_id) DO UPDATE
            SET status = EXCLUDED.status,
                verdict = EXCLUDED.verdict,
                confidence = EXCLUDED.confidence,
                completed_at = EXCLUDED.completed_at,
                result_data = EXCLUDED.result_data
        "#,
    )
    .execute(&mut *tx)
    .await
    .expect("upsert engine_analysis_results");

    // Read path used by the results endpoint.
    sqlx::query(
        "SELECT analysis_id, status, verdict, confidence, created_at, completed_at, result_data \
         FROM engine_analysis_results ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("select from engine_analysis_results");

    tx.rollback().await.expect("rollback tx");
    pool.close().await;
}
