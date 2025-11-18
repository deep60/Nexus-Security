#!/bin/bash

# Nexus Security - Docker Build Script
# This script builds all Docker images for the Nexus Security platform

set -e  # Exit on error

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DOCKER_REGISTRY="${DOCKER_REGISTRY:-nexus-security}"
VERSION="${VERSION:-latest}"
BUILD_MODE="${BUILD_MODE:-production}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}   Nexus Security - Docker Build Script${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
echo -e "Registry: ${YELLOW}${DOCKER_REGISTRY}${NC}"
echo -e "Version:  ${YELLOW}${VERSION}${NC}"
echo -e "Mode:     ${YELLOW}${BUILD_MODE}${NC}"
echo ""

cd "$PROJECT_ROOT"

# Function to build a service
build_service() {
    local service=$1
    local dockerfile=$2
    local image_name="${DOCKER_REGISTRY}/${service}:${VERSION}"

    echo -e "${YELLOW}Building ${service}...${NC}"

    if docker build \
        -f "${dockerfile}" \
        -t "${image_name}" \
        --build-arg BUILD_MODE="${BUILD_MODE}" \
        .; then
        echo -e "${GREEN} Successfully built ${service}${NC}"

        # Also tag as latest if not already
        if [ "${VERSION}" != "latest" ]; then
            docker tag "${image_name}" "${DOCKER_REGISTRY}/${service}:latest"
            echo -e "${GREEN} Tagged ${service} as latest${NC}"
        fi

        return 0
    else
        echo -e "${RED} Failed to build ${service}${NC}"
        return 1
    fi
}

# Build services
echo -e "${GREEN}Building backend services...${NC}"
echo ""

# PostgreSQL
build_service "postgres" "infrastructure/docker/postgress.Dockerfile"

# API Gateway
build_service "api-gateway" "infrastructure/docker/production/api-gateway/api-gateway.Dockerfile"

# Analysis Engine
build_service "analysis-engine" "infrastructure/docker/production/analysis-engine/analysis-engine.Dockerfile"

# Bounty Manager
build_service "bounty-manager" "infrastructure/docker/production/bounty-manager/bounty-manager.Dockerfile"

# Frontend
echo ""
echo -e "${GREEN}Building frontend...${NC}"
echo ""
build_service "frontend" "infrastructure/docker/production/frontend/frontend.Dockerfile"

# Summary
echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}   Build Summary${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
docker images | grep "${DOCKER_REGISTRY}" | grep "${VERSION}"
echo ""
echo -e "${GREEN}All images built successfully!${NC}"
echo ""
echo -e "To run the services locally:"
echo -e "  ${YELLOW}docker-compose -f infrastructure/docker/development/docker-compose.yml up${NC}"
echo ""
echo -e "To run in production mode:"
echo -e "  ${YELLOW}docker-compose -f infrastructure/docker/production/docker-compose.prod.yml up -d${NC}"
echo ""
echo -e "To push images to registry:"
echo -e "  ${YELLOW}./infrastructure/docker/scripts/push.sh${NC}"
echo ""
