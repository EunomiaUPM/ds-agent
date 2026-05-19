#!/bin/zsh
# =============================================================================
# populate_tck.sh — Seeds the provider catalog and agreements for DSP TCK
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

PROVIDER_URL="${PROVIDER_URL:-http://127.0.0.1:1200}"
CONSUMER_URL="${CONSUMER_URL:-http://127.0.0.1:1100}"
OUTPUT_MAPPING="${OUTPUT_MAPPING:-$ROOT_DIR/tck/tck_mapping.json}"

# ── helpers ───────────────────────────────────────────────────────────────────
log()  { echo -e "\033[36m▶ $*\033[0m" >&2; }
ok()   { echo -e "\033[32m  ✓ $*\033[0m" >&2; }
die()  { echo -e "\033[31m  ✗ $*\033[0m" >&2; exit 1; }
new_uuid() { uuidgen 2>/dev/null | tr '[:upper:]' '[:lower:]' || cat /proc/sys/kernel/random/uuid; }

curl_j() {
  local method=$1 url=$2 body=${3:-}
  if [[ -n "$body" ]]; then
    curl -sf -X "$method" -H "Content-Type: application/json" -d "$body" "$url"
  else
    curl -sf -X "$method" -H "Content-Type: application/json" "$url"
  fi
}

# ── Globals ──────────────────────────────────────────────────────────────────
CATALOG_ID=""
DS_SVC_ID=""
TPL_NAME=""
TPL_VER=""
CONSUMER_DID=""
PROVIDER_DID=""
declare -A DS OFFER AGR_PROV AGR_CONS

# ── Catalog seeding ──────────────────────────────────────────────────────────
seed_catalog() {
  log "Seeding provider catalog for TCK"

  CATALOG_ID=$(curl_j POST "$PROVIDER_URL/api/v1/catalog-agent/catalogs/main" '{}' | jq -r '.id')
  [[ -z "$CATALOG_ID" || "$CATALOG_ID" == "null" ]] && die "Could not get main catalog ID"
  ok "Catalog: $CATALOG_ID"

  DS_SVC_ID=$(curl_j GET "$PROVIDER_URL/api/v1/catalog-agent/data-services/main" | jq -r '.id')
  [[ -z "$DS_SVC_ID" || "$DS_SVC_ID" == "null" ]] && die "Could not get main data-service ID"
  ok "DataService: $DS_SVC_ID"

  local tpl_resp
  tpl_resp=$(curl_j POST "$PROVIDER_URL/api/v1/connector/templates" \
    '{"authentication":{"type":"NO_AUTH"},"interaction":{"mode":"PULL","dataAccess":{"protocol":"HTTP","urlTemplate":"{{__ACCESS_URL__}}","method":"{{__ACCESS_METHODS__}}","headers":"{{__ACCESS_HEADERS__}}"}},"parameters":[{"paramType":"STRING","name":"ACCESS_URL","title":"URL","required":true},{"paramType":"VEC<STRING>","name":"ACCESS_METHODS","title":"Methods","required":true},{"paramType":"MAP<STRING,STRING>","name":"ACCESS_HEADERS","title":"Headers","required":true}]}')
  TPL_NAME=$(echo "$tpl_resp" | jq -r '.name')
  TPL_VER=$(echo  "$tpl_resp" | jq -r '.version')
  [[ -z "$TPL_NAME" || "$TPL_NAME" == "null" ]] && die "Could not create connector template"
  ok "Template: $TPL_NAME v$TPL_VER"

  CONSUMER_DID=$(curl_j GET "$PROVIDER_URL/api/v1/mates/all" \
    | jq -r '[.[] | select(.is_me==false)] | first | .participant_id // empty')
  [[ -z "$CONSUMER_DID" ]] && die "Could not resolve consumer DID from provider mates"

  PROVIDER_DID=$(curl -sf "$PROVIDER_URL/.well-known/did.json" | jq -r '.id // empty')
  [[ -z "$PROVIDER_DID" ]] && die "Could not resolve provider DID"
  ok "Provider: ${PROVIDER_DID:0:25}..."
  ok "Consumer: ${CONSUMER_DID:0:25}..."

  local all_keys=(
    CAT0101 CAT0102 CAT0103
    CN0101 CN0102 CN0103 CN0104
    CN0201 CN0202 CN0203 CN0204 CN0205 CN0206 CN0207
    CN0301 CN0302 CN0303 CN0304
    CNC0101 CNC0102 CNC0103 CNC0104
    CNC0201 CNC0202 CNC0203 CNC0204 CNC0205 CNC0206
    CNC0301 CNC0302 CNC0303 CNC0304 CNC0305 CNC0306
  )

  log "Creating ${#all_keys[@]} datasets..."
  for key in "${all_keys[@]}"; do
    local pair
    pair=$(make_dataset "tck-$key")
    DS[$key]="${pair%% *}"
    OFFER[$key]="${pair##* }"
    ok "$key - ds=${DS[$key]:0:30}..."
  done
}

