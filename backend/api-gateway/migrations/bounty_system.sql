-- bounty_system.sql - Bounty System

-- Bounties: Threat analysis bounties with rewards
-- Participations: Engine participation and stakes
-- Analysis Results: Detailed analysis outputs from engines
-- Consensus: Aggregated results and final verdicts
-- Rewards: Payout tracking and distribution
-- Tagging System: Bounty categorization
-- Performance Views: Statistics and reporting

-- Bounties table for threat analysis incentives
CREATE TABLE bounties (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    description TEXT,
    reward_amount DECIMAL(20,8) NOT NULL, -- Amount in wei
    min_stake_amount DECIMAL(20,8) NOT NULL DEFAULT 0, -- Minimum stake required
    max_participants INTEGER DEFAULT NULL, -- NULL means unlimited
    deadline TIMESTAMP WITH TIME ZONE,
    bounty_status VARCHAR(20) DEFAULT 'active' CHECK (
        bounty_status IN ('active', 'completed', 'expired', 'cancelled')
    ),
    requires_verification BOOLEAN DEFAULT FALSE,
    priority_level INTEGER DEFAULT 1 CHECK (priority_level BETWEEN 1 AND 5), -- 1=low, 5=critical
    blockchain_tx_hash VARCHAR(66), -- Ethereum transaction hash
    smart_contract_address VARCHAR(42), -- Contract address for this bounty
    total_staked DECIMAL(20,8) DEFAULT 0,
    participant_count INTEGER DEFAULT 0,
    consensus_threshold DECIMAL(3,2) DEFAULT 0.60, -- 60% consensus required
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Engine participation in bounties (stakes and analyses)
CREATE TABLE bounty_participations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bounty_id UUID NOT NULL REFERENCES bounties(id) ON DELETE CASCADE,
    engine_id UUID NOT NULL REFERENCES engines(id) ON DELETE CASCADE,
    stake_amount DECIMAL(20,8) NOT NULL,
    predicted_verdict VARCHAR(20) NOT NULL CHECK (
        predicted_verdict IN ('malicious', 'benign', 'suspicious', 'unknown')
    ),
    confidence_level DECIMAL(3,2) NOT NULL CHECK (confidence_level BETWEEN 0.00 AND 1.00),
    analysis_data JSONB DEFAULT '{}', -- Detailed analysis results
    stake_tx_hash VARCHAR(66), -- Blockchain transaction for stake
    participation_status VARCHAR(20) DEFAULT 'active' CHECK (
        participation_status IN ('active', 'withdrawn', 'slashed', 'rewarded')
    ),
    is_winner BOOLEAN DEFAULT FALSE,
    reward_earned DECIMAL(20,8) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(bounty_id, engine_id) -- Each engine can only participate once per bounty
);

-- Detailed analysis results from engines
CREATE TABLE analysis_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    participation_id UUID NOT NULL REFERENCES bounty_participations(id) ON DELETE CASCADE,
    engine_id UUID NOT NULL REFERENCES engines(id) ON DELETE CASCADE,
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    verdict VARCHAR(20) NOT NULL CHECK (
        verdict IN ('malicious', 'benign', 'suspicious', 'unknown')
    ),
    confidence_score DECIMAL(5,4) NOT NULL CHECK (confidence_score BETWEEN 0.0000 AND 1.0000),
    threat_types TEXT[], -- Array of detected threat types
    analysis_duration INTEGER, -- Analysis time in seconds
    detailed_report JSONB DEFAULT '{}',
    yara_matches JSONB DEFAULT '[]', -- YARA rule matches
    behavioral_indicators JSONB DEFAULT '{}',
    static_features JSONB DEFAULT '{}',
    network_indicators JSONB DEFAULT '{}',
    error_message TEXT, -- If analysis failed
    analysis_status VARCHAR(20) DEFAULT 'completed' CHECK (
        analysis_status IN ('pending', 'running', 'completed', 'failed', 'timeout')
    ),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Consensus calculations and final verdicts
CREATE TABLE consensus_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bounty_id UUID NOT NULL REFERENCES bounties(id) ON DELETE CASCADE,
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    final_verdict VARCHAR(20) NOT NULL CHECK (
        final_verdict IN ('malicious', 'benign', 'suspicious', 'inconclusive')
    ),
    confidence_score DECIMAL(5,4) NOT NULL,
    malicious_votes INTEGER DEFAULT 0,
    benign_votes INTEGER DEFAULT 0,
    suspicious_votes INTEGER DEFAULT 0,
    unknown_votes INTEGER DEFAULT 0,
    total_participants INTEGER NOT NULL,
    weighted_score DECIMAL(10,8), -- Reputation-weighted consensus
    consensus_algorithm VARCHAR(50) DEFAULT 'majority_vote',
    calculation_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Reward distributions and payouts
