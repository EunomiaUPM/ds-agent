#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
DATA_SPACE_PROVIDER="${DATA_SPACE_PROVIDER:-http://127.0.0.1:1200}"
#DATA_SPACE_PROVIDER=https://dev-dataspaces.dit.upm.es:1200
STATIC_API="${STATIC_API:-http://127.0.0.1:8081}"
DYNAMIC_API="${DYNAMIC_API:-http://127.0.0.1:8082}"
#STATIC_API="${STATIC_API:-http://host.docker.internal:8081}"
#DYNAMIC_API="${DYNAMIC_API:-http://host.docker.internal:8082}"

JSON_HEADER_CT="Content-Type: application/json"

# ---------------------------------------------------------------------------
# Helper
# ---------------------------------------------------------------------------
provider_get()  { curl -sf -H "${JSON_HEADER_CT}"  "${DATA_SPACE_PROVIDER}${1}"; }
provider_post() { curl -sf -X POST -H "${JSON_HEADER_CT}"  -d "${2}" "${DATA_SPACE_PROVIDER}${1}"; }

# ---------------------------------------------------------------------------
# 1. Get main catalog ID
# ---------------------------------------------------------------------------
echo "==> Fetching main catalog..."
CATALOG_RESP=$(provider_get "/api/v1/catalog-agent/catalogs/main")
CATALOG_ID=$(echo "${CATALOG_RESP}" | jq -r '.id')
echo "    catalog_id = ${CATALOG_ID}"

# ---------------------------------------------------------------------------
# 2. Get main data service ID
# ---------------------------------------------------------------------------
echo "==> Fetching main data service..."
DATA_SERVICE_RESP=$(provider_get "/api/v1/catalog-agent/data-services/main")
DATA_SERVICE_ID=$(echo "${DATA_SERVICE_RESP}" | jq -r '.id')
echo "    data_service_id = ${DATA_SERVICE_ID}"

# ---------------------------------------------------------------------------
# 3. Create dataset
# ---------------------------------------------------------------------------
echo "==> Creating dataset..."
DATASET_PAYLOAD=$(jq -n \
  --arg title "My dataspace dataset" \
  --arg catalogId "${CATALOG_ID}" \
  '{ dctTitle: $title, catalogId: $catalogId }')

DATASET_RESP=$(provider_post "/api/v1/catalog-agent/datasets" "${DATASET_PAYLOAD}")
DATASET_ID=$(echo "${DATASET_RESP}" | jq -r '.id')
echo "    dataset_id = ${DATASET_ID}"

# ---------------------------------------------------------------------------
# 4. Create PULL distribution
# ---------------------------------------------------------------------------
echo "==> Creating PULL distribution..."
DIST_PULL_PAYLOAD=$(jq -n \
  --arg title "Distribution of my dataset" \
  --arg desc  "This is the distribution of my dataset" \
  --arg svc   "${DATA_SERVICE_ID}" \
  --arg ds    "${DATASET_ID}" \
  --arg fmt   "http+asd" \
  '{ dctTitle: $title, dctDescription: $desc, dcatAccessService: $svc, datasetId: $ds, dctFormats: $fmt }')

DIST_PULL_RESP=$(provider_post "/api/v1/catalog-agent/distributions" "${DIST_PULL_PAYLOAD}")
DISTRIBUTION_PULL_ID=$(echo "${DIST_PULL_RESP}" | jq -r '.id')
echo "    distribution_pull_id = ${DISTRIBUTION_PULL_ID}"

# ---------------------------------------------------------------------------
# 5. Create PUSH distribution
# ---------------------------------------------------------------------------
echo "==> Creating PUSH distribution..."
DIST_PUSH_PAYLOAD=$(jq -n \
  --arg title "Distribution of a streaming dataset" \
  --arg desc  "This is the distribution of my streaming dataset" \
  --arg svc   "${DATA_SERVICE_ID}" \
  --arg ds    "${DATASET_ID}" \
  --arg fmt   "http+pushing" \
  '{ dctTitle: $title, dctDescription: $desc, dcatAccessService: $svc, datasetId: $ds, dctFormats: $fmt }')

DIST_PUSH_RESP=$(provider_post "/api/v1/catalog-agent/distributions" "${DIST_PUSH_PAYLOAD}")
DISTRIBUTION_PUSH_ID=$(echo "${DIST_PUSH_RESP}" | jq -r '.id')
echo "    distribution_push_id = ${DISTRIBUTION_PUSH_ID}"

