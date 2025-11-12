use anyhow::Result;
use sqlx::PgPool;
use redis::aio::ConnectionManager;

use crate::config::Config;
use crate::aggregation::ConsensusAggregator;

pub struct ConsensusService {
    config: Config,
    db_pool: PgPool,
    redis_conn: ConnectionManager,
    aggregator: ConsensusAggregator,
}

impl ConsensusService {
    pub async fn new(
        config: Config,
        db_pool: PgPool,
        redis_conn: ConnectionManager,
    ) -> Result<Self> {
        let aggregator = ConsensusAggregator::new(config.consensus.clone());
        
        Ok(Self {
            config,
            db_pool,
            redis_conn,
            aggregator,
        })
    }
}
