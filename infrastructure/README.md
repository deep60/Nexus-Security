# Verdyx Infrastructure

> **📖 Read [`DEPLOYMENT.md`](./DEPLOYMENT.md) first — it is the authoritative deployment guide.**
> Launch runs on **Docker Compose on a single VM** (root `docker-compose.yml`, deployed by
> `.github/workflows/deploy.yml`) with **managed RDS + ElastiCache**. The Kubernetes / EKS / Helm
> assets below are **Phase 2 (not live)** — CI does not deploy to them. Some sections of this
> README (the `docker-compose.prod.yml` and single-`verdyx`-DB references) are outdated; where they
> conflict with `DEPLOYMENT.md`, `DEPLOYMENT.md` wins.

This directory contains all infrastructure-as-code configurations for deploying the Verdyx platform.

## Directory Structure

```
infrastructure/
├── docker/                    # Docker configurations
│   ├── development/           # Development docker-compose
│   ├── production/            # Production Dockerfiles & compose
│   │   ├── api-gateway/
│   │   ├── analysis-engine/
│   │   ├── bounty-manager/
│   │   └── frontend/
│   ├── scripts/               # Build & push scripts
│   └── postgres.Dockerfile   # Custom PostgreSQL image
├── kubernetes/                # Kubernetes manifests
│   ├── api-gateway/
│   ├── analysis-engine/
│   ├── database/
│   ├── ingress.yaml
│   └── namespace.yaml
├── terraform/                 # Cloud infrastructure (AWS/GCP/Azure)
└── ansible/                   # Configuration management
```

## Quick Start

### Prerequisites

- Docker & Docker Compose v2+
- kubectl (for Kubernetes deployment)
- Rust 1.82+ (for local backend development)
- Node.js 18+ (for frontend development)

### Development Setup

1. **Clone and navigate to project:**
   ```bash
   cd /path/to/Verdyx
   ```

2. **Copy environment file:**
   ```bash
   cp infrastructure/docker/.env.example infrastructure/docker/.env
   # Edit .env with your configuration
   ```

3. **Start development services:**
   ```bash
   docker-compose -f infrastructure/docker/development/docker-compose.yml up
   ```

4. **Access services:**
   - Frontend: http://localhost:3000
   - API Gateway: http://localhost:8080
   - Analysis Engine: http://localhost:8082
   - Bounty Manager: http://localhost:8083
   - PostgreSQL: localhost:5432
   - Redis: localhost:6379
   - Grafana: http://localhost:3001 (admin/admin)
   - Prometheus: http://localhost:9090

### Production Deployment

1. **Build all images:**
   ```bash
   ./infrastructure/docker/scripts/build.sh
   ```

2. **Push to registry:**
   ```bash
   export DOCKER_REGISTRY=your-registry.com/verdyx
   export VERSION=1.0.0
   ./infrastructure/docker/scripts/push.sh
   ```

3. **Deploy with Docker Compose:**
   ```bash
   docker-compose -f infrastructure/docker/production/docker-compose.prod.yml up -d
   ```

## Backend Integration

### Connecting Backend Services to Infrastructure

The backend Rust services in `/backend` are designed to work with this infrastructure:

#### 1. Environment Variables

Each backend service expects these environment variables:

```bash
# Database
DATABASE_URL=postgresql://verdyx:password@postgres:5432/verdyx

# Redis
REDIS_URL=redis://:password@redis:6379

# JWT Authentication
JWT_SECRET=your-secret-key
JWT_REFRESH_SECRET=your-refresh-secret

# Blockchain
BLOCKCHAIN_RPC_URL=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
CONTRACT_ADDRESS_BOUNTY=0x...
CONTRACT_ADDRESS_TOKEN=0x...

# Service Discovery
ANALYSIS_ENGINE_URL=http://analysis-engine:8082
BOUNTY_MANAGER_URL=http://bounty-manager:8083
```

#### 2. Service Ports

| Service | Internal Port | External Port |
|---------|--------------|---------------|
| api-gateway | 8080 | 8080 |
| analysis-engine | 8082 | 8082 |
| bounty-manager | 8083 | 8083 |
| user-service | 8084 | - |
| reputation-service | 8085 | - |
| notification-service | 8086 | - |

