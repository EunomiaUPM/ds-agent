#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
AGENT_URL="http://127.0.0.1:1200"


# ==========================================
# HELPER: CURL & LOGGING
# ==========================================
invoke_curl() {
    local method="$1"
    local url="$2"
    local body="$3"
    local description="$4"

    # Log header to STDERR
    # Log header to STDERR
    echo "--- $description ---" >&2
    echo "Request: $method $url" >&2

    if [[ -n "$body" ]]; then
        response=$(curl -s -X "$method" -H "Content-Type: application/json" -d "$body" "$url")
    else
        response=$(curl -s -X "$method" "$url")
    fi

    # Log preview to STDERR
    echo "Response (preview): ${response:0:100}..." >&2
    echo "---------------------------------------------------" >&2

    # Output JSON to STDOUT
    echo "$response"
}

# ==========================================
# API WRAPPERS (ATOMIC OPERATIONS)
# ==========================================

# 1. Create Process
# Args: consumer_did, initial_state
create_process() {
    local consumer_did="$1"
    local state="$2"

    local body=$(jq -n \
        --arg peer "$consumer_did" \
        --arg st "$state" \
        '{
        "state": $st,
        "associatedAgentPeer": $peer,
        "protocol": "dataspace-protocol-http",
        "role": "PROVIDER",
        "identifiers": {
            "localId": ("urn:uuid:" + (now|tostring)),
            "remoteId": ("urn:uuid:" + (now|tostring) + "-remote")
        }
    }')
    invoke_curl "POST" "$AGENT_URL/api/v1/negotiation-agent/negotiation-processes" "$body" "Creating Process ($state)" | jq -r '.id // empty'
}

# 2. Update Process State (To simulate transitions in the DB entity)
update_process_state() {
    local process_id="$1"
    local new_state="$2"

    local body=$(jq -n --arg st "$new_state" '{ "state": $st }')
    # Assuming PUT endpoint exists based on standard REST patterns, or we just rely on messages to logically move it.
    # If your API updates state automatically upon receiving a message, this might be optional.
    # For this script, we assume we create messages to document history.
    invoke_curl "PUT" "$AGENT_URL/api/v1/negotiation-agent/negotiation-processes/$process_id" "$body" "Updating Process State -> $new_state" >/dev/null
}

# 3. Create Message
# Args: process_id, type, from_state, to_state
create_message() {
    local process_id="$1"
    local type="$2"
    local from="$3"
    local to="$4"

    local body=$(jq -n \
        --arg pid "$process_id" \
        --arg type "$type" \
        --arg from "$from" \
        --arg to "$to" \
        '{
        "negotiationAgentProcessId": $pid,
        "direction": (if $type == "CONTRACT_REQUEST" or $type == "CONTRACT_AGREEMENT_VERIFICATION" then "INCOMING" else "OUTGOING" end),
        "protocol": "dataspace-protocol-http",
        "messageType": $type,
        "stateTransitionFrom": $from,
        "stateTransitionTo": $to,
        "payload": { "mock_timestamp": (now|tostring) }
    }')
    invoke_curl "POST" "$AGENT_URL/api/v1/negotiation-agent/negotiation-messages" "$body" "Msg: $type ($from -> $to)" | jq -r '.id // empty'
}

# 4. Create Offer Object
create_offer() {
    local process_id="$1"
    local message_id="$2"
    local target_id="$3"

    local offer_uid="urn:offer:$(date +%s%N)"

    local body=$(jq -n \
        --arg pid "$process_id" \
        --arg mid "$message_id" \
        --arg oid "$offer_uid" \
        --arg target "$target_id" \
        '{
        "negotiationAgentProcessId": $pid,
        "negotiationAgentMessageId": $mid,
        "offerId": $oid,
        "offerContent": {
            "uid": $oid,
            "target": $target,
            "assigner": "did:web:provider",
            "permission": [{ "action": "use", "target": $target }]
        }
    }')
    invoke_curl "POST" "$AGENT_URL/api/v1/negotiation-agent/offers" "$body" "Creating Offer Object" | jq -r '.id // empty'
}

# 5. Create Agreement Object
create_agreement() {
    local process_id="$1"
    local message_id="$2"
    local consumer="$3"
    local provider="$4"
    local target="$5"

    local body=$(jq -n \
        --arg pid "$process_id" \
        --arg mid "$message_id" \
        --arg cons "$consumer" \
        --arg prov "$provider" \
        --arg target "$target" \
        '{
        "negotiationAgentProcessId": $pid,
        "negotiationAgentMessageId": $mid,
        "consumerParticipantId": $cons,
        "providerParticipantId": $prov,
        "target": $target,
        "agreementContent": {
            "uid": ("urn:agreement:" + (now|tostring)),
            "target": $target,
            "permission": [{ "action": "use", "target": $target }]
        }
    }')
    invoke_curl "POST" "$AGENT_URL/api/v1/negotiation-agent/agreements" "$body" "Creating Agreement Object" | jq -r '.id // empty'
}


# ==========================================
# SCENARIO SIMULATIONS
# ==========================================

