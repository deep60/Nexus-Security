# 🚀 Nexus Security - Quick Start Guide

## Setup in 3 Steps

### Step 1: Configure Environment

```bash
# Navigate to project directory
cd /Users/arjun/Developer/Nexus-Security

# Copy environment template
cp .env.example .env

# Edit configuration (use your preferred editor)
nano .env
```

**Required changes in `.env`:**
- `JWT_SECRET` → Change to a random 32+ character string
- `SMTP_USERNAME` → Your email for notifications
- `SMTP_PASSWORD` → Your email app password

### Step 2: Start Services

```bash
# Make startup script executable (if not already)
chmod +x docker-start.sh

# Start everything
./docker-start.sh
```

This will:
- ✅ Pull PostgreSQL and Redis images
- ✅ Build all Nexus Security services
- ✅ Start all containers
- ✅ Set up networking

### Step 3: Verify Everything Works

```bash
# Check all services are running
docker-compose ps

# Test API Gateway
curl http://localhost:8080/api/v1/health

# View logs
docker-compose logs -f api-gateway
```

## 🎯 What's Running?

After startup, you have:

| Service | Port | Status |
|---------|------|--------|
| PostgreSQL | 5432 | ✅ Running |
| Redis | 6379 | ✅ Running |
| API Gateway | 8080 | ✅ Running |
| User Service | 8081 | ✅ Running |
| Analysis Engine | 8082 | ✅ Running |
| Bounty Manager | 8083 | ✅ Running |
| Submission Service | 8084 | ✅ Running |
| Consensus Service | 8085 | ✅ Running |
| Payment Service | 8086 | ✅ Running |
| Reputation Service | 8087 | ✅ Running |
| Notification Service | 8088 | ✅ Running |
| PgAdmin | 5050 | ✅ Running |

## 🔧 Common Commands

```bash
# Stop everything
docker-compose down

# Restart specific service
docker-compose restart user-service

# View logs
docker-compose logs -f user-service

# Rebuild after code changes
docker-compose up -d --build user-service

# Access database
docker-compose exec postgres psql -U nexus_user -d nexus_security

# Clear everything (⚠️ deletes data)
docker-compose down -v
```

## 🧪 Testing the APIs

### 1. Health Check
```bash
curl http://localhost:8080/api/v1/health
```

### 2. Register User
```bash
curl -X POST http://localhost:8081/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePassword123!",
    "ethereum_address": "0x1234567890123456789012345678901234567890"
  }'
```

### 3. Login
```bash
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePassword123!"
  }'
```

## 📊 Database Management

### Using PgAdmin (GUI)

1. Open http://localhost:5050
2. Login with:
   - Email: `admin@nexus-security.io`
   - Password: `admin`
3. Add server:
   - Host: `postgres`
   - Port: `5432`
   - Username: `nexus_user`
   - Password: `nexus_password`

### Using Command Line

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U nexus_user -d nexus_security

# List tables
\dt

# Query users
SELECT * FROM users LIMIT 10;

# Exit
\q
```

## 🐛 Troubleshooting

### "Port already in use"

```bash
# Find what's using the port
lsof -i :8080

# Kill the process
kill -9 <PID>

# Or change port in docker-compose.yml
```

### "Container exits immediately"

```bash
# Check logs
docker-compose logs service-name

# Common issues:
# - Database not ready → Wait a few seconds and restart
# - Missing environment variables → Check .env file
# - Build errors → Run: docker-compose build service-name
```

### "Cannot connect to database"

```bash
# Ensure PostgreSQL is healthy
docker-compose ps postgres

# Should show: "healthy"
# If not, restart: docker-compose restart postgres
```

## 📚 Next Steps

- Read [DOCKER.md](./DOCKER.md) for detailed Docker documentation
- Check [README.md](./README.md) for architecture overview
- Review API documentation at `/api/v1/docs` (when implemented)

## 🆘 Getting Help

- Documentation: `./DOCKER.md`
- Issues: GitHub Issues
- Logs: `docker-compose logs -f`
