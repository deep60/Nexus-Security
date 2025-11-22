-- Nexus Security Database Initialization Script

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm"; -- For fuzzy text search
CREATE EXTENSION IF NOT EXISTS "pgcrypto"; -- For cryptographic functions

-- Create custom types
CREATE TYPE bounty_status AS ENUM ('draft', 'open', 'in_progress', 'under_review', 'completed', 'cancelled', 'expired');
CREATE TYPE threat_verdict AS ENUM ('malicious', 'benign', 'suspicious', 'unknown');
CREATE TYPE artifact_type AS ENUM ('file', 'url', 'hash', 'ip', 'domain');
CREATE TYPE kyc_status AS ENUM ('not_submitted', 'pending', 'under_review', 'approved', 'rejected');
CREATE TYPE analysis_status AS ENUM ('pending', 'in_progress', 'completed', 'failed');

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE nexus_security TO nexus_user;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO nexus_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO nexus_user;

-- Log initialization
INSERT INTO pg_catalog.pg_ts_config_map VALUES ('default', 'postgres', 'pg_catalog.simple');

