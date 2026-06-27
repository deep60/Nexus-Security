FROM rust:1.88-slim-bookworm AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy entire workspace (required for Cargo workspace)
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

# Build only this service
RUN cargo build --release --bin api-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5 && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 app
WORKDIR /app
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/app
# Contract ABIs are read at runtime relative to the working directory.
COPY --from=builder /app/api-gateway/abis ./abis
RUN chown -R app:app /usr/local/bin/app /app
USER app
EXPOSE 8080
CMD ["/usr/local/bin/app"]
