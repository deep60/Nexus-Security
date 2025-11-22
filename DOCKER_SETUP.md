# 🐳 Complete Docker Setup Guide

## ⚠️ What Was Wrong (and Fixed)

**The Error:**
```
error: failed to load manifest for workspace member `/app/api-gateway`
No such file or directory (os error 2)
```

**Why it happened:**
- Your backend uses **Cargo workspace** (all services share dependencies)
- The old Dockerfiles only copied ONE service at a time
- Cargo needs ALL workspace members to be present
- When building `user-service`, Cargo looked for `api-gateway`, `analysis-engine`, etc. and failed

**The Fix:**
✅ All Dockerfiles now copy the **entire workspace** before building
✅ Each service still builds independently (only compiles what it needs)
✅ Docker caching makes subsequent builds fast

---

## 🚀 How to Build & Run (Step by Step)

### Step 1: Configure Environment (ONE TIME)

```bash
cd /Users/arjun/Developer/Nexus-Security

# Copy environment template
cp .env.example .env

# Edit configuration
nano .env
```

**Change these values:**
```bash
JWT_SECRET=change-this-to-a-random-32-character-string-minimum
SMTP_USERNAME=your-email@gmail.com     # Optional, for notifications
SMTP_PASSWORD=your-app-password        # Optional
```

Save and exit (Ctrl+X, Y, Enter)

---

### Step 2: Start Databases Only (Fast)

```bash
# Start PostgreSQL and Redis first
docker-compose up -d postgres redis pgadmin

# Wait for them to be healthy (10 seconds)
sleep 10

# Verify they're running
docker-compose ps
```

You should see:
```
nexus-postgres    Up (healthy)
nexus-redis       Up (healthy)
nexus-pgadmin     Up
```

---

### Step 3: Build Services (Takes 10-15 minutes first time)

```bash
# Build all Rust services
docker-compose build api-gateway user-service

# This will take time because:
# - Downloading Rust dependencies
# - Compiling all crates
# - Optimizing for release mode
```

**Progress indicators:**
- `[builder 2/8]` = Installing system dependencies
- `[builder 5/8]` = Copying workspace files  
- `[builder 8/8]` = Compiling Rust code (SLOWEST - 5-10 min)
- `[stage-1 2/5]` = Building runtime image (fast)

---

### Step 4: Start Services

```bash
# Start API Gateway and User Service
docker-compose up -d api-gateway user-service

# Check logs
docker-compose logs -f api-gateway user-service
```

**Look for these messages:**
```
api-gateway    | Starting server on 0.0.0.0:8080
user-service   | Starting server on 0.0.0.0:8080
```

Press Ctrl+C to stop viewing logs (services keep running)

---

### Step 5: Test Everything Works

```bash
# Test API Gateway
curl http://localhost:8080/api/v1/health

# Expected response:
# {"status":"healthy","service":"api-gateway","version":"0.1.0"}

# Test User Service
curl http://localhost:8081/api/v1/health
```

---

## 🎯 Quick Commands Reference

### Check What's Running
```bash
docker-compose ps
```

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f user-service

# Last 50 lines
docker-compose logs --tail=50 api-gateway
```

### Restart Service
```bash
docker-compose restart user-service
```

### Stop Everything
```bash
docker-compose down
```

### Stop and Delete Data
```bash
# ⚠️ WARNING: Deletes database!
docker-compose down -v
```

### Rebuild After Code Changes
```bash
# Rebuild specific service
docker-compose build user-service
docker-compose up -d user-service

# Or combined
docker-compose up -d --build user-service
```

---

## 🔧 Build Only What You Need

You don't need to build all services at once. Start with essentials:

```bash
# Minimal setup (databases + API gateway + user service)
docker-compose up -d postgres redis pgadmin
docker-compose build api-gateway user-service
docker-compose up -d api-gateway user-service

# Add more services as needed
docker-compose build bounty-manager
docker-compose up -d bounty-manager
```

---

## 🐛 Troubleshooting

### Build Still Fails

```bash
# Complete cleanup
docker-compose down -v
docker system prune -a -f

# Rebuild from scratch
docker-compose build --no-cache user-service
```

### "Out of disk space"

```bash
# Remove unused Docker data
docker system prune -a

# Check disk usage
docker system df
```

### "Build is too slow"

**First build is ALWAYS slow (10-15 min)** because:
- Rust compiles everything from source
- Many dependencies to download
- Release mode optimization takes time

**Second build is FAST (1-2 min)** because:
- Docker caches compiled dependencies
- Only changed code recompiles

**Speed it up:**
- Increase Docker Desktop RAM to 8GB
- Increase CPUs to 4+ cores
- Use SSD for Docker storage

### "Port already in use"

```bash
# Find what's using port 8080
lsof -i :8080

# Kill it
kill -9 <PID>

# Or change port in docker-compose.yml
# Change "8080:8080" to "8090:8080"
```

### Service Crashes Immediately

```bash
# Check logs
docker-compose logs user-service

# Common issues:
# 1. Database not ready - Wait 10s, restart service
# 2. Missing .env - Create from .env.example  
# 3. Wrong DATABASE_URL - Check .env matches docker-compose.yml
```

---

## 📊 Expected Resource Usage

| Resource | First Build | Running |
|----------|-------------|---------|
| **Time** | 10-15 min | - |
| **CPU** | 80-100% | 5-10% |
| **RAM** | 4-6 GB | 2-3 GB |
| **Disk** | +3 GB | +500 MB |

---

## ✅ Build Success Checklist

Before building:
- [ ] Docker Desktop is running
- [ ] At least 8GB RAM allocated to Docker
- [ ] `.env` file exists with JWT_SECRET changed
- [ ] In `/Users/arjun/Developer/Nexus-Security` directory

Build process:
- [ ] Databases start successfully
- [ ] `docker-compose build` completes without errors
- [ ] Services start with `docker-compose up -d`
- [ ] `curl http://localhost:8080/api/v1/health` returns JSON

---

## 🎉 Full Startup Script

For convenience, use this complete startup:

```bash
#!/bin/bash
cd /Users/arjun/Developer/Nexus-Security

# 1. Start databases
echo "📦 Starting databases..."
docker-compose up -d postgres redis pgadmin
sleep 10

# 2. Build services
echo "🔨 Building services (this takes 10-15 min first time)..."
docker-compose build api-gateway user-service

# 3. Start services  
echo "🚀 Starting services..."
docker-compose up -d api-gateway user-service

# 4. Show status
echo "✅ Status:"
docker-compose ps

# 5. Test
echo ""
echo "🧪 Testing API:"
sleep 5
curl -s http://localhost:8080/api/v1/health | jq '.' || curl -s http://localhost:8080/api/v1/health
```

Save as `quick-start.sh`, make executable: `chmod +x quick-start.sh`, then run: `./quick-start.sh`

---

## 📚 Next Steps

After everything is running:
- Access PgAdmin: http://localhost:5050
- Test APIs with the examples in QUICKSTART.md
- Build additional services as needed
- Check logs with `docker-compose logs -f`

