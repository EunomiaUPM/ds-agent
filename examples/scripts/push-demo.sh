#!/usr/bin/env bash
# Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
# SPDX-License-Identifier: GPL-3.0-or-later
#
# End-to-end ECOSTARS-inspired HTTP PUSH demonstration:
# - Keystore secret `/ecostars/webhook-token` resolved at runtime via `{{__RUNTIME_SECRET_{/ecostars/webhook-token}__}}`
# - `BEARER_TOKEN` `PUSH` ConnectorTemplate using `{{__RUNTIME_INGRESS__}}` (which embeds `?ingress_token=...`)
#   and `DELETE .../esg/webhooks/{{__RUNTIME_JSON_{.subscription.id}__}}` on unsubscribe
# - DSP 2025-1 Contract Negotiation (`FINALIZED`) + `http+push` Transfer (`STARTED`)
# - Upstream emits an ESG assessment -> Provider `HttpDataProxy` verifies `?ingress_token=...`, strips it,
#   and forwards to Consumer `HttpDataProxy` -> Consumer sink receives payload with zero provider secret leakage
# - Termination triggers automatic `DELETE /esg/webhooks/sub-...` unsubscription and reaches `Terminated`

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./common.sh
source "$SCRIPT_DIR/common.sh"

require_tools

RUN_TAG="$(date +%s)"
TEMPLATE_VERSION="1.0.$RUN_TAG"
PROVIDER_DID=$(api_json GET "$PROVIDER_URL/.well-known/did.json" | jq -r '.id')

# Clear previous sink events before starting
api_json DELETE "$MOCK_URL/consumer-sink/events" >/dev/null

log_step "1. Seeding upstream ESG webhook secret in Provider keystore"
upsert_keystore_secret \
    "$PROVIDER_URL" \
    "/ecostars/webhook-token" \
    "ecostars-demo-secret-2026" \
    "Bearer token for registering/deregistering ECOSTARS upstream webhooks"
log_ok "Stored /ecostars/webhook-token in Provider keystore"

log_step "2. Registering ECOSTARS-style BEARER_TOKEN PUSH ConnectorTemplate"
TEMPLATE_BODY=$(jq -n \
    --arg ctx "$CONNECTOR_CONTEXT" \
    --arg ver "$TEMPLATE_VERSION" \
    '{
        "@context": $ctx,
        "@type": "ConnectorTemplate",
        "name": "ecostars-webhook-push",
        "version": $ver,
        "description": "Event-driven ESG webhook subscription with automatic JSON-path unsubscription",
        "authentication": {
            "type": "BEARER_TOKEN",
            "token": {
                "type": "PLAIN",
                "content": "{{__RUNTIME_SECRET_{/ecostars/webhook-token}__}}"
            }
        },
        "interaction": {
            "mode": "PUSH",
            "subscribe": {
                "protocol": "HTTP",
                "urlTemplate": "{{__WEBHOOK_BASE_URL__}}",
                "method": ["POST"],
                "headers": {
                    "Content-Type": "application/json"
                },
                "bodyTemplate": "{\"callbackUrl\":\"http://provider:1200{{__RUNTIME_INGRESS__}}\",\"topic\":\"{{__ESG_TOPIC__}}\"}"
            },
            "unsubscribe": {
                "protocol": "HTTP",
                "urlTemplate": "{{__WEBHOOK_BASE_URL__}}/{{__RUNTIME_JSON_{.subscription.id}__}}",
                "method": ["DELETE"],
                "headers": {},
                "bodyTemplate": ""
            }
        },
        "parameters": [
            {
                "paramType": "STRING",
                "name": "WEBHOOK_BASE_URL",
                "title": "Upstream Webhook Endpoint",
                "required": true
            },
            {
                "paramType": "STRING",
                "name": "ESG_TOPIC",
                "title": "Subscribed ESG Topic",
                "required": true
            }
        ]
    }')
api_json POST "$PROVIDER_URL/api/v1/connector/templates" "$TEMPLATE_BODY" >/dev/null
log_ok "Registered ConnectorTemplate ecostars-webhook-push@$TEMPLATE_VERSION"

