# syntax=docker/dockerfile:1
# Build context must be the ds-agent root directory:
#   docker build -f deployment/Dockerfile .

ARG RUST_VERSION=1.88.0

# =========================
# Stage 1: Build Frontend
# =========================
FROM node:20-slim AS frontend-builder

WORKDIR /app

# Install dependencies (package files only for cache efficiency)
COPY gui/package*.json ./gui/
COPY gui/admin/package*.json ./gui/admin/
COPY gui/shared/package*.json ./gui/shared/

WORKDIR /app/gui
RUN npm install

# Copy source and build
COPY gui/admin ./admin
COPY gui/shared ./shared

RUN npm run build -w admin

# =========================
# Stage 2: Build Backend
# =========================
FROM rust:${RUST_VERSION}-slim-bookworm AS builder

WORKDIR /app

# System dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev curl protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# Copy Rust workspace manifests
COPY Cargo.toml Cargo.lock ./

# Copy all crates
COPY crates/ ./crates/

# Inject frontend build into the bff crate's static dir (rust-embed picks it up at compile time)
COPY --from=frontend-builder /app/gui/admin/dist ./crates/bff/src/static/admin/dist/

# Build the monolith binary (with layer caches for registry and target)
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    set -e; \
    cargo build --locked --release -p monolith; \
    cp ./target/release/monolith /bin/monolith

# =========================
# Stage 3: Runtime image
# =========================
FROM debian:bookworm-slim AS final

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates wget && \
    rm -rf /var/lib/apt/lists/*

# Non-privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

# Copy binary
COPY --from=builder /bin/monolith /usr/local/bin/monolith
RUN chmod +x /usr/local/bin/monolith

USER appuser

# Entrypoint: subcommand (setup / start) and -e <config> are passed by docker-compose
ENTRYPOINT ["/usr/local/bin/monolith"]