#!/bin/bash
# Creates a tenant on a running monolith and shows it is isolated from the others.
#
# A tenant is born with its OAuth user. Creating the user publishes `oauth:user:create`,
# and catalog-agent reacts by provisioning the tenant's main catalog and main data service.
# The explicit provision call below is idempotent and only confirms the result.
#
# Usage: scripts/new-tenant.sh <tenant> [role]
#   role: owner (default) | reader
# Env:   BASE_URL, ADMIN_EMAIL, ADMIN_PASSWORD, TENANT_EMAIL, TENANT_PASSWORD
set -euo pipefail

TENANT="${1:?usage: $0 <tenant> [owner|reader]}"
ROLE="${2:-owner}"
BASE_URL="${BASE_URL:-http://127.0.0.1:1200}"
ADMIN_EMAIL="${ADMIN_EMAIL:-admin@admin.local}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-admin}"
TENANT_EMAIL="${TENANT_EMAIL:-$TENANT@$TENANT.local}"
TENANT_PASSWORD="${TENANT_PASSWORD:-$TENANT-password}"
CATALOG_API="$BASE_URL/api/v1/catalog-agent"

command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }

step() { printf '\n\033[1m== %s\033[0m\n' "$*" >&2; }

# Calls the API and fails loudly on HTTP errors; prints the JSON body on success.
call() {
    local method="$1" url="$2" token="$3" body="${4:-}" tenant_header="${5:-}"
    local args=(-sS -X "$method" -w '\n%{http_code}' -H "Authorization: Bearer $token")
    [[ -n "$body" ]] && args+=(-H "Content-Type: application/json" -d "$body")
    [[ -n "$tenant_header" ]] && args+=(-H "x-tenant-id: $tenant_header")
    local response status
    response=$(curl "${args[@]}" "$url")
    status="${response##*$'\n'}"
    response="${response%$'\n'*}"
    if [[ "$status" -ge 400 ]]; then
        echo "$method $url -> HTTP $status: $response" >&2
        return 1
    fi
    echo "$response"
}

login() {
    local email="$1" password="$2"
    curl -sS -X POST "$BASE_URL/oauth/token" -H "Content-Type: application/json" \
        -d "$(jq -n --arg u "$email" --arg p "$password" \
            '{grant_type: "password", client_id: "eunomia-admin-gui", username: $u, password: $p}')" \
        | jq -er '.access_token'
}

step "Admin login ($ADMIN_EMAIL)"
ADMIN_TOKEN=$(login "$ADMIN_EMAIL" "$ADMIN_PASSWORD")

step "Create $ROLE user $TENANT_EMAIL, which creates tenant '$TENANT'"
call POST "$BASE_URL/oauth/users" "$ADMIN_TOKEN" "$(jq -n \
    --arg t "$TENANT" --arg e "$TENANT_EMAIL" --arg p "$TENANT_PASSWORD" --arg r "$ROLE" \
    '{tenantId: $t, email: $e, password: $p, role: $r}')" | jq '{tenantId, email, role}'

step "Provision tenant (idempotent; the event listener normally did it already)"
call POST "$CATALOG_API/tenants/$TENANT/provision" "$ADMIN_TOKEN" \
    | jq '{tenantId, catalog: .catalog.id, dataService: .dataService.id, dsp: .dataService.dcatEndpointUrl}'

step "Tenant login ($TENANT_EMAIL)"
TENANT_TOKEN=$(login "$TENANT_EMAIL" "$TENANT_PASSWORD")

step "Tenant's own main catalog and data service"
call GET "$CATALOG_API/catalogs/main" "$TENANT_TOKEN" | jq '{id, tenantId, dspaceParticipantId}'
call GET "$CATALOG_API/data-services/main" "$TENANT_TOKEN" | jq '{id, tenantId, dcatEndpointUrl}'

step "Catalogs visible to the tenant (only its own)"
call GET "$CATALOG_API/catalogs" "$TENANT_TOKEN" | jq '[.items[] | {id, tenantId}]'

step "Catalogs visible to the admin: every tenant, then pinned to '$TENANT'"
call GET "$CATALOG_API/catalogs" "$ADMIN_TOKEN" | jq '[.items[] | {id, tenantId}]'
call GET "$CATALOG_API/catalogs" "$ADMIN_TOKEN" "" "$TENANT" | jq '[.items[] | {id, tenantId}]'

step "The tenant cannot act on another tenant"
if call GET "$CATALOG_API/catalogs" "$TENANT_TOKEN" "" "admin" >/dev/null 2>&1; then
    echo "UNEXPECTED: tenant '$TENANT' could read tenant 'admin'" >&2
    exit 1
fi
echo "rejected, as expected" >&2

step "Events of the tenant (catalog creation included)"
call GET "$BASE_URL/api/v1/events?limit=10" "$TENANT_TOKEN" | jq '[.items[] | {topic, tenant_id}]'

cat >&2 <<EOF

Tenant '$TENANT' ready. Log into the GUI with $TENANT_EMAIL / $TENANT_PASSWORD,
or as admin and pick '$TENANT' in the tenant selector.
EOF
