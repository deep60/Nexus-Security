-- blockchain.sql - Web3 Features

-- Blockchain Networks: Multi-chain support (Ethereum, Polygon, BSC)
-- Smart Contracts: Contract deployment tracking
-- Transactions: On-chain transaction monitoring
-- Token Balances: ERC-20 token management
-- Staking Pools: Various staking mechanisms
-- Wallet Connections: Web3 wallet integration
-- Oracle Feeds: External data integration
-- Governance: DAO voting system

-- Multi-network support (mainnet/testnet)
-- Transaction tracking and verification
-- Stake management with slashing protection
-- Governance system with reputation-weighted voting
-- Oracle integration for external data feeds

-- Blockchain networks and contract addresses
CREATE TABLE blockchain_networks (
    id SERIAL PRIMARY KEY,
    network_name VARCHAR(50) UNIQUE NOT NULL,
    chain_id INTEGER UNIQUE NOT NULL,
    rpc_url VARCHAR(255) NOT NULL,
    block_explorer_url VARCHAR(255),
    native_currency_symbol VARCHAR(10) NOT NULL,
    native_currency_decimals INTEGER DEFAULT 18,
    is_testnet BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    gas_price_gwei DECIMAL(10,2) DEFAULT 20.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Smart contract deployments
CREATE TABLE smart_contracts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_name VARCHAR(100) NOT NULL,
    contract_address VARCHAR(42) NOT NULL,
    network_id INTEGER NOT NULL REFERENCES blockchain_networks(id),
    contract_type VARCHAR(30) NOT NULL CHECK (
        contract_type IN ('bounty_manager', 'threat_token', 'reputation_system', 'governance')
    ),
    abi_json JSONB NOT NULL,
    deployment_tx_hash VARCHAR(66) NOT NULL,
    deployment_block BIGINT,
    deployer_address VARCHAR(42) NOT NULL,
    is_verified BOOLEAN DEFAULT FALSE,
    version VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(contract_address, network_id)
);

-- Blockchain transactions tracking
CREATE TABLE blockchain_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tx_hash VARCHAR(66) UNIQUE NOT NULL,
    network_id INTEGER NOT NULL REFERENCES blockchain_networks(id),
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42) NOT NULL,
    value DECIMAL(30,18) DEFAULT 0, -- Value in native currency (ETH, etc.)
    gas_used BIGINT,
    gas_price BIGINT, -- In wei
    block_number BIGINT,
    block_timestamp TIMESTAMP WITH TIME ZONE,
    transaction_type VARCHAR(30) NOT NULL CHECK (
        transaction_type IN (
            'stake_deposit', 'stake_withdrawal', 'reward_payout', 'bounty_creation',
            'bounty_participation', 'token_transfer', 'reputation_update', 'governance_vote'
        )
    ),
    status VARCHAR(20) DEFAULT 'pending' CHECK (
        status IN ('pending', 'confirmed', 'failed', 'cancelled')
    ),
    error_message TEXT,
    related_bounty_id UUID REFERENCES bounties(id) ON DELETE SET NULL,
    related_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    confirmed_at TIMESTAMP WITH TIME ZONE
);

-- Token balances and transfers
CREATE TABLE token_balances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    network_id INTEGER NOT NULL REFERENCES blockchain_networks(id),
    token_address VARCHAR(42) NOT NULL, -- Contract address for ERC-20 tokens
    token_symbol VARCHAR(10) NOT NULL,
    token_name VARCHAR(50) NOT NULL,
    balance DECIMAL(30,18) NOT NULL DEFAULT 0,
    locked_balance DECIMAL(30,18) NOT NULL DEFAULT 0, -- Staked or locked tokens
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(user_id, network_id, token_address)
);

-- Staking pools and liquidity provision
CREATE TABLE staking_pools (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pool_name VARCHAR(100) NOT NULL,
    contract_address VARCHAR(42) NOT NULL,
    network_id INTEGER NOT NULL REFERENCES blockchain_networks(id),
    pool_type VARCHAR(20) NOT NULL CHECK (
        pool_type IN ('bounty_staking', 'liquidity_provision', 'governance_staking', 'reputation_staking')
    ),
    token_address VARCHAR(42) NOT NULL,
    total_staked DECIMAL(30,18) DEFAULT 0,
    reward_rate DECIMAL(8,6) DEFAULT 0, -- Annual percentage rate
    min_stake_amount DECIMAL(20,8) NOT NULL,
    lock_period_days INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- User stakes in various pools
CREATE TABLE user_stakes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pool_id UUID NOT NULL REFERENCES staking_pools(id) ON DELETE CASCADE,
    staked_amount DECIMAL(30,18) NOT NULL,
    reward_earned DECIMAL(30,18) DEFAULT 0,
    stake_tx_hash VARCHAR(66),
    unstake_tx_hash VARCHAR(66),
    staked_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    lock_expires_at TIMESTAMP WITH TIME ZONE,
    unstaked_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT TRUE,
    
    UNIQUE(user_id, pool_id) DEFERRABLE INITIALLY DEFERRED
);

