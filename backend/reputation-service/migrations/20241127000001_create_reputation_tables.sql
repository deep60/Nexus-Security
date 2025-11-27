-- Create user_reputation table
CREATE TABLE IF NOT EXISTS user_reputation (
    user_id UUID PRIMARY KEY,
    current_score INTEGER NOT NULL DEFAULT 0,
    highest_score INTEGER NOT NULL DEFAULT 0,
    lowest_score INTEGER NOT NULL DEFAULT 0,
    total_submissions INTEGER NOT NULL DEFAULT 0,
    correct_submissions INTEGER NOT NULL DEFAULT 0,
    incorrect_submissions INTEGER NOT NULL DEFAULT 0,
    accuracy_rate DECIMAL(5, 2) NOT NULL DEFAULT 0.00,
    current_streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    total_earned DECIMAL(20, 8) NOT NULL DEFAULT 0,
    rank INTEGER,
    percentile DECIMAL(5, 2),
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create reputation_history table
CREATE TABLE IF NOT EXISTS reputation_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    score_before INTEGER NOT NULL,
    score_after INTEGER NOT NULL,
    score_change INTEGER NOT NULL,
    reason TEXT NOT NULL,
    bounty_id UUID,
    submission_id UUID,
    details JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES user_reputation(user_id) ON DELETE CASCADE
);

-- Create badges table
CREATE TABLE IF NOT EXISTS badges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    icon VARCHAR(255) NOT NULL,
    rarity VARCHAR(50) NOT NULL CHECK (rarity IN ('common', 'uncommon', 'rare', 'epic', 'legendary')),
    min_score INTEGER,
    min_accuracy DECIMAL(5, 2),
    min_submissions INTEGER,
    min_streak INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create user_badges table
CREATE TABLE IF NOT EXISTS user_badges (
    user_id UUID NOT NULL,
    badge_id UUID NOT NULL,
    awarded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    progress DECIMAL(5, 2),
    PRIMARY KEY (user_id, badge_id),
    FOREIGN KEY (user_id) REFERENCES user_reputation(user_id) ON DELETE CASCADE,
    FOREIGN KEY (badge_id) REFERENCES badges(id) ON DELETE CASCADE
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_reputation_history_user_id ON reputation_history(user_id);
CREATE INDEX IF NOT EXISTS idx_reputation_history_created_at ON reputation_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_reputation_score ON user_reputation(current_score DESC);
CREATE INDEX IF NOT EXISTS idx_user_reputation_rank ON user_reputation(rank);
CREATE INDEX IF NOT EXISTS idx_user_badges_user_id ON user_badges(user_id);
CREATE INDEX IF NOT EXISTS idx_user_badges_badge_id ON user_badges(badge_id);

-- Insert some default badges
INSERT INTO badges (name, description, icon, rarity, min_submissions)
VALUES
    ('First Steps', 'Submitted your first analysis', '🎯', 'common', 1),
    ('Persistent Analyst', 'Submitted 10 analyses', '💪', 'common', 10),
    ('Dedicated Researcher', 'Submitted 50 analyses', '🔍', 'uncommon', 50),
    ('Expert Analyst', 'Submitted 100 analyses', '⭐', 'rare', 100),
    ('Master Researcher', 'Submitted 500 analyses', '👑', 'epic', 500),
    ('Legend', 'Submitted 1000 analyses', '🏆', 'legendary', 1000)
ON CONFLICT (name) DO NOTHING;

INSERT INTO badges (name, description, icon, rarity, min_accuracy)
VALUES
    ('Sharp Eye', 'Achieved 80% accuracy', '👁️', 'uncommon', 80.00),
    ('Precision Expert', 'Achieved 90% accuracy', '🎯', 'rare', 90.00),
    ('Perfect Vision', 'Achieved 95% accuracy', '💎', 'epic', 95.00),
    ('Oracle', 'Achieved 99% accuracy', '✨', 'legendary', 99.00)
ON CONFLICT (name) DO NOTHING;

INSERT INTO badges (name, description, icon, rarity, min_streak)
VALUES
    ('On Fire', '5 correct analyses in a row', '🔥', 'uncommon', 5),
    ('Unstoppable', '10 correct analyses in a row', '⚡', 'rare', 10),
    ('Untouchable', '20 correct analyses in a row', '💫', 'epic', 20),
    ('Flawless', '50 correct analyses in a row', '🌟', 'legendary', 50)
ON CONFLICT (name) DO NOTHING;

INSERT INTO badges (name, description, icon, rarity, min_score)
VALUES
    ('Rising Star', 'Reached reputation score of 100', '🌠', 'common', 100),
    ('Respected Analyst', 'Reached reputation score of 500', '🎖️', 'uncommon', 500),
    ('Renowned Expert', 'Reached reputation score of 1000', '🏅', 'rare', 1000),
    ('Elite Researcher', 'Reached reputation score of 5000', '👑', 'epic', 5000),
    ('Hall of Fame', 'Reached reputation score of 10000', '🏆', 'legendary', 10000)
ON CONFLICT (name) DO NOTHING;