# ---------------------------------------------------------------------------
# 6. Create policy from template (policy-1)
# ---------------------------------------------------------------------------
echo "==> Instantiating policy from template (time-limited research access)..."
POLICY_TMPL_PAYLOAD=$(jq -n \
  --arg ds "${DATASET_ID}" \
  '{
    id: "policy-1",
    version: "1.0",
    parameters: { "$bla": "did:web:university.upm.es" },
    entityId: $ds,
    entityType: "Dataset",
    description: "Research access for UPM: grants read access to did:web:university.upm.es valid until 2025-12-31."
  }')

provider_post "/api/v1/catalog-agent/policy-templates/instantiate-odrl-offer" "${POLICY_TMPL_PAYLOAD}" | jq .

# ---------------------------------------------------------------------------
# 7. Create policies directly via API
# ---------------------------------------------------------------------------

echo "==> Creating ODRL policy: commercial use with attribution and time window..."
POLICY_COMMERCIAL_PAYLOAD=$(jq -n \
  --arg ds "${DATASET_ID}" \
  '{
    odrlOffer: {
      permission: [{
        action: "use",
        constraint: [
          {
            leftOperand: "dateTime",
            operator: "gteq",
            rightOperand: "2025-01-01T00:00:00Z"
          },
          {
            leftOperand: "dateTime",
            operator: "lteq",
            rightOperand: "2025-12-31T23:59:59Z"
          },
          {
            leftOperand: "purpose",
            operator: "isA",
            rightOperand: "commercial"
          }
        ]
      }],
      obligation: [{
        action: "attribut",
        constraint: [{
          leftOperand: "recipient",
          operator: "isA",
          rightOperand: "public"
        }]
      }],
      prohibition: [{
        action: "derive",
        constraint: []
      }]
    },
    entityId: $ds,
    entityType: "Dataset",
    description: "Commercial use licence valid for 2025: permits usage for commercial purposes with mandatory attribution. Derivation of the dataset is prohibited."
  }')

provider_post "/api/v1/catalog-agent/odrl-policies" "${POLICY_COMMERCIAL_PAYLOAD}" | jq .

echo "==> Creating ODRL policy: research-only access with 6-month trial window..."
POLICY_DIRECT_PAYLOAD=$(jq -n \
  --arg ds "${DATASET_ID}" \
  '{
    odrlOffer: {
      permission: [{
        action: "read",
        constraint: [
          {
            leftOperand: "dateTime",
            operator: "gteq",
            rightOperand: "2025-03-01T00:00:00Z"
          },
          {
            leftOperand: "dateTime",
            operator: "lteq",
            rightOperand: "2025-09-01T00:00:00Z"
          },
          {
            leftOperand: "purpose",
            operator: "isA",
            rightOperand: "research"
          },
          {
            leftOperand: "count",
            operator: "lteq",
            rightOperand: "10000"
          }
        ]
      }],
      obligation: [],
      prohibition: [
        {
          action: "distribute",
          constraint: []
        },
        {
          action: "reproduce",
          constraint: []
        }
      ]
    },
    entityId: $ds,
    entityType: "Dataset",
    description: "Research trial access (Mar–Sep 2025): read-only, limited to 10 000 requests, restricted to research purposes. Redistribution and reproduction are prohibited."
  }')

provider_post "/api/v1/catalog-agent/odrl-policies" "${POLICY_DIRECT_PAYLOAD}" | jq .

# ---------------------------------------------------------------------------
# 8. Create PULL connector template
# ---------------------------------------------------------------------------
echo "==> Creating PULL connector template..."
CONN_PULL_TMPL_PAYLOAD=$(cat <<'EOF'
{
  "authentication": { "type": "NO_AUTH" },
  "interaction": {
    "mode": "PULL",
    "dataAccess": {
      "protocol": "HTTP",
      "urlTemplate": "{{__ACCESS_URL__}}",
      "method": "{{__ACCESS_METHODS__}}",
      "headers": "{{__ACCESS_HEADERS__}}"
    }
  },
  "parameters": [
    { "paramType": "STRING",           "name": "ACCESS_URL",     "title": "Url",     "required": true },
    { "paramType": "VEC<STRING>",      "name": "ACCESS_METHODS", "title": "Methods", "required": true },
    { "paramType": "MAP<STRING,STRING>","name": "ACCESS_HEADERS", "title": "Headers", "required": true }
  ]
}
EOF
)

