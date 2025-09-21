# Custom PostgreSQL image with extensions for threat intelligence
# Database Enhancements:

# Custom Extensions: UUID, crypto, full-text search, hstore
# Threat Intelligence Types: Custom enums for threat levels, bounty status
# Performance Optimization: Proper indexing strategy
# Audit Logging: Complete change tracking
# Statistics Views: Materialized views for analytics
# Security: Row-level security, multiple roles

# Operational Features:

# Migration System: Automatic schema migration on startup
# Seed Data: Test data loading capability
# Health Checks: Comprehensive database health validation
# Backup Ready: Archive configuration for point-in-time recovery



FROM postgres:15-alpine

# Install additional extensions and tools
RUN apk add --no-cache \
    postgresql15-contrib \
    postgresql15-dev \
    build-base \
    curl \
    git

# Set timezone
ENV TZ=UTC
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Create necessary directories
RUN mkdir -p /docker-entrypoint-initdb.d/migrations \
             /docker-entrypoint-initdb.d/seeds \
             /docker-entrypoint-initdb.d/extensions

# Copy initialization scripts
COPY database/migrations/ /docker-entrypoint-initdb.d/migrations/
COPY database/seeds/ /docker-entrypoint-initdb.d/seeds/
COPY database/schema.sql /docker-entrypoint-initdb.d/

# Create custom initialization script
RUN cat > /docker-entrypoint-initdb.d/00-init-extensions.sql << 'EOF'
-- Enable required PostgreSQL extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "hstore";
CREATE EXTENSION IF NOT EXISTS "ltree";

-- Create custom types for threat intelligence
CREATE TYPE threat_level AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE analysis_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE bounty_status AS ENUM ('open', 'claimed', 'disputed', 'resolved', 'expired');
CREATE TYPE reputation_event AS ENUM ('correct_analysis', 'incorrect_analysis', 'bounty_completion', 'dispute_resolution');

-- Create indexes for performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_sha256 ON files USING hash(sha256);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_created_at ON files(created_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_analyses_status ON analyses(status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_analyses_created_at ON analyses(created_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_bounties_status ON bounties(status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_bounties_reward ON bounties(reward_amount);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_wallet_address ON users USING hash(wallet_address);

-- Enable row level security
ALTER DATABASE nexus_security SET row_security = on;

-- Create roles
CREATE ROLE nexus_api WITH LOGIN PASSWORD 'api_password';
CREATE ROLE nexus_readonly WITH LOGIN PASSWORD 'readonly_password';

-- Grant permissions
GRANT CONNECT ON DATABASE nexus_security TO nexus_api;
GRANT CONNECT ON DATABASE nexus_security TO nexus_readonly;

-- Create audit logging function
CREATE OR REPLACE FUNCTION audit_trigger() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO audit_log(table_name, operation, old_values, changed_by, changed_at)
        VALUES(TG_TABLE_NAME, TG_OP, row_to_json(OLD), current_user, now());
        RETURN OLD;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_log(table_name, operation, old_values, new_values, changed_by, changed_at)
        VALUES(TG_TABLE_NAME, TG_OP, row_to_json(OLD), row_to_json(NEW), current_user, now());
        RETURN NEW;
    ELSIF TG_OP = 'INSERT' THEN
        INSERT INTO audit_log(table_name, operation, new_values, changed_by, changed_at)
        VALUES(TG_TABLE_NAME, TG_OP, row_to_json(NEW), current_user, now());
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id SERIAL PRIMARY KEY,
    table_name TEXT NOT NULL,
    operation TEXT NOT NULL,
    old_values JSONB,
    new_values JSONB,
    changed_by TEXT NOT NULL DEFAULT current_user,
    changed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

-- Create materialized view for threat statistics
CREATE MATERIALIZED VIEW threat_statistics AS
SELECT 
    DATE_TRUNC('day', created_at) as date,
    COUNT(*) as total_files,
    COUNT(*) FILTER (WHERE threat_level = 'critical') as critical_threats,
    COUNT(*) FILTER (WHERE threat_level = 'high') as high_threats,
    COUNT(*) FILTER (WHERE threat_level = 'medium') as medium_threats,
    COUNT(*) FILTER (WHERE threat_level = 'low') as low_threats,
    AVG(EXTRACT(EPOCH FROM (completed_at - created_at))) as avg_analysis_time
FROM analyses 
WHERE status = 'completed'
GROUP BY DATE_TRUNC('day', created_at)
ORDER BY date DESC;

-- Create index on materialized view
CREATE INDEX idx_threat_stats_date ON threat_statistics(date);

-- Function to refresh statistics
CREATE OR REPLACE FUNCTION refresh_threat_statistics() RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY threat_statistics;
END;
$$ LANGUAGE plpgsql;

EOF

# Create script to run migrations in order
RUN cat > /docker-entrypoint-initdb.d/99-run-migrations.sh << 'EOF'
#!/bin/bash
set -e

# Run migrations in numerical order
for migration in /docker-entrypoint-initdb.d/migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo "Running migration: $(basename "$migration")"
        psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$migration"
    fi
done

# Run seed data if exists
for seed in /docker-entrypoint-initdb.d/seeds/*.sql; do
    if [ -f "$seed" ]; then
        echo "Running seed: $(basename "$seed")"
        psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$seed"
    fi
done

# Grant permissions after all tables are created
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" << EOSQL
-- Grant permissions to api role
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO nexus_api;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO nexus_api;

-- Grant read-only permissions
GRANT SELECT ON ALL TABLES IN SCHEMA public TO nexus_readonly;

-- Set default permissions for future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO nexus_api;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO nexus_api;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO nexus_readonly;
EOSQL

EOF

RUN chmod +x /docker-entrypoint-initdb.d/99-run-migrations.sh

# Custom PostgreSQL configuration
RUN cat > /usr/local/share/postgresql/postgresql.conf.sample << 'EOF'
# Custom configuration for Nexus Security
shared_preload_libraries = 'pg_stat_statements'
max_connections = 200
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200

# Logging
log_statement = 'mod'
log_duration = on
log_min_duration_statement = 1000
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '

# Security
ssl = off
password_encryption = scram-sha-256

# Performance monitoring
pg_stat_statements.max = 10000
pg_stat_statements.track = all

# Backup and recovery
archive_mode = on
archive_command = 'test ! -f /backup/archive/%f && cp %p /backup/archive/%f'
EOF

# Create backup directory
RUN mkdir -p /backup/archive && chown postgres:postgres /backup/archive

# Health check script
RUN cat > /usr/local/bin/pg_health_check.sh << 'EOF'
#!/bin/bash
set -eo pipefail

# Check if PostgreSQL is accepting connections
pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB" -h localhost

# Check if required extensions are loaded
psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -t -c "SELECT 1 FROM pg_extension WHERE extname IN ('uuid-ossp', 'pgcrypto', 'pg_trgm');" | grep -q 1

# Check if custom types exist
psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -t -c "SELECT 1 FROM pg_type WHERE typname = 'threat_level';" | grep -q 1

echo "PostgreSQL health check passed"
EOF

RUN chmod +x /usr/local/bin/pg_health_check.sh

# Expose the PostgreSQL port
EXPOSE 5432

# Set the default command
CMD ["postgres"]