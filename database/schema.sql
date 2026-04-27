-- Verdyx Complete Database Schema
-- This is the master schema file that orchestrates all database migrations
-- Run this file to initialize the complete database structure

-- Enable required PostgreSQL extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\echo '================================'
\echo 'Verdyx Database Schema'
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
\echo 'Step 5/7: Applying schema fixes and updates...'
\i postgres/migrations/005_fix_user_schema.sql

\echo ''
\echo 'Step 6/7: Creating API gateway service tables...'
\i postgres/migrations/006_api_gateway_tables.sql

\echo ''
\echo 'Step 7/7: Creating payout tables...'
\i postgres/migrations/007_payout_tables.sql

\echo ''
\echo '================================'
\echo 'Database Schema Creation Complete!'
\echo '================================'
\echo ''
\echo 'Summary:'
\echo '  - Extensions enabled: uuid-ossp, pgcrypto'
\echo '  - Migrations applied: 7'
\echo '  - Tables created: 45+'
\echo '  - Indexes created: 120+'
\echo '  - Functions created: 5+'
\echo '  - Views created: 3'
\echo ''
\echo 'Next steps:'
\echo '  1. Review the schema for any customizations'
\echo '  2. Load seed data: psql -d verdyx -f postgres/seeds/test_data.sql'
\echo '  3. Verify tables: \\dt'
\echo '  4. Connect your application'
\echo ''
\echo 'Default Database Configuration:'
\echo '  - Database: verdyx'
\echo '  - User: postgres (change in production!)'
\echo '  - Port: 5432'
\echo ''
