#!/bin/bash
set -e

# 1. Variables de entorno
export VAULT_PATH="./../../static/vault/provider/secrets"
export VAULT_APP_DB="db.json.example"
export VAULT_APP_WALLET="wallet.json.example"
export VAULT_APP_PRIV_KEY="private_key.json.example"
export VAULT_APP_PUB_PKEY="public_key.json.example"
export VAULT_APP_CERT="cert.json.example"
export RUST_BACKTRACE="1"

# 2. Levantar dependencias
echo -e "\033[0;36mLevantando dependencias (provider)...\033[0m"
docker-compose -f docker-compose.mini.dev.provider.yaml up -d

# 3. Esperar a que la DB esté lista
echo -e "\033[0;36mEsperando a que la DB esté lista...\033[0m"
until docker exec provider-db pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
done
echo -e "\033[0;32mDB lista\033[0m"

# 4. Setup
echo -e "\033[0;36mCompilando React...\033[0m"
cd ../../crates/bff
cargo run build -e ../../static/environment/config/dev/dev.provider.yaml
if [ $? -ne 0 ]; then
    echo -e "\033[0;31mCompilado de react fallido, abortando\033[0m"
    exit 1
fi

echo -e "\033[0;36mEjecutando setup...\033[0m"
cd ../monolith
cargo run setup -e ../../static/environment/config/dev/dev.provider.yaml
if [ $? -ne 0 ]; then
    echo -e "\033[0;31mSetup fallido, abortando\033[0m"
    exit 1
fi

# 5. Start
echo -e "\033[0;36mArrancando provider...\033[0m"
cargo watch -x "run start -e ../../static/environment/config/dev/dev.provider.yaml"