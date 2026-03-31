# Testing & DevOps Guide

Complete guide for testing, containerization, and deployment of Nexus Security platform.

## Table of Contents
- [Testing Infrastructure](#testing-infrastructure)
- [Docker Setup](#docker-setup)
- [CI/CD Pipeline](#cicd-pipeline)
- [Deployment Guide](#deployment-guide)
- [Monitoring & Maintenance](#monitoring--maintenance)

---

## Testing Infrastructure

### Overview

The platform uses **Vitest** for testing with comprehensive unit, integration, and E2E tests.

**Test Coverage:**
- Unit tests for storage layer
- API integration tests
- E2E tests for complete user workflows
- Minimum 60% code coverage enforced

### Running Tests

```bash
cd frontend

# Run all tests (watch mode)
npm test

# Run tests once
npm run test:run

# Run with UI
npm run test:ui

# Generate coverage report
npm run test:coverage
```

### Test Files

- **`tests/storage.test.ts`** - Unit tests for MemStorage
  - User operations (CRUD)
  - Security engine management
  - Submission handling
  - Analysis tracking
  - Consensus results
  - Bounty management

- **`tests/api.test.ts`** - API integration tests
  - Authentication endpoints
  - Security engines endpoints
  - Submissions workflow
  - Statistics endpoints
  - Rate limiting
  - Security headers

- **`tests/e2e.test.ts`** - End-to-end tests
  - Complete user registration flow
  - File submission and analysis workflow
  - Security researcher workflow
  - Multiple submissions handling
  - Error handling and edge cases

### Test Configuration

**`vitest.config.ts`:**
```typescript
{
  environment: 'happy-dom',
  coverage: {
    provider: 'v8',
    thresholds: {
      lines: 60,
      functions: 60,
      branches: 60,
      statements: 60
    }
  }
}
```

### Writing Tests

#### Unit Test Example
```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { MemStorage } from '../server/storage';

describe('Feature', () => {
  let storage: MemStorage;

  beforeEach(() => {
    storage = new MemStorage();
  });

  it('should do something', async () => {
    const result = await storage.method();
    expect(result).toBeDefined();
  });
});
```

#### API Test Example
```typescript
import request from 'supertest';

it('POST /api/endpoint - should work', async () => {
  const response = await request(app)
    .post('/api/endpoint')
    .send({ data: 'value' })
    .expect(200);

  expect(response.body).toHaveProperty('id');
});
```

### Coverage Reports

After running `npm run test:coverage`, view reports:

```bash
# HTML report
open coverage/index.html

# Terminal summary
cat coverage/lcov-report/index.html
```

**Coverage Locations:**
- `coverage/lcov.info` - LCOV format for CI
- `coverage/html/` - HTML reports
- `coverage/json/` - JSON format

---

## Docker Setup

### Overview

The platform uses Docker and Docker Compose for containerization.

**Services:**
- Frontend (Node.js + Express + Vite)
- PostgreSQL database
- Redis cache
- Backend microservices (Rust)
- MinIO (S3-compatible storage)
- ClamAV (antivirus scanning)
- PgAdmin (database management)

### Quick Start

#### Development (Frontend Only)

```bash
# Start essential services for frontend dev
docker-compose -f docker-compose.dev.yml up

# Run frontend locally
cd frontend && npm run dev
```

#### Full Stack (All Services)

```bash
# Build and start all services
docker-compose up --build

# Start in background
docker-compose up -d

# View logs
docker-compose logs -f frontend

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

### Docker Files

#### `frontend/Dockerfile`
Multi-stage build for production:
- Stage 1: Build application
- Stage 2: Production image with minimal dependencies

**Features:**
- Non-root user for security
- Health checks
- Optimized layer caching
- Multi-stage builds

#### `.dockerignore`
Excludes unnecessary files:
- `node_modules/`
- Test files
- Development configs
- Documentation

### Docker Compose Files

#### `docker-compose.yml` - Production
Full stack with all backend services.

**Services:**
- postgres (port 5432)
- redis (port 6379)
- frontend (port 5000)
- api-gateway (port 8080)
- user-service (port 8081)
- analysis-engine (port 8082)
- bounty-manager (port 8083)
- submission-service (port 8084)
- consensus-service (port 8085)
- payment-service (port 8086)
- reputation-service (port 8087)
- notification-service (port 8088)
- minio (ports 9000, 9001)
- clamav (port 3310)
- pgadmin (port 5050)

#### `docker-compose.dev.yml` - Development
Simplified for frontend development.

**Services:**
- postgres (port 5432)
- redis (port 6379)
- frontend (ports 5000, 5173)

**Features:**
- Volume mounts for hot reload
- Lower resource limits
- Faster bcrypt rounds
- Higher rate limits

### Docker Commands

```bash
# Build specific service
docker-compose build frontend

# Rebuild without cache
docker-compose build --no-cache frontend

# Start specific services
docker-compose up postgres redis

# Scale services
docker-compose up --scale frontend=3

# Execute command in container
docker-compose exec frontend npm run test

# View service logs
docker-compose logs -f frontend postgres redis

# Inspect service
docker-compose ps
docker-compose top frontend

# Remove stopped containers
docker-compose rm

# Prune unused images/volumes
docker system prune -a
```

### Environment Variables

Create `.env` file in project root:

```bash
# Database
POSTGRES_PASSWORD=your_secure_password

# JWT
JWT_SECRET=your-super-secret-jwt-key

# Frontend
FRONTEND_URL=http://localhost:5173

# Optional: Docker registry
DOCKER_USERNAME=your_username
```

### Health Checks

All services have health checks:

```bash
# Check health status
docker-compose ps

# View health check logs
docker inspect --format='{{json .State.Health}}' nexus-frontend
```

### Troubleshooting Docker

#### Container won't start
```bash
# Check logs
docker-compose logs frontend

# Check config
docker-compose config

# Rebuild
docker-compose build --no-cache frontend
```

#### Port already in use
```bash
# Find process using port
lsof -i :5000

# Kill process
kill -9 <PID>

# Or change port in docker-compose.yml
```

#### Volume issues
```bash
# Remove volumes
docker-compose down -v

# Remove specific volume
docker volume rm nexus-security_postgres_data
```

---

## CI/CD Pipeline

### Overview

GitHub Actions workflows for automated testing, building, and deployment.

**Workflows:**
1. **Frontend CI** - Runs on every PR/push
2. **Deploy** - Runs on main branch push

### Frontend CI Workflow

**File:** `.github/workflows/frontend-ci.yml`

**Triggers:**
- Pull requests to `main` or `develop`
- Pushes to `main` or `develop`
- Only when frontend files change

**Jobs:**

1. **lint-and-test**
   - Runs ESLint
   - Type checks with TypeScript
   - Runs all tests with coverage
   - Uploads coverage to Codecov
   - Comments coverage on PR

2. **build**
   - Builds production bundle
   - Uploads artifacts
   - Verifies build succeeds

3. **security-scan**
   - Runs `npm audit`
   - Runs Snyk security scan

**Services:**
- PostgreSQL (for tests)
- Redis (for tests)

### Deploy Workflow

**File:** `.github/workflows/deploy.yml`

**Triggers:**
- Pushes to `main` branch
- Manual trigger (`workflow_dispatch`)

**Jobs:**

1. **test**
   - Runs full test suite
   - Must pass before deployment

2. **build-and-push**
   - Builds Docker image
   - Pushes to Docker Hub
   - Tags with branch and SHA

3. **deploy**
   - Deploys to production server via SSH
   - Pulls latest image
   - Restarts containers
   - Runs health check
   - Notifies via Slack

### Required Secrets

Set in GitHub repository settings:

```bash
# Docker Hub
DOCKER_USERNAME=your_username
DOCKER_PASSWORD=your_password

# Deployment Server
SSH_PRIVATE_KEY=your_private_key
SSH_USER=deploy
SSH_HOST=your_server_ip

# Optional
SNYK_TOKEN=your_snyk_token
SLACK_WEBHOOK=your_slack_webhook
CODECOV_TOKEN=your_codecov_token
```

### Local CI Testing

Test workflows locally with [act](https://github.com/nektos/act):

```bash
# Install act
brew install act

# Run CI workflow
act pull_request

# Run specific job
act -j lint-and-test

# Use custom secrets
act -s GITHUB_TOKEN=your_token
```

### Workflow Status Badges

Add to README.md:

```markdown
[![Frontend CI](https://github.com/your-org/nexus-security/workflows/Frontend%20CI/badge.svg)](https://github.com/your-org/nexus-security/actions)
[![Deploy](https://github.com/your-org/nexus-security/workflows/Deploy%20to%20Production/badge.svg)](https://github.com/your-org/nexus-security/actions)
[![codecov](https://codecov.io/gh/your-org/nexus-security/branch/main/graph/badge.svg)](https://codecov.io/gh/your-org/nexus-security)
```

---

## Deployment Guide

### Prerequisites

- Docker and Docker Compose installed
- PostgreSQL 16
- Redis 7
- Node.js 20
- Domain with SSL certificate

### Server Setup

#### 1. Prepare Server

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Install Docker Compose
sudo apt install docker-compose-plugin

# Create deploy user
sudo adduser deploy
sudo usermod -aG docker deploy
```

#### 2. Clone Repository

```bash
su - deploy
git clone https://github.com/your-org/nexus-security.git
cd nexus-security
```

#### 3. Configure Environment

```bash
# Create production .env
cp .env.example .env

# Edit with production values
nano .env
```

**Required variables:**
```bash
NODE_ENV=production
POSTGRES_PASSWORD=strong_password_here
JWT_SECRET=very-long-random-string
FRONTEND_URL=https://nexus-security.io
```

#### 4. Initialize Database

```bash
# Start PostgreSQL and Redis
docker-compose up -d postgres redis

# Wait for health check
docker-compose ps

# Run migrations
cd frontend
npx drizzle-kit push
```

#### 5. Start Application

```bash
# Start all services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f frontend
```

### SSL/TLS Setup

#### Using Nginx

```nginx
server {
    listen 443 ssl http2;
    server_name nexus-security.io;

    ssl_certificate /etc/ssl/certs/nexus-security.crt;
    ssl_certificate_key /etc/ssl/private/nexus-security.key;

    location / {
        proxy_pass http://localhost:5000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

#### Using Caddy

```caddyfile
nexus-security.io {
    reverse_proxy localhost:5000
}
```

### Deployment Strategies

#### Blue-Green Deployment

```bash
# Terminal 1: Start new version (green)
docker-compose -f docker-compose.green.yml up -d

# Verify green deployment
curl http://localhost:5001/api/engines

# Switch traffic (update load balancer)

# Terminal 2: Stop old version (blue)
docker-compose -f docker-compose.blue.yml down
```

#### Rolling Update

```bash
# Update one instance at a time
docker-compose up -d --no-deps --scale frontend=3 frontend

# Health check each instance
docker-compose ps
```

### Database Migrations

```bash
# Generate migration
cd frontend
npx drizzle-kit generate

# Review migration
cat drizzle/0001_*.sql

# Apply migration
npx drizzle-kit push

# Rollback (if needed)
# Restore from backup
```

### Backup Strategy

#### Automated Backups

```bash
# Create backup script
cat > /opt/backup-nexus.sh << 'EOF'
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR=/opt/backups

# Backup PostgreSQL
docker-compose exec -T postgres pg_dump -U nexus_user nexus_security > $BACKUP_DIR/postgres_$DATE.sql

# Backup Redis
docker-compose exec -T redis redis-cli SAVE
cp /var/lib/docker/volumes/nexus-security_redis_data/_data/dump.rdb $BACKUP_DIR/redis_$DATE.rdb

# Compress
tar -czf $BACKUP_DIR/nexus_backup_$DATE.tar.gz $BACKUP_DIR/*_$DATE.*

# Cleanup old backups (keep 7 days)
find $BACKUP_DIR -name "nexus_backup_*.tar.gz" -mtime +7 -delete
EOF

chmod +x /opt/backup-nexus.sh

# Add to crontab
crontab -e
# Add: 0 2 * * * /opt/backup-nexus.sh
```

---

## Monitoring & Maintenance

### Health Checks

```bash
# Check all services
docker-compose ps

# Check specific service
curl http://localhost:5000/api/engines

# Check database
docker-compose exec postgres pg_isready

# Check Redis
docker-compose exec redis redis-cli ping
```

### Logs

```bash
# View logs
docker-compose logs -f

# Specific service
docker-compose logs -f frontend

# Last 100 lines
docker-compose logs --tail=100 frontend

# Export logs
docker-compose logs > logs.txt
```

### Metrics

Monitor with Docker stats:

```bash
# Real-time stats
docker stats

# Specific containers
docker stats nexus-frontend nexus-postgres
```

### Alerts

Set up alerts for:
- Container health check failures
- High memory/CPU usage
- Disk space low
- Database connection failures
- High error rates

### Maintenance

#### Update Dependencies

```bash
cd frontend
npm audit fix
npm update
npm run test:run
git commit -am "Update dependencies"
```

#### Update Docker Images

```bash
# Pull latest images
docker-compose pull

# Restart services
docker-compose up -d

# Remove old images
docker image prune -a
```

#### Database Maintenance

```bash
# Vacuum database
docker-compose exec postgres vacuumdb -U nexus_user -d nexus_security -v

# Reindex
docker-compose exec postgres reindexdb -U nexus_user -d nexus_security
```

### Troubleshooting

#### High Memory Usage

```bash
# Check memory
docker stats

# Restart service
docker-compose restart frontend

# Adjust limits in docker-compose.yml
```

#### Slow Queries

```bash
# Enable query logging
docker-compose exec postgres psql -U nexus_user -d nexus_security
# ALTER DATABASE nexus_security SET log_min_duration_statement = 1000;

# View slow queries
docker-compose exec postgres tail -f /var/log/postgresql/postgresql.log
```

#### Connection Errors

```bash
# Check network
docker network inspect nexus-security_nexus-network

# Recreate network
docker-compose down
docker network prune
docker-compose up -d
```

---

## Quick Reference

### Common Commands

```bash
# Development
npm test                        # Run tests
npm run dev                     # Start dev server
docker-compose -f docker-compose.dev.yml up

# Testing
npm run test:coverage           # Generate coverage
npm run test:ui                 # Open test UI

# Docker
docker-compose up -d            # Start services
docker-compose logs -f          # View logs
docker-compose down             # Stop services
docker-compose ps               # Check status

# Deployment
git pull origin main            # Update code
docker-compose pull             # Pull images
docker-compose up -d            # Restart
docker system prune -af         # Cleanup
```

### Port Reference

- **5000** - Frontend application
- **5173** - Vite dev server
- **5432** - PostgreSQL
- **6379** - Redis
- **8080-8088** - Backend services
- **9000-9001** - MinIO
- **3310** - ClamAV
- **5050** - PgAdmin

---

## Support

For issues or questions:
1. Check logs: `docker-compose logs`
2. Review this documentation
3. Check GitHub issues
4. Contact DevOps team