log_step "3. Publishing DCAT Dataset, Distribution (http+push), ODRL Offer, and ConnectorInstance"
MAIN_CATALOG_ID=$(api_json GET "$PROVIDER_URL/api/v1/catalog-agent/catalogs/main" | jq -r '.["@id"]')
MAIN_SERVICE_ID=$(api_json GET "$PROVIDER_URL/api/v1/catalog-agent/data-services/main" | jq -r '.["@id"]')

DATASET_ID=$(api_json POST "$PROVIDER_URL/api/v1/catalog-agent/datasets" "$(jq -n \
    --arg ctx "$CATALOG_CONTEXT" \
    --arg cat "$MAIN_CATALOG_ID" \
    --arg title "ECOSTARS Fleet ESG Ratings Stream ($RUN_TAG)" \
    '{"@context": $ctx, "@type": "Dataset", "title": {"en": $title}, "inCatalog": $cat}')" | jq -r '.["@id"]')

DISTRIBUTION_ID=$(api_json POST "$PROVIDER_URL/api/v1/catalog-agent/distributions" "$(jq -n \
    --arg ctx "$CATALOG_CONTEXT" \
    --arg svc "$MAIN_SERVICE_ID" \
    --arg ds "$DATASET_ID" \
    '{"@context": $ctx, "@type": "Distribution", "title": {"en": "ESG Webhook Push Stream"}, "accessService": [$svc], "ofDataset": $ds, "format": "http+push"}')" | jq -r '.["@id"]')

api_json POST "$PROVIDER_URL/api/v1/catalog-agent/odrl-policies" "$(jq -n \
    --arg ctx "$ODRL_CONTEXT" \
    --arg ds "$DATASET_ID" \
    '{"odrlOffer": {"@context": $ctx, "@type": "Offer", "permission": [{"action": "use", "constraint": []}]}, "entityId": $ds, "entityType": "Dataset", "description": "Real-time ESG fleet event delivery"}')" >/dev/null

api_json POST "$PROVIDER_URL/api/v1/connector/instances" "$(jq -n \
    --arg ctx "$CONNECTOR_CONTEXT" \
    --arg ver "$TEMPLATE_VERSION" \
    --arg dist "$DISTRIBUTION_ID" \
    '{
        "@context": $ctx,
        "@type": "ConnectorInstance",
        "templateName": "ecostars-webhook-push",
        "templateVersion": $ver,
        "distributionId": $dist,
        "parameters": {
            "WEBHOOK_BASE_URL": "http://mock-backend:8080/esg/webhooks",
            "ESG_TOPIC": "fleet.esg.madrid"
        }
    }')" >/dev/null
log_ok "Bound ConnectorInstance to Distribution $DISTRIBUTION_ID"

log_step "4. Negotiating Contract Agreement over DSP 2025-1"
AGREEMENT_ID=$(negotiate_dataset_to_finalized "$PROVIDER_DID" "$DATASET_ID")
log_ok "Contract Agreement finalized: $AGREEMENT_ID"

log_step "5. Requesting http+push Transfer (with Consumer sink DataAddress) and starting subscription"
REQ_RESP=$(api_json POST "$CONSUMER_URL/dsp/current/transfers/rpc/setup-request" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" \
    --arg peer "$PROVIDER_DID" \
    --arg ag "$AGREEMENT_ID" \
    '{
        "@context": $ctx,
        "@type": "RpcTransferRequestMessage",
        "associatedAgentPeer": $peer,
        "agreementId": $ag,
        "format": "http+push",
        "dataAddress": {
            "endpointType": "HTTP",
            "endpoint": "http://mock-backend:8080/consumer-sink/events"
        },
        "providerAddress": "http://provider:1200/dsp/current"
    }')")
CONSUMER_TP_ID=$(printf '%s' "$REQ_RESP" | jq -r '.transferAgentModel.id')
PROVIDER_PID=$(printf '%s' "$REQ_RESP" | jq -r '.transferAgentModel.correlation.providerPid')
PROVIDER_TP_ID=$(api_json GET "$PROVIDER_URL/api/v1/transfer-agent-ref/transfer-processes" \
    | jq -r --arg ppid "$PROVIDER_PID" '.items[] | select(.correlation.providerPid == $ppid) | .id')

