FROM node:22-slim AS client-builder
WORKDIR /app/client
COPY client/package*.json ./
RUN npm ci
COPY client/ ./
RUN npm run build

FROM rust:1.94-slim-bookworm AS base
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

FROM base AS dev
RUN cargo install cargo-watch
CMD ["cargo", "watch", "-x", "run"]

FROM base AS builder
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim AS prod
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/duo /app/duo
COPY --from=client-builder /app/client/dist /app/client/dist
EXPOSE 8000
CMD ["./duo"]
