-- reputation_system.sql - Reputation & Scoring

-- Reputation Events: All reputation-affecting actions
-- Reputation Scores: Detailed scoring breakdown
-- Performance Metrics: Time-based performance tracking
-- Expertise Areas: Specialized knowledge domains
-- Trust Relationships: Inter-user/engine trust
-- Leaderboards: Ranking system
-- Calculation Functions: Automated score computation

-- Reputation events table to track all reputation-affecting actions
CREATE TABLE reputation_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    engine_id UUID REFERENCES engines(id) ON DELETE CASCADE,
    event_type VARCHAR(30) NOT NULL CHECK (
        event_type IN (
            'correct_analysis', 'incorrect_analysis', 'bounty_win', 'bounty_loss',
            'stake_slash', 'early_detection', 'false_positive', 'false_negative',
            'consensus_agreement', 'consensus_disagreement', 'quality_bonus',
            'speed_bonus', 'verification_success', 'verification_fail'
        )
    ),
    bounty_id UUID REFERENCES bounties(id) ON DELETE SET NULL,
    submission_id UUID REFERENCES submissions(id) ON DELETE SET NULL,
    reputation_change INTEGER NOT NULL, -- Can be negative
    multiplier DECIMAL(4,2) DEFAULT 1.00, -- Reputation multiplier for special cases
    description TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Detailed reputation scores breakdown
CREATE TABLE reputation_scores (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    engine_id UUID REFERENCES engines(id) ON DELETE CASCADE,
    accuracy_score INTEGER DEFAULT 0, -- Based on correct predictions
    speed_score INTEGER DEFAULT 0, -- Based on analysis speed
    consistency_score INTEGER DEFAULT 0, -- Based on consistent performance
    expertise_score INTEGER DEFAULT 0, -- Based on specialized knowledge
    community_score INTEGER DEFAULT 0, -- Based on community interactions
    total_score INTEGER GENERATED ALWAYS AS (
        accuracy_score + speed_score + consistency_score + expertise_score + community_score
    ) STORED,
    rank_position INTEGER,
    percentile DECIMAL(5,2),
    last_calculated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure only one of user_id or engine_id is set
    CONSTRAINT check_user_or_engine CHECK (
        (user_id IS NOT NULL AND engine_id IS NULL) OR 
        (user_id IS NULL AND engine_id IS NOT NULL)
    )
);

-- Performance metrics for engines and users
CREATE TABLE performance_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    engine_id UUID REFERENCES engines(id) ON DELETE CASCADE,
    time_period VARCHAR(20) NOT NULL CHECK (time_period IN ('daily', 'weekly', 'monthly', 'yearly', 'all_time')),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    
    -- Analysis metrics
    total_analyses INTEGER DEFAULT 0,
    correct_analyses INTEGER DEFAULT 0,
    false_positives INTEGER DEFAULT 0,
    false_negatives INTEGER DEFAULT 0,
    accuracy_rate DECIMAL(5,4) DEFAULT 0.0000,
    precision_rate DECIMAL(5,4) DEFAULT 0.0000,
    recall_rate DECIMAL(5,4) DEFAULT 0.0000,
    f1_score DECIMAL(5,4) DEFAULT 0.0000,
    
    -- Speed metrics
    avg_analysis_time INTEGER, -- Average analysis time in seconds
    fastest_analysis_time INTEGER,
    slowest_analysis_time INTEGER,
    
    -- Financial metrics
    total_earnings DECIMAL(20,8) DEFAULT 0,
    total_stakes DECIMAL(20,8) DEFAULT 0,
    total_losses DECIMAL(20,8) DEFAULT 0,
    roi_percentage DECIMAL(8,4) DEFAULT 0.0000, -- Return on investment
    
    -- Participation metrics
    bounties_participated INTEGER DEFAULT 0,
    bounties_won INTEGER DEFAULT 0,
    consensus_agreements INTEGER DEFAULT 0,
    consensus_disagreements INTEGER DEFAULT 0,
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure only one of user_id or engine_id is set
    CONSTRAINT check_user_or_engine_metrics CHECK (
        (user_id IS NOT NULL AND engine_id IS NULL) OR 
        (user_id IS NULL AND engine_id IS NOT NULL)
    ),
    
    -- Unique constraint for time periods
    UNIQUE(user_id, engine_id, time_period, period_start, period_end)
);

