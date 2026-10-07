#!/bin/bash
# Populates every mock dataset as the user of lib/auth.sh (EUNOMIA_TOKEN, or none in static mode).
#
# Usage: scripts/populate_all.sh
# Env:   EUNOMIA_TOKEN, VISIBILITY (of the mates)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/auth.sh
source "$SCRIPT_DIR/lib/auth.sh"

# Tenants are gone (the token's user decides what each call creates and sees), so the former
# creation of a tenant with its OAuth user and main catalog is disabled:
# TENANT="${1:-${TENANT:-}}"
# ROLE="${2:-owner}"
# BASE_URL="${BASE_URL:-http://127.0.0.1:1200}"

# # Creates the tenant's user and main catalog unless the tenant already has a user.
# # The admin calls without x-tenant-id (TENANT= per call) so it is not pinned to the new tenant.
# ensure_tenant() {
#     local email="${TENANT_EMAIL:-$TENANT@$TENANT.local}"
#     local password="${TENANT_PASSWORD:-$TENANT-password}"
#     local users body
#     users=$(eunomia_curl GET "$BASE_URL/oauth/users?limit=1" | jq -r '.total // empty')
#     if [ -z "$users" ]; then
#         echo "Cannot list users of tenant '$TENANT' as $ADMIN_EMAIL" >&2
#         return 1
#     fi
#     if [ "$users" -gt 0 ]; then
#         echo "Tenant '$TENANT' already exists" >&2
#         return 0
#     fi
#     echo "Creating tenant '$TENANT' ($ROLE user $email / $password)" >&2
#     body=$(jq -n --arg t "$TENANT" --arg e "$email" --arg p "$password" --arg r "$ROLE" \
#         '{tenantId: $t, email: $e, password: $p, role: $r}')
#     TENANT= eunomia_curl POST "$BASE_URL/oauth/users" "$body" | jq -e '.tenantId' >/dev/null || {
#         echo "Creating the user of tenant '$TENANT' failed" >&2
#         return 1
#     }
#     # Idempotent: the user-created event normally provisioned it already.
#     TENANT= eunomia_curl POST "$BASE_URL/api/v1/catalog-agent/tenants/$TENANT/provision" \
#         | jq -e '.tenantId' >/dev/null || {
#         echo "Provisioning tenant '$TENANT' failed" >&2
#         return 1
#     }
# }

# if [ -n "$TENANT" ]; then
#     ensure_tenant || exit 1
#     export TENANT
# fi
# echo "Populating tenant '${TENANT:-admin}'" >&2

echo "Populating as the scripts' user (static mode: the root)" >&2

bash "$SCRIPT_DIR/populate_mock_data.sh"
bash "$SCRIPT_DIR/populate_mock_contracts.sh"
bash "$SCRIPT_DIR/populate_mock_transfers.sh"
bash "$SCRIPT_DIR/populate_mock_mates.sh"

#bash "$SCRIPT_DIR/populate_tck.sh"
