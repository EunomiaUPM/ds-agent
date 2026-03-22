#!/bin/bash
set -euo pipefail

# -----------------------------------------------------------------------------
# Eunomia DS-Agent SERVICE START SCRIPT
# This script manages the full startup of the Eunomia DS-Agent environment:
# 1. Validates the target module (core, catalog, etc.).
# 2. Starts necessary databases using Docker Compose.
# 3. Launches the Authority, Consumer, and Provider services, each in its own
#    dedicated terminal window/tab to display live logs.
# -----------------------------------------------------------------------------

# ----------------------------
# 1. Base Directory Calculation
# Base directory (two levels up from scripts/bash, assuming script location)
# ----------------------------
BASE_DIR="$(cd "$(dirname "$0")" && pwd)/../.."

# ----------------------------
# 2. Global Configuration & Module Validation
# ----------------------------
MODULE=${1:-core}  # Default to 'core' if no module is passed
DOCKER_COMPOSE_PATH="$BASE_DIR/deployment/docker-compose.core.yaml"

echo "=== Starting Eunomia DS-Agent environment for module '$MODULE'..."

# List of valid modules that can be initialized
VALID_MODULES=("core" "catalog" "contracts" "transfer" "auth")

# Check if the provided module argument is valid
if [[ ! " ${VALID_MODULES[*]} " =~ " ${MODULE} " ]]; then
    echo "ERROR: Invalid module '$MODULE'. Valid options are: ${VALID_MODULES[*]}" >&2
    exit 1
fi

# ----------------------------
# 3. Start Databases
# ----------------------------
echo "Starting databases with Docker Compose..."
# Starts the containers and runs them in detached (background) mode.
docker-compose -f "$DOCKER_COMPOSE_PATH" up -d
echo "Waiting 5 seconds for databases to be ready..."
sleep 5

# ----------------------------
# 4. Helper function to open a new terminal window
# This function attempts to use OS-specific commands to spawn a new terminal.
# ----------------------------
spawn_terminal() {
    local title="$1"
    local command_to_run="$2"
    local run_dir="$3"

    echo "Launching $title in a new terminal window/tab..."

    # The full command to be executed INSIDE the new terminal.
    # We use 'exec bash' at the end to ensure the terminal remains open
    # after the main service command (cargo run) finishes or exits.
    local full_command="cd '$run_dir' && $command_to_run; exec bash"

    # Attempt to use operating system specific commands
    if command -v gnome-terminal &> /dev/null; then
        # Linux (GNOME)
        gnome-terminal --title="$title" --working-directory="$run_dir" -- bash -c "$full_command" &
    elif command -v konsole &> /dev/null; then
        # Linux (KDE)
        konsole --separate --caption "$title" -e bash -c "$full_command" &
    elif [[ "$(uname)" == "Darwin" ]]; then
        # macOS
        # Uses AppleScript to open a new Terminal window/tab and execute the command.
        osascript -e "tell application \"Terminal\" to do script \"$full_command\"" &
    elif command -v xterm &> /dev/null; then
        # Linux (Fallback: xterm)
        xterm -T "$title" -e bash -c "$full_command" &
    elif command -v cmd.exe &> /dev/null; then
        # Windows (WSL/Git Bash fallback - using PowerShell for better command handling)
        # This relies on cmd.exe's 'start' command to launch a PowerShell window.
        cmd.exe /C start "Eunomia DS-Agent $title" powershell -NoExit -Command "cd '$run_dir'; & $command_to_run"
    else
        echo "WARNING: Could not find a suitable terminal emulator (gnome-terminal, konsole, xterm, Terminal). Starting in background (nohup)..." >&2
        # Fallback method: runs the command in the background using nohup,
        # but the user will not see the logs easily.
        (
            cd "$run_dir"
            nohup $command_to_run > "/dev/null" 2>&1 &
        )
        echo "  -> PID: $!" >&2
    fi
}

# ----------------------------
# 5. Paths and Commands Definition
# ----------------------------
CONSUMER_DIR="$BASE_DIR/crates/$MODULE"
PROVIDER_DIR="$BASE_DIR/crates/$MODULE"

# Adjust directories for the 'core' module case
if [[ "$MODULE" == "core" ]]; then
    CONSUMER_DIR="$BASE_DIR/crates/monolith"
    PROVIDER_DIR="$BASE_DIR/crates/monolith"
fi

# Service commands to run inside the new terminals
CONSUMER_CMD="cargo run --manifest-path Cargo.toml consumer start --env-file static/envs/core.consumer.yaml"
PROVIDER_CMD="cargo run --manifest-path Cargo.toml provider start --env-file static/envs/core.provider.yaml"


# ----------------------------
# 6. Start Services
# ----------------------------

# Start Consumer service
spawn_terminal "Eunomia DS-Agent Consumer" "$CONSUMER_CMD" "$CONSUMER_DIR"

# Start Provider service
spawn_terminal "Eunomia DS-Agent Provider" "$PROVIDER_CMD" "$PROVIDER_DIR"

echo ""
echo "Eunomia DS-Agent services started. Check the new terminal windows/tabs for live logs."
echo "The main script window will now exit, but the services will remain active in their respective terminals."
