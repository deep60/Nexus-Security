-- Migration 006: Decouple gateway domains from the local `users` table.
--
-- Under the microservices design, user identity is owned by `user-service`
-- (its own database). Users authenticate/register there and are referenced
-- across the platform by their UUID. The gateway's own tables (bounties,
-- submissions, engines, webhooks, etc.) previously declared FOREIGN KEY
-- constraints to the local `users` table, which means any insert carrying a
-- user id minted by user-service fails with a foreign-key violation because
-- that row does not exist in the gateway database.
--
-- This migration drops those cross-boundary FK constraints. The `user_id` /
-- `creator_id` / `submitter_id` / `owner_id` columns are kept as plain UUIDs
-- that reference the user-service-owned identity (referential integrity for
-- users is now enforced by user-service, not the gateway DB).
--
-- The local `users` table itself is intentionally retained (empty for
-- user-service-created accounts) so legacy read paths that still query it do
-- not error at runtime; those endpoints are being migrated to proxy
-- user-service separately.
--
-- Idempotent: constraint names are Postgres's inline defaults
-- (`<table>_<column>_fkey`); DROP ... IF EXISTS makes re-runs safe.

ALTER TABLE engines             DROP CONSTRAINT IF EXISTS engines_owner_id_fkey;
ALTER TABLE submissions         DROP CONSTRAINT IF EXISTS submissions_submitter_id_fkey;
ALTER TABLE bounties            DROP CONSTRAINT IF EXISTS bounties_creator_id_fkey;
ALTER TABLE user_sessions       DROP CONSTRAINT IF EXISTS user_sessions_user_id_fkey;
ALTER TABLE api_keys            DROP CONSTRAINT IF EXISTS api_keys_user_id_fkey;
ALTER TABLE wallet_transactions DROP CONSTRAINT IF EXISTS wallet_transactions_user_id_fkey;
ALTER TABLE reputation_history  DROP CONSTRAINT IF EXISTS reputation_history_user_id_fkey;
ALTER TABLE webhooks            DROP CONSTRAINT IF EXISTS webhooks_user_id_fkey;

-- Keep the user-id columns indexed for lookups now that they're standalone
-- references (most already have indexes from 001/002; these are safety nets).
CREATE INDEX IF NOT EXISTS idx_engines_owner ON engines(owner_id);
CREATE INDEX IF NOT EXISTS idx_wallet_tx_user_id ON wallet_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_reputation_history_user_id ON reputation_history(user_id);
CREATE INDEX IF NOT EXISTS idx_webhooks_user_id ON webhooks(user_id);
