use redis::AsyncCommands;
use uuid::Uuid;

/// Redis queue key for analysis tasks
const ANALYSIS_QUEUE_KEY: &str = "analysis_queue";

/// Publish a submission ID to the analysis queue
pub async fn publish_to_analysis_queue(
    redis_client: &redis::Client,
    submission_id: Uuid,
) -> Result<(), redis::RedisError> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    // Push submission ID to the queue (LPUSH for FIFO with BRPOP)
    let submission_id_str = submission_id.to_string();
    conn.lpush(ANALYSIS_QUEUE_KEY, &submission_id_str).await?;

    tracing::info!(
        "Published submission {} to analysis queue",
        submission_id
    );

    Ok(())
}

/// Get queue length (useful for monitoring)
pub async fn get_queue_length(
    redis_client: &redis::Client,
) -> Result<usize, redis::RedisError> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;
    let len: usize = conn.llen(ANALYSIS_QUEUE_KEY).await?;
    Ok(len)
}

/// Publish multiple submission IDs in bulk
pub async fn publish_bulk_to_analysis_queue(
    redis_client: &redis::Client,
    submission_ids: Vec<Uuid>,
) -> Result<(), redis::RedisError> {
    if submission_ids.is_empty() {
        return Ok(());
    }

    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    let submission_id_strings: Vec<String> = submission_ids
        .iter()
        .map(|id| id.to_string())
        .collect();

    // Use pipeline for bulk insert
    let mut pipe = redis::pipe();
    for id_str in &submission_id_strings {
        pipe.lpush(ANALYSIS_QUEUE_KEY, id_str);
    }
    pipe.query_async(&mut conn).await?;

    tracing::info!(
        "Published {} submissions to analysis queue",
        submission_ids.len()
    );

    Ok(())
}
