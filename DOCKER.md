# 🐳 Docker Setup Guide - Nexus Security

This guide explains how to run the Nexus Security platform using Docker.

## Prerequisites

- Docker Desktop installed and running
- At least 8GB RAM allocated to Docker
- 20GB free disk space

## Quick Start

### 1. Configure Environment Variables

```bash
# Copy the example environment file
cp .env.example .env

# Edit the .env file with your configuration
nano .env  # or use your preferred editor
```

**Important variables to configure:**
- `JWT_SECRET` - Generate a secure random string (at least 32 characters)
- `SMTP_USERNAME` and `SMTP_PASSWORD` - For email notifications
- `BLOCKCHAIN_RPC_URL` and `TREASURY_PRIVATE_KEY` - For payment service (if using blockchain features)

### 2. Start All Services

```bash
# Using the startup script (recommended)
./docker-start.sh

# Or manually with docker-compose
docker-compose up -d
```

### 3. Access Services

Once started, services will be available at:

| Service | URL | Description |
|---------|-----|-------------|
| **API Gateway** | http://localhost:8080 | Main API endpoint |
| **User Service** | http://localhost:8081 | User management |
| **PgAdmin** | http://localhost:5050 | Database admin UI |
| **PostgreSQL** | localhost:5432 | Database |
| **Redis** | localhost:6379 | Cache |

**PgAdmin Login:**
- Email: `admin@nexus-security.io`
- Password: `admin`

## Docker Commands Reference

### Starting Services

```bash
# Start all services
docker-compose up -d

# Start specific service
docker-compose up -d user-service

# Start with logs visible
docker-compose up
```

### Stopping Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (⚠️  deletes all data)
docker-compose down -v

# Stop specific service
docker-compose stop user-service
```

### Viewing Logs

```bash
# View all logs
docker-compose logs

# View logs for specific service
docker-compose logs user-service

# Follow logs in real-time
docker-compose logs -f api-gateway

# View last 100 lines
docker-compose logs --tail=100 postgres
```

### Rebuilding Services

```bash
# Rebuild all services
docker-compose build

# Rebuild specific service
docker-compose build user-service

# Rebuild and restart
docker-compose up -d --build user-service
```

### Executing Commands in Containers

```bash
# Open shell in container
docker-compose exec user-service sh

# Run database migrations
docker-compose exec user-service /usr/local/bin/app migrate

# Check PostgreSQL
docker-compose exec postgres psql -U nexus_user -d nexus_security

# Check Redis
docker-compose exec redis redis-cli
```

### Managing Data

```bash
# List volumes
docker volume ls

# Inspect volume
docker volume inspect nexus-security_postgres_data

# Backup database
docker-compose exec postgres pg_dump -U nexus_user nexus_security > backup.sql

# Restore database
docker-compose exec -T postgres psql -U nexus_user nexus_security < backup.sql
```

## Service Architecture

```
┌─────────────────┐
│   API Gateway   │ :8080
│  (Entry Point)  │
└────────┬────────┘
         │
    ┌────┴────┬──────────┬───────────┬──────────┬──────────┐
    │         │          │           │          │          │
┌───▼──┐  ┌──▼───┐  ┌───▼────┐  ┌──▼───┐  ┌───▼───┐  ┌──▼────┐
│ User │  │Bounty│  │Analysis│  │Submi-│  │Consen-│  │Payment│
│ :8081│  │:8083 │  │ :8082  │  │ssion │  │sus    │  │ :8086 │
└──┬───┘  └──┬───┘  └───┬────┘  │:8084 │  │:8085  │  └───┬───┘
   │         │          │        └──┬───┘  └───┬───┘      │
   │         │          │           │          │          │
   └─────────┴──────────┴───────────┴──────────┴──────────┘
                        │                      │
                  ┌─────▼──────┐        ┌─────▼─────┐
                  │ PostgreSQL │        │   Redis   │
                  │   :5432    │        │   :6379   │
                  └────────────┘        └───────────┘
```

## Troubleshooting

### Services Won't Start

```bash
# Check Docker is running
docker info

# Check service status
docker-compose ps

# View service logs
docker-compose logs service-name

# Restart service
docker-compose restart service-name
```

### Database Connection Issues

```bash
# Check postgres is healthy
docker-compose ps postgres

# Check logs
docker-compose logs postgres

# Test connection
docker-compose exec postgres psql -U nexus_user -d nexus_security -c "SELECT 1;"
```

### Port Already in Use

```bash
# Find what's using the port
lsof -i :8080

# Change port in docker-compose.yml
# For example: "8090:8080" instead of "8080:8080"
```

### Out of Disk Space

```bash
# Remove unused containers and images
docker system prune

# Remove everything (⚠️  dangerous)
docker system prune -a --volumes
```

### Service Crashes on Startup

```bash
# Check resource limits
docker stats

# Increase Docker memory in Docker Desktop settings
# Recommended: At least 8GB RAM
```

## Production Deployment

For production deployment, make these changes:

1. **Security:**
   - Change all default passwords
   - Use strong JWT secret (64+ characters)
   - Enable HTTPS/TLS
   - Use secrets management (Docker Secrets, Vault)

2. **Scaling:**
   ```yaml
   # In docker-compose.yml
   user-service:
     deploy:
       replicas: 3
       resources:
         limits:
           cpus: '1.0'
           memory: 512M
   ```

3. **Monitoring:**
   - Add Prometheus and Grafana
   - Set up log aggregation (ELK stack)
   - Configure health checks

4. **Networking:**
   - Use reverse proxy (Nginx/Traefik)
   - Implement rate limiting
   - Set up load balancer

## Kubernetes Deployment

For Kubernetes deployment, see `kubernetes/` directory for manifests.

```bash
# Apply manifests
kubectl apply -f kubernetes/

# Check status
kubectl get pods -n nexus-security
```

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Reference](https://docs.docker.com/compose/compose-file/)
- [Rust Docker Best Practices](https://docs.docker.com/language/rust/)

## Support

For issues or questions:
- GitHub Issues: https://github.com/nexus-security/deep60/issues
- Documentation: https://docs.nexus-security.io
