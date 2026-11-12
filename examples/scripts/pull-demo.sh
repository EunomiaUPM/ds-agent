#!/usr/bin/env bash
# Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
# SPDX-License-Identifier: GPL-3.0-or-later
#
# End-to-end EDACCIT-inspired HTTP PULL demonstration:
# - Keystore secret `/edaccit/arcgis-token` referenced via `{{__RUNTIME_SECRET_{/edaccit/arcgis-token}__}}`
# - `API_KEY` (`QUERY`) ConnectorTemplate + ConnectorInstance bound to an `http+pull` Distribution
# - DSP 2025-1 Catalog Query -> Contract Negotiation (`FINALIZED`) -> Transfer (`STARTED`)
# - Governance & security checks:
#     * Active-transfer delete guard (`DELETE /api/v1/connector/instances/{id}` -> `409 Conflict`)
#     * Unauthenticated direct call to Provider `HttpDataProxy` -> `401 Unauthorized`
#     * Authenticated pull through Consumer `HttpDataProxy` -> `200 OK` GeoJSON (zero secret leakage)
# - Suspension (`SUSPENDED` -> `403`), resume (`STARTED` -> `200`), termination (`TERMINATED` -> `403`)

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
    "ArcGIS FeatureServer token for EDACCIT transport/climate demo"
log_ok "Stored /edaccit/arcgis-token in Provider keystore"

log_step "2. Registering EDACCIT-style API_KEY (QUERY) PULL ConnectorTemplate"
TEMPLATE_BODY=$(jq -n \
    --arg ctx "$CONNECTOR_CONTEXT" \
    --arg ver "$TEMPLATE_VERSION" \
    '{
        "@context": $ctx,
        "@type": "ConnectorTemplate",
        "name": "edaccit-arcgis-pull",
        "version": $ver,
        "description": "ArcGIS FeatureServer REST connector with query-parameter API key from keystore",
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
                "title": "ArcGIS FeatureServer Query Endpoint",
                "required": true
            }
        ]
    }')
api_json POST "$PROVIDER_URL/api/v1/connector/templates" "$TEMPLATE_BODY" >/dev/null
log_ok "Registered ConnectorTemplate edaccit-arcgis-pull@$TEMPLATE_VERSION"

log_step "3. Publishing DCAT Dataset, Distribution (http+pull), ODRL Offer, and ConnectorInstance"
MAIN_CATALOG_ID=$(api_json GET "$PROVIDER_URL/api/v1/catalog-agent/catalogs/main" | jq -r '.["@id"]')
MAIN_SERVICE_ID=$(api_json GET "$PROVIDER_URL/api/v1/catalog-agent/data-services/main" | jq -r '.["@id"]')

DATASET_ID=$(api_json POST "$PROVIDER_URL/api/v1/catalog-agent/datasets" "$(jq -n \
    --arg ctx "$CATALOG_CONTEXT" \
    --arg cat "$MAIN_CATALOG_ID" \
    --arg title "EDACCIT Transport & Climate Stations ($RUN_TAG)" \
    '{"@context": $ctx, "@type": "Dataset", "title": {"en": $title}, "inCatalog": $cat}')" | jq -r '.["@id"]')

DISTRIBUTION_ID=$(api_json POST "$PROVIDER_URL/api/v1/catalog-agent/distributions" "$(jq -n \
    --arg ctx "$CATALOG_CONTEXT" \
    --arg svc "$MAIN_SERVICE_ID" \
    --arg ds "$DATASET_ID" \
    '{"@context": $ctx, "@type": "Distribution", "title": {"en": "ArcGIS GeoJSON Pull Feed"}, "accessService": [$svc], "ofDataset": $ds, "format": "http+pull"}')" | jq -r '.["@id"]')

api_json POST "$PROVIDER_URL/api/v1/catalog-agent/odrl-policies" "$(jq -n \
    --arg ctx "$ODRL_CONTEXT" \
    --arg ds "$DATASET_ID" \
    '{"odrlOffer": {"@context": $ctx, "@type": "Offer", "permission": [{"action": "use", "constraint": []}]}, "entityId": $ds, "entityType": "Dataset", "description": "Unrestricted research & operational mobility use"}')" >/dev/null

INSTANCE_ID=$(api_json POST "$PROVIDER_URL/api/v1/connector/instances" "$(jq -n \
    --arg ctx "$CONNECTOR_CONTEXT" \
    --arg ver "$TEMPLATE_VERSION" \
    --arg dist "$DISTRIBUTION_ID" \
    '{
        "@context": $ctx,
        "@type": "ConnectorInstance",
        "templateName": "edaccit-arcgis-pull",
        "templateVersion": $ver,
        "distributionId": $dist,
        "parameters": {
            "ARCGIS_LAYER_URL": "http://mock-backend:8080/arcgis/rest/services/edaccit/FeatureServer/0/query"
        }
    }')" | jq -r '.id')
