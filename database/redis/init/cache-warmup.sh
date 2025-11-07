#!/bin/bash

# Redis Cache Warmup Script for Nexus-Security
# This script pre-populates Redis with commonly used data

set -e

REDIS_HOST="${REDIS_HOST:-localhost}"
REDIS_PORT="${REDIS_PORT:-6379}"
REDIS_PASSWORD="${REDIS_PASSWORD:-}"

echo "Warming up Redis cache for Nexus-Security..."

# Function to execute Redis commands
redis_cmd() {
    if [ -n "$REDIS_PASSWORD" ]; then
        redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" -a "$REDIS_PASSWORD" "$@"
    else
        redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" "$@"
    fi
}

# ==================== DB 0: Session Storage ====================
echo "Setting up DB 0 (Session Storage)..."
redis_cmd SELECT 0
redis_cmd CONFIG SET maxmemory-policy allkeys-lru

# ==================== DB 1: API Rate Limiting ====================
echo "Setting up DB 1 (Rate Limiting)..."
redis_cmd SELECT 1

# Set default rate limits for different user tiers
redis_cmd HSET rate_limit:free requests_per_minute 10
redis_cmd HSET rate_limit:free burst 20

redis_cmd HSET rate_limit:basic requests_per_minute 60
redis_cmd HSET rate_limit:basic burst 100

redis_cmd HSET rate_limit:premium requests_per_minute 300
redis_cmd HSET rate_limit:premium burst 500

redis_cmd HSET rate_limit:enterprise requests_per_minute 1000
redis_cmd HSET rate_limit:enterprise burst 2000

echo "Rate limit configurations created"

# ==================== DB 2: Analysis Results Cache ====================
echo "Setting up DB 2 (Analysis Cache)..."
redis_cmd SELECT 2

# Set TTL for cached analysis results (24 hours)
redis_cmd CONFIG SET maxmemory-policy volatile-lru

# ==================== DB 3: Engine Reputation Cache ====================
echo "Setting up DB 3 (Reputation Cache)..."
redis_cmd SELECT 3

# Initialize some common configuration values
redis_cmd SET consensus:min_engines 3
redis_cmd SET consensus:threshold 0.7
redis_cmd SET reputation:initial_score 50
redis_cmd SET stake:minimum 0.01

echo "Reputation system defaults set"

# ==================== DB 4: Temporary File Processing ====================
echo "Setting up DB 4 (File Processing)..."
redis_cmd SELECT 4

# Set TTL for temporary processing data (1 hour)
redis_cmd CONFIG SET maxmemory-policy volatile-ttl

# ==================== DB 5: WebSocket Connections ====================
echo "Setting up DB 5 (WebSocket)..."
redis_cmd SELECT 5

# Initialize connection tracking
redis_cmd SET ws:connections 0
redis_cmd SET ws:max_connections 10000

echo "WebSocket configuration initialized"

# ==================== Pub/Sub Channels ====================
echo "Setting up Pub/Sub channels..."
redis_cmd SELECT 0

# Document the pub/sub channels (for reference)
redis_cmd SET channel:analysis:started "Channel for analysis start notifications"
redis_cmd SET channel:analysis:completed "Channel for analysis completion"
redis_cmd SET channel:consensus:reached "Channel for consensus notifications"
redis_cmd SET channel:payment:processed "Channel for payment notifications"
redis_cmd SET channel:reputation:updated "Channel for reputation updates"

# Set expiry on these documentation keys (7 days)
for channel in started completed reached processed updated; do
    redis_cmd EXPIRE "channel:*:$channel" 604800
done

echo "Pub/Sub channels documented"

# ==================== Health Check ====================
redis_cmd PING > /dev/null
echo "Redis health check: OK"

# ==================== Statistics ====================
echo ""
echo "Redis Cache Warmup Complete!"
echo "================================"
redis_cmd INFO stats | grep -E "total_commands_processed|total_connections_received"
redis_cmd DBSIZE

echo ""
echo "Cache warmup completed successfully"
