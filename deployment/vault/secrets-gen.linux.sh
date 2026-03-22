#!/bin/sh
set -e

SECRETS_DIR=$1


ROLE=$2

if [ -z "$ROLE" ]; then
    echo "Usage: $0 <secrets_dir> <role>"
    echo "Role must be 'provider' 'consumer' or 'authority'"
    exit 1
fi

mkdir -p "$SECRETS_DIR"

if [ ! -f "$SECRETS_DIR/private_key.pem" ]; then
    echo "Generating private_key.pem..."
    openssl genrsa -out "$SECRETS_DIR/private_key.pem" 2048
else
    echo "private_key.pem already exists. Skipping."
fi

if [ ! -f "$SECRETS_DIR/public_key.pem" ]; then
    echo "Generating public_key.pem..."
    openssl rsa -in "$SECRETS_DIR/private_key.pem" -pubout -out "$SECRETS_DIR/public_key.pem"
else
    echo "public_key.pem already exists. Skipping."
fi

if [ ! -f "$SECRETS_DIR/cert.pem" ]; then
    echo "Generating cert.pem..."
    openssl req -new -x509 -key "$SECRETS_DIR/private_key.pem" -out "$SECRETS_DIR/cert.pem" -days 365 -subj "/C=ES/ST=Madrid/L=Madrid/O=Eunomia/CN=localhost"
else
    echo "cert.pem already exists. Skipping."
fi

if [ ! -f "$SECRETS_DIR/db.json" ]; then
    echo "Generating db.json for $ROLE..."
    if [ "$ROLE" = "provider" ]; then
        echo '{
  "user": "ds_provider",
  "password": "ds_provider",
  "name": "ds_provider"
}' > "$SECRETS_DIR/db.json"
    elif [ "$ROLE" = "consumer" ]; then
        echo '{
  "user": "ds_consumer",
  "password": "ds_consumer",
  "name": "ds_consumer"
}' > "$SECRETS_DIR/db.json"
    elif [ "$ROLE" = "authority" ]; then
            echo '{
      "user": "ds_authority",
      "password": "ds_authority",
      "name": "ds_authority"
    }' > "$SECRETS_DIR/db.json"
   fi
else
    echo "db.json already exists. Skipping."
fi

if [ ! -f "$SECRETS_DIR/wallet.json" ]; then
    echo "Generating wallet.json for $ROLE..."
    if [ "$ROLE" = "provider" ]; then
        echo '{
  "type": "email",
  "name": "provider",
  "email": "provider@eunomia.dev",
  "password": "provider"
}' > "$SECRETS_DIR/wallet.json"
    elif [ "$ROLE" = "consumer" ]; then
        echo '{
  "type": "email",
  "name": "consumer",
  "email": "consumer@eunomia.dev",
  "password": "consumer"
}' > "$SECRETS_DIR/wallet.json"

elif [ "$ROLE" = "authority" ]; then
        echo '{
  "type": "email",
  "name": "authority",
  "email": "authority@eunomia.dev",
  "password": "authority"
}' > "$SECRETS_DIR/wallet.json"
    fi
else
    echo "wallet.json already exists. Skipping."
fi

echo "Secrets generation complete in $SECRETS_DIR for $ROLE"