#### 3. Running Backend Locally (without Docker)

```bash
cd backend

# Set environment variables
export DATABASE_URL=postgresql://verdyx:password@localhost:5432/verdyx
export REDIS_URL=redis://:password@localhost:6379

# Run specific service
cargo run --bin api-gateway
cargo run --bin analysis-engine
cargo run --bin bounty-manager
```

#### 4. Database Migrations

The PostgreSQL Dockerfile automatically runs migrations from:
- `database/migrations/` - Schema migrations
- `database/seeds/` - Test data

To run migrations manually:
```bash
cd backend/api-gateway
sqlx migrate run --database-url $DATABASE_URL
```

### Adding New Backend Services

1. **Create Dockerfile:**
   ```dockerfile
   # infrastructure/docker/production/new-service/new-service.Dockerfile
   FROM rust:1.82-slim as builder
   # ... (follow existing patterns)
   ```

2. **Add to docker-compose.yml:**
   ```yaml
   new-service:
     build:
       context: ../../..
       dockerfile: infrastructure/docker/production/new-service/new-service.Dockerfile
     environment:
       DATABASE_URL: postgresql://verdyx:${POSTGRES_PASSWORD}@postgres:5432/verdyx
     depends_on:
       postgres:
         condition: service_healthy
   ```

3. **Create Kubernetes manifests:**
   ```bash
   mkdir infrastructure/kubernetes/new-service
   # Create deployment.yaml, service.yaml, configmap.yaml
   ```

## Kubernetes Deployment

### Prerequisites

- kubectl configured with cluster access
- Kubernetes cluster (EKS, GKE, AKS, or local)

### Deployment Steps

1. **Create namespace:**
   ```bash
   kubectl apply -f infrastructure/kubernetes/namespace.yaml
   ```

2. **Create secrets:**
   ```bash
   kubectl create secret generic verdyx-database-secret \
     --from-literal=database_url='postgresql://verdyx:password@postgresql-service:5432/verdyx' \
     -n verdyx

   kubectl create secret generic verdyx-auth-secret \
     --from-literal=jwt_secret='your-jwt-secret' \
     --from-literal=jwt_refresh_secret='your-refresh-secret' \
     -n verdyx
   ```

3. **Deploy database:**
   ```bash
   kubectl apply -f infrastructure/kubernetes/database/
   ```

4. **Deploy services:**
   ```bash
   kubectl apply -f infrastructure/kubernetes/api-gateway/
   kubectl apply -f infrastructure/kubernetes/analysis-engine/
   ```

5. **Configure ingress:**
   ```bash
   kubectl apply -f infrastructure/kubernetes/ingress.yaml
   ```

### Monitoring

- **Prometheus**: Collects metrics from all services
- **Grafana**: Visualizes metrics with pre-configured dashboards
- **Health endpoints**: Each service exposes `/health` and `/ready`

## Troubleshooting

### Common Issues

1. **Database connection failed:**
   ```bash
   # Check if PostgreSQL is healthy
   docker-compose logs postgres
   docker exec -it verdyx-postgres pg_isready -U verdyx
   ```

2. **Service can't connect to Redis:**
   ```bash
   # Verify Redis is running
   docker exec -it verdyx-redis redis-cli ping
   ```

3. **Build failures:**
   ```bash
   # Clean and rebuild
   docker-compose down -v
   docker system prune -f
   ./infrastructure/docker/scripts/build.sh
   ```

4. **Kubernetes pods not starting:**
   ```bash
   kubectl describe pod <pod-name> -n verdyx
   kubectl logs <pod-name> -n verdyx
   ```

## Security Considerations

- All services run as non-root users
- Secrets are managed via environment variables (use Kubernetes Secrets in production)
- Network policies restrict inter-service communication
- TLS is required for all external traffic
- CORS is configured for specific origins only

## Contributing

When adding infrastructure changes:
1. Test locally with docker-compose first
2. Update documentation
3. Ensure all health checks pass
4. Review security implications
