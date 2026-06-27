// Storage module for S3/MinIO integration

pub mod s3_client;

// Re-export the S3Client for convenience
pub use s3_client::{FileMetadata, S3Client};
