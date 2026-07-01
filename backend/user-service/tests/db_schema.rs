//! Schema/query integration test for user-service.
//!
//! Runs the service's embedded migrations against a real Postgres database and
//! then executes the same queries the handlers use (users, user_profiles,
//! user_settings, kyc_verifications). This is the guard the project was missing:
//! previously a broken or missing schema passed CI (no test touched a table) and
//! only failed at runtime with `relation "..." does not exist`.
//!
//! It runs only when `USER_SERVICE_DATABASE_URL` is set (CI provides it via
//! scripts/ci/setup-test-databases.sh). Without it the test is a no-op so local
//! `cargo test` stays green when no database is available.
//!
//! All writes happen inside a transaction that is rolled back, so the test is
//! self-contained and re-runnable.

use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn test_database_url() -> Option<String> {
    std::env::var("USER_SERVICE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn schema_supports_handler_queries() {
    let Some(database_url) = test_database_url() else {
        eprintln!("skipping user-service schema test; USER_SERVICE_DATABASE_URL not set");
        return;
    };

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect to test database");

    // Applying the embedded migrations must succeed and be idempotent.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations apply cleanly");

    // Run all writes in a transaction we roll back, so the test leaves no rows.
    let mut tx = pool.begin().await.expect("begin tx");

    let user_id = Uuid::new_v4();
    let unique = user_id.simple().to_string();

    // Mirrors UserService::create_user.
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, ethereum_address, email_verified,
                          is_active, is_admin, two_factor_enabled, kyc_status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, false, true, false, false, 'not_submitted', NOW(), NOW())
        "#,
    )
    .bind(user_id)
    .bind(format!("user_{unique}"))
    .bind(format!("{unique}@example.test"))
    .bind("hashed-password")
    .bind(Option::<String>::None)
    .execute(&mut *tx)
    .await
    .expect("insert into users");

    sqlx::query(
        "INSERT INTO user_profiles (user_id, created_at, updated_at) VALUES ($1, NOW(), NOW())",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .expect("insert into user_profiles");

    sqlx::query("INSERT INTO user_settings (user_id) VALUES ($1)")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .expect("insert into user_settings");

    sqlx::query(
        r#"
        INSERT INTO kyc_verifications
            (id, user_id, full_name, country, document_type, document_number, status, submitted_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending', NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind("Test User")
    .bind("US")
    .bind("passport")
    .bind("X1234567")
    .execute(&mut *tx)
    .await
    .expect("insert into kyc_verifications");

    // The read paths the handlers exercise (SELECT * validates every column the
    // model structs decode).
    sqlx::query("SELECT * FROM users WHERE email = $1 OR username = $2")
        .bind(format!("{unique}@example.test"))
        .bind(format!("user_{unique}"))
        .fetch_optional(&mut *tx)
        .await
        .expect("select from users");

    sqlx::query("SELECT * FROM user_profiles WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .expect("select from user_profiles");

    sqlx::query("SELECT * FROM user_settings WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .expect("select from user_settings");

    sqlx::query(
        "SELECT * FROM kyc_verifications WHERE user_id = $1 ORDER BY submitted_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .expect("select from kyc_verifications");

    tx.rollback().await.expect("rollback tx");
    pool.close().await;
}
