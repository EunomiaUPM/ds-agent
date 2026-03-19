#!/bin/sh
set -e

# Use a reliable mirror instead of the default CDN
cat > /etc/apk/repositories << EOF
https://mirrors.aliyun.com/alpine/v3.21/main
https://mirrors.aliyun.com/alpine/v3.21/community
EOF

# Install minimal dependencies
apk add --no-cache libcap unzip ca-certificates openssl jq curl

# Download and install Vault binary if not present
if [ ! -f "/usr/bin/vault" ]; then
    # Detect architecture to download correct binary
    ARCH="amd64"
    [ "$(uname -m)" = "aarch64" ] && ARCH="arm64"

    # Download to /tmp to avoid volume conflicts
    cd /tmp
    wget -q "https://releases.hashicorp.com/vault/1.13.3/vault_1.13.3_linux_${ARCH}.zip"
    unzip -o -q "vault_1.13.3_linux_${ARCH}.zip"
    mv vault /usr/bin/vault
    chmod +x /usr/bin/vault
    rm "vault_1.13.3_linux_${ARCH}.zip"
fi

# Handover to initialization logic
chmod +x /vault-init.sh
exec /vault-init.sh