#!/usr/bin/env bash
# Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Zero-authority SSI onboarding between Provider (:1200) and Consumer (:1100):
# 1. Links each agent's Fafnir wallet (`POST /api/v1/wallet/link`).
# 2. Self-attests Gaia-X `LegalPerson` VCs into each wallet (`POST /api/v1/gaia/generate`).
# 3. Performs mutual GNAP + OID4VP peer connection (`POST /api/v1/peer-connection/connect`).

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./common.sh
source "$SCRIPT_DIR/common.sh"

require_tools

wait_for_url() {
    local label="$1"
    local url="$2"
    for _ in $(seq 1 60); do
        if curl -sSf "$url" >/dev/null 2>&1; then
            log_ok "$label is ready ($url)"
            return 0
        fi
        sleep 2
    done
    echo "ERROR: Timed out waiting for $label at $url" >&2
    exit 1
}

wait_for_peer_token() {
    local caller_url="$1"
    local peer_did="$2"
    for _ in $(seq 1 60); do
        local token
        token=$(api_json GET "$caller_url/api/v1/peer-connection/request/all" \
            | jq -r --arg did "$peer_did" \
                '.items[] | select(.participant_id == $did and .status == "Approved") | .final_token // empty' \
            | head -n 1)
        if [[ -n "$token" ]]; then
            printf '%s' "$token"
            return 0
        fi
        sleep 1
    done
    echo "ERROR: Timed out waiting for Approved GNAP peer token towards $peer_did" >&2
    exit 1
}

log_step "1. Checking readiness of Provider, Consumer, and Mock Backend"
wait_for_url "Mock Backend" "$MOCK_URL/health"
wait_for_url "Provider" "$PROVIDER_URL/.well-known/dspace-version"
wait_for_url "Consumer" "$CONSUMER_URL/.well-known/dspace-version"

log_step "2. Linking Fafnir wallets and issuing self-attested Gaia-X LegalPerson VCs"
for url in "$PROVIDER_URL" "$CONSUMER_URL"; do
    api_json POST "$url/api/v1/wallet/link" '{}' >/dev/null
    api_json POST "$url/api/v1/gaia/generate" '{}' >/dev/null
done

PROVIDER_DID=$(api_json GET "$PROVIDER_URL/.well-known/did.json" | jq -r '.id')
CONSUMER_DID=$(api_json GET "$CONSUMER_URL/.well-known/did.json" | jq -r '.id')
log_ok "Provider DID: $PROVIDER_DID"
log_ok "Consumer DID: $CONSUMER_DID"

log_step "3. Onboarding Consumer -> Provider over GNAP + OID4VP"
api_json POST "$CONSUMER_URL/api/v1/peer-connection/connect" "$(jq -n \
    --arg id "$PROVIDER_DID" \
    '{"id": $id, "nick": "provider", "url": "http://provider:1200/api/v1/gate/access", "actions": ["talk"], "visibility": "Public", "auto": true}')" >/dev/null
C2P_TOKEN=$(wait_for_peer_token "$CONSUMER_URL" "$PROVIDER_DID")
VERIFIED_C=$(api_json POST "$PROVIDER_URL/api/v1/gate/token/verify" "$(jq -n --arg t "$C2P_TOKEN" '{"token": $t}')" | jq -r '.participant_id')
[[ "$VERIFIED_C" == "$CONSUMER_DID" ]]
log_ok "Consumer holds verified GNAP token for Provider"

log_step "4. Onboarding Provider -> Consumer over GNAP + OID4VP"
api_json POST "$PROVIDER_URL/api/v1/peer-connection/connect" "$(jq -n \
    --arg id "$CONSUMER_DID" \
    '{"id": $id, "nick": "consumer", "url": "http://consumer:1100/api/v1/gate/access", "actions": ["talk"], "visibility": "Public", "auto": true}')" >/dev/null
P2C_TOKEN=$(wait_for_peer_token "$PROVIDER_URL" "$CONSUMER_DID")
VERIFIED_P=$(api_json POST "$CONSUMER_URL/api/v1/gate/token/verify" "$(jq -n --arg t "$P2C_TOKEN" '{"token": $t}')" | jq -r '.participant_id')
[[ "$VERIFIED_P" == "$PROVIDER_DID" ]]
log_ok "Provider holds verified GNAP token for Consumer"

printf '\n\033[1;32mOnboarding complete! Run ./examples/scripts/pull-demo.sh or ./examples/scripts/push-demo.sh next.\033[0m\n'