make_dataset() {
  local title=$1
  local d_id dist_id offer_id

  d_id=$(curl_j POST "$PROVIDER_URL/api/v1/catalog-agent/datasets" \
    "$(jq -n --arg t "$title" --arg c "$CATALOG_ID" '{"dctTitle":$t,"catalogId":$c}')" \
    | jq -r '.id')

  dist_id=$(curl_j POST "$PROVIDER_URL/api/v1/catalog-agent/distributions" \
    "$(jq -n --arg d "$d_id" --arg s "$DS_SVC_ID" \
      '{"dctTitle":"dist","dcatAccessService":$s,"datasetId":$d,"dctFormats":"http+pull"}')" \
    | jq -r '.id')

  curl_j POST "$PROVIDER_URL/api/v1/connector/instances" \
    "$(jq -n --arg tn "$TPL_NAME" --arg tv "$TPL_VER" --arg di "$dist_id" \
      '{"templateName":$tn,"templateVersion":$tv,"distributionId":$di,"metadata":{"description":"tck","ownerId":"tck"},"dryRun":false,"parameters":{"ACCESS_URL":"https://jsonplaceholder.typicode.com/posts","ACCESS_METHODS":["GET"],"ACCESS_HEADERS":{"Content-Type":"application/json"}}}')" \
    >/dev/null

  offer_id=$(curl_j POST "$PROVIDER_URL/api/v1/catalog-agent/odrl-policies" \
    "$(jq -n --arg eid "$d_id" \
      '{"odrlOffer":{"permission":[{"action":"use"}]},"entityId":$eid,"entityType":"Dataset"}')" \
    | jq -r '.id')

  echo "$d_id $offer_id"
}

# ── Agreement seeding ────────────────────────────────────────────────────────
seed_agreements() {
  log "Seeding agreements for TCK (all stored in provider DB)"

  local anchor_ds="${DS[CN0101]}"

  log "Creating 16 provider-role agreements (TP*)..."
  for key in TP0101 TP0102 TP0103 TP0104 TP0105 \
             TP0201 TP0202 TP0203 TP0204 TP0205 \
             TP0301 TP0302 TP0303 TP0304 TP0305 TP0306; do
    AGR_PROV[$key]=$(make_agreement "$anchor_ds")
    ok "$key - ${AGR_PROV[$key]:0:40}..."
  done

  log "Creating 16 consumer-role agreements (TPC*)..."
  for key in TPC0101 TPC0102 TPC0103 TPC0104 TPC0105 \
             TPC0201 TPC0202 TPC0203 TPC0204 TPC0205 \
             TPC0301 TPC0302 TPC0303 TPC0304 TPC0305 TPC0306; do
    AGR_CONS[$key]=$(make_agreement "$anchor_ds")
    ok "$key - ${AGR_CONS[$key]:0:40}..."
  done
}