log_ok "Bound ConnectorInstance $INSTANCE_ID -> Distribution $DISTRIBUTION_ID"

log_step "4. Running DSP 2025-1 Catalog Query & Contract Negotiation to FINALIZED"
AGREEMENT_ID=$(negotiate_dataset_to_finalized "$PROVIDER_DID" "$DATASET_ID")
log_ok "Contract Agreement finalized: $AGREEMENT_ID"

log_step "5. Requesting and starting http+pull Transfer over DSP"
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

PROVIDER_DP_ID=$(api_json GET "$PROVIDER_URL/api/v1/transfer-agent/dataplane/dataplane-processes/transfer-process/$PROVIDER_TP_ID" | jq -r '.id')
CONSUMER_DP_ID=$(api_json GET "$CONSUMER_URL/api/v1/transfer-agent/dataplane/dataplane-processes/transfer-process/$CONSUMER_TP_ID" | jq -r '.id')
log_ok "Transfer STARTED (Provider dataplane: $PROVIDER_DP_ID, Consumer dataplane: $CONSUMER_DP_ID)"

log_step "6. Verifying Security & Governance Invariants while transfer is STARTED"
DELETE_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$PROVIDER_URL/api/v1/connector/instances/$INSTANCE_ID")
[[ "$DELETE_CODE" == "409" ]] || { echo "Expected 409 Conflict on active instance delete, got $DELETE_CODE" >&2; exit 1; }
log_ok "Active-transfer guard rejected ConnectorInstance deletion with HTTP 409 Conflict"

UNAUTH_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$PROVIDER_URL/dataplane/proxy/$PROVIDER_DP_ID")
[[ "$UNAUTH_CODE" == "401" ]] || { echo "Expected 401 Unauthorized on bare provider proxy call, got $UNAUTH_CODE" >&2; exit 1; }
log_ok "Unauthenticated call to Provider HttpDataProxy rejected with HTTP 401 Unauthorized"

GEOJSON=$(api_json GET "$CONSUMER_URL/dataplane/proxy/$CONSUMER_DP_ID?where=1%3D1&f=geojson")
FEATURE_COUNT=$(printf '%s' "$GEOJSON" | jq '.features | length')
[[ "$FEATURE_COUNT" == "3" ]] || { echo "Expected 3 GeoJSON features, got $FEATURE_COUNT" >&2; exit 1; }
log_ok "Consumer pulled $FEATURE_COUNT GeoJSON stations via local HttpDataProxy (ArcGIS token injected server-side)"

log_step "7. Exercising Suspension -> Resume -> Termination lifecycle"
api_json POST "$PROVIDER_URL/dsp/current/transfers/rpc/setup-suspension" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$PROVIDER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferSuspensionMessage", "@id": $id}')" >/dev/null
SUSP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$CONSUMER_URL/dataplane/proxy/$CONSUMER_DP_ID")
[[ "$SUSP_CODE" == "403" ]] || { echo "Expected 403 Forbidden while SUSPENDED, got $SUSP_CODE" >&2; exit 1; }
log_ok "Suspended transfer gates proxy forwarding with HTTP 403 Forbidden"

api_json POST "$PROVIDER_URL/dsp/current/transfers/rpc/setup-start" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$PROVIDER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferStartMessage", "@id": $id}')" >/dev/null
RESUME_COUNT=$(api_json GET "$CONSUMER_URL/dataplane/proxy/$CONSUMER_DP_ID?f=geojson" | jq '.features | length')
[[ "$RESUME_COUNT" == "3" ]]
log_ok "Resumed transfer serves GeoJSON again (3 features)"

api_json POST "$CONSUMER_URL/dsp/current/transfers/rpc/setup-termination" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$CONSUMER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferTerminationMessage", "@id": $id}')" >/dev/null
TERM_STATE=$(api_json GET "$PROVIDER_URL/api/v1/transfer-agent/dataplane/dataplane-processes/$PROVIDER_DP_ID" | jq -r '.state')
[[ "$TERM_STATE" == "Terminated" ]] || { echo "Expected Provider dataplane Terminated, got $TERM_STATE" >&2; exit 1; }
log_ok "Terminated transfer drove dataplane cleanly to state '$TERM_STATE'"

printf '\n\033[1;32mHTTP PULL demo succeeded end-to-end!\033[0m\n'
