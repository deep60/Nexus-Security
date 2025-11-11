use anyhow::Result;
use sqlx::PgPool;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use ethers::prelude::*;

use crate::config::Config;
use crate::blockchain::BlockchainProvider;

pub struct PaymentService {
    config: Config,
    db_pool: PgPool,
    redis_conn: ConnectionManager,
    provider: BlockchainProvider,
}

impl PaymentService {
    pub async fn new(
        config: Config,
        db_pool: PgPool,
        redis_conn: ConnectionManager,
        provider: BlockchainProvider,
    ) -> Result<Self> {
        Ok(Self {
            config,
            db_pool,
            redis_conn,
            provider,
        })
    }
}
