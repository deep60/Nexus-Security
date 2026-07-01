//! Schema/query integration test for reputation-service.
//!
//! Applies the embedded migrations to a real Postgres database and exercises
//! the user_reputation, reputation_history and badges tables (including the
//! FK from history to user_reputation and the seeded badge rows). Runs only
//! when `REPUTATION_SERVICE_DATABASE_URL` (or `DATABASE_URL`) is set; otherwise
//! it is a no-op. All writes run inside a rolled-back transaction.

use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn test_database_url() -> Option<String> {
    std::env::var("REPUTATION_SERVICE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn schema_supports_service_queries() {
    let Some(database_url) = test_database_url() else {
        eprintln!(
            "skipping reputation-service schema test; REPUTATION_SERVICE_DATABASE_URL not set"
        );
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

    // Mirrors ReputationService::create for a new user.
    let user_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO user_reputation (user_id, current_score, highest_score, lowest_score)
        VALUES (gen_random_uuid(), 100, 100, 100)
        RETURNING user_id
        "#,
    )
    .fetch_one(&mut *tx)
    .await
    .expect("insert into user_reputation");

    // Mirrors the reputation_history append (FK to user_reputation).
    sqlx::query(
        r#"
        INSERT INTO reputation_history
            (user_id, score_before, score_after, score_change, reason, bounty_id, submission_id, details)
        VALUES ($1, 100, 110, 10, 'correct_submission', gen_random_uuid(), gen_random_uuid(), '{}'::jsonb)
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .expect("insert into reputation_history");

    // Leaderboard read path.
    sqlx::query("SELECT * FROM user_reputation ORDER BY current_score DESC LIMIT 10")
        .fetch_all(&mut *tx)
        .await
        .expect("select leaderboard from user_reputation");

    // The migration seeds badge rows; the awarding path reads them.
    let _badge_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM badges")
        .fetch_one(&mut *tx)
        .await
        .expect("count badges");

    tx.rollback().await.expect("rollback tx");
    pool.close().await;
}
