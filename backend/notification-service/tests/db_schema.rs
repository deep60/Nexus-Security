//! Schema/query integration test for notification-service.
//!
//! Applies the embedded migrations to a real Postgres database and exercises
//! the notification_preferences, notification_history and notification_templates
//! tables (including the seeded default templates). Runs only when
//! `NOTIFICATION_SERVICE_DATABASE_URL` (or `DATABASE_URL`) is set; otherwise it
//! is a no-op. All writes run inside a rolled-back transaction.

use sqlx::postgres::PgPoolOptions;

fn test_database_url() -> Option<String> {
    std::env::var("NOTIFICATION_SERVICE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn schema_supports_service_queries() {
    let Some(database_url) = test_database_url() else {
        eprintln!(
            "skipping notification-service schema test; NOTIFICATION_SERVICE_DATABASE_URL not set"
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

    // Mirrors the preferences upsert column list.
    sqlx::query(
        r#"
        INSERT INTO notification_preferences
            (user_id, email_enabled, push_enabled, webhook_enabled, websocket_enabled,
             email_address, push_token, webhook_url, webhook_secret)
        VALUES (gen_random_uuid(), true, true, false, true,
                'user@example.test', NULL, NULL, NULL)
        "#,
    )
    .execute(&mut *tx)
    .await
    .expect("insert into notification_preferences");

    // Mirrors the notification delivery history append.
    sqlx::query(
        r#"
        INSERT INTO notification_history (user_id, channel, event_type, payload, status)
        VALUES (gen_random_uuid(), 'email', 'analysis_complete', '{"file_name":"x"}'::jsonb, 'pending')
        "#,
    )
    .execute(&mut *tx)
    .await
    .expect("insert into notification_history");

    // The migration seeds default templates; the render path reads them.
    let _template_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notification_templates")
        .fetch_one(&mut *tx)
        .await
        .expect("count notification_templates");

    tx.rollback().await.expect("rollback tx");
    pool.close().await;
}
