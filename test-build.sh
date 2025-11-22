#!/bin/bash

echo "🧪 Testing Docker Build Fix..."
echo ""

cd /Users/arjun/Developer/Nexus-Security

# Step 1: Check .env exists
if [ ! -f .env ]; then
    echo "⚠️  Creating .env file..."
    cp .env.example .env
    echo "✅ .env created (you should edit JWT_SECRET later)"
else
    echo "✅ .env exists"
fi

# Step 2: Start databases
echo ""
echo "📦 Starting PostgreSQL and Redis..."
docker-compose up -d postgres redis

echo "⏳ Waiting for databases to be healthy (15 seconds)..."
sleep 15

# Step 3: Test build ONE service
echo ""
echo "🔨 Testing build with user-service (this will take 5-10 minutes)..."
echo "    Progress: Downloading dependencies → Compiling → Optimizing"
docker-compose build user-service

# Step 4: Check if build succeeded
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ BUILD SUCCESSFUL!"
    echo ""
    echo "🚀 Now you can start the service:"
    echo "   docker-compose up -d user-service"
    echo ""
    echo "📊 Check status:"
    echo "   docker-compose ps"
    echo ""
    echo "🔍 View logs:"
    echo "   docker-compose logs -f user-service"
else
    echo ""
    echo "❌ BUILD FAILED"
    echo ""
    echo "Check logs above for errors"
    echo "Common fixes:"
    echo "  1. docker system prune -a -f"
    echo "  2. docker-compose build --no-cache user-service"
fi
