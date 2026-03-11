#!/bin/bash
# Vault Teardown Script
# Cleans up generated Vault data while preserving example files and directory structure

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATIC_VAULT_DIR="${SCRIPT_DIR}/../../static/vault"

ROLES=("authority" "consumer" "provider")

echo "🧹 Vault Teardown Script"
echo "========================"

for role in "${ROLES[@]}"; do
    ROLE_DIR="${STATIC_VAULT_DIR}/${role}"
    
    if [ ! -d "$ROLE_DIR" ]; then
        echo "⚠️  Skipping ${role}: directory not found"
        continue
    fi
    
    echo ""
    echo "📁 Cleaning ${role}..."
    
    # Clean data directory (Vault's internal storage)
    DATA_DIR="${ROLE_DIR}/data"
    if [ -d "$DATA_DIR" ]; then
        echo "   - Wiping data directory..."
        rm -rf "${DATA_DIR:?}"/*
        # Create empty vault.env placeholder
        touch "${DATA_DIR}/vault.env"
        echo "   - Created empty vault.env"
    fi
    
    # Clean secrets directory (keep .example files)
    SECRETS_DIR="${ROLE_DIR}/secrets"
    if [ -d "$SECRETS_DIR" ]; then
        echo "   - Cleaning secrets (keeping .example files)..."
        find "$SECRETS_DIR" -type f ! -name "*.example" -delete
    fi
    
    echo "   ✅ ${role} cleaned"
done

echo ""
echo "✅ Teardown complete!"
echo ""
echo "To start fresh, run:"
echo "  docker compose -f deployment/docker-compose.dev.yaml up -d"
