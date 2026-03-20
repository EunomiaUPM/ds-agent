# syntax=docker/dockerfile:1
# Build context must be the PARENT directory of ds-protocol (to include ymir):
#   docker build -f ds-protocol/deployment/Dockerfile --build-arg APP_NAME=... ..

ARG RUST_VERSION=1.88.0
ARG APP_NAME=monolith

# --- builder ---
FROM rust:${RUST_VERSION}-slim-bookworm AS builder
ARG APP_NAME

WORKDIR /app

# system dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev curl protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# app building
# https://docs.docker.com/guides/rust/develop/
# source paths are relative to the build context (parent of ds-protocol)
RUN --mount=type=bind,source=ds-protocol/Cargo.toml,target=/app/Cargo.toml \
    --mount=type=bind,source=ds-protocol/Cargo.lock,target=/app/Cargo.lock \
    --mount=type=bind,source=ds-protocol/crates/monolith,target=/app/crates/monolith \
    --mount=type=bind,source=ds-protocol/crates/catalog-agent,target=/app/crates/catalog-agent \
    --mount=type=bind,source=ds-protocol/crates/negotiation-agent,target=/app/crates/negotiation-agent \
    --mount=type=bind,source=ds-protocol/crates/transfer-agent,target=/app/crates/transfer-agent \
    --mount=type=bind,source=ds-protocol/crates/dataplane,target=/app/crates/dataplane \
    --mount=type=bind,source=ds-protocol/crates/common,target=/app/crates/common \
    --mount=type=bind,source=ds-protocol/crates/auth,target=/app/crates/auth \
    --mount=type=bind,source=ds-protocol/crates/events,target=/app/crates/events \
    --mount=type=bind,source=ds-protocol/crates/bff,target=/app/crates/bff \
    --mount=type=bind,source=ds-protocol/crates/connector,target=/app/crates/connector \
    --mount=type=bind,source=ds-protocol/static,target=/app/static \
    --mount=type=bind,source=ymir,target=/ymir \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    /bin/sh -c "\
    set -e; \
    cargo build --locked --release -p ${APP_NAME}; \
    cp ./target/release/${APP_NAME} /bin/${APP_NAME}; \
"

# --- image ---
FROM debian:bookworm-slim AS final
ARG APP_NAME
ENV APP_NAME=${APP_NAME}


# update on image
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# not privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

# binary copy
COPY --from=builder /bin/${APP_NAME} /usr/local/bin/

# user permission
RUN chmod +x /usr/local/bin/${APP_NAME}
USER appuser

# entrypoint
ENTRYPOINT ["/usr/local/bin/${APP_NAME}"]