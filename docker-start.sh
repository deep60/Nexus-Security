#!/bin/bash

# Verdyx - Docker Startup Script

set -e

echo "🚀 Starting Verdyx Platform..."

# Check if .env file exists
if [ ! -f .env ]; then
    echo "⚠️  .env file not found. Creating from .env.example..."
    cp .env.example .env
    echo "📝 Please edit .env file with your actual configuration values"
    echo "Press Enter to continue or Ctrl+C to exit and edit .env first"
    read
fi

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

echo "🐳 Docker is running"

# Pull latest images
echo "📦 Pulling latest base images..."
docker-compose pull postgres redis pgadmin

# Build services
echo "🔨 Building Verdyx services..."
docker-compose build

# Start services
echo "▶️  Starting services..."
docker-compose up -d

# Wait for services to be healthy
echo "⏳ Waiting for services to be healthy..."
sleep 5

# Check service health
echo "🏥 Checking service health..."
docker-compose ps

echo ""
echo "✅ Verdyx Platform is running!"
echo ""
echo "📊 Service URLs:"
echo "   - API Gateway:         http://localhost:8080"
echo "   - User Service:        http://localhost:8081"
echo "   - PgAdmin:             http://localhost:5050"
echo "   - PostgreSQL:          localhost:5432"
echo "   - Redis:               localhost:6379"
echo ""
echo "📝 To view logs:"
echo "   docker-compose logs -f [service-name]"
echo ""
echo "🛑 To stop all services:"
echo "   docker-compose down"
echo ""
echo "🗑️  To stop and remove all data:"
echo "   docker-compose down -v"
echo ""
