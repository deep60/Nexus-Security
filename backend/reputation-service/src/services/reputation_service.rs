use anyhow::Result;
use sqlx::PgPool;
use redis::aio::ConnectionManager;

use crate::config::Config;
use crate::scoring::ReputationScorer;

pub struct ReputationService {
    config: Config,
    db_pool: PgPool,
    redis_conn: ConnectionManager,
    scorer: ReputationScorer,
}

impl ReputationService {
    pub async fn new(
        config: Config,
        db_pool: PgPool,
        redis_conn: ConnectionManager,
    ) -> Result<Self> {
        let scorer = ReputationScorer::new(config.reputation.clone());
        
        Ok(Self {
            config,
            db_pool,
            redis_conn,
            scorer,
        })
    }
}
