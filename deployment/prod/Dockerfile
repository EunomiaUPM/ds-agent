# ----------------------------
# Stage 1: Build Frontend
# ----------------------------
FROM node:20-slim AS frontend-builder

WORKDIR /app

# Copiar package.json para monorepo
COPY gui/package*.json ./gui/
COPY gui/admin/package*.json ./gui/admin/
COPY gui/shared/package*.json ./gui/shared/
COPY gui/business/package*.json ./gui/business/

# Instalar dependencias del monorepo
WORKDIR /app/gui
RUN npm install --legacy-peer-deps

# Copiar código fuente completo
COPY gui/admin ./admin
COPY gui/shared ./shared
# COPY gui/business ./business

# Build React + TypeScript
WORKDIR /app/gui/admin
RUN npm run build


# ----------------------------
# Stage 2: Build Rust backend (rainbow-monolith)
# ----------------------------
FROM rust:alpine3.23 AS backend-builder

WORKDIR /app

# Copiar el backend completo
COPY . .
RUN apk add --no-cache \
    protobuf \
    protobuf-dev \
    musl-dev \
    build-base

    # Copiar build del frontend al path esperado por rust_embed
COPY --from=frontend-builder /app/gui/admin/dist ./rainbow-monolith/src/static/admin/dist

# Build final del monolito
RUN cargo build --release --manifest-path rainbow-monolith/Cargo.toml


# ----------------------------
# Stage 3: Runtime minimal
# ----------------------------
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copiar binario release
COPY --from=backend-builder /app/target/release/rainbow_monolith /app/rainbow-monolith

# Exponer puerto por defecto (ajustar si es otro)
EXPOSE 1500

# Variable de entorno opcional para log
ENV RUST_LOG=info

ENV CONFIG_PATH=/app/static/environment/config/prod/core.consumer.yaml

ENTRYPOINT ["/bin/sh", "-c"]
CMD ["/app/rainbow-monolith setup --env-file $CONFIG_PATH && /app/rainbow-monolith start --env-file $CONFIG_PATH"]
