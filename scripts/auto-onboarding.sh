#!/bin/bash
set -e

# ----------------------------
# Configuración de URLs
# ----------------------------
AUTHORITY_URL="${AUTHORITY_URL:-http://127.0.0.1:1500}"
CONSUMER_URL="${CONSUMER_URL:-http://127.0.0.1:1100}"
PROVIDER_URL="${PROVIDER_URL:-http://127.0.0.1:1200}"

DOCKER_AUTHORITY_URL="${DOCKER_AUTHORITY_URL:-http://host.docker.internal:1500}"
DOCKER_CONSUMER_URL="${DOCKER_CONSUMER_URL:-http://host.docker.internal:1100}"
DOCKER_PROVIDER_URL="${DOCKER_PROVIDER_URL:-http://host.docker.internal:1200}"

# ----------------------------
# Helpers de logging
# ----------------------------
log_step()    { echo -e "\n\033[36m$1\033[0m"; }
log_success() { echo -e "\033[32m$1\033[0m"; }
log_error()   { echo -e "\033[31m$1\033[0m"; exit 1; }
log_info()    { echo -e "\033[33m$1\033[0m"; }

# ----------------------------
# Helper HTTP
# ----------------------------
invoke_curl_json() {
    local method=${1:-GET}
    local url=$2
    local body=$3
    local parse_json=${4:-true}

    local response

    if [ -n "$body" ]; then
        response=$(curl -s -w "%{http_code}" -X "$method" "$url" \
            -H "Content-Type: application/json" \
            -d "$body")
    else
        response=$(curl -s -w "%{http_code}" -X "$method" "$url" \
            -H "Content-Type: application/json")
    fi

    http_code="${response: -3}"
    content="${response::-3}"

    if [[ "$http_code" =~ ^2 ]]; then
        log_success "SUCCESS: $method $url -> $http_code"
    else
        log_error "ERROR: $method $url -> $http_code"
    fi

    if [ "$parse_json" = true ] && [ -n "$content" ]; then
        echo "$content" | jq .
    else
        echo "$content"
    fi
}

echo -e "\n======================================"
echo "      AUTO ONBOARDING SCRIPT"
echo "======================================"

# ----------------------------
# STEP 1 - Link Authority Wallet
# ----------------------------
log_step "STEP 1 - Linking Authority wallet"
invoke_curl_json POST "$AUTHORITY_URL/api/v1/wallet/link" "" false

# ----------------------------
# STEP 2 - Link Consumer Wallet
# ----------------------------
log_step "STEP 2 - Linking Consumer wallet"
invoke_curl_json POST "$CONSUMER_URL/api/v1/wallet/link" "" false

# ----------------------------
# STEP 3 - Link Provider Wallet
# ----------------------------
log_step "STEP 3 - Linking Provider wallet"
invoke_curl_json POST "$PROVIDER_URL/api/v1/wallet/link" "" false

# ----------------------------
# STEP 4 - Retrieve DIDs
# ----------------------------
log_step "STEP 4 - Retrieving DIDs"

AUTH_DID=$(invoke_curl_json GET "$AUTHORITY_URL/.well-known/did.json" | jq -r '.id')
log_success "Authority DID: $AUTH_DID"

CONSUMER_DID=$(invoke_curl_json GET "$CONSUMER_URL/.well-known/did.json" | jq -r '.id')
log_success "Consumer DID: $CONSUMER_DID"

PROVIDER_DID=$(invoke_curl_json GET "$PROVIDER_URL/.well-known/did.json" | jq -r '.id')
log_success "Provider DID: $PROVIDER_DID"

# ----------------------------
# STEP 5 - Consumer requests credential
# ----------------------------
log_step "STEP 5 - Consumer requests credential from Authority"

C_BEG_BODY=$(jq -n \
    --arg url "$DOCKER_AUTHORITY_URL/api/v1/gate/access" \
    --arg id "$AUTH_DID" \
    --arg slug "authority" \
    --arg vc_type "DataspaceParticipant_jwt_vc_json" \
    --arg method "cert" \
    '{url: $url, id: $id, slug: $slug, vc_type: $vc_type, method: $method}')

invoke_curl_json POST "$CONSUMER_URL/api/v1/vc-request/beg" "$C_BEG_BODY" false
log_success "Consumer credential request sent"

# ----------------------------
# STEP 6 - Authority retrieves requests
# ----------------------------
log_step "STEP 6 - Authority retrieving pending requests"
ALL_REQUESTS=$(invoke_curl_json GET "$AUTHORITY_URL/api/v1/approver/all")
PETITION_ID=$(echo "$ALL_REQUESTS" | jq -r '.[-1].id')
log_info "Petition ID: $PETITION_ID"

# ----------------------------
# STEP 7 - Authority approves request
# ----------------------------
log_step "STEP 7 - Authority approving request"
APPROVE_BODY='{"approve": true}'
invoke_curl_json POST "$AUTHORITY_URL/api/v1/approver/$PETITION_ID" "$APPROVE_BODY" false
log_success "Request approved"

# ----------------------------
# STEP 8 - Consumer retrieves credential URI
# ----------------------------
log_step "STEP 8 - Consumer retrieving OIDC4VCI URI"
ALL_AUTHORITY=$(invoke_curl_json GET "$CONSUMER_URL/api/v1/vc-request/all")
OIDC4VCI_URI=$(echo "$ALL_AUTHORITY" | jq -r '.[-1].vc_uri')
log_info "OIDC4VCI URI: $OIDC4VCI_URI"

# ----------------------------
# STEP 9 - Consumer processes OIDC4VCI
# ----------------------------
log_step "STEP 9 - Consumer processing credential"
invoke_curl_json POST "$CONSUMER_URL/api/v1/wallet/oidc4vci" "{\"uri\":\"$OIDC4VCI_URI\"}" false
log_success "OIDC4VCI processed"

# ----------------------------
# STEP 10 - Consumer requests Provider access
# ----------------------------
log_step "STEP 10 - Consumer requesting Provider access"

OIDC4VP_BODY=$(jq -n \
    --arg url "$DOCKER_PROVIDER_URL/api/v1/gate/access" \
    --arg id "$PROVIDER_DID" \
    --arg slug "provider" \
    '{url: $url, id: $id, slug: $slug, actions:["talk"]}')

OIDC4VP_URI=$(invoke_curl_json POST "$CONSUMER_URL/api/v1/onboard/provider" "$OIDC4VP_BODY" false)
log_info "OIDC4VP URI: $OIDC4VP_URI"

# ----------------------------
# STEP 11 - Consumer processes OIDC4VP
# ----------------------------
log_step "STEP 11 - Consumer processing OIDC4VP"
invoke_curl_json POST "$CONSUMER_URL/api/v1/wallet/oidc4vp" "{\"uri\":\"$OIDC4VP_URI\"}" false
log_success "OIDC4VP processed"

echo -e "\n======================================"
echo "   ONBOARDING FINISHED SUCCESSFULLY"
echo "======================================"