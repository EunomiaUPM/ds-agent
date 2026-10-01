#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/auth.sh
source "$SCRIPT_DIR/lib/auth.sh"

# ----------------------------
# Configuración de URLs
# ----------------------------
AUTHORITY_URL="${AUTHORITY_URL:-http://127.0.0.1:1500}"
CONSUMER_URL="${CONSUMER_URL:-http://127.0.0.1:1100}"
PROVIDER_URL="${PROVIDER_URL:-http://127.0.0.1:1200}"

INTERNAL_AUTHORITY_URL="${INTERNAL_AUTHORITY_URL:-$AUTHORITY_URL}"
INTERNAL_PROVIDER_URL="${INTERNAL_PROVIDER_URL:-$PROVIDER_URL}"
# Provider tenant the consumer onboards into (tenants share the provider DID).
PROVIDER_TENANT="${PROVIDER_TENANT:-admin}"

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
CURL_TIMEOUT="${CURL_TIMEOUT:-60}"

# Human-readable reason for the curl exit codes seen when a service is down or
# is not speaking HTTP at all (e.g. a Postgres port).
curl_reason() {
    case "$1" in
        6)  echo "no se resuelve el host" ;;
        7)  echo "conexión rechazada, no hay nada escuchando" ;;
        28) echo "timeout tras ${CURL_TIMEOUT}s" ;;
        35) echo "fallo de TLS" ;;
        52) echo "respuesta vacía: el puerto responde pero no habla HTTP" ;;
        56) echo "conexión cortada por el otro extremo" ;;
        *)  echo "error de curl" ;;
    esac
}

curl_raw() {
    local method=${1:-GET}
    local url=$2
    local body=${3:-}
    eunomia_auth "$url"
    if [ -n "$body" ]; then
        curl -s --max-time "$CURL_TIMEOUT" -X "$method" "$url" "${AUTH_ARGS[@]}" \
            -H "Content-Type: application/json" -d "$body"
    else
        curl -s --max-time "$CURL_TIMEOUT" -X "$method" "$url" "${AUTH_ARGS[@]}" \
            -H "Content-Type: application/json"
    fi
}

# Como curl_raw pero aborta ante cualquier respuesta no-2xx. Úsalo para todo lo
# que muta estado: curl_raw se traga los errores y el script reportaría éxito
# aunque un paso hubiese fallado.
curl_checked() {
    local method=${1:-GET}
    local url=$2
    local body=${3:-}
    local out code rc
    out=$(mktemp)
    eunomia_auth "$url"
    # `code=$(curl ...)` alone would abort the script under `set -e` before the
    # HTTP check runs, losing the reason. Capture curl's exit status instead.
    if [ -n "$body" ]; then
        code=$(curl -s -o "$out" -w '%{http_code}' --max-time "$CURL_TIMEOUT" \
            -X "$method" "$url" "${AUTH_ARGS[@]}" -H "Content-Type: application/json" -d "$body") \
            && rc=0 || rc=$?
    else
        code=$(curl -s -o "$out" -w '%{http_code}' --max-time "$CURL_TIMEOUT" \
            -X "$method" "$url" "${AUTH_ARGS[@]}" -H "Content-Type: application/json") \
            && rc=0 || rc=$?
    fi
    if [ "$rc" -ne 0 ]; then
        rm -f "$out"
        log_error "$method $url -> $(curl_reason "$rc") (curl exit $rc)"
    fi
    if [[ ! "$code" =~ ^2 ]]; then
        local snippet
        snippet=$(head -c 400 "$out")
        rm -f "$out"
        log_error "$method $url -> HTTP $code
$snippet"
    fi
    cat "$out"
    rm -f "$out"
}

# Fails with the full list of unreachable services instead of dying on the
# first one, so a broken dev environment is diagnosed in a single run.
preflight() {
    local failed=0 name url rc
    for entry in "authority|$AUTHORITY_URL" "consumer|$CONSUMER_URL" "provider|$PROVIDER_URL"; do
        name=${entry%%|*}
        url=${entry#*|}
        curl -s -o /dev/null --max-time 10 "$url/api/v1/wallet/is-linked" && rc=0 || rc=$?
        if [ "$rc" -ne 0 ]; then
            log_info "  $name ($url) -> $(curl_reason "$rc") (curl exit $rc)"
            failed=1
        else
            log_success "  $name ($url) -> OK"
        fi
    done
    if [ "$failed" -ne 0 ]; then
        log_error "Hay servicios inalcanzables. Arráncalos o apunta a los suyos con
AUTHORITY_URL / CONSUMER_URL / PROVIDER_URL antes de relanzar el onboarding."
    fi
}

# ----------------------------
# HEADER
# ----------------------------
echo -e "\n======================================"
echo "      AUTO ONBOARDING (DEV)"
echo "======================================"

VC_TYPE="DataSpaceParticipant_jwt_vc_json"

# ----------------------------
# STEP 0 - Reachability
# ----------------------------
log_step "STEP 0 - Comprobando servicios"
preflight

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
        'any(.items[]?; .participant_id == $id
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
log_step "STEP 4 - Consumer authenticates with provider (tenant $PROVIDER_TENANT)"

if curl_raw GET "$CONSUMER_URL/api/v1/mates/all" \
    | jq -e --arg id "$PROVIDER_DID" \
        'any(.items[]?; .participant_id == $id and (.token // "") != "")' >/dev/null; then
    log_info "Consumer ya está autenticado con el provider, saltando"
else
    CONNECT_BODY=$(jq -n \
        --arg id "$PROVIDER_DID" \
        --arg nick "provider" \
        --arg url "$INTERNAL_PROVIDER_URL/api/v1/gate/$PROVIDER_TENANT/access" \
        '{id:$id, nick:$nick, url:$url, actions:["talk"], auto:true}')
    curl_checked POST "$CONSUMER_URL/api/v1/peer-connection/connect" "$CONNECT_BODY" >/dev/null
    log_success "Authentication complete"
fi

echo -e "\n======================================"
echo "   ONBOARDING FINISHED SUCCESSFULLY"
echo "======================================"
