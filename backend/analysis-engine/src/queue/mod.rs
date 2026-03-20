pub mod consumer;
// NOTE: scheduler is temporarily disabled — it depends on the `shared` crate
// (KafkaProducer, RedisClient, etc.) which is not a dependency of analysis-engine.
// pub mod scheduler;