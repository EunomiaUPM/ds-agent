#!/usr/bin/env bash
# Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Shared helpers for the reproducible HTTP PULL and HTTP PUSH example scripts.

set -euo pipefail

PROVIDER_URL="${PROVIDER_URL:-http://127.0.0.1:1200}"
CONSUMER_URL="${CONSUMER_URL:-http://127.0.0.1:1100}"
MOCK_URL="${MOCK_URL:-http://127.0.0.1:18080}"

EUNOMIA_CONTEXT="https://w3id.org/eunomia/v1/context.jsonld"
CATALOG_CONTEXT="https://w3id.org/eunomia/v1/catalog-context.jsonld"
CONNECTOR_CONTEXT="https://w3id.org/eunomia/v1/connector-context.jsonld"
KEYSTORE_CONTEXT="https://w3id.org/eunomia/v1/keystore-context.jsonld"
ODRL_CONTEXT="http://www.w3.org/ns/odrl.jsonld"

require_tools() {
    for bin in curl jq; do
        if ! command -v "$bin" >/dev/null 2>&1; then
            echo "ERROR: '$bin' is required to run this example." >&2
            exit 1
        fi
    done
}

log_step() {
    printf '\n\033[1;34m==> %s\033[0m\n' "$1" >&2
}

log_ok() {
    printf '\033[1;32m  [OK]\033[0m %s\n' "$1" >&2
}

# Performs an HTTP JSON request and fails loudly on >= 400 status codes.
api_json() {
    local method="$1"
    local url="$2"
    local body="${3:-}"
    local tmp http_code

    tmp="$(mktemp)"
    if [[ -n "$body" ]]; then
        http_code=$(curl -sS -o "$tmp" -w "%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$body" \
            "$url")
    else
        http_code=$(curl -sS -o "$tmp" -w "%{http_code}" -X "$method" "$url")
    fi

    if [[ "$http_code" -lt 200 || "$http_code" -ge 300 ]]; then
        echo "ERROR: $method $url -> HTTP $http_code" >&2
        cat "$tmp" >&2
        echo "" >&2
        rm -f "$tmp"
        return 1
    fi
    cat "$tmp"
    rm -f "$tmp"
}

# Upserts a secret in an agent's keystore (`POST /api/v1/keystore/secrets`, or `PUT` if it exists).
upsert_keystore_secret() {
    local agent_url="$1"
    local key="$2"
    local value="$3"
    local description="$4"
    local key_no_slash="${key#/}"
    local tmp code

    tmp="$(mktemp)"
    code=$(curl -sS -o "$tmp" -w "%{http_code}" -X POST \
        -H "Content-Type: application/json" \
        -d "$(jq -n \
            --arg ctx "$KEYSTORE_CONTEXT" \
            --arg key "$key" \
            --arg val "$value" \
            --arg desc "$description" \
            '{"@context": $ctx, "@type": "Secret", "key": $key, "value": $val, "description": $desc}')" \
        "$agent_url/api/v1/keystore/secrets")

    if [[ "$code" == "201" || "$code" == "200" ]]; then
        rm -f "$tmp"
        return 0
    fi

    # Already exists: fetch current version and update optimistically.
    local current_ver
    current_ver=$(api_json GET "$agent_url/api/v1/keystore/secrets/$key_no_slash" | jq -r '.version')
    api_json PUT "$agent_url/api/v1/keystore/secrets/$key_no_slash" "$(jq -n \
        --arg ctx "$KEYSTORE_CONTEXT" \
        --arg val "$value" \
        --argjson ver "$current_ver" \
        --arg desc "$description" \
        '{"@context": $ctx, "@type": "Secret", "value": $val, "expectedVersion": $ver, "description": $desc}')" >/dev/null
    rm -f "$tmp"
}