CONN_PULL_TMPL_RESP=$(provider_post "/api/v1/connector/templates" "${CONN_PULL_TMPL_PAYLOAD}")
CONN_PULL_NAME=$(echo "${CONN_PULL_TMPL_RESP}" | jq -r '.name')
CONN_PULL_VERSION=$(echo "${CONN_PULL_TMPL_RESP}" | jq -r '.version')
echo "    pull_connector_template = ${CONN_PULL_NAME}  v${CONN_PULL_VERSION}"

# ---------------------------------------------------------------------------
# 9. Instantiate PULL connector
# ---------------------------------------------------------------------------
echo "==> Creating PULL connector instance..."
CONN_PULL_INST_PAYLOAD=$(jq -n \
  --arg name      "${CONN_PULL_NAME}" \
  --arg version   "${CONN_PULL_VERSION}" \
  --arg distId    "${DISTRIBUTION_PULL_ID}" \
  --arg staticApi "${STATIC_API}" \
  '{
    templateName:    $name,
    templateVersion: $version,
    distributionId:  $distId,
    metadata: { description: "asasdasddas", ownerId: "asdas" },
    dryRun: false,
    parameters: {
      ACCESS_URL:     $staticApi,
      ACCESS_METHODS: ["GET", "HEAD"],
      ACCESS_HEADERS: { "Content-type": "application/json" }
    }
  }')

provider_post "/api/v1/connector/instances" "${CONN_PULL_INST_PAYLOAD}" | jq .

# ---------------------------------------------------------------------------
# 10. Create PUSH connector template
# ---------------------------------------------------------------------------
echo "==> Creating PUSH connector template..."
CONN_PUSH_TMPL_PAYLOAD=$(cat <<'EOF'
{
  "authentication": { "type": "NO_AUTH" },
  "interaction": {
    "mode": "PUSH",
    "subscribe": {
      "protocol": "HTTP",
      "urlTemplate": "{{__SUB_URL__}}",
      "method": "POST",
      "headers": {},
      "bodyTemplate": "{\"url\": \"{{__RUNTIME_CB_URL_DOCKERIZED__}}\", \"event_type\": \"hotel_waste_generated\"}"
    },
    "unsubscribe": {
      "protocol": "HTTP",
      "urlTemplate": "{{__UNSUB_URL__}}/{{__RUNTIME_SUB_RESPONSE_{.data.ID}__}}",
      "method": "POST",
      "headers": {},
      "bodyTemplate": "{}"
    }
  },
  "parameters": [
    { "paramType": "STRING", "name": "SUB_URL",   "title": "Subscription URL",   "required": true },
    { "paramType": "STRING", "name": "UNSUB_URL", "title": "Unsubscription URL", "required": true }
  ]
}
EOF
)

CONN_PUSH_TMPL_RESP=$(provider_post "/api/v1/connector/templates" "${CONN_PUSH_TMPL_PAYLOAD}")
CONN_PUSH_NAME=$(echo "${CONN_PUSH_TMPL_RESP}" | jq -r '.name')
CONN_PUSH_VERSION=$(echo "${CONN_PUSH_TMPL_RESP}" | jq -r '.version')
echo "    push_connector_template = ${CONN_PUSH_NAME}  v${CONN_PUSH_VERSION}"

# ---------------------------------------------------------------------------
# 11. Instantiate PUSH connector
# ---------------------------------------------------------------------------
echo "==> Creating PUSH connector instance..."
CONN_PUSH_INST_PAYLOAD=$(jq -n \
  --arg name       "${CONN_PUSH_NAME}" \
  --arg version    "${CONN_PUSH_VERSION}" \
  --arg distId     "${DISTRIBUTION_PUSH_ID}" \
  --arg dynamicApi "${DYNAMIC_API}" \
  '{
    templateName:    $name,
    templateVersion: $version,
    distributionId:  $distId,
    parameters: {
      SUB_URL:   ($dynamicApi + "/subscriptions/subscribe"),
      UNSUB_URL: ($dynamicApi + "/subscriptions/unsubscribe")
    }
  }')

provider_post "/api/v1/connector/instances" "${CONN_PUSH_INST_PAYLOAD}" | jq .

echo ""
echo "Done! Catalog populated successfully."