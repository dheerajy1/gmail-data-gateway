# Builds the jobs-api-gateway API as one standalone binary.
# The build context is the repo root, see docker/docker-compose.yml.
# public/global.css is committed, so no Tailwind CLI is needed here. Run cargo css and commit it after a style change.

FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY public ./public
COPY xtask ./xtask
COPY subscriber ./subscriber

RUN cargo build --release --package jobs-api-gateway

FROM debian:bookworm-slim

# CA certificates let the API trust the Azure SQL server and Google certificates
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/main /app/main

# The API listens on this port outside Vercel, it matches APP_PORT in .env.production
ENV VERCEL_DEV_PORT=8091

EXPOSE 8091

CMD ["/app/main"]