# Queries the Provider's DSP catalog from the Consumer, locates `$target_dataset_id`,
# and drives DSP 2025-1 contract negotiation over RPC to `FINALIZED`.
# Prints the resulting `agreementId` to stdout.
negotiate_dataset_to_finalized() {
    local provider_did="$1"
    local target_dataset_id="$2"

    local catalog_resp dataset_json policy_id provider_dsp callback
    catalog_resp=$(api_json POST "$CONSUMER_URL/dsp/current/catalog/rpc/setup-catalog-request" "$(jq -n \
        --arg ctx "$EUNOMIA_CONTEXT" \
        --arg peer "$provider_did" \
        '{"@context": $ctx, "@type": "RpcCatalogRequestMessage", "associatedAgentPeer": $peer, "noCache": true}')")

    dataset_json=$(printf '%s' "$catalog_resp" | jq -c --arg ds "$target_dataset_id" \
        '.response.dataset[] | select(.["@id"] == $ds)')
    if [[ -z "$dataset_json" ]]; then
        echo "ERROR: Dataset $target_dataset_id not found in provider DSP catalog" >&2
        exit 1
    fi

    policy_id=$(printf '%s' "$dataset_json" | jq -r '.hasPolicy[0]["@id"]')
    provider_dsp=$(printf '%s' "$catalog_resp" | jq -r '.response.service[0].endpointURL')
    callback="http://consumer:1100/dsp/current"

    local offer_obj
    offer_obj=$(jq -n --arg pid "$policy_id" --arg target "$target_dataset_id" \
        '{"@id": $pid, "target": $target, "@type": "Offer", "permission": [{"action": "use"}]}')

    local init_resp consumer_proc_id consumer_pid provider_proc_id
    init_resp=$(api_json POST "$CONSUMER_URL/dsp/current/negotiations/rpc/setup-request-init" "$(jq -n \
        --arg ctx "$EUNOMIA_CONTEXT" \
        --arg peer "$provider_did" \
        --arg pdsp "$provider_dsp" \
        --arg cb "$callback" \
        --argjson offer "$offer_obj" \
        '{"@context": $ctx, "@type": "RpcNegotiationRequestInitMessage", "associatedAgentPeer": $peer, "providerAddress": $pdsp, "callbackAddress": $cb, "offer": $offer}')")

    consumer_proc_id=$(printf '%s' "$init_resp" | jq -r '.negotiationAgentModel.id')
    consumer_pid=$(printf '%s' "$init_resp" | jq -r '.negotiationAgentModel.correlation.identifiers.consumerPid')

    provider_proc_id=$(api_json GET "$PROVIDER_URL/api/v1/negotiation-agent/negotiation-processes" \
        | jq -r --arg cpid "$consumer_pid" '.items[] | select(.correlation.identifiers.consumerPid == $cpid) | .id')

    # REQUESTED -> OFFERED -> ACCEPTED -> AGREED -> VERIFIED -> FINALIZED
    api_json POST "$PROVIDER_URL/dsp/current/negotiations/rpc/setup-offer" "$(jq -n \
        --arg ctx "$EUNOMIA_CONTEXT" --arg id "$provider_proc_id" --argjson offer "$offer_obj" \
        '{"@context": $ctx, "@type": "RpcNegotiationOfferMessage", "@id": $id, "offer": $offer}')" >/dev/null

    api_json POST "$CONSUMER_URL/dsp/current/negotiations/rpc/setup-acceptance" "$(jq -n \
        --arg ctx "$EUNOMIA_CONTEXT" --arg id "$consumer_proc_id" \
        '{"@context": $ctx, "@type": "RpcNegotiationAcceptanceMessage", "@id": $id}')" >/dev/null

    api_json POST "$PROVIDER_URL/dsp/current/negotiations/rpc/setup-agreement" "$(jq -n \
        --arg ctx "$EUNOMIA_CONTEXT" --arg id "$provider_proc_id" \
        '{"@context": $ctx, "@type": "RpcNegotiationAgreementMessage", "@id": $id}')" >/dev/null

    api_json POST "$CONSUMER_URL/dsp/current/negotiations/rpc/setup-verification" "$(jq -n \
        --arg ctx "$EUNOMIA_CONTEXT" --arg id "$consumer_proc_id" \
        '{"@context": $ctx, "@type": "RpcNegotiationVerificationMessage", "@id": $id}')" >/dev/null

    api_json POST "$PROVIDER_URL/dsp/current/negotiations/rpc/setup-finalization" "$(jq -n \
        --arg ctx "$EUNOMIA_CONTEXT" --arg id "$provider_proc_id" \
        '{"@context": $ctx, "@type": "RpcNegotiationFinalizationMessage", "@id": $id}')" >/dev/null

    api_json GET "$PROVIDER_URL/api/v1/negotiation-agent/agreements/process/$provider_proc_id" \
        | jq -r '.items[0].id'
}
