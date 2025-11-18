#!/bin/bash

# Nexus Security - Docker Push Script
# This script pushes all Docker images to the container registry

set -e  # Exit on error

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DOCKER_REGISTRY="${DOCKER_REGISTRY:-nexus-security}"
VERSION="${VERSION:-latest}"

echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}   Nexus Security - Docker Push Script${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
echo -e "Registry: ${YELLOW}${DOCKER_REGISTRY}${NC}"
echo -e "Version:  ${YELLOW}${VERSION}${NC}"
echo ""

# Check if logged in to registry
if [ "${DOCKER_REGISTRY}" != "nexus-security" ]; then
    echo -e "${YELLOW}Checking Docker registry login...${NC}"
    if ! docker info | grep -q "Username"; then
        echo -e "${RED}Not logged in to Docker registry${NC}"
        echo -e "Please login first: ${YELLOW}docker login${NC}"
        exit 1
    fi
    echo -e "${GREEN} Logged in to Docker registry${NC}"
    echo ""
fi

# Function to push a service image
push_service() {
    local service=$1
    local image_name="${DOCKER_REGISTRY}/${service}:${VERSION}"

    echo -e "${YELLOW}Pushing ${service}...${NC}"

    # Check if image exists
    if ! docker images | grep -q "${DOCKER_REGISTRY}/${service}"; then
        echo -e "${RED} Image ${service} not found. Please build first.${NC}"
        return 1
    fi

    if docker push "${image_name}"; then
        echo -e "${GREEN} Successfully pushed ${service}${NC}"

        # Also push latest tag if not already latest
        if [ "${VERSION}" != "latest" ]; then
            docker push "${DOCKER_REGISTRY}/${service}:latest"
            echo -e "${GREEN} Pushed ${service}:latest${NC}"
        fi

        return 0
    else
        echo -e "${RED} Failed to push ${service}${NC}"
        return 1
    fi
}

# List of services to push
SERVICES=(
    "postgres"
    "api-gateway"
    "analysis-engine"
    "bounty-manager"
    "frontend"
)

# Push all services
echo -e "${GREEN}Pushing images to registry...${NC}"
echo ""

FAILED_SERVICES=()
for service in "${SERVICES[@]}"; do
    if ! push_service "$service"; then
        FAILED_SERVICES+=("$service")
    fi
    echo ""
done

# Summary
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}   Push Summary${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""

if [ ${#FAILED_SERVICES[@]} -eq 0 ]; then
    echo -e "${GREEN}All images pushed successfully!${NC}"
    echo ""
    echo -e "Images pushed:"
    for service in "${SERVICES[@]}"; do
        echo -e "  ${GREEN}${NC} ${DOCKER_REGISTRY}/${service}:${VERSION}"
    done
else
    echo -e "${RED}Some images failed to push:${NC}"
    for service in "${FAILED_SERVICES[@]}"; do
        echo -e "  ${RED}${NC} ${service}"
    done
    echo ""
    exit 1
fi

echo ""
echo -e "To deploy to Kubernetes:"
echo -e "  ${YELLOW}kubectl apply -f infrastructure/kubernetes/${NC}"
echo ""
