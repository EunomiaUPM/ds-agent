#!/bin/bash

# This script automates the Onboarding process for wallets (A, C, P)
# and the sequence of credential exchange and access (OIDC4VCI and OIDC4VP).
#
# The utility function now prioritizes clarity by showing the full 'curl'
# command before execution.

# Requirements: curl and jq must be installed on the system.

# --- 1. Define URLs ---
AuthorityUrl="http://127.0.0.1:1500"
ConsumerUrl="http://127.0.0.1:1100"
ProviderUrl="http://127.0.0.1:1200"

# Stop script execution on error (fail-fast)
set -e

# --- Utility Function ---
# Robust function to execute curl commands with error handling and logging.
# Shows the curl command being executed.
# Arguments: 1=Method, 2=URL, 3=JSON Body (optional), 4=Log Message
function invoke_curl_robust {
    local method="$1"
    local url="$2"
    local body="$3"
    local log_message="$4"
    local output=""
    local curl_base_command="curl -s -X $method"
    local full_curl_command=""

    echo "" >&2
    echo "--- Executing step: $log_message ---" >&2
    echo "   URL: $url" >&2

    # Build and display the full command
    if [[ -n "$body" ]]; then
        # Command with JSON data
        full_curl_command="$curl_base_command -H \"Content-Type: application/json\" -d '$body' $url"
    else
        # Simple GET/POST command without body
        full_curl_command="$curl_base_command $url"
    fi

    echo "   CMD: $full_curl_command" >&2

    # Execute the command
    # We use eval for correct interpretation of quotes within the command,
    # especially for -d '...'
    output=$(eval "$full_curl_command")
    local exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        echo "--- ERROR ($log_message): curl connection/command failed (Code: $exit_code) ---" >&2
        echo "--- Exiting script. ---" >&2
        exit 1
    fi

    # Success Log (STDERR)
    echo "   STATUS: SUCCESS" >&2

    # Show output in the log (STDERR) for visibility
    # echo "   Output: $output" >&2
    echo "---------------------------------------------------" >&2

    # Return the output (JSON/text) to STDOUT for capture or piping
    echo "$output"
}

# --- 2. Onboarding and Processing Sequence ---

# 2.1 A Onboarding
invoke_curl_robust "POST" "$AuthorityUrl/api/v1/wallet/onboard" "" "1. Onboarding Authority (A)"

# 2.2 C Onboarding
invoke_curl_robust "POST" "$ConsumerUrl/api/v1/wallet/onboard" "" "2. Onboarding Consumer (C)"

# 2.3 P Onboarding
invoke_curl_robust "POST" "$ProviderUrl/api/v1/wallet/onboard" "" "3. Onboarding Provider (P)"


# 2.4 Get DIDs
echo "" >&2
echo "--- 4. Getting DIDs of participants ---" >&2
AuthorityDid=$(invoke_curl_robust "GET" "$AuthorityUrl/api/v1/wallet/did.json" "" "4.1 Get A Did" | jq -r '.id')
ConsumerDid=$(invoke_curl_robust "GET" "$ConsumerUrl/api/v1/wallet/did.json" "" "4.2 Get C Did" | jq -r '.id')
ProviderDid=$(invoke_curl_robust "GET" "$ProviderUrl/api/v1/wallet/did.json" "" "4.3 Get P Did" | jq -r '.id')

echo "AuthorityDid: $AuthorityDid" >&2
echo "ConsumerDid: $ConsumerDid" >&2
echo "ProviderDid: $ProviderDid" >&2
echo "-----------------------------------------------" >&2


# 2.5 C Beg 4 Credential (Consumer Requests Credential from Authority)
C_BEG_BODY=$(jq -n --arg url "$AuthorityUrl/api/v1/gate/access" \
    --arg id "$AuthorityDid" \
    '{"url": $url, "id": $id, "slug": "authority", "vc_type": "DataspaceParticipant"}')

invoke_curl_robust "POST" "$ConsumerUrl/api/v1/vc-request/beg/cross-user" "$C_BEG_BODY" "5. C Requests Credential (Beg 4 Credential)"


# 2.6 A All Requests & Get PetitionId
ALL_REQUESTS_JSON=$(invoke_curl_robust "GET" "$AuthorityUrl/api/v1/approver/all" "" "6. A Retrieves all Requests")

PetitionId=$(echo "$ALL_REQUESTS_JSON" | jq -r '.[-1].id')
if [[ -z "$PetitionId" ]]; then
    echo "ERROR: Could not get PetitionId. Exiting." >&2
    exit 1
fi
echo "PetitionId retrieved: $PetitionId" >&2


# 2.7 A Accept Request (Authority Approves)
APPROVE_BODY='{"approve": true}'
invoke_curl_robust "POST" "$AuthorityUrl/api/v1/approver/$PetitionId" "$APPROVE_BODY" "7. A Accepts the Request"


# 2.8 C All Authorities & Get OIDC4VCI_URI
ALL_AUTHORITY_JSON=$(invoke_curl_robust "GET" "$ConsumerUrl/api/v1/vc-request/all" "" "8. C Retrieves Authorities (Get OIDC4VCI URI)")

OIDC4VCI_URI=$(echo "$ALL_AUTHORITY_JSON" | jq -r '.[-1].vc_uri')
if [[ -z "$OIDC4VCI_URI" ]]; then
    echo "ERROR: Could not get OIDC4VCI_URI. Exiting." >&2
    exit 1
fi
echo "OIDC4VCI_URI retrieved: $OIDC4VCI_URI" >&2


# 2.9 C Manage OIDC4VCI (Process Credential Issuance)
OIDC4VCI_PROCESS_BODY=$(jq -n --arg uri "$OIDC4VCI_URI" '{"uri": $uri}')
invoke_curl_robust "POST" "$ConsumerUrl/api/v1/wallet/oidc4vci" "$OIDC4VCI_PROCESS_BODY" "9. C Processes OIDC4VCI (Get VC)"


# 2.10 C Grant Request & Get OIDC4VP_URI
OIDC4VP_BODY=$(jq -n --arg url "$ProviderUrl/api/v1/gate/access" \
    --arg id "$ProviderDid" \
    '{"url": $url, "id": $id, "slug": "provider", "actions": "talk"}')

# The response is the OIDC4VP URI in plain text
OIDC4VP_URI_RAW=$(invoke_curl_robust "POST" "$ConsumerUrl/api/v1/onboard/provider" "$OIDC4VP_BODY" "10. C Requests Access from Provider (Get OIDC4VP URI)")

# Clean up whitespace or carriage returns to get only the URI
OIDC4VP_URI=$(echo "$OIDC4VP_URI_RAW" | tr -d '[:space:]')
if [[ -z "$OIDC4VP_URI" ]]; then
    echo "ERROR: Could not get OIDC4VP_URI. Exiting." >&2
    exit 1
fi
echo "OIDC4VP_URI retrieved: $OIDC4VP_URI" >&2


# 2.11 C Manage OIDC4VP (Process Credential Presentation)
OIDC4VP_PROCESS_BODY=$(jq -n --arg uri "$OIDC4VP_URI" '{"uri": $uri}')
invoke_curl_robust "POST" "$ConsumerUrl/api/v1/wallet/oidc4vp" "$OIDC4VP_PROCESS_BODY" "11. C Processes OIDC4VP (Present VC)"

echo "" >&2
echo "================================================================" >&2
echo "=> Onboarding Script Finished Successfully. <= " >&2
echo "================================================================" >&2
