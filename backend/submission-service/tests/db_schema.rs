//! Schema/query integration test for submission-service.
//!
//! Runs the service's embedded migrations against a real Postgres database and
//! then executes the same queries the repository layer uses against the
//! `submissions` table. Guards against the "relation does not exist" class of
//! runtime failures that previously slipped through CI.
//!
//! Runs only when `SUBMISSION_SERVICE_DATABASE_URL` is set (CI provides it via
//! scripts/ci/setup-test-databases.sh); otherwise it is a no-op so local
//! `cargo test` stays green without a database.
//!
//! All writes happen inside a transaction that is rolled back.

use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn test_database_url() -> Option<String> {
    std::env::var("SUBMISSION_SERVICE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn schema_supports_repository_queries() {
    let Some(database_url) = test_database_url() else {
        eprintln!(
            "skipping submission-service schema test; SUBMISSION_SERVICE_DATABASE_URL not set"
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

    let submission_id = Uuid::new_v4();

    // Mirrors db::repository::create_submission.
    sqlx::query(
        r#"
        INSERT INTO submissions (
            id, submitter_id, file_hash, url, original_filename, file_size, mime_type,
            file_path, submission_type, analysis_status, metadata, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', $10, NOW(), NOW())
        "#,
    )
    .bind(submission_id)
    .bind(Some(Uuid::new_v4()))
    .bind(Some("abc123def456"))
    .bind(Option::<String>::None)
    .bind(Some("sample.exe"))
    .bind(Some(1024_i64))
    .bind(Some("application/octet-stream"))
    .bind(Some("submissions/sample.exe"))
    .bind("file")
    .bind(Option::<serde_json::Value>::None)
    .execute(&mut *tx)
    .await
    .expect("insert into submissions");

    sqlx::query("SELECT * FROM submissions WHERE id = $1")
        .bind(submission_id)
        .fetch_optional(&mut *tx)
        .await
        .expect("select from submissions");

    sqlx::query("UPDATE submissions SET analysis_status = $1, updated_at = $2 WHERE id = $3")
        .bind("analyzing")
        .bind(chrono::Utc::now())
        .bind(submission_id)
        .execute(&mut *tx)
        .await
        .expect("update submissions");

    let _count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM submissions WHERE file_hash = $1")
        .bind("abc123def456")
        .fetch_one(&mut *tx)
        .await
        .expect("count submissions by hash");

    tx.rollback().await.expect("rollback tx");
    pool.close().await;
}
