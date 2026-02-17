# ----------------------------
# Stage 1: Build Frontend
# ----------------------------
FROM node:20-slim AS frontend-builder

WORKDIR /app/gui/admin

# Copiar solo package.json y package-lock.json primero (npm install)
COPY gui/admin/package*.json ./

RUN npm install --legacy-peer-deps

# Copiar todo el frontend
COPY gui/admin ./

# Build React
RUN npm run build


# ----------------------------
# Stage 2: Build Rust backend (rainbow-monolith)
# ----------------------------
FROM rust:1.81-slim-bookworm AS backend-builder

WORKDIR /app

# Instalar dependencias de compilación
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copiar el backend completo
COPY . .

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
COPY --from=backend-builder /app/rainbow-monolith/target/release/rainbow-monolith /app/rainbow-monolith

# Exponer puerto por defecto (ajustar si es otro)
EXPOSE 1500

# Variable de entorno opcional para log
ENV RUST_LOG=info

# ENTRYPOINT fijo al binario
ENTRYPOINT ["/app/rainbow-monolith"]

# CMD por defecto usando archivo config por defecto, pero permite sobrescribir
CMD ["setup", "start", "--env-file", "/app/static/environment/envs/.env"]
