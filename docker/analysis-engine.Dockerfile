# Multi-stage build for Rust analysis engine
FROM rust:1.75-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace Cargo.toml first for better caching
COPY backend/Cargo.toml ./Cargo.toml

# Copy all backend source code
COPY backend/ .

# Build the analysis-engine in release mode
RUN cargo build --release --bin analysis-engine

# Runtime stage - use slim debian for smaller image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    curl \
    file \
    binutils \
    python3 \
    python3-pip \
    yara \
    && rm -rf /var/lib/apt/lists/*

# Install additional analysis tools
RUN pip3 install --no-cache-dir \
    pefile \
    yara-python \
    ssdeep \
    magic \
    && apt-get purge -y python3-pip \
    && apt-get autoremove -y

# Create non-root user for security
RUN groupadd -r nexus && useradd -r -g nexus -s /bin/bash nexus

# Create necessary directories
RUN mkdir -p /app/rules /app/uploads /app/logs && \
    chown -R nexus:nexus /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/analysis-engine /app/analysis-engine

# Copy YARA rules
COPY backend/analysis-engine/rules/ /app/rules/

# Set up working directory and permissions
WORKDIR /app
RUN chmod +x /app/analysis-engine

# Switch to non-root user
USER nexus

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8082/health || exit 1

# Expose port
EXPOSE 8082

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV YARA_RULES_PATH=/app/rules
ENV UPLOAD_DIR=/app/uploads

# Run the analysis engine
CMD ["./analysis-engine"]


# analysis-engine.Dockerfile
# This dockerfile builds the threat analysis service with:

# Multi-stage build to keep the final image small
# Analysis tools like YARA, pefile, ssdeep for malware detection
# Security hardening with non-root user
# Health checks for container orchestration
# YARA rules copied into the container
# File analysis capabilities with proper permissions

# Key Features of Both:

# Security-first approach - non-root users, minimal attack surface
# Production-ready - health checks, proper logging, error handling
# Optimized builds - multi-stage builds to reduce image size
# Monitoring support - health endpoints for orchestration
# Environment flexibility - configurable through environment variables


# # Build analysis engine
# docker build -f docker/analysis-engine.Dockerfile -t nexus-analysis-engine .

# # Build API gateway  
# docker build -f docker/api-gateway.Dockerfile -t nexus-api-gateway .