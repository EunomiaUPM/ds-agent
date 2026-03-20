#!/bin/bash
# start-dev.sh — Bash equivalent of start-dev.ps1
# Starts the dev Docker Compose stack, waits for vault setup to complete,
# then force-recreates heimdall so it picks up the freshly generated vault.env.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# docker-compose must run from deployment/ so ${PWD} resolves vault paths correctly
COMPOSE_DIR="$SCRIPT_DIR/.."

if ! command -v docker &>/dev/null; then
    echo "[ERROR] Docker not found." >&2
    exit 1
fi

# Use 'docker compose' (v2) or fall back to 'docker-compose' (v1)
if docker compose version &>/dev/null 2>&1; then
    DC="docker compose"
else
    DC="docker-compose"
fi

run_compose() {
    (cd "$COMPOSE_DIR" && $DC -f docker-compose.dev.yaml "$@")
}

echo "=== Eunomia Dev Environment Setup ==="

# 1. Start infrastructure (DBs, Redis, IDM, Vault servers)
#    Vault server containers MUST be (re)started before the setup containers run,
#    so they reflect the current state of the data directory (empty after teardown).
echo "[1/4] Starting infrastructure..."
run_compose up -d --remove-orphans

echo "[1/4] Restarting vault servers to align with current data directories..."
run_compose restart heimdall-vault provider_vault consumer_vault
sleep 3   # give vault processes time to come up after restart

# 2. Force-run vault setup containers (recreate so they always execute)
echo "[2/4] Running vault setup..."
run_compose up -d --force-recreate \
    heimdall_vault_setup \
    provider_vault_setup \
    consumer_vault_setup

# 3. Wait for vault setup containers to finish
echo "[3/4] Waiting for vault setup to complete (up to 2 minutes)..."

wait_for_container() {
    local container="$1"
    local max=120
    local count=0
    printf "  Waiting for %s" "$container"
    while [ $count -lt $max ]; do
        status=$(docker inspect -f '{{.State.Status}}' "$container" 2>/dev/null || echo "missing")
        if [ "$status" = "exited" ]; then
            exit_code=$(docker inspect -f '{{.State.ExitCode}}' "$container" 2>/dev/null || echo "1")
            if [ "$exit_code" = "0" ]; then
                echo " ✓"
                return 0
            else
                echo " ✗ (exit code $exit_code)"
                docker logs "$container" 2>&1 | tail -10 >&2
                return 1
            fi
        fi
        printf "."
        sleep 1
        count=$((count + 1))
    done
    echo " timed out"
    return 1
}

wait_for_container "heimdall_vault_setup"
wait_for_container "provider_vault_setup"
wait_for_container "consumer_vault_setup"

# Small buffer for file sync
sleep 2

# 4. Force-recreate heimdall so Docker re-reads the now-populated vault.env
echo "[4/4] Recreating heimdall with fresh vault.env..."
run_compose up -d --force-recreate heimdall

echo ""
echo "=== Dev environment ready ==="
echo "Attaching logs (Ctrl+C to stop)..."
run_compose logs -f
