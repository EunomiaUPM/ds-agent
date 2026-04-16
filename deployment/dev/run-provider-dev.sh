#!/bin/bash
set -e

# 1. Environment variables
export VAULT_PATH="./../../static/vault/provider/secrets"
export VAULT_APP_DB="db.json.example"
export VAULT_APP_WALLET="wallet.json.example"
export VAULT_APP_PRIV_KEY="private_key.json.example"
export VAULT_APP_PUB_PKEY="public_key.json.example"
export VAULT_APP_CERT="cert.json.example"
export RUST_BACKTRACE="1"
export VITE_GATEWAY_PORT=1200

cleanup() {
    if [ -n "$FRONTEND_PID" ]; then
        echo -e "\n\033[0;33mStopping Vite front-end dev server (PID: $FRONTEND_PID)...\033[0m"
        kill $FRONTEND_PID 2>/dev/null || true
    fi
}
# fire cleanup
trap cleanup EXIT INT TERM


# 2. Start dependencies
echo -e "\033[0;36mStarting dependencies (provider)...\033[0m"
docker compose -f docker-compose.mini.dev.provider.yaml up -d

# 3. Wait for DB
echo -e "\033[0;36mWaiting for DB to be ready...\033[0m"
until docker exec provider-db pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
done
echo -e "\033[0;32mDB ready\033[0m"

# 4. Build frontend
echo -e "\033[0;36mStarting React frontend in DEV mode...\033[0m"

cd ../../gui

npm install
VITE_GATEWAY_PORT=1200 npm run dev -w admin &
FRONTEND_PID=$!

echo -e "\033[0;32mFrontend dev server spawned\033[0m"


# 5. Backend setup
echo -e "\033[0;36mRunning setup...\033[0m"
cd ../crates/monolith

cargo run setup -e ../../static/environment/config/dev/dev.provider.yaml
if [ $? -ne 0 ]; then
    echo -e "\033[0;31mSetup failed, aborting\033[0m"
    exit 1
fi

# 6. Start
echo -e "\033[0;36mStarting provider...\033[0m"
cargo watch -x "run start -e ../../static/environment/config/dev/dev.provider.yaml"