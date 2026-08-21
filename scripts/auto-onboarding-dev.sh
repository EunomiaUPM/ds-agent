#!/bin/bash
set -euo pipefail

# ----------------------------
# Configuración de URLs
# ----------------------------
AUTHORITY_URL="${AUTHORITY_URL:-http://127.0.0.1:1500}"
CONSUMER_URL="${CONSUMER_URL:-http://127.0.0.1:1100}"
PROVIDER_URL="${PROVIDER_URL:-http://127.0.0.1:1200}"

INTERNAL_AUTHORITY_URL="${INTERNAL_AUTHORITY_URL:-$AUTHORITY_URL}"
INTERNAL_PROVIDER_URL="${INTERNAL_PROVIDER_URL:-$PROVIDER_URL}"

# ----------------------------
# Logging (solo stderr)
# ----------------------------
log_step()    { echo -e "\n\033[36m$1\033[0m" >&2; }
log_success() { echo -e "\033[32m$1\033[0m" >&2; }
log_error()   { echo -e "\033[31m$1\033[0m" >&2; exit 1; }
log_info()    { echo -e "\033[33m$1\033[0m" >&2; }

# ----------------------------
# CURL
# ----------------------------
curl_raw() {
    local method=${1:-GET}
    local url=$2
    local body=${3:-}
    if [ -n "$body" ]; then
        curl -s -X "$method" "$url" -H "Content-Type: application/json" -d "$body"
    else
        curl -s -X "$method" "$url" -H "Content-Type: application/json"
    fi
}

# Como curl_raw pero aborta ante cualquier respuesta no-2xx. Úsalo para todo lo
# que muta estado: curl_raw se traga los errores y el script reportaría éxito
# aunque un paso hubiese fallado.
curl_checked() {
    local method=${1:-GET}
    local url=$2
    local body=${3:-}
    local out code
    out=$(mktemp)
    if [ -n "$body" ]; then
        code=$(curl -s -o "$out" -w '%{http_code}' -X "$method" "$url" \
            -H "Content-Type: application/json" -d "$body")
    else
        code=$(curl -s -o "$out" -w '%{http_code}' -X "$method" "$url" \
            -H "Content-Type: application/json")
    fi
    if [[ ! "$code" =~ ^2 ]]; then
        log_error "$method $url -> HTTP $code
$(head -c 400 "$out")"
    fi
    cat "$out"
    rm -f "$out"
}

# ----------------------------
# HEADER
# ----------------------------
echo -e "\n======================================"
echo "      AUTO ONBOARDING (DEV)"
echo "======================================"

VC_TYPE="DataSpaceParticipant_jwt_vc_json"

# ----------------------------
# STEP 1 - Link wallets
# ----------------------------
log_step "STEP 1 - Linking wallets"
curl_checked POST "$AUTHORITY_URL/api/v1/wallet/link" >/dev/null
curl_checked POST "$CONSUMER_URL/api/v1/wallet/link" >/dev/null
curl_checked POST "$PROVIDER_URL/api/v1/wallet/link" >/dev/null
log_success "Wallets linked"

# ----------------------------
# STEP 2 - DIDs
# ----------------------------
log_step "STEP 2 - Retrieving DIDs"
AUTH_DID=$(curl_raw GET "$AUTHORITY_URL/.well-known/did.json" | jq -r '.id')
PROVIDER_DID=$(curl_raw GET "$PROVIDER_URL/.well-known/did.json" | jq -r '.id')
log_success "Authority DID: $AUTH_DID"
log_success "Provider DID: $PROVIDER_DID"

# ----------------------------
# STEP 3 - Consumer requests credential from authority
# ----------------------------
# Payload = ReachAuthority (crates/auth/src/types/entities/reacher.rs):
#   { id, nick, url, vc_type, method, auto }
log_step "STEP 3 - Consumer requests credential"

if curl_raw GET "$CONSUMER_URL/api/v1/vc-request/all" \
    | jq -e --arg id "$AUTH_DID" --arg vc "$VC_TYPE" \
        'any(.[]?; .participant_id == $id
                   and .status == "Finalized"
                   and (.vc_type_config // [] | index($vc)))' >/dev/null; then
    log_info "Consumer ya tiene la credencial, saltando"
else
    BEG_BODY=$(jq -n \
        --arg url "$INTERNAL_AUTHORITY_URL/api/v1/gate/access" \
        --arg id "$AUTH_DID" \
        --arg nick "authority" \
        --arg vc_type "$VC_TYPE" \
        --arg method "cert" \
        '{url:$url, id:$id, nick:$nick, vc_type:$vc_type, method:$method, auto:true}')
    curl_checked POST "$CONSUMER_URL/api/v1/vc-request/beg" "$BEG_BODY" >/dev/null
    log_success "Credential request finished"
fi

# ----------------------------
# STEP 4 - Consumer authenticates with provider
# ----------------------------
# Payload = ReachProvider (crates/auth/src/types/entities/reacher.rs):
#   { id, nick, url, actions, auto }
log_step "STEP 4 - Consumer authenticates with provider"

if curl_raw GET "$CONSUMER_URL/api/v1/mates/all" \
    | jq -e --arg id "$PROVIDER_DID" \
        'any(.[]?; .participant_id == $id and (.token // "") != "")' >/dev/null; then
    log_info "Consumer ya está autenticado con el provider, saltando"
else
    CONNECT_BODY=$(jq -n \
        --arg id "$PROVIDER_DID" \
        --arg nick "provider" \
        --arg url "$INTERNAL_PROVIDER_URL/api/v1/gate/access" \
        '{id:$id, nick:$nick, url:$url, actions:["talk"], auto:true}')
    curl_checked POST "$CONSUMER_URL/api/v1/peer-connection/connect" "$CONNECT_BODY" >/dev/null
    log_success "Authentication complete"
fi

echo -e "\n======================================"
echo "   ONBOARDING FINISHED SUCCESSFULLY"
echo "======================================"
