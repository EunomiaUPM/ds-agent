#!/bin/bash
# Populates every mock dataset into one tenant. Without a tenant the admin's own tenant is used.
# A missing tenant is created first, with its OAuth user, so it can log into the GUI.
#
# Usage: scripts/populate_all.sh [tenant] [owner|reader]
# Env:   BASE_URL, ADMIN_EMAIL, ADMIN_PASSWORD, TENANT_EMAIL, TENANT_PASSWORD
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/auth.sh
source "$SCRIPT_DIR/lib/auth.sh"

TENANT="${1:-${TENANT:-}}"
ROLE="${2:-owner}"
BASE_URL="${BASE_URL:-http://127.0.0.1:1200}"

# Creates the tenant's user and main catalog unless the tenant already has a user.
# The admin calls without x-tenant-id (TENANT= per call) so it is not pinned to the new tenant.
ensure_tenant() {
    local email="${TENANT_EMAIL:-$TENANT@$TENANT.local}"
    local password="${TENANT_PASSWORD:-$TENANT-password}"
    local users body
    users=$(eunomia_curl GET "$BASE_URL/oauth/users?limit=1" | jq -r '.total // empty')
    if [ -z "$users" ]; then
        echo "Cannot list users of tenant '$TENANT' as $ADMIN_EMAIL" >&2
        return 1
    fi
    if [ "$users" -gt 0 ]; then
        echo "Tenant '$TENANT' already exists" >&2
        return 0
    fi
    echo "Creating tenant '$TENANT' ($ROLE user $email / $password)" >&2
    body=$(jq -n --arg t "$TENANT" --arg e "$email" --arg p "$password" --arg r "$ROLE" \
        '{tenantId: $t, email: $e, password: $p, role: $r}')
    TENANT= eunomia_curl POST "$BASE_URL/oauth/users" "$body" | jq -e '.tenantId' >/dev/null || {
        echo "Creating the user of tenant '$TENANT' failed" >&2
        return 1
    }
    # Idempotent: the user-created event normally provisioned it already.
    TENANT= eunomia_curl POST "$BASE_URL/api/v1/catalog-agent/tenants/$TENANT/provision" \
        | jq -e '.tenantId' >/dev/null || {
        echo "Provisioning tenant '$TENANT' failed" >&2
        return 1
    }
}

if [ -n "$TENANT" ]; then
    ensure_tenant || exit 1
    export TENANT
fi
echo "Populating tenant '${TENANT:-admin}'" >&2

bash "$SCRIPT_DIR/populate_mock_data.sh"
bash "$SCRIPT_DIR/populate_mock_contracts.sh"
bash "$SCRIPT_DIR/populate_mock_transfers.sh"
bash "$SCRIPT_DIR/populate_mock_mates.sh"

#bash "$SCRIPT_DIR/populate_tck.sh"