api_json POST "$PROVIDER_URL/dsp/current/transfers/rpc/setup-start" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$PROVIDER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferStartMessage", "@id": $id}')" >/dev/null

PROVIDER_DP_ID=$(api_json GET "$PROVIDER_URL/api/v1/transfer-agent/dataplane/dataplane-processes/transfer-process/$PROVIDER_TP_ID" | jq -r '.id')
ACTIVE_SUBS=$(api_json GET "$MOCK_URL/esg/webhooks" | jq -r '.count')
[[ "$ACTIVE_SUBS" -ge 1 ]] || { echo "Expected >= 1 active upstream webhook subscription, got $ACTIVE_SUBS" >&2; exit 1; }
log_ok "Provider registered upstream webhook subscription (active count: $ACTIVE_SUBS)"

log_step "6. Verifying Ingress Capability Token Enforcement & End-to-End Push Delivery"
SPOOF_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d '{"spoofed": true}' \
    "$PROVIDER_URL/dataplane/proxy/$PROVIDER_DP_ID")
[[ "$SPOOF_CODE" == "401" ]] || { echo "Expected 401 on spoofed push without ?ingress_token, got $SPOOF_CODE" >&2; exit 1; }
log_ok "Spoofed webhook push without ?ingress_token rejected with HTTP 401 Unauthorized"

EMIT_RESP=$(api_json POST "$MOCK_URL/esg/emit" '{"fleetId": "ES-MAD-EMT-901", "euroRating": "5-Star-ZeroTailpipe", "co2GramsPerKm": 0.0}')
DELIVERED_STATUS=$(printf '%s' "$EMIT_RESP" | jq -r '.delivered[0].status')
[[ "$DELIVERED_STATUS" == "200" ]] || { echo "Expected upstream webhook delivery HTTP 200, got: $EMIT_RESP" >&2; exit 1; }

SINK_RESP=$(api_json GET "$MOCK_URL/consumer-sink/events")
SINK_COUNT=$(printf '%s' "$SINK_RESP" | jq -r '.count')
SINK_FLEET=$(printf '%s' "$SINK_RESP" | jq -r '.items[-1].payload.fleetId')
SINK_PATH=$(printf '%s' "$SINK_RESP" | jq -r '.items[-1].path')
SINK_AUTH=$(printf '%s' "$SINK_RESP" | jq -r '.items[-1].headers.authorization // empty')
[[ "$SINK_COUNT" == "1" && "$SINK_FLEET" == "ES-MAD-EMT-901" ]] || { echo "Unexpected sink state: $SINK_RESP" >&2; exit 1; }
[[ "$SINK_PATH" != *"ingress_token"* ]] || { echo "ingress_token leaked to consumer sink path: $SINK_PATH" >&2; exit 1; }
[[ "$SINK_AUTH" != *"ecostars-demo-secret-2026"* ]] || { echo "Provider secret leaked to consumer sink!" >&2; exit 1; }
log_ok "Consumer sink received ESG event for $SINK_FLEET (query token stripped, provider secret isolated)"

log_step "7. Terminating Transfer and verifying automatic upstream DELETE unsubscription"
api_json POST "$CONSUMER_URL/dsp/current/transfers/rpc/setup-termination" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$CONSUMER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferTerminationMessage", "@id": $id}')" >/dev/null

REMAINING_SUBS=$(api_json GET "$MOCK_URL/esg/webhooks" | jq -r '.count')
TERM_STATE=$(api_json GET "$PROVIDER_URL/api/v1/transfer-agent/dataplane/dataplane-processes/$PROVIDER_DP_ID" | jq -r '.state')
[[ "$REMAINING_SUBS" == "0" ]] || { echo "Expected 0 remaining webhook subscriptions after termination, got $REMAINING_SUBS" >&2; exit 1; }
[[ "$TERM_STATE" == "Terminated" ]] || { echo "Expected Provider dataplane Terminated, got $TERM_STATE" >&2; exit 1; }
log_ok "Unsubscribed upstream webhook via {{__RUNTIME_JSON_{.subscription.id}__}} (remaining: 0, state: $TERM_STATE)"

printf '\n\033[1;32mHTTP PUSH demo succeeded end-to-end!\033[0m\n'
