-- The api-gateway analysis handlers (handlers/analysis.rs) read, insert, and
-- update an `analyses` table — analyst-submitted verdicts tied to a bounty.
-- It was never created by earlier migrations (only the distinct engine
-- `analysis_results` table exists), so /api/v1/analysis endpoints failed with
-- `relation "analyses" does not exist` (HTTP 500). This creates it to match the
-- columns the handlers use. No FKs on bounty_id/analyst_id, consistent with the
-- FK-decoupling in 006_decouple_user_fks.sql.
CREATE TABLE IF NOT EXISTS analyses (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bounty_id    UUID,
    analyst_id   UUID,
    file_hash    VARCHAR(64),
    verdict      VARCHAR(20),
    confidence   DOUBLE PRECISION,
    status       VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_analyses_bounty  ON analyses(bounty_id);
CREATE INDEX IF NOT EXISTS idx_analyses_analyst ON analyses(analyst_id);
CREATE INDEX IF NOT EXISTS idx_analyses_status  ON analyses(status);