-- Web3 authentication and wallet connections
CREATE TABLE wallet_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallet_address VARCHAR(42) NOT NULL,
    wallet_type VARCHAR(20) NOT NULL CHECK (
        wallet_type IN ('metamask', 'walletconnect', 'coinbase', 'ledger', 'trezor', 'other')
    ),
    network_id INTEGER NOT NULL REFERENCES blockchain_networks(id),
    is_primary BOOLEAN DEFAULT FALSE,
    signature_message TEXT,
    signature VARCHAR(132), -- Ethereum signature
    nonce VARCHAR(64) NOT NULL,
    verified_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(user_id, wallet_address, network_id)
);

-- Oracle data feeds for external price/data
CREATE TABLE oracle_feeds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    feed_name VARCHAR(100) NOT NULL,
    oracle_address VARCHAR(42) NOT NULL,
    network_id INTEGER NOT NULL REFERENCES blockchain_networks(id),
    data_type VARCHAR(30) NOT NULL CHECK (
        data_type IN ('price_feed', 'threat_score', 'reputation_data', 'market_data')
    ),
    latest_value DECIMAL(20,8),
    latest_timestamp TIMESTAMP WITH TIME ZONE,
    update_frequency_seconds INTEGER DEFAULT 3600, -- 1 hour default
    is_active BOOLEAN DEFAULT TRUE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Governance proposals and voting
CREATE TABLE governance_proposals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    proposal_id BIGINT NOT NULL, -- On-chain proposal ID
    proposer_id UUID NOT NULL REFERENCES users(id),
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    proposal_type VARCHAR(30) NOT NULL CHECK (
        proposal_type IN ('parameter_change', 'upgrade', 'treasury', 'feature_request', 'emergency')
    ),
    voting_start TIMESTAMP WITH TIME ZONE NOT NULL,
    voting_end TIMESTAMP WITH TIME ZONE NOT NULL,
    execution_deadline TIMESTAMP WITH TIME ZONE,
    min_quorum DECIMAL(5,2) NOT NULL, -- Minimum participation percentage
    approval_threshold DECIMAL(5,2) NOT NULL, -- Required approval percentage
    total_votes_for DECIMAL(30,18) DEFAULT 0,
    total_votes_against DECIMAL(30,18) DEFAULT 0,
    total_votes_abstain DECIMAL(30,18) DEFAULT 0,
    proposal_status VARCHAR(20) DEFAULT 'pending' CHECK (
        proposal_status IN ('pending', 'active', 'succeeded', 'failed', 'executed', 'cancelled')
    ),
    execution_tx_hash VARCHAR(66),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Individual votes on governance proposals
CREATE TABLE governance_votes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    proposal_id UUID NOT NULL REFERENCES governance_proposals(id) ON DELETE CASCADE,
    voter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vote_choice VARCHAR(10) NOT NULL CHECK (vote_choice IN ('for', 'against', 'abstain')),
    voting_power DECIMAL(30,18) NOT NULL,
    vote_tx_hash VARCHAR(66),
    voted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(proposal_id, voter_id)
);

-- Insert default blockchain networks
INSERT INTO blockchain_networks (network_name, chain_id, rpc_url, block_explorer_url, native_currency_symbol, is_testnet) VALUES
('Ethereum Mainnet', 1, 'https://mainnet.infura.io/v3/', 'https://etherscan.io', 'ETH', FALSE),
('Polygon', 137, 'https://polygon-rpc.com/', 'https://polygonscan.com', 'MATIC', FALSE),
('BSC Mainnet', 56, 'https://bsc-dataseed.binance.org/', 'https://bscscan.com', 'BNB', FALSE),
('Ethereum Sepolia', 11155111, 'https://sepolia.infura.io/v3/', 'https://sepolia.etherscan.io', 'ETH', TRUE),
('Polygon Mumbai', 80001, 'https://rpc-mumbai.maticvigil.com/', 'https://mumbai.polygonscan.com', 'MATIC', TRUE);

-- Indexes for blockchain tables
CREATE INDEX idx_smart_contracts_address ON smart_contracts(contract_address);
CREATE INDEX idx_smart_contracts_network ON smart_contracts(network_id);
CREATE INDEX idx_smart_contracts_type ON smart_contracts(contract_type);

CREATE INDEX idx_blockchain_transactions_hash ON blockchain_transactions(tx_hash);
CREATE INDEX idx_blockchain_transactions_network ON blockchain_transactions(network_id);
CREATE INDEX idx_blockchain_transactions_type ON blockchain_transactions(transaction_type);
CREATE INDEX idx_blockchain_transactions_status ON blockchain_transactions(status);
CREATE INDEX idx_blockchain_transactions_block ON blockchain_transactions(block_number);
CREATE INDEX idx_blockchain_transactions_timestamp ON blockchain_transactions(block_timestamp);
CREATE INDEX idx_blockchain_transactions_bounty ON blockchain_transactions(related_bounty_id);
CREATE INDEX idx_blockchain_transactions_user ON blockchain_transactions(related_user_id);