CREATE TABLE reward_distributions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bounty_id UUID NOT NULL REFERENCES bounties(id) ON DELETE CASCADE,
    participation_id UUID NOT NULL REFERENCES bounty_participations(id) ON DELETE CASCADE,
    engine_id UUID NOT NULL REFERENCES engines(id) ON DELETE CASCADE,
    reward_type VARCHAR(20) NOT NULL CHECK (
        reward_type IN ('winner_share', 'participation_bonus', 'accuracy_bonus', 'stake_return')
    ),
    amount DECIMAL(20,8) NOT NULL,
    blockchain_tx_hash VARCHAR(66),
    payout_status VARCHAR(20) DEFAULT 'pending' CHECK (
        payout_status IN ('pending', 'processing', 'completed', 'failed')
    ),
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP WITH TIME ZONE
);

-- Bounty tags for categorization
CREATE TABLE bounty_tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    color VARCHAR(7) DEFAULT '#3B82F6' -- Hex color code
);

-- Many-to-many relationship for bounty tags
CREATE TABLE bounty_tag_assignments (
    bounty_id UUID REFERENCES bounties(id) ON DELETE CASCADE,
    tag_id INTEGER REFERENCES bounty_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (bounty_id, tag_id)
);

-- Insert default bounty tags
INSERT INTO bounty_tags (name, description, color) VALUES
('malware', 'General malware analysis', '#EF4444'),
('ransomware', 'Ransomware analysis', '#DC2626'),
('trojan', 'Trojan analysis', '#B91C1C'),
('phishing', 'Phishing attempt analysis', '#F59E0B'),
('apt', 'Advanced Persistent Threat', '#8B5CF6'),
('zero-day', 'Zero-day exploit analysis', '#EC4899'),
('cryptocurrency', 'Crypto-related threats', '#10B981'),
('mobile', 'Mobile malware analysis', '#6366F1'),
('iot', 'IoT security analysis', '#84CC16'),
('urgent', 'High priority analysis', '#FF0000');

-- Indexes for bounty system performance
CREATE INDEX idx_bounties_creator ON bounties(creator_id);
CREATE INDEX idx_bounties_submission ON bounties(submission_id);
CREATE INDEX idx_bounties_status ON bounties(bounty_status);
CREATE INDEX idx_bounties_deadline ON bounties(deadline);
CREATE INDEX idx_bounties_reward ON bounties(reward_amount DESC);
CREATE INDEX idx_bounties_created ON bounties(created_at DESC);

CREATE INDEX idx_participations_bounty ON bounty_participations(bounty_id);
CREATE INDEX idx_participations_engine ON bounty_participations(engine_id);
CREATE INDEX idx_participations_status ON bounty_participations(participation_status);
CREATE INDEX idx_participations_verdict ON bounty_participations(predicted_verdict);

CREATE INDEX idx_analysis_results_participation ON analysis_results(participation_id);
CREATE INDEX idx_analysis_results_engine ON analysis_results(engine_id);
CREATE INDEX idx_analysis_results_submission ON analysis_results(submission_id);
CREATE INDEX idx_analysis_results_verdict ON analysis_results(verdict);
CREATE INDEX idx_analysis_results_status ON analysis_results(analysis_status);

CREATE INDEX idx_consensus_bounty ON consensus_results(bounty_id);
CREATE INDEX idx_consensus_submission ON consensus_results(submission_id);
CREATE INDEX idx_consensus_verdict ON consensus_results(final_verdict);

CREATE INDEX idx_rewards_bounty ON reward_distributions(bounty_id);
CREATE INDEX idx_rewards_engine ON reward_distributions(engine_id);
CREATE INDEX idx_rewards_status ON reward_distributions(payout_status);
CREATE INDEX idx_rewards_type ON reward_distributions(reward_type);

-- Add updated_at triggers
CREATE TRIGGER update_bounties_updated_at BEFORE UPDATE ON bounties FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_participations_updated_at BEFORE UPDATE ON bounty_participations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- View for bounty statistics
CREATE VIEW bounty_stats AS
SELECT 
    b.id,
    b.title,
    b.reward_amount,
    b.bounty_status,
    b.participant_count,
    b.total_staked,
    COUNT(bp.id) as actual_participants,
    AVG(bp.confidence_level) as avg_confidence,
    MAX(ar.created_at) as last_analysis,
    cr.final_verdict,
    cr.confidence_score as consensus_confidence
FROM bounties b
LEFT JOIN bounty_participations bp ON b.id = bp.bounty_id
LEFT JOIN analysis_results ar ON bp.id = ar.participation_id
LEFT JOIN consensus_results cr ON b.id = cr.bounty_id
GROUP BY b.id, b.title, b.reward_amount, b.bounty_status, b.participant_count, 
         b.total_staked, cr.final_verdict, cr.confidence_score;