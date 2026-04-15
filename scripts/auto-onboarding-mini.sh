#!/bin/bash
set -euo pipefail

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
# Logging (solo stderr)
# ----------------------------
log_step()    { echo -e "\n\033[36m$1\033[0m" >&2; }
log_success() { echo -e "\033[32m$1\033[0m" >&2; }
log_error()   { echo -e "\033[31m$1\033[0m" >&2; exit 1; }
log_info()    { echo -e "\033[33m$1\033[0m" >&2; }

# ----------------------------
# CURL RAW (SIEMPRE JSON LIMPIO)
# ----------------------------
curl_raw() {
    local method=${1:-GET}
    local url=$2
    local body=${3:-}

    if [ -n "$body" ]; then
        curl -s -X "$method" "$url" \
            -H "Content-Type: application/json" \
            -d "$body"
    else
        curl -s -X "$method" "$url" \
            -H "Content-Type: application/json"
    fi
}

# ----------------------------
# HEADER
# ----------------------------
echo -e "\n======================================"
echo "      AUTO ONBOARDING SCRIPT"
echo "======================================"

# ----------------------------
# STEP 1 - Link wallets
# ----------------------------
log_step "STEP 1 - Linking Authority wallet"
curl_raw POST "$AUTHORITY_URL/api/v1/wallet/link" >/dev/null

log_step "STEP 2 - Linking Consumer wallet"
curl_raw POST "$CONSUMER_URL/api/v1/wallet/link" >/dev/null

log_step "STEP 3 - Linking Provider wallet"
curl_raw POST "$PROVIDER_URL/api/v1/wallet/link" >/dev/null

# ----------------------------
# STEP 4 - DIDs (FIXED jq)
# ----------------------------
log_step "STEP 4 - Retrieving DIDs"

AUTH_DID=$(curl_raw GET "$AUTHORITY_URL/.well-known/did.json" | jq -r '.id')
CONSUMER_DID=$(curl_raw GET "$CONSUMER_URL/.well-known/did.json" | jq -r '.id')
PROVIDER_DID=$(curl_raw GET "$PROVIDER_URL/.well-known/did.json" | jq -r '.id')

log_success "Authority DID: $AUTH_DID"
log_success "Consumer DID: $CONSUMER_DID"
log_success "Provider DID: $PROVIDER_DID"

# ----------------------------
# STEP 5 - Consumer request credential
# ----------------------------
log_step "STEP 5 - Consumer requests credential"

C_BEG_BODY=$(jq -n \
    --arg url "$DOCKER_AUTHORITY_URL/api/v1/gate/access" \
    --arg id "$AUTH_DID" \
    --arg slug "authority" \
    --arg vc_type "DataspaceParticipant_jwt_vc_json" \
    --arg method "cert" \
    '{url:$url,id:$id,slug:$slug,vc_type:$vc_type,method:$method}')

curl_raw POST "$CONSUMER_URL/api/v1/vc-request/beg" "$C_BEG_BODY" >/dev/null
log_success "Credential request sent"

# ----------------------------
# STEP 6 - Approver requests
# ----------------------------
log_step "STEP 6 - Authority retrieving requests"

ALL_REQUESTS=$(curl_raw GET "$AUTHORITY_URL/api/v1/approver/all")
PETITION_ID=$(echo "$ALL_REQUESTS" | jq -r '.[-1].id')

log_info "Petition ID: $PETITION_ID"

# ----------------------------
# STEP 7 - Approve
# ----------------------------
log_step "STEP 7 - Approving request"

APPROVE_BODY='{"approve": true}'
curl_raw POST "$AUTHORITY_URL/api/v1/approver/$PETITION_ID" "$APPROVE_BODY" >/dev/null

log_success "Request approved"

# ----------------------------
# STEP 8 - OIDC4VCI URI
# ----------------------------
log_step "STEP 8 - Retrieving OIDC4VCI URI"

ALL_AUTHORITY=$(curl_raw GET "$CONSUMER_URL/api/v1/vc-request/all")
OIDC4VCI_URI=$(echo "$ALL_AUTHORITY" | jq -r '.[-1].vc_uri')

log_info "OIDC4VCI URI: $OIDC4VCI_URI"

# ----------------------------
# STEP 9 - Process credential
# ----------------------------
log_step "STEP 9 - Processing credential"

curl_raw POST "$CONSUMER_URL/api/v1/wallet/oidc4vci" \
"{\"uri\":\"$OIDC4VCI_URI\"}" >/dev/null

log_success "OIDC4VCI processed"

# ----------------------------
# STEP 10 - Provider access
# ----------------------------
log_step "STEP 10 - Provider request"

OIDC4VP_BODY=$(jq -n \
    --arg url "$DOCKER_PROVIDER_URL/api/v1/gate/access" \
    --arg id "$PROVIDER_DID" \
    --arg slug "provider" \
    '{url:$url,id:$id,slug:$slug,actions:["talk"]}')

OIDC4VP_URI=$(curl_raw POST "$CONSUMER_URL/api/v1/onboard/provider" "$OIDC4VP_BODY")

log_info "OIDC4VP URI: $OIDC4VP_URI"

# ----------------------------
# STEP 11 - Process VP
# ----------------------------
log_step "STEP 11 - Processing OIDC4VP"

curl_raw POST "$CONSUMER_URL/api/v1/wallet/oidc4vp" \
"{\"uri\":\"$OIDC4VP_URI\"}" >/dev/null

log_success "OIDC4VP processed"

echo -e "\n======================================"
echo "   ONBOARDING FINISHED SUCCESSFULLY"
echo "======================================"