# SCENARIO A: HAPPY PATH (Complete Success)
# REQUESTED -> OFFERED -> ACCEPTED -> AGREED -> FINALIZED
simulate_successful_negotiation() {
    local consumer="$1"
    local dataset="$2"
    local label="$3"

    echo ">>> Starting Scenario: SUCCESSFUL ($label)" >&2

    # 1. State: REQUESTED
    local PID=$(create_process "$consumer" "REQUESTED")
    create_message "$PID" "CONTRACT_REQUEST" "INITIAL" "REQUESTED" >/dev/null

    # 2. State: OFFERED
    update_process_state "$PID" "OFFERED"
    local MID_OFFER=$(create_message "$PID" "CONTRACT_OFFER" "REQUESTED" "OFFERED")
    create_offer "$PID" "$MID_OFFER" "$dataset" >/dev/null

    # 3. State: ACCEPTED
    update_process_state "$PID" "ACCEPTED"
    create_message "$PID" "CONTRACT_NEGOTIATION_EVENT" "OFFERED" "ACCEPTED" >/dev/null

    # 4. State: AGREED
    update_process_state "$PID" "AGREED"
    local MID_AGREE=$(create_message "$PID" "CONTRACT_AGREEMENT" "ACCEPTED" "AGREED")
    # Provider DID is hardcoded mock for simplicity
    create_agreement "$PID" "$MID_AGREE" "$consumer" "did:web:provider:mock" "$dataset" >/dev/null

    # 5. State: FINALIZED
    update_process_state "$PID" "FINALIZED"
    create_message "$PID" "CONTRACT_NEGOTIATION_EVENT" "AGREED" "FINALIZED" >/dev/null

    echo ">>> Finished Scenario: SUCCESSFUL (Process: $PID)" >&2
    echo "" >&2
}

# SCENARIO B: TERMINATED (Failed Negotiation)
# REQUESTED -> OFFERED -> TERMINATED (e.g., Consumer didn't like the price)
simulate_terminated_negotiation() {
    local consumer="$1"
    local dataset="$2"
    local label="$3"

    echo ">>> Starting Scenario: TERMINATED ($label)" >&2

    # 1. State: REQUESTED
    local PID=$(create_process "$consumer" "REQUESTED")
    create_message "$PID" "CONTRACT_REQUEST" "INITIAL" "REQUESTED" >/dev/null

    # 2. State: OFFERED
    update_process_state "$PID" "OFFERED"
    local MID_OFFER=$(create_message "$PID" "CONTRACT_OFFER" "REQUESTED" "OFFERED")
    create_offer "$PID" "$MID_OFFER" "$dataset" >/dev/null

    # 3. State: TERMINATED
    update_process_state "$PID" "TERMINATED"
    # Event Message for termination
    create_message "$PID" "CONTRACT_NEGOTIATION_EVENT" "OFFERED" "TERMINATED" >/dev/null

    echo ">>> Finished Scenario: TERMINATED (Process: $PID)" >&2
    echo "" >&2
}

# SCENARIO C: PENDING (In Progress)
# REQUESTED -> OFFERED (Waiting for consumer action)
simulate_pending_negotiation() {
    local consumer="$1"
    local dataset="$2"
    local label="$3"

    echo ">>> Starting Scenario: PENDING ($label)" >&2

    # 1. State: REQUESTED
    local PID=$(create_process "$consumer" "REQUESTED")
    create_message "$PID" "CONTRACT_REQUEST" "INITIAL" "REQUESTED" >/dev/null

    # 2. State: OFFERED
    update_process_state "$PID" "OFFERED"
    local MID_OFFER=$(create_message "$PID" "CONTRACT_OFFER" "REQUESTED" "OFFERED")
    create_offer "$PID" "$MID_OFFER" "$dataset" >/dev/null

    echo ">>> Finished Scenario: PENDING (Process: $PID)" >&2
    echo "" >&2
}

# ==========================================
# MAIN
# ==========================================

main() {
    echo "=== GENERATING MOCK NEGOTIATION DATA ===" >&2

    # MOCK DATASETS & PARTICIPANTS
    # In a real script, get these from the previous catalog steps
    CONSUMER_A="did:web:consumer:company-a"
    CONSUMER_B="did:web:consumer:company-b"
    DS_WEATHER="urn:dataset:weather-2024"
    DS_TRAFFIC="urn:dataset:traffic-logs"
    DS_SENSITIVE="urn:dataset:sensitive-financials"

    # --- BATCH 1: Successful Agreements ---
    simulate_successful_negotiation "$CONSUMER_A" "$DS_WEATHER" "Weather Data Purchase - Completed"
    simulate_successful_negotiation "$CONSUMER_B" "$DS_TRAFFIC" "Traffic Data Access - Completed"
    simulate_successful_negotiation "$CONSUMER_A" "$DS_TRAFFIC" "Traffic Data Access (Repeated) - Completed"

    # --- BATCH 2: Failed / Terminated ---
    simulate_terminated_negotiation "$CONSUMER_B" "$DS_SENSITIVE" "Sensitive Data Access - Rejected by User"

    # --- BATCH 3: Ongoing / Pending ---
    simulate_pending_negotiation "$CONSUMER_A" "$DS_SENSITIVE" "Sensitive Data Access - Waiting for Approval"

    echo "=== DONE. GENERATED 5 NEGOTIATION PROCESSES ===" >&2
}

main "$@"