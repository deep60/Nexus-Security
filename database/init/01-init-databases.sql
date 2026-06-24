-- Verdyx Database Initialization Script
-- Creates one database per microservice so each owns its own schema and
-- its own _sqlx_migrations table (no cross-service migration version collisions).
--
-- This runs only on first initialization of an empty Postgres data volume.
-- It is executed by the postgres entrypoint as ${POSTGRES_USER} connected to
-- the default ${POSTGRES_DB} database.

-- ---------------------------------------------------------------------------
-- Create one database per service (owned by verdyx_user)
-- ---------------------------------------------------------------------------
CREATE DATABASE verdyx_gateway        OWNER verdyx_user;
CREATE DATABASE verdyx_users          OWNER verdyx_user;
CREATE DATABASE verdyx_analysis       OWNER verdyx_user;
CREATE DATABASE verdyx_bounty         OWNER verdyx_user;
CREATE DATABASE verdyx_submissions    OWNER verdyx_user;
CREATE DATABASE verdyx_consensus      OWNER verdyx_user;
CREATE DATABASE verdyx_payments       OWNER verdyx_user;
CREATE DATABASE verdyx_reputation     OWNER verdyx_user;
CREATE DATABASE verdyx_notifications  OWNER verdyx_user;

-- ---------------------------------------------------------------------------
-- Install required extensions in every database.
-- (Extensions are per-database, so we connect to each one.)
-- Service migrations are responsible for creating their own enums/tables.
-- ---------------------------------------------------------------------------

\connect verdyx
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_gateway
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_users
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_analysis
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_bounty
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_submissions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_consensus
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_payments
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_reputation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\connect verdyx_notifications
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
