#!/bin/bash
# Seeds data schemas: infers one from a sample, hosts it (v1, then v2), sets it as the schema of
# the "Weather Data 2024" dataset (its distributions inherit it) and gives "Traffic Sensor Logs"
# an external schema URL. Run populate_mock_data.sh first.
#
# Usage: scripts/data/populate_mock_schemas.sh
# Env:   AGENT_URL (default EUNOMIA_PROVIDER_URL), EUNOMIA_TOKEN
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/auth.sh
source "$SCRIPT_DIR/../lib/auth.sh"

AGENT_URL="${AGENT_URL:-$EUNOMIA_PROVIDER_URL}"
API="$AGENT_URL/api/v1/catalog-agent"
EXTERNAL_SCHEMA="${EXTERNAL_SCHEMA:-https://geojson.org/schema/FeatureCollection.json}"

# Two weather readings: `note` is missing in one, so it comes out optional.
SAMPLES='[
  {"station": "MAD-01", "at": "2026-10-29T10:00:00Z", "temperature": 14.5, "humidity": 61,
   "location": {"lat": 40.41, "lon": -3.70}, "note": "clear"},
  {"station": "BCN-02", "at": "2026-10-29T10:00:00Z", "temperature": 18, "humidity": 70,
   "location": {"lat": 41.39, "lon": 2.17}}
]'

fail() {
    echo "Error: $*" >&2
    exit 1
}

# The id of the first dataset whose title contains $1.
dataset_id() {
    eunomia_curl GET "$API/datasets?title=$(jq -rn --arg t "$1" '$t|@uri')&limit=1" \
        | jq -r '(.["@graph"] // [])[0]["@id"] // empty'
}

# Replaces the dataset's `schema` with $2, keeping the rest of its description.
set_schema() {
    local id="$1" schema="$2" etag doc body
    etag=$(eunomia_curl GET "$API/datasets/$id" "" -D - -o /dev/null \
        | awk 'tolower($1)=="etag:" {print $2}' | tr -d '\r')
    doc=$(eunomia_curl GET "$API/datasets/$id")
    body=$(echo "$doc" | jq --arg s "$schema" 'del(.issued, .modified) | .schema = $s')
    eunomia_curl PUT "$API/datasets/$id" "$body" -H "If-Match: $etag" \
        | jq -e '.["@id"]' >/dev/null || fail "setting the schema of $id"
    echo ">> $id conforms to $schema" >&2
}

main() {
    echo "=== STARTING MOCK SCHEMA POPULATION ===" >&2

    local inferred created id v2 url weather traffic
    inferred=$(eunomia_curl POST "$API/schemas/infer" "$(jq -n --argjson s "$SAMPLES" '{samples: $s}')")
    echo "$inferred" | jq -e '.properties' >/dev/null || fail "inferring: $inferred"
    echo ">> Inferred fields: $(echo "$inferred" | jq -r '.properties | keys | join(", ")')" >&2

    created=$(eunomia_curl POST "$API/schemas" "$(jq -n --argjson s "$inferred" '{
        title: "Weather readings",
        description: "One reading of a weather station",
        visibility: "Public",
        schema: $s
    }')")
    id=$(echo "$created" | jq -r '.id // empty')
    [ -n "$id" ] || fail "creating the schema: $created"
    echo ">> Schema $id v1 at $(echo "$created" | jq -r .url)" >&2

    # v2 documents the temperature unit; v1 keeps its URL and content.
    v2=$(eunomia_curl POST "$API/schemas/$id/versions" "$(jq -n --argjson s "$inferred" '{
        schema: ($s | .properties.temperature.description = "Degrees Celsius")
    }')")
    url=$(echo "$v2" | jq -r '.url // empty')
    [ -n "$url" ] || fail "creating version 2: $v2"
    echo ">> Schema $id v2 at $url" >&2

    weather=$(dataset_id "Weather Data 2024")
    traffic=$(dataset_id "Traffic Sensor Logs")
    [ -n "$weather" ] && set_schema "$weather" "$url" || echo "Warning: no 'Weather Data 2024' dataset" >&2
    [ -n "$traffic" ] && set_schema "$traffic" "$EXTERNAL_SCHEMA" || echo "Warning: no 'Traffic Sensor Logs' dataset" >&2

    # Anyone dereferences the hosted version, with no token.
    echo ">> GET $url (no token):" >&2
    curl -s "$url" | jq -c '{"$id", title, required}' >&2

    echo "=== MOCK SCHEMA POPULATION FINISHED ===" >&2
}

main "$@"
