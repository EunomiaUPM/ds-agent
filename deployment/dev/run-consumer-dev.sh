#!/bin/bash
set -e

# =========================
# 1. Environment variables
# =========================
export VAULT_PATH="./../../static/vault/consumer/secrets"
export VAULT_APP_DB="db.json.example"
export VAULT_APP_WALLET="wallet.json.example"
export VAULT_APP_PRIV_KEY="private_key.json.example"
export VAULT_APP_PUB_PKEY="public_key.json.example"
export VAULT_APP_CERT="cert.json.example"
export RUST_BACKTRACE="1"
export VITE_GATEWAY_PORT=1100

cleanup() {
    if [ -n "$FRONTEND_PID" ]; then
        echo -e "\n\033[0;33mStopping Vite front-end dev server (PID: $FRONTEND_PID)...\033[0m"
        kill $FRONTEND_PID 2>/dev/null || true
    fi
}
# fire cleanup
trap cleanup EXIT INT TERM


# =========================
# 2. Start dependencies
# =========================
echo -e "\033[0;36mStarting dependencies (consumer)...\033[0m"
docker compose -f docker-compose.mini.dev.consumer.yaml up -d

# =========================
# 3. Wait for DB
# =========================
echo -e "\033[0;36mWaiting for DB to be ready...\033[0m"
until docker exec consumer-db pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
done
echo -e "\033[0;32mDB ready\033[0m"


# =========================
# 4. Build frontend
# =========================
echo -e "\033[0;36mStarting React frontend in DEV mode...\033[0m"

cd ../../gui

npm install
npm run dev -w admin &
FRONTEND_PID=$!

echo -e "\033[0;32mFrontend dev server spawned\033[0m"

# =========================
# 5. Copy to backend
# =========================
echo -e "\033[0;36mCopying build to backend...\033[0m"

cd ../crates/bff

rm -rf ./src/static/admin/*
mkdir -p ./src/static/admin/dist

cp -r ../../gui/admin/dist/* ./src/static/admin/dist/

echo -e "\033[0;32mFrontend built and copied successfully\033[0m"

# 3.5. Wait for fafnir-wallet
echo -e "\033[0;36mWaiting for fafnir-wallet to be ready...\033[0m"
WALLET_URL="http://localhost:7001/readiness"   # 7002 en provider
until curl -fs "$WALLET_URL" > /dev/null 2>&1; do
    sleep 2
done
echo -e "\033[0;32mfafnir-wallet ready\033[0m"


# =========================
# 6. Backend setup
# =========================
echo -e "\033[0;36mRunning setup...\033[0m"

cd ../monolith

cargo run setup -e ../../static/environment/config/dev/dev.consumer.yaml

# =========================
# 7. Start server
# =========================
echo -e "\033[0;36mStarting consumer...\033[0m"

cargo watch -i "vault/*" -x "run start -e ../../static/environment/config/dev/dev.consumer.yaml"