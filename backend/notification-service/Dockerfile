FROM rust:1.85-slim-bookworm as builder
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
RUN cargo build --release --bin notification-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5 && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 app
COPY --from=builder /app/target/release/notification-service /usr/local/bin/app
RUN chown -R app:app /usr/local/bin/app
USER app
EXPOSE 8080
CMD ["/usr/local/bin/app"]
