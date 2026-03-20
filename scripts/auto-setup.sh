#!/bin/bash
set -euo pipefail

# -----------------------------------------------------------------------------
# RAINBOW ENVIRONMENT SETUP SCRIPT
# This script initializes the environment for the Rainbow project. It performs
# the following steps:
# 1. Determines the base directory of the project.
# 2. Validates the target module (e.g., core, catalog, etc.).
# 3. Restarts the required databases using Docker Compose.
# 4. Executes the necessary 'setup' commands for Authority, Consumer, and Provider.
# Usage: ./rainbow_setup.sh [module_name] (e.g., ./rainbow_setup.sh core)
# -----------------------------------------------------------------------------

# ----------------------------
# 1. Base Directory Calculation
# Base directory (two levels up from scripts/bash, assuming script location)
# ----------------------------
BASE_DIR="$(cd "$(dirname "$0")" && pwd)/../.."

# ----------------------------
# 2. Global Configuration & Module Validation
# ----------------------------
# Default to 'core' module if no argument is passed
MODULE=${1:-core}
# Define the path to the main docker-compose file for core services
DOCKER_COMPOSE_PATH="$BASE_DIR/deployment/docker-compose.core.yaml"

echo "=== Starting Rainbow environment setup for module '$MODULE'..."

# List of valid modules that can be initialized
VALID_MODULES=("core" "catalog" "contracts" "transfer" "auth")

# Check if the provided module argument is valid
# shellcheck disable=SC2076
if [[ ! " ${VALID_MODULES[*]} " =~ " ${MODULE} " ]]; then
    echo "ERROR: Invalid module '$MODULE'. Valid options are: ${VALID_MODULES[*]}" >&2
    exit 1
fi


# ----------------------------
# 3. Start and Wait for Databases
# ----------------------------
echo ""
echo "--- Restarting databases with Docker Compose ---"
# 'down -v' stops and removes containers, and removes associated volumes
docker-compose -f "$DOCKER_COMPOSE_PATH" down -v
# 'up -d' recreates and starts services in detached mode
docker-compose -f "$DOCKER_COMPOSE_PATH" up -d
echo "Waiting 5 seconds for databases to stabilize..."
sleep 5


# ----------------------------
# 5. Consumer Setup
# The setup command structure depends on whether 'core' or a specific module is targeted.
# ----------------------------
echo ""
if [[ "$MODULE" == "core" ]]; then
  echo "--- Running setup for Consumer (all core modules) ---"
  cargo run --manifest-path "$BASE_DIR/rainbow-core/Cargo.toml" consumer setup \
    --env-file "$BASE_DIR/static/envs/.env.consumer.core"
else
  echo "--- Running setup for Consumer module: $MODULE ---"
  # Targets the specific module's manifest file (e.g., rainbow-catalog/Cargo.toml)
  cargo run --manifest-path "$BASE_DIR/rainbow-$MODULE/Cargo.toml" consumer setup \
    --env-file "$BASE_DIR/static/envs/.env.consumer.core"
fi


# ----------------------------
# 6. Provider Setup
# Similar to Consumer setup, handles 'core' vs. specific module targeting.
# ----------------------------
echo ""
if [[ "$MODULE" == "core" ]]; then
  echo "--- Running setup for Provider (all core modules) ---"
  cargo run --manifest-path "$BASE_DIR/rainbow-core/Cargo.toml" provider setup \
    --env-file "$BASE_DIR/static/envs/core.consumer.yaml"
else
  echo "--- Running setup for Provider module: $MODULE ---"
  # Targets the specific module's manifest file (e.g., rainbow-catalog/Cargo.toml)
  cargo run --manifest-path "$BASE_DIR/rainbow-$MODULE/Cargo.toml" provider setup \
    --env-file "$BASE_DIR/static/envs/core.consumer.yaml"
fi

echo ""
echo "================================================================"
echo "Setup completed successfully for module '$MODULE'."
echo "================================================================"
