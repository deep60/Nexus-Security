# System Design

## Overview

Nexus Security implements a microservices architecture optimized for security analysis workloads and blockchain integration. This document details the design decisions, component responsibilities, and interaction patterns.

## Core Components

### 1. API Gateway

**Purpose**: Single entry point for all client requests.

**Responsibilities**:
- JWT authentication and authorization
- Rate limiting and throttling
- Request routing to appropriate services
- CORS handling
- Request/response logging
- WebSocket connection management

**Technology**: Rust with Actix-web

**Ports**: 8080 (HTTP), 9090 (Metrics)

```rust
// Key endpoints
POST /api/v1/auth/login       // User authentication
POST /api/v1/auth/register    // User registration
GET  /api/v1/bounties         // List bounties
POST /api/v1/analyze          // Submit file for analysis
WS   /ws                      // Real-time updates
```

### 2. Analysis Engine

**Purpose**: Core malware analysis service.

**Responsibilities**:
- Static analysis (PE parsing, string extraction)
- YARA rule matching
- File entropy calculation
- Hash generation (MD5, SHA256, SSDeep)
- Behavioral indicator detection
- Analysis result aggregation

**Technology**: Rust with YARA bindings

**Ports**: 8082 (HTTP), 9090 (Metrics)

**Analysis Pipeline**:
```
File Upload ’ Validation ’ Static Analysis ’ YARA Scan ’
Hash Calculation ’ Behavioral Check ’ Result Aggregation ’ Storage
```

### 3. Bounty Manager

**Purpose**: Manages bounty lifecycle and reward distribution.

**Responsibilities**:
- Bounty creation and validation
- Analyst stake management
- Consensus calculation
- Reward distribution via smart contracts
- Dispute resolution
- Expiry handling

**Technology**: Rust with ethers-rs

**Ports**: 8083 (HTTP), 9090 (Metrics)

### 4. User Service

**Purpose**: User management and authentication.

**Responsibilities**:
- User registration and profile management
- Wallet address verification
- Password hashing (bcrypt)
- Session management
- Role-based access control

**Technology**: Rust

**Ports**: 8084 (HTTP)

### 5. Reputation Service

**Purpose**: Tracks and calculates user reputation scores.

**Responsibilities**:
- Reputation score calculation
- Historical performance tracking
- Leaderboard management
- Trust level determination

**Technology**: Rust

**Ports**: 8085 (HTTP)

### 6. Notification Service

**Purpose**: Handles all user notifications.

**Responsibilities**:
- Email notifications
- Webhook deliveries
- In-app notifications via WebSocket
- Notification preferences

**Technology**: Rust

**Ports**: 8086 (HTTP)

## Data Storage

### PostgreSQL (Primary Database)

**Schema Design**:
- `users` - User accounts and profiles
- `bounties` - Bounty metadata and status
- `analyses` - Analysis submissions and results
- `files` - Uploaded file metadata
- `reputation_events` - Reputation change history
- `audit_log` - System audit trail

**Key Features**:
- UUID primary keys
- JSONB for flexible analysis data
- Materialized views for statistics
- Row-level security for multi-tenancy

### Redis (Cache & Message Queue)

**Usage Patterns**:
- Session storage (TTL: 24 hours)
- Rate limiting counters
- Analysis result caching
- Pub/Sub for real-time updates
- Distributed locks for consensus

## Blockchain Integration

### Smart Contracts

1. **BountyContract.sol**
   - Create bounties with token deposits
   - Submit analysis verdicts with stakes
   - Automatic reward distribution
   - Dispute resolution mechanism

2. **ReputationToken.sol**
   - ERC-20 token for platform operations
   - Staking mechanism
   - Slashing for incorrect analyses

3. **GovernanceContract.sol**
   - Platform parameter updates
   - Fee structure changes
   - Community voting

### Integration Flow

```
User Action ’ API Gateway ’ Bounty Manager ’ ethers-rs ’
Ethereum RPC ’ Smart Contract ’ Event Emission ’
Event Listener ’ Database Update ’ WebSocket Notification
```

## Communication Patterns

### Synchronous (REST)
- Client-to-Gateway
- Gateway-to-Service (internal)
- Service-to-Database

### Asynchronous (Events)
- Analysis completion notifications
- Bounty state changes
- Consensus reached events
- Reward distribution confirmations

### Real-time (WebSocket)
- Analysis progress updates
- New bounty alerts
- Chat/comments on bounties
- Reputation changes

## Security Architecture

### Authentication Flow

```
1. User submits credentials
2. API Gateway validates against User Service
3. JWT token generated (1 hour expiry)
4. Refresh token stored in Redis (7 days)
5. Token sent with each request
6. Gateway validates signature and expiry
```

### Authorization Levels

| Role | Permissions |
|------|-------------|
| Guest | View public bounties, read analyses |
| User | Submit files, create bounties, submit analyses |
| Analyst | All User + claim bounties, stake tokens |
| Admin | All Analyst + moderate, manage platform |

### Data Protection

- Passwords: bcrypt with 12+ rounds
- API Keys: HMAC-SHA256
- Tokens: RS256 signed JWTs
- Database: Encrypted at rest (AES-256)
- Network: TLS 1.3 for all traffic

## Scalability Considerations

### Horizontal Scaling

| Service | Scaling Strategy |
|---------|-----------------|
| API Gateway | HPA based on CPU/requests |
| Analysis Engine | HPA based on queue depth |
| Bounty Manager | 2-3 replicas (blockchain calls) |
| Database | Read replicas for queries |
| Redis | Redis Cluster for HA |

### Performance Optimizations

1. **Connection Pooling**: SQLx with pool size 10-50
2. **Caching**: 5-minute TTL for frequent queries
3. **Async Processing**: Tokio runtime for I/O
4. **Batch Processing**: Bulk database operations
5. **CDN**: Static assets via CloudFront/Cloudflare

## Monitoring & Observability

### Metrics (Prometheus)
- Request latency (p50, p95, p99)
- Error rates by endpoint
- Database query duration
- Redis hit/miss ratio
- Blockchain transaction success rate

### Logging (Structured JSON)
- Request ID correlation
- User ID tracking
- Error stack traces
- Audit events

### Tracing (OpenTelemetry)
- Distributed trace IDs
- Service-to-service spans
- Database query spans

## Failure Handling

### Circuit Breaker
- Blockchain RPC calls
- External API integrations
- Email service

### Retry Logic
- Database connections (3 retries)
- Redis operations (2 retries)
- Blockchain transactions (5 retries with backoff)

### Graceful Degradation
- Cache miss ’ Database query
- Blockchain unavailable ’ Queue transaction
- Analysis timeout ’ Partial results

## Future Considerations

1. **IPFS Integration**: Decentralized storage for analysis reports
2. **Multi-chain Support**: Polygon, Arbitrum for lower fees
3. **ML Models**: Automated preliminary threat classification
4. **Federation**: Cross-platform threat intelligence sharing