CREATE INDEX idx_token_balances_user ON token_balances(user_id);
CREATE INDEX idx_token_balances_network ON token_balances(network_id);
CREATE INDEX idx_token_balances_token ON token_balances(token_address);
CREATE INDEX idx_token_balances_symbol ON token_balances(token_symbol);

CREATE INDEX idx_staking_pools_network ON staking_pools(network_id);
CREATE INDEX idx_staking_pools_type ON staking_pools(pool_type);
CREATE INDEX idx_staking_pools_token ON staking_pools(token_address);
CREATE INDEX idx_staking_pools_active ON staking_pools(is_active);

CREATE INDEX idx_user_stakes_user ON user_stakes(user_id);
CREATE INDEX idx_user_stakes_pool ON user_stakes(pool_id);
CREATE INDEX idx_user_stakes_active ON user_stakes(is_active);
CREATE INDEX idx_user_stakes_expires ON user_stakes(lock_expires_at);

CREATE INDEX idx_wallet_connections_user ON wallet_connections(user_id);
CREATE INDEX idx_wallet_connections_address ON wallet_connections(wallet_address);
CREATE INDEX idx_wallet_connections_network ON wallet_connections(network_id);
CREATE INDEX idx_wallet_connections_type ON wallet_connections(wallet_type);
CREATE INDEX idx_wallet_connections_primary ON wallet_connections(is_primary);

CREATE INDEX idx_oracle_feeds_network ON oracle_feeds(network_id);
CREATE INDEX idx_oracle_feeds_type ON oracle_feeds(data_type);
CREATE INDEX idx_oracle_feeds_active ON oracle_feeds(is_active);
CREATE INDEX idx_oracle_feeds_timestamp ON oracle_feeds(latest_timestamp);

CREATE INDEX idx_governance_proposals_proposer ON governance_proposals(proposer_id);
CREATE INDEX idx_governance_proposals_status ON governance_proposals(proposal_status);
CREATE INDEX idx_governance_proposals_voting_period ON governance_proposals(voting_start, voting_end);
CREATE INDEX idx_governance_proposals_type ON governance_proposals(proposal_type);

CREATE INDEX idx_governance_votes_proposal ON governance_votes(proposal_id);
CREATE INDEX idx_governance_votes_voter ON governance_votes(voter_id);
CREATE INDEX idx_governance_votes_choice ON governance_votes(vote_choice);

-- Add updated_at triggers
CREATE TRIGGER update_staking_pools_updated_at BEFORE UPDATE ON staking_pools FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_oracle_feeds_updated_at BEFORE UPDATE ON oracle_feeds FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_governance_proposals_updated_at BEFORE UPDATE ON governance_proposals FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to get user's total staked amount across all pools
CREATE OR REPLACE FUNCTION get_user_total_stake(user_uuid UUID)
RETURNS DECIMAL(30,18) AS $$
DECLARE
    total_stake DECIMAL(30,18) := 0;
BEGIN
    SELECT COALESCE(SUM(staked_amount), 0)
    INTO total_stake
    FROM user_stakes us
    JOIN staking_pools sp ON us.pool_id = sp.id
    WHERE us.user_id = user_uuid 
    AND us.is_active = TRUE
    AND sp.is_active = TRUE;
    
    RETURN total_stake;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate voting power based on stakes and reputation
CREATE OR REPLACE FUNCTION calculate_voting_power(user_uuid UUID)
RETURNS DECIMAL(30,18) AS $$
DECLARE
    stake_power DECIMAL(30,18) := 0;
    reputation_multiplier DECIMAL(4,2) := 1.0;
    total_power DECIMAL(30,18);
BEGIN
    -- Get staked amount in governance pools
    SELECT COALESCE(SUM(us.staked_amount), 0)
    INTO stake_power
    FROM user_stakes us
    JOIN staking_pools sp ON us.pool_id = sp.id
    WHERE us.user_id = user_uuid
    AND us.is_active = TRUE
    AND sp.pool_type = 'governance_staking';
    
    -- Get reputation multiplier (max 2.0x for high reputation users)
    SELECT LEAST(2.0, 1.0 + (COALESCE(rs.total_score, 0) / 10000.0))
    INTO reputation_multiplier
    FROM reputation_scores rs
    WHERE rs.user_id = user_uuid;
    
    total_power := stake_power * reputation_multiplier;
    
    RETURN total_power;
END;
$$ LANGUAGE plpgsql;

-- View for comprehensive blockchain activity
CREATE VIEW blockchain_activity AS
SELECT 
    bt.id,
    bt.tx_hash,
    bn.network_name,
    bt.from_address,
    bt.to_address,
    bt.value,
    bt.transaction_type,
    bt.status,
    bt.block_timestamp,
    u.username as related_user,
    b.title as related_bounty,
    bt.metadata
FROM blockchain_transactions bt
LEFT JOIN blockchain_networks bn ON bt.network_id = bn.id
LEFT JOIN users u ON bt.related_user_id = u.id
LEFT JOIN bounties b ON bt.related_bounty_id = b.id
ORDER BY bt.block_timestamp DESC NULLS LAST, bt.created_at DESC;