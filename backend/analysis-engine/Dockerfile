FROM rust:1.85-slim-bookworm as builder

# Install build dependencies including YARA
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    automake \
    libtool \
    make \
    gcc \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install YARA from source
RUN wget https://github.com/VirusTotal/yara/archive/v4.5.0.tar.gz && \
    tar -zxf v4.5.0.tar.gz && \
    cd yara-4.5.0 && \
    ./bootstrap.sh && \
    ./configure && \
    make && \
    make install && \
    ldconfig && \
    cd .. && \
    rm -rf yara-4.5.0 v4.5.0.tar.gz

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

# Install runtime dependencies including YARA
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    automake \
    libtool \
    make \
    gcc \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install YARA from source (runtime)
RUN wget https://github.com/VirusTotal/yara/archive/v4.5.0.tar.gz && \
    tar -zxf v4.5.0.tar.gz && \
    cd yara-4.5.0 && \
    ./bootstrap.sh && \
    ./configure && \
    make && \
    make install && \
    ldconfig && \
    cd .. && \
    rm -rf yara-4.5.0 v4.5.0.tar.gz

RUN useradd -m -u 1000 app
COPY --from=builder /app/target/release/analysis-engine /usr/local/bin/app
RUN chown -R app:app /usr/local/bin/app
USER app
EXPOSE 8080
CMD ["/usr/local/bin/app"]
