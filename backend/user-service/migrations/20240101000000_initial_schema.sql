-- Initial schema for user-service (database: verdyx_users).
--
-- Owns the user identity, profile, settings, and KYC tables that the service
-- queries at runtime. Extensions (uuid-ossp, pgcrypto, pg_trgm) are installed
-- by /database/init/01-init-databases.sql when the data volume is first
-- created; this migration only creates service-local tables.

-- ---------------------------------------------------------------------------
-- users: core identity / auth record
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username            VARCHAR(50) NOT NULL UNIQUE,
    email               VARCHAR(255) NOT NULL UNIQUE,
    password_hash       TEXT NOT NULL,
    ethereum_address    VARCHAR(42),
    email_verified      BOOLEAN NOT NULL DEFAULT false,
    is_active           BOOLEAN NOT NULL DEFAULT true,
    is_admin            BOOLEAN NOT NULL DEFAULT false,
    two_factor_enabled  BOOLEAN NOT NULL DEFAULT false,
    two_factor_secret   TEXT,
    kyc_status          VARCHAR(32) NOT NULL DEFAULT 'not_submitted',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login          TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_kyc_status ON users(kyc_status);
CREATE INDEX IF NOT EXISTS idx_users_ethereum_address ON users(ethereum_address);

-- ---------------------------------------------------------------------------
-- user_profiles: one-to-one public-facing profile
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS user_profiles (
    user_id         UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    display_name    VARCHAR(100),
    bio             TEXT,
    avatar_url      TEXT,
    location        VARCHAR(255),
    website         VARCHAR(255),
    twitter         VARCHAR(255),
    github          VARCHAR(255),
    specializations TEXT[] NOT NULL DEFAULT '{}',
    public_email    VARCHAR(255),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ---------------------------------------------------------------------------
-- user_settings: one-to-one preferences
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS user_settings (
    user_id                 UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    email_notifications     BOOLEAN NOT NULL DEFAULT true,
    push_notifications      BOOLEAN NOT NULL DEFAULT true,
    webhook_notifications   BOOLEAN NOT NULL DEFAULT false,
    privacy_public_profile  BOOLEAN NOT NULL DEFAULT true,
    privacy_show_email      BOOLEAN NOT NULL DEFAULT false,
    privacy_show_stats      BOOLEAN NOT NULL DEFAULT true,
    language                VARCHAR(10) NOT NULL DEFAULT 'en',
    timezone                VARCHAR(64) NOT NULL DEFAULT 'UTC',
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ---------------------------------------------------------------------------
-- kyc_verifications: KYC submissions (latest per user is the active one)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS kyc_verifications (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    full_name           VARCHAR(255) NOT NULL,
    date_of_birth       DATE,
    country             VARCHAR(100) NOT NULL,
    document_type       VARCHAR(50) NOT NULL,
    document_number     VARCHAR(100) NOT NULL,
    document_front_url  TEXT NOT NULL DEFAULT '',
    document_back_url   TEXT,
    selfie_url          TEXT NOT NULL DEFAULT '',
    status              VARCHAR(32) NOT NULL DEFAULT 'pending',
    rejection_reason    TEXT,
    verified_by         UUID,
    submitted_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at         TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_kyc_user_id ON kyc_verifications(user_id);
CREATE INDEX IF NOT EXISTS idx_kyc_submitted_at ON kyc_verifications(user_id, submitted_at DESC);
