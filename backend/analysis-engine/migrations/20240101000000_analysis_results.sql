-- Analysis Engine: results storage
-- Stores the full AnalysisResult as JSONB with extracted indexed columns for querying

CREATE TABLE IF NOT EXISTS engine_analysis_results (
    analysis_id UUID PRIMARY KEY,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    verdict VARCHAR(20),
    confidence REAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    result_data JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_engine_results_status ON engine_analysis_results(status);
CREATE INDEX IF NOT EXISTS idx_engine_results_verdict ON engine_analysis_results(verdict);
CREATE INDEX IF NOT EXISTS idx_engine_results_created ON engine_analysis_results(created_at DESC);
