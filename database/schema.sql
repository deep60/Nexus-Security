-- Nexus-Security Complete Database Schema
-- This is the master schema file that orchestrates all database migrations
-- Run this file to initialize the complete database structure

-- Enable required PostgreSQL extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\echo '================================'
\echo 'Nexus-Security Database Schema'
\echo 'Version: 1.0.0'
\echo '================================'

\echo ''
\echo 'Step 1/5: Creating core user and engine tables...'
\i postgres/migrations/001_user_engine.sql

\echo ''
\echo 'Step 2/5: Creating bounty system tables...'
\i postgres/migrations/002_bounty_system.sql

\echo ''
\echo 'Step 3/5: Creating blockchain integration tables...'
\i postgres/migrations/003_blockchain.sql

\echo ''
\echo 'Step 4/5: Creating reputation system tables...'
\i postgres/migrations/004_reputation_system.sql

\echo ''
\echo 'Step 5/5: Applying schema fixes and updates...'
\i postgres/migrations/005_fix_user_schema.sql

\echo ''
\echo '================================'
\echo 'Database Schema Creation Complete!'
\echo '================================'
\echo ''
\echo 'Summary:'
\echo '  - Extensions enabled: uuid-ossp, pgcrypto'
\echo '  - Migrations applied: 5'
\echo '  - Tables created: 40+'
\echo '  - Indexes created: 100+'
\echo '  - Functions created: 5+'
\echo '  - Views created: 3'
\echo ''
\echo 'Next steps:'
\echo '  1. Review the schema for any customizations'
\echo '  2. Load seed data: psql -d nexus_security -f postgres/seeds/test_data.sql'
\echo '  3. Verify tables: \\dt'
\echo '  4. Connect your application'
\echo ''
\echo 'Default Database Configuration:'
\echo '  - Database: nexus_security'
\echo '  - User: postgres (change in production!)'
\echo '  - Port: 5432'
\echo ''
