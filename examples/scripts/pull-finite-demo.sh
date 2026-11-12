#!/usr/bin/env bash
# Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
# SPDX-License-Identifier: GPL-3.0-or-later
#
# End-to-end FINITE (single-shot) HTTP PULL demonstration:
# - ConnectorTemplate with `"lifetime": "FINITE"` and `"maxRetries": 1`
# - DSP 2025-1 Catalog Query -> Contract Negotiation (`FINALIZED`) -> Transfer (`STARTED`)
# - Consumer pulls GeoJSON snapshot once through its local `HttpDataProxy` -> `200 OK`
# - Provider data plane emits `DataplaneSignal::Completed` at stream EOF, driving
#   `TransferCompletionMessage` so BOTH Provider and Consumer automatically transition to
#   `COMPLETED` (and dataplanes to `Terminated`)
# - A second pull attempt is rejected (`403 Forbidden` / `410 Gone`)

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./common.sh
source "$SCRIPT_DIR/common.sh"

require_tools

RUN_TAG="$(date +%s)"
TEMPLATE_VERSION="1.0.$RUN_TAG"
PROVIDER_DID=$(api_json GET "$PROVIDER_URL/.well-known/did.json" | jq -r '.id')

log_step "1. Seeding upstream ArcGIS API key in Provider keystore"
upsert_keystore_secret \
    "$PROVIDER_URL" \
    "/edaccit/arcgis-token" \
    "edaccit-demo-key-2026" \
    "ArcGIS FeatureServer token for EDACCIT single-shot snapshot demo"
log_ok "Stored /edaccit/arcgis-token in Provider keystore"

log_step "2. Registering FINITE (single-shot) PULL ConnectorTemplate"
TEMPLATE_BODY=$(jq -n \
    --arg ctx "$CONNECTOR_CONTEXT" \
    --arg ver "$TEMPLATE_VERSION" \
    '{
        "@context": $ctx,
        "@type": "ConnectorTemplate",
        "name": "edaccit-arcgis-pull-finite",
        "version": $ver,
        "description": "Single-shot snapshot PULL connector that auto-completes the DSP transfer on EOF",
        "authentication": {
            "type": "API_KEY",
            "key": "token",
            "location": "QUERY",
            "value": {
                "type": "PLAIN",
                "content": "{{__RUNTIME_SECRET_{/edaccit/arcgis-token}__}}"
            }
        },
        "interaction": {
            "mode": "PULL",
            "lifetime": "FINITE",
            "maxRetries": 1,
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "{{__ARCGIS_LAYER_URL__}}",
                "method": ["GET"],
                "headers": {},
                "bodyTemplate": ""
            }
        },
        "parameters": [
            {
                "paramType": "STRING",
                "name": "ARCGIS_LAYER_URL",
                "title": "ArcGIS FeatureServer Snapshot Endpoint",
                "required": true
            }
        ]
    }')
api_json POST "$PROVIDER_URL/api/v1/connector/templates" "$TEMPLATE_BODY" >/dev/null
log_ok "Registered ConnectorTemplate edaccit-arcgis-pull-finite@$TEMPLATE_VERSION (lifetime=FINITE, maxRetries=1)"

log_step "3. Publishing DCAT Dataset, Distribution (http+pull), ODRL Offer, and ConnectorInstance"
MAIN_CATALOG_ID=$(api_json GET "$PROVIDER_URL/api/v1/catalog-agent/catalogs/main" | jq -r '.["@id"]')
MAIN_SERVICE_ID=$(api_json GET "$PROVIDER_URL/api/v1/catalog-agent/data-services/main" | jq -r '.["@id"]')

DATASET_ID=$(api_json POST "$PROVIDER_URL/api/v1/catalog-agent/datasets" "$(jq -n \
    --arg ctx "$CATALOG_CONTEXT" \
    --arg cat "$MAIN_CATALOG_ID" \
    --arg title "EDACCIT Single-Shot Snapshot ($RUN_TAG)" \
    '{"@context": $ctx, "@type": "Dataset", "title": {"en": $title}, "inCatalog": $cat}')" | jq -r '.["@id"]')

DISTRIBUTION_ID=$(api_json POST "$PROVIDER_URL/api/v1/catalog-agent/distributions" "$(jq -n \
    --arg ctx "$CATALOG_CONTEXT" \
    --arg svc "$MAIN_SERVICE_ID" \
    --arg ds "$DATASET_ID" \
    '{"@context": $ctx, "@type": "Distribution", "title": {"en": "ArcGIS GeoJSON Snapshot"}, "accessService": [$svc], "ofDataset": $ds, "format": "http+pull"}')" | jq -r '.["@id"]')

api_json POST "$PROVIDER_URL/api/v1/catalog-agent/odrl-policies" "$(jq -n \
    --arg ctx "$ODRL_CONTEXT" \
    --arg ds "$DATASET_ID" \
    '{"odrlOffer": {"@context": $ctx, "@type": "Offer", "permission": [{"action": "use", "constraint": []}]}, "entityId": $ds, "entityType": "Dataset", "description": "Single snapshot download"}')" >/dev/null

