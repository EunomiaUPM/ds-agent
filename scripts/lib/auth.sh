#!/bin/bash
# OAuth helpers shared by the scripts. Source it, then add "${AUTH_ARGS[@]}" to curl calls
# after `eunomia_auth <base_url>`, or use `eunomia_curl`.
#
# Env: EUNOMIA_TOKEN (optional access token issued by the identity provider, e.g. Keycloak).
#
# The agents no longer issue tokens nor take a tenant: the user comes from the token. Without
# EUNOMIA_TOKEN the calls go without one, which an agent in `static` mode serves as its
# configured user (by default the root, /admin).

# Former login of every agent, with tokens cached per agent:
# ADMIN_EMAIL="${ADMIN_EMAIL:-admin@admin.local}"
# ADMIN_PASSWORD="${ADMIN_PASSWORD:-admin}"
# export EUNOMIA_TOKEN_DIR="${EUNOMIA_TOKEN_DIR:-$(mktemp -d)}"

# Prints the scheme://host:port part of a URL.
eunomia_origin() {
    printf '%s' "$1" | sed -E 's#^(https?://[^/]+).*#\1#'
}

# Prints the access token to send to the agent serving <url> (empty: none).
eunomia_token() {
    printf '%s' "${EUNOMIA_TOKEN:-}"
}

# Former version, logging into the agent's own /oauth/token (gone with the built-in OAuth module):
# # Prints an access token for the agent serving <url>, logging in on first use.
# eunomia_token() {
#     local origin cache token
#     origin=$(eunomia_origin "$1")
#     cache="$EUNOMIA_TOKEN_DIR/$(printf '%s' "$origin" | tr -c 'A-Za-z0-9' '_')"
#     if [ ! -s "$cache" ]; then
#         token=$(curl -s -X POST "$origin/oauth/token" -H "Content-Type: application/json" \
#             -d "{\"grant_type\":\"password\",\"client_id\":\"eunomia-admin-gui\",\"username\":\"$ADMIN_EMAIL\",\"password\":\"$ADMIN_PASSWORD\"}" \
#             | jq -r '.access_token // empty')
#         if [ -z "$token" ]; then
#             echo "Login as $ADMIN_EMAIL on $origin failed" >&2
#             return 1
#         fi
#         printf '%s' "$token" > "$cache"
#     fi
#     cat "$cache"
# }

# Sets AUTH_ARGS to the curl header arguments for the agent serving <url>.
# Without a token the request goes without the header.
eunomia_auth() {
    local token
    token=$(eunomia_token "$1") || token=""
    AUTH_ARGS=()
    if [ -n "$token" ]; then
        AUTH_ARGS+=(-H "Authorization: Bearer $token")
    fi
    # Tenants are gone: the token's user decides what the call sees.
    # if [ -n "${TENANT:-}" ]; then
    #     AUTH_ARGS+=(-H "x-tenant-id: $TENANT")
    # fi
}

# curl with the agent's credentials: eunomia_curl <method> <url> [json body] [extra curl args...]
eunomia_curl() {
    local method="$1" url="$2" body="${3:-}"
    shift 3 2>/dev/null || shift $#
    eunomia_auth "$url"
    if [ -n "$body" ]; then
        curl -s -X "$method" "${AUTH_ARGS[@]}" -H "Content-Type: application/json" -d "$body" "$@" "$url"
    else
        curl -s -X "$method" "${AUTH_ARGS[@]}" "$@" "$url"
    fi
}
