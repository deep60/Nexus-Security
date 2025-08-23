-- user_engine.sql - Core Foundation

-- Users & Authentication: User accounts, sessions, API keys
-- Engines: Both automated and human expert analyzers
-- File Categories: Supported file types and limits
-- Submissions: Files/URLs submitted for analysis
-- Indexes & Triggers: Performance optimization and automatic timestamp updates

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users table for platform participants
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    wallet_address VARCHAR(42) UNIQUE, -- Ethereum address
    reputation_score INTEGER DEFAULT 0,
    total_submissions INTEGER DEFAULT 0,
    successful_submissions INTEGER DEFAULT 0,
    is_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Engines table (both automated and human experts)
CREATE TABLE engines (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    engine_type VARCHAR(20) NOT NULL CHECK (engine_type IN ('automated', 'human', 'hybrid')),
    description TEXT,
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    api_endpoint VARCHAR(255), -- For automated engines
    is_active BOOLEAN DEFAULT TRUE,
    accuracy_rate DECIMAL(5,4) DEFAULT 0.0000, -- 0.0000 to 1.0000
    total_analyses INTEGER DEFAULT 0,
    correct_analyses INTEGER DEFAULT 0,
    stake_amount DECIMAL(20,8) DEFAULT 0, -- Amount staked in wei
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- File types and categories
CREATE TABLE file_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    mime_types TEXT[], -- Array of supported MIME types
    max_file_size BIGINT DEFAULT 104857600 -- 100MB default
);

-- Insert common file categories
INSERT INTO file_categories (name, description, mime_types, max_file_size) VALUES
('executable', 'Executable files', ARRAY['application/x-executable', 'application/x-msdos-program', 'application/x-msdownload'], 104857600),
('document', 'Document files', ARRAY['application/pdf', 'application/msword', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'], 52428800),
('archive', 'Archive files', ARRAY['application/zip', 'application/x-rar-compressed', 'application/x-7z-compressed'], 209715200),
('script', 'Script files', ARRAY['text/javascript', 'application/x-python-code', 'text/x-shellscript'], 10485760),
('image', 'Image files', ARRAY['image/jpeg', 'image/png', 'image/gif', 'image/bmp'], 20971520),
('url', 'URL/Website analysis', ARRAY['text/uri-list'], 0);

-- Submissions table for files/URLs to be analyzed
CREATE TABLE submissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    submitter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_hash VARCHAR(64) UNIQUE, -- SHA-256 hash for files
    url TEXT, -- For URL submissions
    original_filename VARCHAR(255),
    file_size BIGINT,
    mime_type VARCHAR(100),
    category_id INTEGER REFERENCES file_categories(id),
    file_path TEXT, -- Storage path
    submission_type VARCHAR(10) NOT NULL CHECK (submission_type IN ('file', 'url')),
    is_malicious BOOLEAN, -- Final consensus result
    confidence_score DECIMAL(5,4), -- 0.0000 to 1.0000
    analysis_status VARCHAR(20) DEFAULT 'pending' CHECK (analysis_status IN ('pending', 'analyzing', 'completed', 'failed')),
    metadata JSONB DEFAULT '{}', -- Additional file metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Sessions table for authentication
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    ip_address INET,
    user_agent TEXT
);

-- API keys for external integrations
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    permissions TEXT[] DEFAULT ARRAY['read'], -- read, write, admin
    rate_limit INTEGER DEFAULT 1000, -- requests per hour
    is_active BOOLEAN DEFAULT TRUE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for performance
CREATE INDEX idx_users_wallet_address ON users(wallet_address);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);

CREATE INDEX idx_engines_type ON engines(engine_type);
CREATE INDEX idx_engines_active ON engines(is_active);
CREATE INDEX idx_engines_accuracy ON engines(accuracy_rate DESC);

CREATE INDEX idx_submissions_hash ON submissions(file_hash);
CREATE INDEX idx_submissions_submitter ON submissions(submitter_id);
CREATE INDEX idx_submissions_status ON submissions(analysis_status);
CREATE INDEX idx_submissions_created ON submissions(created_at);
CREATE INDEX idx_submissions_type ON submissions(submission_type);

CREATE INDEX idx_sessions_user ON user_sessions(user_id);
CREATE INDEX idx_sessions_token ON user_sessions(token_hash);
CREATE INDEX idx_sessions_expires ON user_sessions(expires_at);

CREATE INDEX idx_api_keys_user ON api_keys(user_id);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_active ON api_keys(is_active);

-- Triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_engines_updated_at BEFORE UPDATE ON engines FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_submissions_updated_at BEFORE UPDATE ON submissions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();