# ----------------------------
# Stage 1: Build Frontend (React)
# ----------------------------
FROM node:20-slim AS frontend-builder
WORKDIR /app

# Copiar archivos necesarios para instalar dependencias
COPY gui/package*.json ./gui/
COPY gui/admin/package*.json ./gui/admin/
COPY gui/shared/package*.json ./gui/shared/
COPY gui/business/package*.json ./gui/business/

WORKDIR /app/gui
RUN npm install --legacy-peer-deps

# Copiar el resto del código del frontend
COPY gui/admin ./admin
COPY gui/shared ./shared

# Construir la aplicación React
WORKDIR /app/gui/admin
RUN npm run build

# ----------------------------
# Stage 2: Build Rust Backend
# ----------------------------
FROM rust:alpine3.23 AS backend-builder
WORKDIR /app

# Instalar dependencias de compilación para Alpine (musl)
RUN apk add --no-cache \
    protobuf \
    protobuf-dev \
    musl-dev \
    build-base

# 1. Copiamos todo el código fuente del backend
COPY . .

# 2. IMPORTANTE: Copiamos los archivos estáticos generados en el Stage 1
# a la ruta exacta donde RustEmbed los busca según tu código:
# #[folder = "src/static/admin/dist"]
RUN mkdir -p rainbow-monolith/src/static/admin/dist
COPY --from=frontend-builder /app/gui/admin/dist ./rainbow-monolith/src/static/admin/dist

# 3. Compilamos el binario (ahora incluirá el frontend dentro)
RUN cargo build --release --manifest-path rainbow-monolith/Cargo.toml

# ----------------------------
# Stage 3: Runtime minimal
# ----------------------------
# Usamos Alpine para coherencia con el builder y menor tamaño
FROM alpine:3.23

WORKDIR /app

# Instalar librerías mínimas necesarias para ejecutar el binario
RUN apk add --no-cache libgcc libstdc++ ca-certificates

# Copiar el binario único (que ya contiene el frontend)
COPY --from=backend-builder /app/target/release/rainbow_monolith /app/rainbow-monolith

# Exponer puerto
EXPOSE 1500

# Variables de entorno
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/static/environment/config/prod/core.consumer.yaml

# Comando de entrada corregido (usando directamente el binario)
ENTRYPOINT ["/bin/sh", "-c"]
CMD ["/app/monolith setup --env-file $CONFIG_PATH && /app/monolith start --env-file $CONFIG_PATH"]