INSTANCE_ID=$(api_json POST "$PROVIDER_URL/api/v1/connector/instances" "$(jq -n \
    --arg ctx "$CONNECTOR_CONTEXT" \
    --arg ver "$TEMPLATE_VERSION" \
    --arg dist "$DISTRIBUTION_ID" \
    '{
        "@context": $ctx,
        "@type": "ConnectorInstance",
        "templateName": "edaccit-arcgis-pull-finite",
        "templateVersion": $ver,
        "distributionId": $dist,
        "parameters": {
            "ARCGIS_LAYER_URL": "http://mock-backend:8080/arcgis/rest/services/edaccit/FeatureServer/0/query"
        }
    }')" | jq -r '.id')
log_ok "Bound ConnectorInstance $INSTANCE_ID -> Distribution $DISTRIBUTION_ID"

log_step "4. Negotiating Agreement and starting http+pull Transfer over DSP"
AGREEMENT_ID=$(negotiate_dataset_to_finalized "$PROVIDER_DID" "$DATASET_ID")
REQ_RESP=$(api_json POST "$CONSUMER_URL/dsp/current/transfers/rpc/setup-request" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" \
    --arg peer "$PROVIDER_DID" \
    --arg ag "$AGREEMENT_ID" \
    '{
        "@context": $ctx,
        "@type": "RpcTransferRequestMessage",
        "associatedAgentPeer": $peer,
        "agreementId": $ag,
        "format": "http+pull",
        "providerAddress": "http://provider:1200/dsp/current"
    }')")
CONSUMER_TP_ID=$(printf '%s' "$REQ_RESP" | jq -r '.transferAgentModel.id')
PROVIDER_PID=$(printf '%s' "$REQ_RESP" | jq -r '.transferAgentModel.correlation.providerPid')
PROVIDER_TP_ID=$(api_json GET "$PROVIDER_URL/api/v1/transfer-agent-ref/transfer-processes" \
    | jq -r --arg ppid "$PROVIDER_PID" '.items[] | select(.correlation.providerPid == $ppid) | .id')

api_json POST "$PROVIDER_URL/dsp/current/transfers/rpc/setup-start" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$PROVIDER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferStartMessage", "@id": $id}')" >/dev/null

CONSUMER_DP_ID=$(api_json GET "$CONSUMER_URL/api/v1/transfer-agent/dataplane/dataplane-processes/transfer-process/$CONSUMER_TP_ID" | jq -r '.id')
log_ok "Transfer STARTED (Consumer dataplane: $CONSUMER_DP_ID)"

log_step "5. Executing single-shot pull and verifying autonomous DSP COMPLETED signaling"
GEOJSON=$(api_json GET "$CONSUMER_URL/dataplane/proxy/$CONSUMER_DP_ID?where=1%3D1&f=geojson")
FEATURE_COUNT=$(printf '%s' "$GEOJSON" | jq '.features | length')
[[ "$FEATURE_COUNT" == "3" ]] || { echo "Expected 3 GeoJSON features, got $FEATURE_COUNT" >&2; exit 1; }
log_ok "Consumer pulled $FEATURE_COUNT GeoJSON stations via single-shot proxy"

for _ in {1..40}; do
    PROV_STATE=$(api_json GET "$PROVIDER_URL/api/v1/transfer-agent-ref/transfer-processes/$PROVIDER_TP_ID" | jq -r '.state')
    CONS_STATE=$(api_json GET "$CONSUMER_URL/api/v1/transfer-agent-ref/transfer-processes/$CONSUMER_TP_ID" | jq -r '.state')
    if [[ "$PROV_STATE" == "COMPLETED" && "$CONS_STATE" == "COMPLETED" ]]; then
        break
    fi
    sleep 0.25
done
[[ "$PROV_STATE" == "COMPLETED" && "$CONS_STATE" == "COMPLETED" ]] || {
    echo "Expected both sides COMPLETED, got provider=$PROV_STATE consumer=$CONS_STATE" >&2
    exit 1
}
log_ok "DataplaneSignal::Completed automatically drove Provider ($PROV_STATE) and Consumer ($CONS_STATE) to COMPLETED"

SECOND_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$CONSUMER_URL/dataplane/proxy/$CONSUMER_DP_ID?where=1%3D1&f=geojson")
[[ "$SECOND_CODE" == "403" || "$SECOND_CODE" == "410" ]] || {
    echo "Expected 403 or 410 on second pull after FINITE completion, got $SECOND_CODE" >&2
    exit 1
}
log_ok "Subsequent pull rejected with HTTP $SECOND_CODE"

printf '\n\033[1;32mFINITE HTTP PULL demo succeeded end-to-end!\033[0m\n'