make_agreement() {
  local ds_id=$1
  local lid="urn:uuid:$(new_uuid)"
  local rid="urn:uuid:$(new_uuid)"

  local proc_id
  proc_id=$(curl_j POST "$PROVIDER_URL/api/v1/negotiation-agent/negotiation-processes" \
    "$(jq -n --arg peer "$CONSUMER_DID" --arg lid "$lid" --arg rid "$rid" '{
      "state": "AGREED",
      "associatedAgentPeer": $peer,
      "protocol": "dataspace-protocol-http",
      "role": "PROVIDER",
      "identifiers": {"localId": $lid, "remoteId": $rid}
    }')" | jq -r '.id')

  local msg_id
  msg_id=$(curl_j POST "$PROVIDER_URL/api/v1/negotiation-agent/negotiation-messages" \
    "$(jq -n --arg pid "$proc_id" '{
      "negotiationAgentProcessId": $pid,
      "direction": "OUTGOING",
      "protocol": "dataspace-protocol-http",
      "messageType": "CONTRACT_AGREEMENT",
      "stateTransitionFrom": "ACCEPTED",
      "stateTransitionTo": "AGREED",
      "payload": {}
    }')" | jq -r '.id')

  local agr_uid="urn:agreement:tck-$(new_uuid)"
  curl_j POST "$PROVIDER_URL/api/v1/negotiation-agent/agreements" \
    "$(jq -n --arg pid "$proc_id" --arg mid "$msg_id" \
          --arg cons "$CONSUMER_DID" --arg prov "$PROVIDER_DID" \
          --arg tgt "$ds_id" --arg uid "$agr_uid" '{
      "negotiationAgentProcessId": $pid,
      "negotiationAgentMessageId": $mid,
      "consumerParticipantId": $cons,
      "providerParticipantId": $prov,
      "target": $tgt,
      "agreementContent": {
        "uid": $uid, "target": $tgt,
        "permission": [{"action": "use", "target": $tgt}]
      }
    }')" | jq -r '.id'
}

# ── Write mapping ─────────────────────────────────────────────────────────────
main() {
  seed_catalog
  seed_agreements

  log "Writing mapping to $OUTPUT_MAPPING"
  mkdir -p "$(dirname "$OUTPUT_MAPPING")"

  local DS_JSON OFFER_JSON AGR_PROV_JSON AGR_CONS_JSON

  DS_JSON=$(for k in "${(@k)DS}"; do printf '%s\t%s\n' "$k" "${DS[$k]}"; done \
    | jq -Rs 'split("\n")|map(select(length>0))|map(split("\t"))|map({(.[0]): .[1]})|add')
  OFFER_JSON=$(for k in "${(@k)OFFER}"; do printf '%s\t%s\n' "$k" "${OFFER[$k]}"; done \
    | jq -Rs 'split("\n")|map(select(length>0))|map(split("\t"))|map({(.[0]): .[1]})|add')
  AGR_PROV_JSON=$(for k in "${(@k)AGR_PROV}"; do printf '%s\t%s\n' "$k" "${AGR_PROV[$k]}"; done \
    | jq -Rs 'split("\n")|map(select(length>0))|map(split("\t"))|map({(.[0]): .[1]})|add')
  AGR_CONS_JSON=$(for k in "${(@k)AGR_CONS}"; do printf '%s\t%s\n' "$k" "${AGR_CONS[$k]}"; done \
    | jq -Rs 'split("\n")|map(select(length>0))|map(split("\t"))|map({(.[0]): .[1]})|add')

  jq -n \
    --argjson ds    "$DS_JSON" \
    --argjson offer "$OFFER_JSON" \
    --argjson agr_p "$AGR_PROV_JSON" \
    --argjson agr_c "$AGR_CONS_JSON" \
    '{datasets: $ds, offers: $offer, provider_agreements: $agr_p, consumer_agreements: $agr_c}' \
    > "$OUTPUT_MAPPING"

  ok "TCK seeding complete - $OUTPUT_MAPPING"
}

main
