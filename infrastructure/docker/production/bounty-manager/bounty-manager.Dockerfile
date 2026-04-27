# Multi-stage build for Rust Bounty Manager Service
FROM rust:1.82-slim as builder

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

# Build the bounty-manager in release mode
RUN cargo build --release --bin bounty-manager

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
RUN groupadd -r verdyx && useradd -r -g verdyx -s /bin/bash verdyx

# Create necessary directories
RUN mkdir -p /app/logs && \
    chown -R verdyx:verdyx /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/bounty-manager /app/bounty-manager

# Set up working directory and permissions
WORKDIR /app
RUN chmod +x /app/bounty-manager

# Switch to non-root user
USER verdyx

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8083/health || exit 1

# Expose port
EXPOSE 8083

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV PORT=8083

# Run the bounty manager
CMD ["./bounty-manager"]
