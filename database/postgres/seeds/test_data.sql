-- Test Data Seeds for Nexus-Security
-- This file populates the database with test data for development and testing
-- DO NOT USE IN PRODUCTION

-- Clean existing test data (for idempotency)
TRUNCATE TABLE governance_votes, governance_proposals, user_stakes, wallet_connections,
               blockchain_transactions, token_balances, smart_contracts,
               reward_distributions, consensus_results, analysis_results, bounty_participations,
               bounty_tag_assignments, bounties, submissions, user_sessions, api_keys,
               reputation_events, reputation_scores, performance_metrics, user_expertise,
               trust_relationships, leaderboards, engines, users CASCADE;

-- Insert test users
INSERT INTO users (id, username, email, password_hash, wallet_address, reputation_score, is_verified, is_active) VALUES
('11111111-1111-1111-1111-111111111111', 'alice_hunter', 'alice@nexus-sec.dev', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5I', '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1', 850, TRUE, TRUE),
('22222222-2222-2222-2222-222222222222', 'bob_analyst', 'bob@nexus-sec.dev', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5I', '0x5A86858aA3b595FD6663c2296741eF4cd8BC4d01', 720, TRUE, TRUE),
('33333333-3333-3333-3333-333333333333', 'charlie_expert', 'charlie@nexus-sec.dev', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5I', '0x1234567890123456789012345678901234567890', 950, TRUE, TRUE),
('44444444-4444-4444-4444-444444444444', 'david_newbie', 'david@nexus-sec.dev', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5I', '0x9876543210987654321098765432109876543210', 350, TRUE, TRUE),
('55555555-5555-5555-5555-555555555555', 'eve_admin', 'eve@nexus-sec.dev', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5I', '0xABCDEF0123456789ABCDEF0123456789ABCDEF01', 1200, TRUE, TRUE);

COMMENT ON COLUMN users.password_hash IS 'Hash of password "TestPassword123!" for all test users';

-- Insert test engines
INSERT INTO engines (id, name, engine_type, description, owner_id, is_active, accuracy_rate, total_analyses, correct_analyses, stake_amount) VALUES
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'VirusTotal Scanner', 'automated', 'Automated malware scanner using VirusTotal API', '11111111-1111-1111-1111-111111111111', TRUE, 0.9200, 1500, 1380, 50000000000000000000),
('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Yara Rule Engine', 'automated', 'YARA-based pattern matching engine', '22222222-2222-2222-2222-222222222222', TRUE, 0.8800, 800, 704, 30000000000000000000),
('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Sandbox Analyzer', 'automated', 'Dynamic analysis sandbox', '33333333-3333-3333-3333-333333333333', TRUE, 0.9500, 600, 570, 75000000000000000000),
('dddddddd-dddd-dddd-dddd-dddddddddddd', 'Expert Manual Review', 'human', 'Human expert manual analysis', '33333333-3333-3333-3333-333333333333', TRUE, 0.9800, 250, 245, 100000000000000000000),
('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'ML Classifier', 'automated', 'Machine learning-based classifier', '55555555-5555-5555-5555-555555555555', TRUE, 0.9100, 2000, 1820, 40000000000000000000);

-- Insert test submissions
INSERT INTO submissions (id, submitter_id, file_hash, original_filename, file_size, mime_type, category_id, file_path, submission_type, is_malicious, confidence_score, analysis_status) VALUES
('sub11111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', '8f14e45fceea167a5a36dedd4bea2543a5a36dedd4bea2543a5a36dedd4bea25', 'suspicious.exe', 245760, 'application/x-executable', 1, '/storage/8f14e45fceea167a5a36dedd4bea2543', 'file', TRUE, 0.9500, 'completed'),
('sub22222-2222-2222-2222-222222222222', '22222222-2222-2222-2222-222222222222', 'a1b2c3d4e5f6789012345678901234567890123456789012345678901234abcd', 'document.pdf', 102400, 'application/pdf', 2, '/storage/a1b2c3d4e5f6789012345678901234567', 'file', FALSE, 0.9200, 'completed'),
('sub33333-3333-3333-3333-333333333333', '44444444-4444-4444-4444-444444444444', NULL, NULL, NULL, NULL, 6, NULL, 'url', TRUE, 0.8800, 'completed'),
('sub44444-4444-4444-4444-444444444444', '11111111-1111-1111-1111-111111111111', 'deadbeef1234567890abcdef1234567890abcdef1234567890abcdef12345678', 'malware.dll', 524288, 'application/x-msdownload', 1, '/storage/deadbeef1234567890abcdef12345678', 'file', TRUE, 0.9900, 'completed');

-- Update URLs for URL submissions
UPDATE submissions SET url = 'http://malicious-phishing-site.example.com/login' WHERE id = 'sub33333-3333-3333-3333-333333333333';

-- Insert test bounties
INSERT INTO bounties (id, creator_id, submission_id, title, description, reward_amount, min_stake_amount, deadline, bounty_status, priority_level, total_staked, participant_count, consensus_threshold) VALUES
('bounty11-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'sub11111-1111-1111-1111-111111111111', 'Analyze Suspicious Executable', 'Need analysis of potentially malicious executable file', 100000000000000000000, 10000000000000000000, CURRENT_TIMESTAMP + INTERVAL '7 days', 'active', 3, 80000000000000000000, 4, 0.66),
('bounty22-2222-2222-2222-222222222222', '22222222-2222-2222-2222-222222222222', 'sub22222-2222-2222-2222-222222222222', 'PDF Document Malware Check', 'Check if PDF contains embedded malware', 50000000000000000000, 5000000000000000000, CURRENT_TIMESTAMP + INTERVAL '5 days', 'completed', 2, 40000000000000000000, 3, 0.66),
('bounty33-3333-3333-3333-333333333333', '44444444-4444-4444-4444-444444444444', 'sub33333-3333-3333-3333-333333333333', 'Phishing URL Analysis', 'Analyze suspected phishing website', 75000000000000000000, 7500000000000000000, CURRENT_TIMESTAMP + INTERVAL '3 days', 'completed', 4, 60000000000000000000, 5, 0.66),
('bounty44-4444-4444-4444-444444444444', '11111111-1111-1111-1111-111111111111', 'sub44444-4444-4444-4444-444444444444', 'Critical DLL Analysis', 'URGENT: Suspected ransomware DLL', 200000000000000000000, 20000000000000000000, CURRENT_TIMESTAMP + INTERVAL '2 days', 'active', 5, 150000000000000000000, 6, 0.75);

-- Insert bounty participations
INSERT INTO bounty_participations (id, bounty_id, engine_id, stake_amount, predicted_verdict, confidence_level, participation_status, is_winner, reward_earned) VALUES
('part1111-1111-1111-1111-111111111111', 'bounty11-1111-1111-1111-111111111111', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 20000000000000000000, 'malicious', 0.92, 'active', FALSE, 0),
('part2222-2222-2222-2222-222222222222', 'bounty11-1111-1111-1111-111111111111', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 15000000000000000000, 'malicious', 0.88, 'active', FALSE, 0),
('part3333-3333-3333-3333-333333333333', 'bounty11-1111-1111-1111-111111111111', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 25000000000000000000, 'malicious', 0.95, 'active', FALSE, 0),
('part4444-4444-4444-4444-444444444444', 'bounty11-1111-1111-1111-111111111111', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 20000000000000000000, 'malicious', 0.98, 'active', FALSE, 0),
('part5555-5555-5555-5555-555555555555', 'bounty22-2222-2222-2222-222222222222', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 15000000000000000000, 'benign', 0.94, 'rewarded', TRUE, 20000000000000000000),
('part6666-6666-6666-6666-666666666666', 'bounty22-2222-2222-2222-222222222222', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 12000000000000000000, 'benign', 0.91, 'rewarded', TRUE, 15000000000000000000),
('part7777-7777-7777-7777-777777777777', 'bounty22-2222-2222-2222-222222222222', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 13000000000000000000, 'benign', 0.89, 'rewarded', TRUE, 15000000000000000000);

-- Insert analysis results
INSERT INTO analysis_results (id, participation_id, engine_id, submission_id, verdict, confidence_score, analysis_duration, analysis_status, completed_at) VALUES
('analy111-1111-1111-1111-111111111111', 'part1111-1111-1111-1111-111111111111', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'sub11111-1111-1111-1111-111111111111', 'malicious', 0.9200, 45, 'completed', CURRENT_TIMESTAMP - INTERVAL '2 hours'),
('analy222-2222-2222-2222-222222222222', 'part2222-2222-2222-2222-222222222222', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'sub11111-1111-1111-1111-111111111111', 'malicious', 0.8800, 120, 'completed', CURRENT_TIMESTAMP - INTERVAL '1 hour'),
('analy333-3333-3333-3333-333333333333', 'part3333-3333-3333-3333-333333333333', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'sub11111-1111-1111-1111-111111111111', 'malicious', 0.9500, 180, 'completed', CURRENT_TIMESTAMP - INTERVAL '30 minutes'),
('analy444-4444-4444-4444-444444444444', 'part4444-4444-4444-4444-444444444444', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 'sub11111-1111-1111-1111-111111111111', 'malicious', 0.9800, 300, 'completed', CURRENT_TIMESTAMP - INTERVAL '15 minutes'),
('analy555-5555-5555-5555-555555555555', 'part5555-5555-5555-5555-555555555555', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'sub22222-2222-2222-2222-222222222222', 'benign', 0.9400, 30, 'completed', CURRENT_TIMESTAMP - INTERVAL '4 hours'),
('analy666-6666-6666-6666-666666666666', 'part6666-6666-6666-6666-666666666666', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'sub22222-2222-2222-2222-222222222222', 'benign', 0.9100, 60, 'completed', CURRENT_TIMESTAMP - INTERVAL '3 hours'),
('analy777-7777-7777-7777-777777777777', 'part7777-7777-7777-7777-777777777777', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'sub22222-2222-2222-2222-222222222222', 'benign', 0.8900, 90, 'completed', CURRENT_TIMESTAMP - INTERVAL '2 hours');

-- Add threat types to analysis results
UPDATE analysis_results SET threat_types = ARRAY['trojan', 'keylogger', 'c2_communication'] WHERE id = 'analy111-1111-1111-1111-111111111111';
UPDATE analysis_results SET threat_types = ARRAY['trojan', 'info_stealer'] WHERE id = 'analy222-2222-2222-2222-222222222222';
UPDATE analysis_results SET threat_types = ARRAY['trojan', 'ransomware_characteristics'] WHERE id = 'analy333-3333-3333-3333-333333333333';
UPDATE analysis_results SET threat_types = ARRAY['trojan', 'data_exfiltration'] WHERE id = 'analy444-4444-4444-4444-444444444444';

-- Insert consensus results
INSERT INTO consensus_results (id, bounty_id, submission_id, final_verdict, confidence_score, malicious_votes, benign_votes, total_participants, weighted_score) VALUES
('conse111-1111-1111-1111-111111111111', 'bounty22-2222-2222-2222-222222222222', 'sub22222-2222-2222-2222-222222222222', 'benign', 0.9133, 0, 3, 3, 0.9200),
('conse222-2222-2222-2222-222222222222', 'bounty33-3333-3333-3333-333333333333', 'sub33333-3333-3333-3333-333333333333', 'malicious', 0.9400, 5, 0, 5, 0.9500);

-- Insert reward distributions
INSERT INTO reward_distributions (id, bounty_id, participation_id, engine_id, reward_type, amount, payout_status, processed_at) VALUES
('reward11-1111-1111-1111-111111111111', 'bounty22-2222-2222-2222-222222222222', 'part5555-5555-5555-5555-555555555555', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'winner_share', 20000000000000000000, 'completed', CURRENT_TIMESTAMP - INTERVAL '1 hour'),
('reward22-2222-2222-2222-222222222222', 'bounty22-2222-2222-2222-222222222222', 'part6666-6666-6666-6666-666666666666', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'winner_share', 15000000000000000000, 'completed', CURRENT_TIMESTAMP - INTERVAL '1 hour'),
('reward33-3333-3333-3333-333333333333', 'bounty22-2222-2222-2222-222222222222', 'part7777-7777-7777-7777-777777777777', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'winner_share', 15000000000000000000, 'completed', CURRENT_TIMESTAMP - INTERVAL '1 hour');

-- Assign tags to bounties
INSERT INTO bounty_tag_assignments (bounty_id, tag_id) VALUES
('bounty11-1111-1111-1111-111111111111', 1), -- malware
('bounty11-1111-1111-1111-111111111111', 3), -- trojan
('bounty22-2222-2222-2222-222222222222', 1), -- malware
('bounty33-3333-3333-3333-333333333333', 4), -- phishing
('bounty33-3333-3333-3333-333333333333', 10), -- urgent
('bounty44-4444-4444-4444-444444444444', 2), -- ransomware
('bounty44-4444-4444-4444-444444444444', 10); -- urgent

-- Insert API keys
INSERT INTO api_keys (user_id, key_hash, name, permissions, rate_limit, is_active) VALUES
('11111111-1111-1111-1111-111111111111', '$2b$12$API_KEY_HASH_ALICE_1234567890ABCDEF', 'Alice Production Key', ARRAY['read', 'write'], 5000, TRUE),
('22222222-2222-2222-2222-222222222222', '$2b$12$API_KEY_HASH_BOB_1234567890ABCDEF01', 'Bob Development Key', ARRAY['read'], 1000, TRUE),
('55555555-5555-5555-5555-555555555555', '$2b$12$API_KEY_HASH_EVE_ADMIN_1234567890ABC', 'Eve Admin Key', ARRAY['read', 'write', 'admin'], 10000, TRUE);

-- Insert reputation scores
INSERT INTO reputation_scores (user_id, engine_id, accuracy_score, speed_score, consistency_score, expertise_score, community_score) VALUES
(NULL, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 920, 85, 90, 75, 50),
(NULL, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 880, 70, 80, 85, 45),
(NULL, 'cccccccc-cccc-cccc-cccc-cccccccccccc', 950, 60, 95, 90, 55),
(NULL, 'dddddddd-dddd-dddd-dddd-dddddddddddd', 980, 40, 75, 100, 80),
(NULL, 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 910, 95, 85, 70, 60);

-- Insert performance metrics for all_time period
INSERT INTO performance_metrics (
    engine_id, time_period, period_start, period_end,
    total_analyses, correct_analyses, false_positives, false_negatives,
    accuracy_rate, avg_analysis_time, total_earnings, bounties_participated, bounties_won
) VALUES
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'all_time', '2023-01-01', CURRENT_DATE, 1500, 1380, 85, 35, 0.9200, 45, 500000000000000000000, 450, 320),
('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'all_time', '2023-01-01', CURRENT_DATE, 800, 704, 50, 46, 0.8800, 105, 250000000000000000000, 250, 175),
('cccccccc-cccc-cccc-cccc-cccccccccccc', 'all_time', '2023-01-01', CURRENT_DATE, 600, 570, 15, 15, 0.9500, 165, 450000000000000000000, 200, 180),
('dddddddd-dddd-dddd-dddd-dddddddddddd', 'all_time', '2023-01-01', CURRENT_DATE, 250, 245, 3, 2, 0.9800, 270, 600000000000000000000, 100, 95),
('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'all_time', '2023-01-01', CURRENT_DATE, 2000, 1820, 120, 60, 0.9100, 52, 800000000000000000000, 600, 520);

-- Insert user expertise
INSERT INTO user_expertise (engine_id, expertise_area_id, proficiency_level, experience_points, specialization_score) VALUES
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 1, 4, 1500, 850), -- malware_analysis
('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 8, 5, 800, 920),  -- reverse_engineering
('cccccccc-cccc-cccc-cccc-cccccccccccc', 1, 5, 600, 950),  -- malware_analysis
('dddddddd-dddd-dddd-dddd-dddddddddddd', 6, 5, 250, 980),  -- forensics
('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 7, 4, 2000, 910); -- threat_intelligence

-- Insert trust relationships
INSERT INTO trust_relationships (trustor_id, trustee_id, trust_level, trust_type, confidence) VALUES
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 85, 'derived', 90),
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 95, 'derived', 95),
('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 90, 'derived', 85),
('cccccccc-cccc-cccc-cccc-cccccccccccc', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 98, 'derived', 98);

-- Insert leaderboards
INSERT INTO leaderboards (category, time_period, engine_id, score, rank_position, period_start, period_end) VALUES
('accuracy', 'all_time', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 0.9800, 1, '2023-01-01', CURRENT_DATE),
('accuracy', 'all_time', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 0.9500, 2, '2023-01-01', CURRENT_DATE),
('accuracy', 'all_time', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 0.9200, 3, '2023-01-01', CURRENT_DATE),
('speed', 'all_time', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 45.0000, 1, '2023-01-01', CURRENT_DATE),
('speed', 'all_time', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 52.0000, 2, '2023-01-01', CURRENT_DATE),
('earnings', 'all_time', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 800000000000000000000, 1, '2023-01-01', CURRENT_DATE),
('earnings', 'all_time', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 600000000000000000000, 2, '2023-01-01', CURRENT_DATE);

-- Insert blockchain networks data (additional testnets for development)
INSERT INTO blockchain_networks (network_name, chain_id, rpc_url, block_explorer_url, native_currency_symbol, is_testnet, is_active) VALUES
('Hardhat Local', 1337, 'http://localhost:8545', 'http://localhost:8545', 'ETH', TRUE, TRUE),
('Ganache Local', 1337, 'http://localhost:7545', 'http://localhost:7545', 'ETH', TRUE, FALSE);

-- Insert smart contract deployments (for local development)
INSERT INTO smart_contracts (contract_name, contract_address, network_id, contract_type, abi_json, deployment_tx_hash, deployer_address, is_verified, version) VALUES
('BountyManager', '0x5FbDB2315678afecb367f032d93F642f64180aa3', 6, 'bounty_manager', '[]'::jsonb, '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef', '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266', TRUE, '1.0.0'),
('ThreatToken', '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512', 6, 'threat_token', '[]'::jsonb, '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890', '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266', TRUE, '1.0.0'),
('ReputationSystem', '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0', 6, 'reputation_system', '[]'::jsonb, '0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321', '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266', TRUE, '1.0.0');

COMMENT ON TABLE smart_contracts IS 'Smart contract addresses above are Hardhat default deployment addresses for local testing';

-- Print summary
DO $$
DECLARE
    user_count INT;
    engine_count INT;
    bounty_count INT;
    submission_count INT;
BEGIN
    SELECT COUNT(*) INTO user_count FROM users;
    SELECT COUNT(*) INTO engine_count FROM engines;
    SELECT COUNT(*) INTO bounty_count FROM bounties;
    SELECT COUNT(*) INTO submission_count FROM submissions;

    RAISE NOTICE '========================================';
    RAISE NOTICE 'Test Data Seeding Complete!';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Users created: %', user_count;
    RAISE NOTICE 'Engines created: %', engine_count;
    RAISE NOTICE 'Submissions created: %', submission_count;
    RAISE NOTICE 'Bounties created: %', bounty_count;
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Test Users (password: TestPassword123!):';
    RAISE NOTICE '  - alice_hunter (alice@nexus-sec.dev)';
    RAISE NOTICE '  - bob_analyst (bob@nexus-sec.dev)';
    RAISE NOTICE '  - charlie_expert (charlie@nexus-sec.dev)';
    RAISE NOTICE '  - david_newbie (david@nexus-sec.dev)';
    RAISE NOTICE '  - eve_admin (eve@nexus-sec.dev)';
    RAISE NOTICE '========================================';
END $$;