-- Expertise areas for specialized reputation
CREATE TABLE expertise_areas (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    icon VARCHAR(50), -- Icon name or class
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- User/Engine expertise in specific areas
CREATE TABLE user_expertise (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    engine_id UUID REFERENCES engines(id) ON DELETE CASCADE,
    expertise_area_id INTEGER NOT NULL REFERENCES expertise_areas(id) ON DELETE CASCADE,
    proficiency_level INTEGER NOT NULL CHECK (proficiency_level BETWEEN 1 AND 5), -- 1=novice, 5=expert
    experience_points INTEGER DEFAULT 0,
    certifications TEXT[], -- Array of relevant certifications
    specialization_score INTEGER DEFAULT 0,
    last_activity TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure only one of user_id or engine_id is set
    CONSTRAINT check_user_or_engine_expertise CHECK (
        (user_id IS NOT NULL AND engine_id IS NULL) OR 
        (user_id IS NULL AND engine_id IS NOT NULL)
    ),
    
    UNIQUE(user_id, engine_id, expertise_area_id)
);

-- Trust relationships between users/engines
CREATE TABLE trust_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trustor_id UUID NOT NULL, -- Who is giving trust (user or engine)
    trustee_id UUID NOT NULL, -- Who is receiving trust (user or engine)
    trust_level INTEGER NOT NULL CHECK (trust_level BETWEEN -100 AND 100), -- -100=distrust, 100=full trust
    trust_type VARCHAR(20) NOT NULL CHECK (trust_type IN ('direct', 'derived', 'algorithmic')),
    basis_description TEXT,
    confidence INTEGER DEFAULT 50 CHECK (confidence BETWEEN 0 AND 100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Prevent self-trust
    CONSTRAINT no_self_trust CHECK (trustor_id != trustee_id),
    
    -- Unique trust relationship
    UNIQUE(trustor_id, trustee_id)
);

-- Leaderboards for different categories
CREATE TABLE leaderboards (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    category VARCHAR(30) NOT NULL CHECK (
        category IN ('accuracy', 'speed', 'earnings', 'participation', 'expertise', 'overall')
    ),
    time_period VARCHAR(20) NOT NULL CHECK (time_period IN ('daily', 'weekly', 'monthly', 'yearly', 'all_time')),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    engine_id UUID REFERENCES engines(id) ON DELETE CASCADE,
    score DECIMAL(15,4) NOT NULL,
    rank_position INTEGER NOT NULL,
    previous_rank INTEGER,
    rank_change INTEGER GENERATED ALWAYS AS (COALESCE(previous_rank, rank_position) - rank_position) STORED,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure only one of user_id or engine_id is set
    CONSTRAINT check_user_or_engine_leaderboard CHECK (
        (user_id IS NOT NULL AND engine_id IS NULL) OR 
        (user_id IS NULL AND engine_id IS NOT NULL)
    ),
    
    UNIQUE(category, time_period, user_id, engine_id, period_start, period_end)
);

-- Insert default expertise areas
INSERT INTO expertise_areas (name, description, icon) VALUES
('malware_analysis', 'Static and dynamic malware analysis', 'bug'),
('network_security', 'Network traffic analysis and intrusion detection', 'network'),
('web_security', 'Web application security and vulnerability assessment', 'globe'),
('mobile_security', 'Mobile application and device security', 'smartphone'),
('cryptography', 'Cryptographic analysis and security', 'key'),
('forensics', 'Digital forensics and incident response', 'search'),
('threat_intelligence', 'Threat hunting and intelligence analysis', 'eye'),
('reverse_engineering', 'Binary analysis and reverse engineering', 'code'),
('iot_security', 'Internet of Things security', 'cpu'),
('blockchain_security', 'Blockchain and smart contract security', 'blocks');

-- Indexes for reputation system
CREATE INDEX idx_reputation_events_user ON reputation_events(user_id);
CREATE INDEX idx_reputation_events_engine ON reputation_events(engine_id);
CREATE INDEX idx_reputation_events_type ON reputation_events(event_type);
CREATE INDEX idx_reputation_events_bounty ON reputation_events(bounty_id);
CREATE INDEX idx_reputation_events_created ON reputation_events(created_at);

CREATE INDEX idx_reputation_scores_user ON reputation_scores(user_id);
CREATE INDEX idx_reputation_scores_engine ON reputation_scores(engine_id);
CREATE INDEX idx_reputation_scores_total ON reputation_scores(total_score DESC);
CREATE INDEX idx_reputation_scores_rank ON reputation_scores(rank_position);

CREATE INDEX idx_performance_metrics_user ON performance_metrics(user_id);
CREATE INDEX idx_performance_metrics_engine ON performance_metrics(engine_id);
CREATE INDEX idx_performance_metrics_period ON performance_metrics(time_period, period_start, period_end);
CREATE INDEX idx_performance_metrics_accuracy ON performance_metrics(accuracy_rate DESC);

CREATE INDEX idx_user_expertise_user ON user_expertise(user_id);
CREATE INDEX idx_user_expertise_engine ON user_expertise(engine_id);
CREATE INDEX idx_user_expertise_area ON user_expertise(expertise_area_id);
CREATE INDEX idx_user_expertise_level ON user_expertise(proficiency_level DESC);

CREATE INDEX idx_trust_relationships_trustor ON trust_relationships(trustor_id);
CREATE INDEX idx_trust_relationships_trustee ON trust_relationships(trustee_id);
CREATE INDEX idx_trust_relationships_level ON trust_relationships(trust_level DESC);

CREATE INDEX idx_leaderboards_category ON leaderboards(category, time_period);
CREATE INDEX idx_leaderboards_user ON leaderboards(user_id);
CREATE INDEX idx_leaderboards_engine ON leaderboards(engine_id);
CREATE INDEX idx_leaderboards_rank ON leaderboards(rank_position);
CREATE INDEX idx_leaderboards_score ON leaderboards(score DESC);

-- Add updated_at triggers
CREATE TRIGGER update_reputation_scores_updated_at BEFORE UPDATE ON reputation_scores FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_performance_metrics_updated_at BEFORE UPDATE ON performance_metrics FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_expertise_updated_at BEFORE UPDATE ON user_expertise FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_trust_relationships_updated_at BEFORE UPDATE ON trust_relationships FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- View for comprehensive reputation overview
CREATE VIEW reputation_overview AS
SELECT 
    COALESCE(u.id, e.owner_id) as owner_id,
    COALESCE(u.username, e.name) as name,
    'user'::text as entity_type,
    rs.accuracy_score,
    rs.speed_score,
    rs.consistency_score,
    rs.expertise_score,
    rs.community_score,
    rs.total_score,
    rs.rank_position,
    rs.percentile,
    pm.accuracy_rate,
    pm.total_earnings,
    pm.bounties_won,
    pm.bounties_participated
FROM reputation_scores rs
LEFT JOIN users u ON rs.user_id = u.id
LEFT JOIN engines e ON rs.engine_id = e.id
LEFT JOIN performance_metrics pm ON (pm.user_id = rs.user_id OR pm.engine_id = rs.engine_id) 
    AND pm.time_period = 'all_time'
ORDER BY rs.total_score DESC;

-- Function to calculate reputation score based on recent performance
CREATE OR REPLACE FUNCTION calculate_reputation_score(entity_id UUID, is_user BOOLEAN DEFAULT TRUE)
RETURNS INTEGER AS $
DECLARE
    base_score INTEGER := 0;
    accuracy_bonus INTEGER := 0;
    speed_bonus INTEGER := 0;
    consistency_bonus INTEGER := 0;
    recent_performance RECORD;
BEGIN
    -- Get recent performance metrics
    SELECT 
        COALESCE(accuracy_rate * 1000, 0)::INTEGER as acc_score,
        CASE 
            WHEN avg_analysis_time IS NULL THEN 0
            WHEN avg_analysis_time < 300 THEN 100  -- Under 5 minutes
            WHEN avg_analysis_time < 900 THEN 75   -- Under 15 minutes
            WHEN avg_analysis_time < 1800 THEN 50  -- Under 30 minutes
            ELSE 25
        END as spd_score,
        CASE 
            WHEN total_analyses >= 100 THEN 100
            WHEN total_analyses >= 50 THEN 75
            WHEN total_analyses >= 20 THEN 50
            ELSE 25
        END as con_score
    INTO recent_performance
    FROM performance_metrics pm
    WHERE (is_user = TRUE AND pm.user_id = entity_id) 
       OR (is_user = FALSE AND pm.engine_id = entity_id)
    AND pm.time_period = 'monthly'
    ORDER BY pm.created_at DESC
    LIMIT 1;

    -- Calculate component scores
    accuracy_bonus := COALESCE(recent_performance.acc_score, 0);
    speed_bonus := COALESCE(recent_performance.spd_score, 0);
    consistency_bonus := COALESCE(recent_performance.con_score, 0);
    
    -- Base score from reputation events
    SELECT COALESCE(SUM(reputation_change * multiplier), 0)::INTEGER
    INTO base_score
    FROM reputation_events
    WHERE (is_user = TRUE AND user_id = entity_id) 
       OR (is_user = FALSE AND engine_id = entity_id);

    RETURN base_score + accuracy_bonus + speed_bonus + consistency_bonus;
END;
$ LANGUAGE plpgsql;

-- Function to update performance metrics for a specific period
CREATE OR REPLACE FUNCTION update_performance_metrics(
    entity_id UUID, 
    is_user BOOLEAN DEFAULT TRUE,
    period_type VARCHAR(20) DEFAULT 'monthly'
) RETURNS VOID AS $
DECLARE
    start_date DATE;
    end_date DATE;
    perf_data RECORD;
BEGIN
    -- Calculate period dates
    CASE period_type
        WHEN 'daily' THEN
            start_date := CURRENT_DATE;
            end_date := CURRENT_DATE;
        WHEN 'weekly' THEN
            start_date := DATE_TRUNC('week', CURRENT_DATE)::DATE;
            end_date := (DATE_TRUNC('week', CURRENT_DATE) + INTERVAL '6 days')::DATE;
        WHEN 'monthly' THEN
            start_date := DATE_TRUNC('month', CURRENT_DATE)::DATE;
            end_date := (DATE_TRUNC('month', CURRENT_DATE) + INTERVAL '1 month - 1 day')::DATE;
        WHEN 'yearly' THEN
            start_date := DATE_TRUNC('year', CURRENT_DATE)::DATE;
            end_date := (DATE_TRUNC('year', CURRENT_DATE) + INTERVAL '1 year - 1 day')::DATE;
        ELSE -- all_time
            start_date := '1900-01-01'::DATE;
            end_date := CURRENT_DATE;
    END CASE;

    -- Calculate performance metrics from analysis results
    SELECT 
        COUNT(*) as total_analyses,
        SUM(CASE WHEN ar.verdict = s.is_malicious::TEXT OR 
                     (ar.verdict = 'benign' AND s.is_malicious = FALSE) OR
                     (ar.verdict = 'malicious' AND s.is_malicious = TRUE)
                THEN 1 ELSE 0 END) as correct_analyses,
        SUM(CASE WHEN ar.verdict = 'malicious' AND s.is_malicious = FALSE THEN 1 ELSE 0 END) as false_positives,
        SUM(CASE WHEN ar.verdict = 'benign' AND s.is_malicious = TRUE THEN 1 ELSE 0 END) as false_negatives,
        AVG(ar.analysis_duration) as avg_time,
        MIN(ar.analysis_duration) as min_time,
        MAX(ar.analysis_duration) as max_time,
        COUNT(DISTINCT bp.bounty_id) as bounties_participated,
        SUM(CASE WHEN bp.is_winner = TRUE THEN 1 ELSE 0 END) as bounties_won,
        SUM(bp.reward_earned) as total_earnings,
        SUM(bp.stake_amount) as total_stakes
    INTO perf_data
    FROM analysis_results ar
    JOIN bounty_participations bp ON ar.participation_id = bp.id
    JOIN submissions s ON ar.submission_id = s.id
    WHERE ((is_user = TRUE AND ar.engine_id IN (SELECT id FROM engines WHERE owner_id = entity_id)) 
           OR (is_user = FALSE AND ar.engine_id = entity_id))
    AND ar.created_at::DATE BETWEEN start_date AND end_date
    AND ar.analysis_status = 'completed';

    -- Insert or update performance metrics
    INSERT INTO performance_metrics (
        user_id, engine_id, time_period, period_start, period_end,
        total_analyses, correct_analyses, false_positives, false_negatives,
        accuracy_rate, avg_analysis_time, fastest_analysis_time, slowest_analysis_time,
        total_earnings, total_stakes, bounties_participated, bounties_won
    ) VALUES (
        CASE WHEN is_user THEN entity_id ELSE NULL END,
        CASE WHEN NOT is_user THEN entity_id ELSE NULL END,
        period_type, start_date, end_date,
        COALESCE(perf_data.total_analyses, 0),
        COALESCE(perf_data.correct_analyses, 0),
        COALESCE(perf_data.false_positives, 0),
        COALESCE(perf_data.false_negatives, 0),
        CASE WHEN COALESCE(perf_data.total_analyses, 0) > 0 
             THEN COALESCE(perf_data.correct_analyses, 0)::DECIMAL / perf_data.total_analyses 
             ELSE 0 END,
        perf_data.avg_time::INTEGER,
        perf_data.min_time,
        perf_data.max_time,
        COALESCE(perf_data.total_earnings, 0),
        COALESCE(perf_data.total_stakes, 0),
        COALESCE(perf_data.bounties_participated, 0),
        COALESCE(perf_data.bounties_won, 0)
    )
    ON CONFLICT (user_id, engine_id, time_period, period_start, period_end) 
    DO UPDATE SET
        total_analyses = EXCLUDED.total_analyses,
        correct_analyses = EXCLUDED.correct_analyses,
        false_positives = EXCLUDED.false_positives,
        false_negatives = EXCLUDED.false_negatives,
        accuracy_rate = EXCLUDED.accuracy_rate,
        avg_analysis_time = EXCLUDED.avg_analysis_time,
        fastest_analysis_time = EXCLUDED.fastest_analysis_time,
        slowest_analysis_time = EXCLUDED.slowest_analysis_time,
        total_earnings = EXCLUDED.total_earnings,
        total_stakes = EXCLUDED.total_stakes,
        bounties_participated = EXCLUDED.bounties_participated,
        bounties_won = EXCLUDED.bounties_won,
        updated_at = CURRENT_TIMESTAMP;
END;
$ LANGUAGE plpgsql; 