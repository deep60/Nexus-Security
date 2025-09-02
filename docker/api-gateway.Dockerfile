# Multi-stage build for Rust API Gateway
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

# Build the api-gateway in release mode
RUN cargo build --release --bin api-gateway

# Runtime stage - use slim debian for smaller image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r nexus && useradd -r -g nexus -s /bin/bash nexus

# Create necessary directories
RUN mkdir -p /app/logs /app/uploads && \
    chown -R nexus:nexus /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/api-gateway /app/api-gateway

# Copy database migrations
COPY backend/api-gateway/migrations/ /app/migrations/

# Set up working directory and permissions
WORKDIR /app
RUN chmod +x /app/api-gateway

# Switch to non-root user
USER nexus

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV PORT=8080
ENV UPLOAD_DIR=/app/uploads

# Run database migrations and start the API gateway
CMD ["./api-gateway"]


# api-gateway.Dockerfile
# This dockerfile builds the main API service with:

# Multi-stage build for optimization
# Database migration support
# Security hardening with non-root user
# Health checks for monitoring
# Upload handling capabilities
# JWT/authentication support through runtime dependencies



# Key Features of Both:

# Security-first approach - non-root users, minimal attack surface
# Production-ready - health checks, proper logging, error handling
# Optimized builds - multi-stage builds to reduce image size
# Monitoring support - health endpoints for orchestration
# Environment flexibility - configurable through environment variables
