#!/bin/bash
set -e

# Parameters
ROLE=${1:-provider}
CMD=${2:-setup}

# Paths — absolute, derived from this script's location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

CONFIG_FILE="$PROJECT_ROOT/static/environment/config/core.${ROLE}.yaml"
ENV_FILE="$PROJECT_ROOT/static/vault/${ROLE}/data/vault.env"

# Validations
if [[ ! "$ROLE" =~ ^(provider|consumer)$ ]]; then
    echo "[ERROR] Invalid role. Usage: ./run.sh [provider|consumer] [setup|start]"
    exit 1
fi

if [[ ! "$CMD" =~ ^(setup|start)$ ]]; then
    echo "[ERROR] Invalid command. Usage: ./run.sh [provider|consumer] [setup|start]"
    exit 1
fi

if [ ! -f "$CONFIG_FILE" ]; then
    echo "[ERROR] Config file not found: $CONFIG_FILE"
    exit 1
fi

if [ ! -f "$ENV_FILE" ]; then
    echo "[ERROR] Secrets file not found: $ENV_FILE"
    echo "        Ensure Docker container is running and Vault is initialized."
    exit 1
fi

# Execution
echo "Running [${CMD}] for [${ROLE}]..."

LOCAL_ENV_FILE="$PROJECT_ROOT/static/vault/${ROLE}/data/local.vault.env"

set -a
source "$ENV_FILE"
if [ -f "$LOCAL_ENV_FILE" ]; then
    source "$LOCAL_ENV_FILE"
fi
set +a

cd "$PROJECT_ROOT/crates/monolith"
cargo run "$CMD" -e "$CONFIG_FILE"