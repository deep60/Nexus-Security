# Local Development Setup

This guide walks you through setting up Nexus Security for local development.

## Prerequisites

- **Docker** >= 20.10
- **Docker Compose** >= 2.0
- **Rust** >= 1.82
- **Node.js** >= 18
- **Git**

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/nexus-security/nexus-security.git
cd nexus-security
```

### 2. Environment Configuration

```bash
# Copy environment template
cp infrastructure/docker/.env.example infrastructure/docker/development/.env

# Edit with your values
nano infrastructure/docker/development/.env
```

**Required variables:**

```bash
POSTGRES_PASSWORD=nexus_dev_password
REDIS_PASSWORD=nexus_redis_password
JWT_SECRET=your_dev_jwt_secret_min_32_chars
```

### 3. Start Services with Docker

```bash
# Start all services
docker-compose -f infrastructure/docker/development/docker-compose.yml up

# Or run in background
docker-compose -f infrastructure/docker/development/docker-compose.yml up -d
```

### 4. Verify Services

| Service | URL | Health Check |
|---------|-----|--------------|
| Frontend | http://localhost:3000 | Browser |
| API Gateway | http://localhost:8080 | http://localhost:8080/health |
| Analysis Engine | http://localhost:8082 | http://localhost:8082/health |
| PostgreSQL | localhost:5432 | `pg_isready` |
| Redis | localhost:6379 | `redis-cli ping` |
| Grafana | http://localhost:3001 | admin/admin |

## Running Backend Locally

For faster development iteration, run backend services outside Docker:

### 1. Start Only Dependencies

```bash
docker-compose -f infrastructure/docker/development/docker-compose.yml up postgres redis
```

### 2. Set Environment Variables

```bash
export DATABASE_URL=postgresql://nexus:nexus_dev_password@localhost:5432/nexus_security
export REDIS_URL=redis://:nexus_redis_password@localhost:6379
export JWT_SECRET=your_dev_jwt_secret_min_32_chars
export RUST_LOG=debug
```

### 3. Run Backend Services

```bash
cd backend

# Run API Gateway
cargo run --bin api-gateway

# In another terminal - Analysis Engine
cargo run --bin analysis-engine

# In another terminal - Bounty Manager
cargo run --bin bounty-manager
```

## Running Frontend Locally

```bash
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev
```

Frontend will be available at http://localhost:5173

## Database Management

### Run Migrations

```bash
cd backend/api-gateway
sqlx migrate run --database-url $DATABASE_URL
```

### Reset Database

```bash
docker-compose -f infrastructure/docker/development/docker-compose.yml down -v
docker-compose -f infrastructure/docker/development/docker-compose.yml up postgres
```

### Access PostgreSQL

```bash
docker exec -it nexus-postgres psql -U nexus -d nexus_security
```

## Testing

### Run Backend Tests

```bash
cd backend
cargo test
```

### Run Frontend Tests

```bash
cd frontend
npm test
```

## Common Issues

### Port Already in Use

```bash
# Find process using port
lsof -i :8080

# Kill process
kill -9 <PID>
```

### Database Connection Failed

```bash
# Check PostgreSQL is running
docker-compose logs postgres

# Verify connection
docker exec -it nexus-postgres pg_isready -U nexus
```

### Redis Connection Failed

```bash
# Check Redis is running
docker exec -it nexus-redis redis-cli -a nexus_redis_password ping
```

## Useful Commands

```bash
# View all logs
docker-compose -f infrastructure/docker/development/docker-compose.yml logs -f

# View specific service logs
docker-compose logs -f api-gateway

# Rebuild specific service
docker-compose up --build api-gateway

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```
