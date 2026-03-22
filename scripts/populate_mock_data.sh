#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
AGENT_URL="http://127.0.0.1:1200"


# ==========================================
# HELPER FUNCTIONS
# ==========================================

# Wrapper for curl to handle logging and clean JSON output
# STDOUT: Only the raw JSON response
# STDERR: Logs, request details, and response preview
invoke_curl() {
    local method="$1"
    local url="$2"
    local body="$3"
    local description="$4"

    # LOGS to stderr (to keep stdout clean)
    echo "--- $description ---" >&2
    echo "Request: $method $url" >&2

    if [[ -n "$body" ]]; then
        response=$(curl -s -X "$method" -H "Content-Type: application/json" -d "$body" "$url")
    else
        response=$(curl -s -X "$method" "$url")
    fi

    # LOG preview to stderr
    echo "Response (preview): ${response:0:100}..." >&2
    echo "---------------------------------------------------" >&2

    # Raw JSON goes to stdout to be captured by jq
    echo "$response"
}

# ==========================================
# CORE LOGIC FUNCTIONS
# ==========================================

# 1. Retrieve DIDs
# Returns: Nothing (updates global variables)
# Output: Logs to stderr
get_dids() {
    echo "Retrieving DIDs..." >&2

    # Provider
    PROVIDER_DID_RAW=$(curl -s "$AGENT_URL/api/v1/mates/myself")
    PROVIDER_DID=$(echo "$PROVIDER_DID_RAW" | jq -r '
        if type=="array" then .[0].participant_id
        else if .error_code == 3120 then "null" else .participant_id end end
    ')

    # Consumer
    CONSUMER_DID_RAW=$(curl -s "$AGENT_URL/api/v1/mates/all")
    CONSUMER_DID=$(echo "$CONSUMER_DID_RAW" | jq -r '
        if type=="array" then (. | last(.[] | select(.is_me == false)).participant_id)
        else if .error_code == 3120 then "null" else "null" end end
    ')

    if [[ "$PROVIDER_DID" == "null" || "$CONSUMER_DID" == "null" || -z "$PROVIDER_DID" ]]; then
        echo "Warning: Could not retrieve DIDs. Using placeholders." >&2
        PROVIDER_DID="urn:did:provider:placeholder"
        CONSUMER_DID="urn:did:consumer:placeholder"
    fi

    echo "Provider DID: $PROVIDER_DID" >&2
    echo "Consumer DID: $CONSUMER_DID" >&2
}

# 2. Create Catalog
# Returns: Catalog ID (stdout)
create_catalog() {
    # Check existence
    local check=$(curl -s "$AGENT_URL/api/v1/catalog-agent/catalogs/main")
    local existing_id=$(echo "$check" | jq -r 'if type=="object" and .error_code == 3120 then "null" else (.id // "null") end' 2>/dev/null)

    if [[ "$existing_id" != "null" && -n "$existing_id" ]]; then
        echo "Log: Catalog already exists." >&2
        echo "$existing_id"
        return
    fi

    local body=$(jq -n '{ "foafHomePage": "superfoaf" }')
    # invoke_curl sends logs to stderr and json to stdout.
    # We capture the json in resp.
    local resp=$(invoke_curl "POST" "$AGENT_URL/api/v1/catalog-agent/catalogs" "$body" "Creating Catalog")

    # Output only the ID to stdout
    echo "$resp" | jq -r '.id // empty'
}

get_main_catalog() {
    # Check existence
    local check=$(curl -s "$AGENT_URL/api/v1/catalog-agent/catalogs/main")
    local existing_id=$(echo "$check" | jq -r 'if type=="object" and .error_code == 3120 then "null" else (.id // "null") end' 2>/dev/null)

    if [[ "$existing_id" != "null" && -n "$existing_id" ]]; then
        echo "Log: Catalog already exists." >&2
        echo "$existing_id"
        return
    fi
    # Output only the ID to stdout
    echo "$check" | jq -r '.id // empty'
}

# 3. Create Data Service
# Returns: Data Service ID (stdout)
create_dataservice() {
    local catalog_id="$1"
    local body=$(jq -n --arg cat_id "$catalog_id" '{
        "dcatEndpointUrl": "superfoaf",
        "catalogId": $cat_id
    }')

    local resp=$(invoke_curl "POST" "$AGENT_URL/api/v1/catalog-agent/data-services" "$body" "Creating Data Service")
    echo "$resp" | jq -r '.id // empty'
}

# 4. Create Dataset
# Returns: Dataset ID (stdout)
create_dataset() {
    local title="$1"
    local catalog_id="$2"

    local body=$(jq -n --arg title "$title" --arg cat_id "$catalog_id" '{
         "dctTitle": $title,
         "catalogId": $cat_id
    }')

    local resp=$(invoke_curl "POST" "$AGENT_URL/api/v1/catalog-agent/datasets" "$body" "Creating Dataset ($title)")
    echo "$resp" | jq -r '.id // empty'
}

# 5. Create Distribution
# Returns: Distribution ID (stdout)
create_distribution() {
    local title="$1"
    local dataset_id="$2"
    local dataservice_id="$3"
    local format="$4"

    local body=$(jq -n \
        --arg title "$title" \
        --arg ds_svc "$dataservice_id" \
        --arg ds_id "$dataset_id" \
        --arg fmt "$format" \
      '{
         "dctTitle": $title,
         "dcatAccessService": $ds_svc,
         "datasetId": $ds_id,
         "dctFormats": $fmt
      }')

    local resp=$(invoke_curl "POST" "$AGENT_URL/api/v1/catalog-agent/distributions" "$body" "Creating Distribution ($title)")
    echo "$resp" | jq -r '.id // empty'
}


# 6. Create Connector Template (Blueprint)
# Returns: JSON object { "id": "...", "name": "...", "version": "..." }
create_blueprint() {
    local body=$(jq -n \
        '{
        "authentication": { "type": "NO_AUTH" },
        "interaction": {
            "mode": "PULL",
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "{{__ACCESS_URL__}}",
                "method": "{{__ALLOWED_METHODS__}}",
                "headers": "{{__ALLOWED_HEADERS__}}"
            }
        },
        "parameters": [
            { "paramType": "STRING", "name": "ACCESS_URL", "title": "Target URL", "required": true },
            { "paramType": "VEC<STRING>", "name": "ALLOWED_METHODS", "title": "Methods", "required": true },
            { "paramType": "MAP<STRING,STRING>", "name": "ALLOWED_HEADERS", "title": "Headers", "required": true }
        ]
    }')

    local resp=$(invoke_curl "POST" "$AGENT_URL/api/v1/connector/templates" "$body" "Creating Connector Template")

    echo "$resp" | jq '{name: .name, version: .version}'
}

# 7. Create Connector Instance
# Args: dist_id, target_url, template_name, template_version
# Returns: Instance ID (stdout)
create_connector_instance() {
    local dist_id="$1"
    local target_url="$2"
    local t_name="$3"
    local t_ver="$4"

    local body=$(jq -n \
        --arg tName "$t_name" \
        --arg tVer "$t_ver" \
        --arg distId "$dist_id" \
        --arg url "$target_url" \
        '{
        "templateName": $tName,
        "templateVersion": $tVer,
        "distributionId": $distId,
        "metadata": { "description": "Auto-generated instance", "ownerId": "provider-admin" },
        "dryRun": false,
        "parameters": {
            "ACCESS_URL": $url,
            "ALLOWED_METHODS": ["GET", "HEAD"],
            "ALLOWED_HEADERS": { "Content-Type": "application/json" }
        }
    }')

    local resp=$(invoke_curl "POST" "$AGENT_URL/api/v1/connector/instances" "$body" "Linking Connector to Dist $dist_id")
    echo "$resp" | jq -r '.id // empty'
}

# 8. Create Policy
# Returns: Policy ID (stdout)
create_policy() {
    local entity_id="$1"

    local body=$(jq -n --arg entityId "$entity_id" '{
        "odrlOffer": {
            "permission": [ { "action": "use" } ]
        },
        "entityId": $entityId,
        "entityType": "Dataset"
    }')

    local resp=$(invoke_curl "POST" "$AGENT_URL/api/v1/catalog-agent/odrl-policies" "$body" "Creating Policy for Dataset/Entity")
    echo "$resp" | jq -r '.id // empty'
}

# ==========================================
# MAIN EXECUTION
# ==========================================

main() {
    echo "=== STARTING MOCK DATA POPULATION ===" >&2

    # 1. Retrieve DIDs
    get_dids

    # 2. Create Catalog
    CATALOG_ID=$(get_main_catalog)
    if [[ -z "$CATALOG_ID" || "$CATALOG_ID" == "null" ]]; then
        echo "Error: Failed to get or create Catalog ID." >&2
        exit 1
    fi
    echo ">> Catalog ID: $CATALOG_ID" >&2

    # 3. Create Data Service
    DATA_SERVICE_ID=$(create_dataservice "$CATALOG_ID")
    echo ">> Data Service ID: $DATA_SERVICE_ID" >&2

    # 4. Create Datasets
    DS1_ID=$(create_dataset "Weather Data 2024" "$CATALOG_ID")
    DS2_ID=$(create_dataset "Traffic Sensor Logs" "$CATALOG_ID")
    DS3_ID=$(create_dataset "Public Transport Schedules" "$CATALOG_ID")
    echo ">> Datasets Created: $DS1_ID, $DS2_ID, $DS3_ID" >&2

    # 5. Create Distributions
    DI1_ID=$(create_distribution "Weather Data - JSON" "$DS1_ID" "$DATA_SERVICE_ID" "http+pull")
    DI2_ID=$(create_distribution "Traffic Logs - JSON" "$DS1_ID" "$DATA_SERVICE_ID" "http+push")
    DI3_ID=$(create_distribution "Public Transport - JSON" "$DS2_ID" "$DATA_SERVICE_ID" "http+pull")
    DI4_ID=$(create_distribution "Weather Data Alt - JSON" "$DS3_ID" "$DATA_SERVICE_ID" "http+pull")
    DI5_ID=$(create_distribution "Traffic Logs Alt - JSON" "$DS3_ID" "$DATA_SERVICE_ID" "http+push")
    DI6_ID=$(create_distribution "Public Transport Alt - JSON" "$DS3_ID" "$DATA_SERVICE_ID" "http+pull")
    echo ">> Distributions Created." >&2

    # 6. Create Blueprint (Template)
    echo ">> Creating Blueprint..." >&2
    BP_DATA=$(create_blueprint)
    BP_NAME=$(echo "$BP_DATA" | jq -r '.name')
    BP_VER=$(echo "$BP_DATA" | jq -r '.version')
    BP_ID=$(echo "$BP_DATA" | jq -r '.id')
    echo ">> Blueprint Created: $BP_NAME v$BP_VER (ID: $BP_ID)" >&2

    # 7. Create Connector Instances (Passing name and version dynamically)
    if [[ -n "$DI1_ID" ]]; then
        CI1=$(create_connector_instance "$DI1_ID" "https://jsonplaceholder.typicode.com/posts" "$BP_NAME" "$BP_VER")
        echo ">> Connector Instance for DI1: $CI1" >&2
    fi
    if [[ -n "$DI3_ID" ]]; then
        CI3=$(create_connector_instance "$DI3_ID" "https://jsonplaceholder.typicode.com/albums" "$BP_NAME" "$BP_VER")
        echo ">> Connector Instance for DI3: $CI3" >&2
    fi
    if [[ -n "$DI6_ID" ]]; then
        CI6=$(create_connector_instance "$DI6_ID" "https://jsonplaceholder.typicode.com/users" "$BP_NAME" "$BP_VER")
        echo ">> Connector Instance for DI6: $CI6" >&2
    fi

    # 8. Create Policies (As requested)
    # The IDs will be captured in variables, and we log them to stderr
    POLICY_ID1=$(create_policy "$DS1_ID")
    echo ">> Policy for DS1 (1): $POLICY_ID1" >&2

    POLICY_ID11=$(create_policy "$DS1_ID")
    echo ">> Policy for DS1 (2): $POLICY_ID11" >&2

    POLICY_ID12=$(create_policy "$DS1_ID")
    echo ">> Policy for DS1 (3): $POLICY_ID12" >&2

    POLICY_ID13=$(create_policy "$DS1_ID")
    echo ">> Policy for DS1 (4): $POLICY_ID13" >&2

    POLICY_ID2=$(create_policy "$DS2_ID")
    echo ">> Policy for DS2: $POLICY_ID2" >&2

    POLICY_ID3=$(create_policy "$DS3_ID")
    echo ">> Policy for DS3: $POLICY_ID3" >&2

    echo "=== MOCK DATA POPULATION FINISHED ===" >&2
}

# Run Main
main "$@"