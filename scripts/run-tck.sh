#!/usr/bin/env bash
# =============================================================================
# run-tck.sh — Prepares the environment and launches the DSP TCK 
#              (eclipse-dataspacetck/dsp-tck release/1.0.0-RC6)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEPLOYMENT_DIR="$ROOT_DIR/deployment"
HEIMDALL_DIR="${HEIMDALL_DIR:-$(cd "$ROOT_DIR/../heimdall" 2>/dev/null && pwd || echo "")}"
TCK_DIR="$ROOT_DIR/tck"
TCK_PROPS="$TCK_DIR/tck.properties"
TCK_MAPPING="$TCK_DIR/tck_mapping.json"

PROVIDER_URL="${PROVIDER_URL:-http://127.0.0.1:1200}"
CONSUMER_URL="${CONSUMER_URL:-http://127.0.0.1:1100}"
AUTHORITY_URL="${AUTHORITY_URL:-http://127.0.0.1:1500}"

# URLs visible from inside Docker containers
DOCKER_PROVIDER_URL="${DOCKER_PROVIDER_URL:-http://host.docker.internal:1200}"
DOCKER_CONSUMER_URL="${DOCKER_CONSUMER_URL:-http://host.docker.internal:1100}"

TCK_IMAGE="${TCK_IMAGE:-eclipsedataspacetck/dsp-tck-runtime:1.0.0-RC6}"
TCK_PORT="${TCK_PORT:-8888}"

SEED_ONLY=false
TCK_ONLY=false
NO_CONSUMER_TESTS=false

for arg in "$@"; do
  case $arg in
    --seed-only)        SEED_ONLY=true ;;
    --tck-only)         TCK_ONLY=true ;;
    --no-consumer-tests) NO_CONSUMER_TESTS=true ;;
  esac
done

# ── helpers ───────────────────────────────────────────────────────────────────
log()  { echo -e "\n\033[36m▶ $*\033[0m" >&2; }
ok()   { echo -e "\033[32m  ✓ $*\033[0m" >&2; }
warn() { echo -e "\033[33m  ! $*\033[0m" >&2; }
die()  { echo -e "\033[31m  ✗ $*\033[0m" >&2; exit 1; }

curl_j() {
  local method=$1 url=$2 body=${3:-}
  if [[ -n "$body" ]]; then
    curl -sf -X "$method" -H "Content-Type: application/json" -d "$body" "$url"
  else
    curl -sf -X "$method" -H "Content-Type: application/json" "$url"
  fi
}

wait_for_health() {
  local name=$1 url=$2
  log "Waiting for $name to be healthy at $url"
  local i=0
  until curl -sf "$url/.well-known/dspace-version" >/dev/null 2>&1; do
    ((i++))
    [[ $i -ge 90 ]] && die "$name did not respond after 3 minutes"
    sleep 2
  done
  ok "$name ready"
}

wait_for_authority() {
  local name=$1 url=$2
  log "Waiting for $name to be healthy at $url"
  local i=0
  until curl -sf "$url/.well-known/openid-credential-issuer" >/dev/null 2>&1; do
    ((i++))
    [[ $i -ge 90 ]] && die "$name did not respond after 3 minutes"
    sleep 2
  done
  ok "$name ready"
}

# ── PHASE 1: Docker stacks ────────────────────────────────────────────────────
start_stacks() {
  log "Starting Docker stacks (heimdall + provider + consumer)"

  if [[ -z "$HEIMDALL_DIR" || ! -f "$HEIMDALL_DIR/docker-compose.yml" ]]; then
    die "Heimdall directory not found. Set HEIMDALL_DIR or place heimdall repo next to ds-protocol."
  fi

  docker compose -f "$HEIMDALL_DIR/docker-compose.yml" up -d
  docker compose -f "$DEPLOYMENT_DIR/docker-compose.mini.provider.yaml" up -d
  docker compose -f "$DEPLOYMENT_DIR/docker-compose.mini.consumer.yaml" up -d
  ok "Containers started"
}

# ── PHASE 2: onboarding ───────────────────────────────────────────────────────
run_onboarding() {
  log "Executing full onboarding"
  AUTHORITY_URL="$AUTHORITY_URL" \
  CONSUMER_URL="$CONSUMER_URL"   \
  PROVIDER_URL="$PROVIDER_URL"   \
  DOCKER_AUTHORITY_URL="${DOCKER_AUTHORITY_URL:-http://host.docker.internal:1500}" \
  DOCKER_CONSUMER_URL="$DOCKER_CONSUMER_URL" \
  DOCKER_PROVIDER_URL="$DOCKER_PROVIDER_URL" \
    bash "$SCRIPT_DIR/auto-onboarding-mini.sh"
  ok "Onboarding completed"
}

