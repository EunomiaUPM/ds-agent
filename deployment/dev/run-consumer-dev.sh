#!/bin/bash
set -e

# 1. Variables de entorno
export VAULT_PATH="./../../static/vault/consumer/secrets"
export VAULT_APP_DB="db.json.example"
export VAULT_APP_WALLET="wallet.json.example"
export VAULT_APP_PRIV_KEY="private_key.json.example"
export VAULT_APP_PUB_PKEY="public_key.json.example"
export VAULT_APP_CERT="cert.json.example"
export RUST_BACKTRACE="1"

# 2. Levantar dependencias
echo -e "\033[0;36mLevantando dependencias (consumer)...\033[0m"
docker-compose -f docker-compose.mini.dev.consumer.yaml up -d

# 3. Esperar a que la DB esté lista
echo -e "\033[0;36mEsperando a que la DB esté lista...\033[0m"
until docker exec consumer-db pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
done
echo -e "\033[0;32mDB lista\033[0m"

# 4. Setup
echo -e "\033[0;36mEjecutando setup...\033[0m"
cd ../../crates/monolith
cargo run setup -e ../../static/environment/config/dev/dev.consumer.yaml
if [ $? -ne 0 ]; then
    echo -e "\033[0;31mSetup fallido, abortando\033[0m"
    exit 1
fi

# 5. Start
echo -e "\033[0;36mArrancando consumer...\033[0m"
cargo watch -x "run start -e ../../static/environment/config/dev/dev.consumer.yaml"