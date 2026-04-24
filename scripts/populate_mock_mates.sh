#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
BASE_URL="${BASE_URL:-http://127.0.0.1:1200}"
MATES_URL="$BASE_URL/api/v1/mates"

# ==========================================
# HELPER: CURL & LOGGING
# ==========================================
invoke_curl() {
    local method="$1"
    local url="$2"
    local body="$3"
    local description="$4"

    echo "--- $description ---" >&2
    echo "Request: $method $url" >&2

    if [[ -n "$body" ]]; then
        response=$(curl -s -X "$method" -H "Content-Type: application/json" -d "$body" "$url")
    else
        response=$(curl -s -X "$method" "$url")
    fi

    echo "Response (preview): ${response:0:120}..." >&2
    echo "---------------------------------------------------" >&2

    echo "$response"
}

# ==========================================
# API WRAPPER
# ==========================================
create_mate() {
    local participant_id="$1"
    local participant_slug="$2"
    local participant_type="$3"   # "Agent" | "Authority"
    local base_url="$4"
    local extra_fields="$5"
    local description="$6"

    local body
    body=$(jq -n \
        --arg pid    "$participant_id" \
        --arg slug   "$participant_slug" \
        --arg type   "$participant_type" \
        --arg url    "$base_url" \
        --argjson ef "$extra_fields" \
        '{
            "participant_id":   $pid,
            "participant_slug": $slug,
            "participant_type": $type,
            "base_url":         $url,
            "token":            null,
            "extra_fields":     $ef,
            "is_me":            false
        }')

    invoke_curl "POST" "$MATES_URL" "$body" "Creating mate: $description" | jq -r '.participant_id // empty'
}

# ==========================================
# MAIN
# ==========================================
main() {
    echo "=== GENERATING MOCK MATES ===" >&2

    # --- Agents ---
    create_mate \
        "did:web:agent:acme-corp" \
        "acme-corp" \
        "Agent" \
        "http://acme-corp.example.com" \
        '{"company_name":"ACME Corp","country":"US","sector":"Manufacturing","contact_email":"dataspace@acme-corp.example.com","vat_id":"US-123456789","certified":true}' \
        "ACME Corp (Agent)" >/dev/null

    create_mate \
        "did:web:agent:globex" \
        "globex" \
        "Agent" \
        "http://globex.example.com" \
        '{"company_name":"Globex Industries","country":"DE","sector":"Energy","contact_email":"ds-ops@globex.example.com","vat_id":"DE-987654321","certified":true}' \
        "Globex Industries (Agent)" >/dev/null

    create_mate \
        "did:web:agent:initech" \
        "initech" \
        "Agent" \
        "http://initech.example.com" \
        '{"company_name":"Initech Solutions","country":"FR","sector":"Finance","contact_email":"tech@initech.example.com","vat_id":"FR-112233445","certified":false}' \
        "Initech Solutions (Agent)" >/dev/null

    create_mate \
        "did:web:agent:umbrella" \
        "umbrella" \
        "Agent" \
        "http://umbrella.example.com" \
        '{"company_name":"Umbrella Data Co","country":"ES","sector":"Healthcare","contact_email":"data@umbrella.example.com","vat_id":"ES-556677889","certified":true}' \
        "Umbrella Data Co (Agent)" >/dev/null

    # --- Authorities ---
    create_mate \
        "did:web:authority:trust-anchor-eu" \
        "trust-anchor-eu" \
        "Authority" \
        "http://trust-anchor.eu.example.com" \
        '{"authority_name":"EU Trust Anchor","jurisdiction":"EU","trust_framework":"GAIA-X","contact_email":"trust@trust-anchor.eu.example.com","accreditation_id":"EU-TA-0001"}' \
        "EU Trust Anchor (Authority)" >/dev/null

    create_mate \
        "did:web:authority:gaia-x-hub" \
        "gaia-x-hub" \
        "Authority" \
        "http://gaia-x-hub.example.com" \
        '{"authority_name":"Gaia-X Hub","jurisdiction":"EU","trust_framework":"GAIA-X","contact_email":"registry@gaia-x-hub.example.com","accreditation_id":"GX-HUB-0042"}' \
        "Gaia-X Hub (Authority)" >/dev/null

    create_mate \
        "did:web:authority:upm-registry" \
        "upm-registry" \
        "Authority" \
        "http://registry.upm.es" \
        '{"authority_name":"UPM Registry","jurisdiction":"ES","trust_framework":"FIWARE","contact_email":"registry@upm.es","accreditation_id":"ES-UPM-REG-001"}' \
        "UPM Registry (Authority)" >/dev/null

    echo "=== DONE. GENERATED 7 MATES (4 Agents + 3 Authorities) ===" >&2
}

main "$@"