# ── PHASE 3: extract Bearer token ─────────────────────────────────────────────
get_provider_token_for_tck() {
  log "Extracting consumer access token (for TCK usage)"
  local mates
  mates=$(curl_j GET "$PROVIDER_URL/api/v1/mates/all")

  local token
  token=$(echo "$mates" | jq -r '
    [ .[] | select(.is_me == false and .token != null) | .token ]
    | first
    // empty
  ')

  if [[ -z "$token" ]]; then
    warn "No token found in provider mates — TCK might fail on authenticated routes"
    echo "NO_TOKEN"
  else
    ok "Token extracted (${#token} chars)"
    echo "$token"
  fi
}

# ── PHASE 4 & 5: Seeding ──────────────────────────────────────────────────────
run_seeding() {
  log "Seeding TCK data using populate_tck.sh"
  PROVIDER_URL="$PROVIDER_URL" \
  CONSUMER_URL="$CONSUMER_URL" \
  OUTPUT_MAPPING="$TCK_MAPPING" \
    zsh "$SCRIPT_DIR/populate_tck.sh"
}

# ── PHASE 6: generate tck.properties ──────────────────────────────────────────
generate_properties() {
  local token=$1
  log "Generating $TCK_PROPS"

  if [[ ! -f "$TCK_MAPPING" ]]; then
    die "Mapping file $TCK_MAPPING not found. Seeding must have failed."
  fi

  local mapping=$(cat "$TCK_MAPPING")
  
  lookup_ds()    { echo "$mapping" | jq -r --arg k "$1" '.datasets[$k] // "MISSING"'; }
  lookup_offer() { echo "$mapping" | jq -r --arg k "$1" '.offers[$k] // "MISSING"'; }
  lookup_agr_p() { echo "$mapping" | jq -r --arg k "$1" '.provider_agreements[$k] // "MISSING"'; }
  lookup_agr_c() { echo "$mapping" | jq -r --arg k "$1" '.consumer_agreements[$k] // "MISSING"'; }

  local agent_id
  agent_id=$(curl_j GET "$PROVIDER_URL/api/v1/mates/myself" | jq -r '.participant_id // "urn:provider"')

  cat > "$TCK_PROPS" <<EOF
# ============================================================
# DSP TCK — eclipse-dataspacetck/dsp-tck release/1.0.0-RC6
# Automatically generated by scripts/run-tck.sh
# ============================================================

dataspacetck.debug=true
dataspacetck.dsp.local.connector=false

dataspacetck.host=0.0.0.0
dataspacetck.port=${TCK_PORT}
dataspacetck.callback.address=http://host.docker.internal:${TCK_PORT}

dataspacetck.dsp.connector.agent.id=${agent_id}
dataspacetck.dsp.connector.http.url=${DOCKER_PROVIDER_URL}/dsp/current
dataspacetck.dsp.connector.http.base.url=${DOCKER_PROVIDER_URL}
dataspacetck.dsp.connector.http.headers.authorization=Bearer ${token}

dataspacetck.dsp.default.wait=15000

dataspacetck.dsp.connector.negotiation.initiate.url=${DOCKER_CONSUMER_URL}/tck/negotiations/requests
dataspacetck.dsp.connector.transfer.initiate.url=${DOCKER_CONSUMER_URL}/tck/transfers/requests

# ── Catalog ───────────────────────────────────────────────────────────────────
CAT_01_01_DATASETID=$(lookup_ds CAT0101)
CAT_01_02_DATASETID=$(lookup_ds CAT0102)
CAT_01_03_DATASETID=$(lookup_ds CAT0103)

# ── Negotiation Provider (CN) ─────────────────────────────────────────────────
CN_01_01_DATASETID=$(lookup_ds CN0101)
CN_01_01_OFFERID=$(lookup_offer CN0101)
CN_01_02_DATASETID=$(lookup_ds CN0102)
CN_01_02_OFFERID=$(lookup_offer CN0102)
CN_01_03_DATASETID=$(lookup_ds CN0103)
CN_01_03_OFFERID=$(lookup_offer CN0103)
CN_01_04_DATASETID=$(lookup_ds CN0104)
CN_01_04_OFFERID=$(lookup_offer CN0104)
CN_02_01_DATASETID=$(lookup_ds CN0201)
CN_02_01_OFFERID=$(lookup_offer CN0201)
CN_02_02_DATASETID=$(lookup_ds CN0202)
CN_02_02_OFFERID=$(lookup_offer CN0202)
CN_02_03_DATASETID=$(lookup_ds CN0203)
CN_02_03_OFFERID=$(lookup_offer CN0203)
CN_02_04_DATASETID=$(lookup_ds CN0204)
CN_02_04_OFFERID=$(lookup_offer CN0204)
CN_02_05_DATASETID=$(lookup_ds CN0205)
CN_02_05_OFFERID=$(lookup_offer CN0205)
CN_02_06_DATASETID=$(lookup_ds CN0206)
CN_02_06_OFFERID=$(lookup_offer CN0206)
CN_02_07_DATASETID=$(lookup_ds CN0207)
CN_02_07_OFFERID=$(lookup_offer CN0207)
CN_03_01_DATASETID=$(lookup_ds CN0301)
CN_03_01_OFFERID=$(lookup_offer CN0301)
CN_03_02_DATASETID=$(lookup_ds CN0302)
CN_03_02_OFFERID=$(lookup_offer CN0302)
CN_03_03_DATASETID=$(lookup_ds CN0303)
CN_03_03_OFFERID=$(lookup_offer CN0303)
CN_03_04_DATASETID=$(lookup_ds CN0304)
CN_03_04_OFFERID=$(lookup_offer CN0304)

# ── Negotiation Consumer (CN_C) ───────────────────────────────────────────────
CN_C_01_01_DATASETID=$(lookup_ds CNC0101)
CN_C_01_02_DATASETID=$(lookup_ds CNC0102)
CN_C_01_03_DATASETID=$(lookup_ds CNC0103)
CN_C_01_04_DATASETID=$(lookup_ds CNC0104)
CN_C_02_01_DATASETID=$(lookup_ds CNC0201)
CN_C_02_02_DATASETID=$(lookup_ds CNC0202)
CN_C_02_03_DATASETID=$(lookup_ds CNC0203)
CN_C_02_04_DATASETID=$(lookup_ds CNC0204)
CN_C_02_05_DATASETID=$(lookup_ds CNC0205)
CN_C_02_06_DATASETID=$(lookup_ds CNC0206)
CN_C_03_01_DATASETID=$(lookup_ds CNC0301)
CN_C_03_02_DATASETID=$(lookup_ds CNC0302)
CN_C_03_03_DATASETID=$(lookup_ds CNC0303)
CN_C_03_04_DATASETID=$(lookup_ds CNC0304)
CN_C_03_05_DATASETID=$(lookup_ds CNC0305)
CN_C_03_06_DATASETID=$(lookup_ds CNC0306)

# ── Transfer Provider (TP) ────────────────────────────────────────────────────
TP_01_01_AGREEMENTID=$(lookup_agr_p TP0101)
TP_01_01_FORMAT=HttpData-PULL
TP_01_02_AGREEMENTID=$(lookup_agr_p TP0102)
TP_01_02_FORMAT=HttpData-PULL
TP_01_03_AGREEMENTID=$(lookup_agr_p TP0103)
TP_01_03_FORMAT=HttpData-PULL
TP_01_04_AGREEMENTID=$(lookup_agr_p TP0104)
TP_01_04_FORMAT=HttpData-PULL
TP_01_05_AGREEMENTID=$(lookup_agr_p TP0105)
TP_01_05_FORMAT=HttpData-PULL
TP_02_01_AGREEMENTID=$(lookup_agr_p TP0201)
TP_02_01_FORMAT=HttpData-PULL
TP_02_02_AGREEMENTID=$(lookup_agr_p TP0202)
TP_02_02_FORMAT=HttpData-PULL
TP_02_03_AGREEMENTID=$(lookup_agr_p TP0203)
TP_02_03_FORMAT=HttpData-PULL
TP_02_04_AGREEMENTID=$(lookup_agr_p TP0204)
TP_02_04_FORMAT=HttpData-PULL
TP_02_05_AGREEMENTID=$(lookup_agr_p TP0205)
TP_02_05_FORMAT=HttpData-PULL
TP_03_01_AGREEMENTID=$(lookup_agr_p TP0301)
TP_03_01_FORMAT=HttpData-PULL
TP_03_02_AGREEMENTID=$(lookup_agr_p TP0302)
TP_03_02_FORMAT=HttpData-PULL
TP_03_03_AGREEMENTID=$(lookup_agr_p TP0303)
TP_03_03_FORMAT=HttpData-PULL
TP_03_04_AGREEMENTID=$(lookup_agr_p TP0304)
TP_03_04_FORMAT=HttpData-PULL
TP_03_05_AGREEMENTID=$(lookup_agr_p TP0305)
TP_03_05_FORMAT=HttpData-PULL
TP_03_06_AGREEMENTID=$(lookup_agr_p TP0306)
TP_03_06_FORMAT=HttpData-PULL

# ── Transfer Consumer (TP_C) ──────────────────────────────────────────────────
TP_C_01_01_AGREEMENTID=$(lookup_agr_c TPC0101)
TP_C_01_01_FORMAT=HttpData-PULL
TP_C_01_02_AGREEMENTID=$(lookup_agr_c TPC0102)
TP_C_01_02_FORMAT=HttpData-PULL
TP_C_01_03_AGREEMENTID=$(lookup_agr_c TPC0103)
TP_C_01_03_FORMAT=HttpData-PULL
TP_C_01_04_AGREEMENTID=$(lookup_agr_c TPC0104)
TP_C_01_04_FORMAT=HttpData-PULL
TP_C_01_05_AGREEMENTID=$(lookup_agr_c TPC0105)
TP_C_01_05_FORMAT=HttpData-PULL
TP_C_02_01_AGREEMENTID=$(lookup_agr_c TPC0201)
TP_C_02_01_FORMAT=HttpData-PULL
TP_C_02_02_AGREEMENTID=$(lookup_agr_c TPC0202)
TP_C_02_02_FORMAT=HttpData-PULL
TP_C_02_03_AGREEMENTID=$(lookup_agr_c TPC0203)
TP_C_02_03_FORMAT=HttpData-PULL
TP_C_02_04_AGREEMENTID=$(lookup_agr_c TPC0204)
TP_C_02_04_FORMAT=HttpData-PULL
TP_C_02_05_AGREEMENTID=$(lookup_agr_c TPC0205)
TP_C_02_05_FORMAT=HttpData-PULL
TP_C_03_01_AGREEMENTID=$(lookup_agr_c TPC0301)
TP_C_03_01_FORMAT=HttpData-PULL
TP_C_03_02_AGREEMENTID=$(lookup_agr_c TPC0302)
TP_C_03_02_FORMAT=HttpData-PULL
TP_C_03_03_AGREEMENTID=$(lookup_agr_c TPC0303)
TP_C_03_03_FORMAT=HttpData-PULL
TP_C_03_04_AGREEMENTID=$(lookup_agr_c TPC0304)
TP_C_03_04_FORMAT=HttpData-PULL
TP_C_03_05_AGREEMENTID=$(lookup_agr_c TPC0305)
TP_C_03_05_FORMAT=HttpData-PULL
TP_C_03_06_AGREEMENTID=$(lookup_agr_c TPC0306)
TP_C_03_06_FORMAT=HttpData-PULL
EOF

  ok "tck.properties generated at $TCK_PROPS"
}

# ── PHASE 7: launch TCK ───────────────────────────────────────────────────────
run_tck() {
  log "Launching TCK Docker: $TCK_IMAGE"
  docker pull "$TCK_IMAGE" >&2
  docker run --rm --name dsp-tck \
    --add-host "host.docker.internal:host-gateway" \
    -p "${TCK_PORT}:${TCK_PORT}" \
    --mount "type=bind,source=${TCK_PROPS},target=/etc/tck/config.properties" \
    "$TCK_IMAGE"
}

# ── Main Orchestration ────────────────────────────────────────────────────────
main() {
  echo ""
  echo "╔══════════════════════════════════════════════════════╗"
  echo "║        DSP TCK runner — eclipse-dataspacetck         ║"
  echo "╚══════════════════════════════════════════════════════╝"

  if [[ "$TCK_ONLY" == "true" ]]; then
    log "--tck-only mode: skipping setup"
    run_tck
    exit 0
  fi

  if [[ "$SEED_ONLY" != "true" ]]; then
    start_stacks
    wait_for_authority "Authority" "$AUTHORITY_URL"
    wait_for_health "Provider" "$PROVIDER_URL"
    wait_for_health "Consumer" "$CONSUMER_URL"
    run_onboarding
  fi

  run_seeding

  local token
  token=$(get_provider_token_for_tck)

  generate_properties "$token"

  if [[ "$SEED_ONLY" == "true" ]]; then
    ok "Seeding completed. tck.properties ready."
    exit 0
  fi

  run_tck
}

main
