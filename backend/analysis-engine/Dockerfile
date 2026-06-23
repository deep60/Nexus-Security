FROM rust:1.88-slim-bookworm AS builder

# Install build dependencies. YARA is installed from Debian bookworm-backports
# (libyara 4.5.x) instead of compiling from source, which avoids the autoconf
# "PKG_CHECK_MODULES / PROTOBUF_C" bootstrap failure.
RUN echo "deb http://deb.debian.org/debian bookworm-backports main" \
        > /etc/apt/sources.list.d/backports.list && \
    apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    make \
    gcc \
    && apt-get install -y -t bookworm-backports libyara-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy entire workspace
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY api-gateway ./api-gateway
COPY analysis-engine ./analysis-engine
COPY bounty-manager ./bounty-manager
COPY consensus-service ./consensus-service
COPY notification-service ./notification-service
COPY payment-service ./payment-service
COPY reputation-service ./reputation-service
COPY submission-service ./submission-service
COPY user-service ./user-service

# Build analysis-engine
RUN cargo build --release --bin analysis-engine

FROM debian:bookworm-slim

# Install runtime dependencies including the YARA shared library from
# bookworm-backports (matches the build stage).
RUN echo "deb http://deb.debian.org/debian bookworm-backports main" \
        > /etc/apt/sources.list.d/backports.list && \
    apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && apt-get install -y -t bookworm-backports libyara10 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
COPY --from=builder /app/target/release/analysis-engine /usr/local/bin/app
RUN chown -R app:app /usr/local/bin/app
USER app
EXPOSE 8080
CMD ["/usr/local/bin/app"]
