#!/usr/bin/env bash
# Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
# SPDX-License-Identifier: GPL-3.0-or-later
#
# End-to-end Kafka Provider PUSH Relay demonstration (Kafka -> HTTP Sink):
# - Ensures the `redpanda` broker container (`--profile kafka`) is running and creates topic `telemetry.sensors.v1.<tag>`
# - Registers a `PUSH` `ConnectorTemplate` (`protocol: "KAFKA"`, `VEC<STRING>` `KAFKA_BROKERS`, `STRING` `KAFKA_TOPIC`, `autoOffsetReset: "EARLIEST"`)
# - Instantiates a `ConnectorInstance` bound to a DCAT `Distribution` (`format: "http+push"`)
# - Negotiates a DSP 2025-1 Contract Agreement (`FINALIZED`) and starts a `PUSH` Transfer (`STARTED`)
# - Produces 3 keyed JSON sensor events via `rpk topic produce` and verifies receipt + `X-Kafka-*` metadata headers at the Consumer HTTP sink
# - Exercises `Suspend` -> produces a 4th event while suspended -> `Resume` (verifying automatic continuation from committed offset 3 with zero duplicate replay) -> `Terminate`

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./common.sh
source "$SCRIPT_DIR/common.sh"

require_tools

REDPANDA_CONTAINER="${REDPANDA_CONTAINER:-examples-redpanda}"
if ! docker ps --format '{{.Names}}' | grep -qx "$REDPANDA_CONTAINER"; then
    echo "Redpanda container '$REDPANDA_CONTAINER' is not running." >&2
    echo "Start it first with: docker compose -f examples/docker-compose.yaml --profile kafka up -d --wait redpanda" >&2
    exit 1
fi

RUN_TAG="$(date +%s)"
TEMPLATE_VERSION="1.0.$RUN_TAG"
TOPIC_NAME="telemetry.sensors.v1.$RUN_TAG"
PROVIDER_DID=$(api_json GET "$PROVIDER_URL/.well-known/did.json" | jq -r '.id')

# Clear previous consumer sink events before starting
api_json DELETE "$MOCK_URL/consumer-sink/events" >/dev/null

log_step "1. Creating Kafka topic '$TOPIC_NAME' on Redpanda"
docker exec "$REDPANDA_CONTAINER" rpk topic create "$TOPIC_NAME" -p 1 -r 1 >/dev/null
log_ok "Created single-partition topic '$TOPIC_NAME'"

log_step "2. Registering Kafka PUSH ConnectorTemplate (kafka-telemetry-push@$TEMPLATE_VERSION)"
TEMPLATE_BODY=$(jq -n \
    --arg ctx "$CONNECTOR_CONTEXT" \
    --arg ver "$TEMPLATE_VERSION" \
    '{
        "@context": $ctx,
        "@type": "ConnectorTemplate",
        "name": "kafka-telemetry-push",
        "version": $ver,
        "description": "Supervised Kafka consumer relay delivering topic records to the consumer DataAddress sink",
        "authentication": {
            "type": "NO_AUTH"
        },
        "interaction": {
            "mode": "PUSH",
            "lifetime": "INFINITE",
            "maxRetries": 3,
            "subscribe": {
                "protocol": "KAFKA",
                "brokers": "{{__KAFKA_BROKERS__}}",
                "topic": "{{__KAFKA_TOPIC__}}",
                "securityProtocol": "PLAINTEXT",
                "autoOffsetReset": "EARLIEST"
            }
        },
        "parameters": [
            {
                "paramType": "VEC<STRING>",
                "name": "KAFKA_BROKERS",
                "title": "Kafka Bootstrap Brokers",
                "required": true
            },
            {
                "paramType": "STRING",
                "name": "KAFKA_TOPIC",
                "title": "Source Kafka Topic",
                "required": true
            }
        ]
    }')
api_json POST "$PROVIDER_URL/api/v1/connector/templates" "$TEMPLATE_BODY" >/dev/null
log_ok "Registered ConnectorTemplate kafka-telemetry-push@$TEMPLATE_VERSION"

log_step "3. Publishing DCAT Dataset, Distribution (http+push), ODRL Offer, and Kafka ConnectorInstance"
MAIN_CATALOG_ID=$(api_json GET "$PROVIDER_URL/api/v1/catalog-agent/catalogs/main" | jq -r '.["@id"]')
MAIN_SERVICE_ID=$(api_json GET "$PROVIDER_URL/api/v1/catalog-agent/data-services/main" | jq -r '.["@id"]')

DATASET_ID=$(api_json POST "$PROVIDER_URL/api/v1/catalog-agent/datasets" "$(jq -n \
    --arg ctx "$CATALOG_CONTEXT" \
    --arg cat "$MAIN_CATALOG_ID" \
    --arg title "Urban IoT Telemetry Kafka Stream ($RUN_TAG)" \
    '{"@context": $ctx, "@type": "Dataset", "title": {"en": $title}, "inCatalog": $cat}')" | jq -r '.["@id"]')

DISTRIBUTION_ID=$(api_json POST "$PROVIDER_URL/api/v1/catalog-agent/distributions" "$(jq -n \
    --arg ctx "$CATALOG_CONTEXT" \
    --arg svc "$MAIN_SERVICE_ID" \
    --arg ds "$DATASET_ID" \
    '{"@context": $ctx, "@type": "Distribution", "title": {"en": "Kafka -> HTTP Push Telemetry Feed"}, "accessService": [$svc], "ofDataset": $ds, "format": "http+push"}')" | jq -r '.["@id"]')

api_json POST "$PROVIDER_URL/api/v1/catalog-agent/odrl-policies" "$(jq -n \
    --arg ctx "$ODRL_CONTEXT" \
    --arg ds "$DATASET_ID" \
    '{"odrlOffer": {"@context": $ctx, "@type": "Offer", "permission": [{"action": "use", "constraint": []}]}, "entityId": $ds, "entityType": "Dataset", "description": "Continuous Kafka telemetry streaming"}')" >/dev/null

api_json POST "$PROVIDER_URL/api/v1/connector/instances" "$(jq -n \
    --arg ctx "$CONNECTOR_CONTEXT" \
    --arg ver "$TEMPLATE_VERSION" \
    --arg dist "$DISTRIBUTION_ID" \
    --arg topic "$TOPIC_NAME" \
    '{
        "@context": $ctx,
        "@type": "ConnectorInstance",
        "templateName": "kafka-telemetry-push",
        "templateVersion": $ver,
        "distributionId": $dist,
        "parameters": {
            "KAFKA_BROKERS": ["redpanda:9092"],
            "KAFKA_TOPIC": $topic
        }
    }')" >/dev/null
log_ok "Bound Kafka ConnectorInstance (redpanda:9092 / $TOPIC_NAME) to Distribution $DISTRIBUTION_ID"

log_step "4. Negotiating Contract Agreement over DSP 2025-1"
AGREEMENT_ID=$(negotiate_dataset_to_finalized "$PROVIDER_DID" "$DATASET_ID")
log_ok "Contract Agreement finalized: $AGREEMENT_ID"

log_step "5. Requesting http+push Transfer and starting supervised KafkaRelayWorker"
REQ_RESP=$(api_json POST "$CONSUMER_URL/dsp/current/transfers/rpc/setup-request" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" \
    --arg peer "$PROVIDER_DID" \
    --arg ag "$AGREEMENT_ID" \
    '{
        "@context": $ctx,
        "@type": "RpcTransferRequestMessage",
        "associatedAgentPeer": $peer,
        "agreementId": $ag,
        "format": "http+push",
        "dataAddress": {
            "endpointType": "HTTP",
            "endpoint": "http://mock-backend:8080/consumer-sink/events"
        },
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
log_ok "Kafka PUSH transfer STARTED (Provider dataplane: $PROVIDER_DP_ID)"

log_step "6. Producing 3 keyed JSON telemetry events to '$TOPIC_NAME' and verifying X-Kafka-* headers at Consumer sink"
printf '%s\n' \
    'sensor-01:{"sensorId":"sensor-01","tempC":22.4,"seq":1}' \
    'sensor-02:{"sensorId":"sensor-02","tempC":23.1,"seq":2}' \
    'sensor-03:{"sensorId":"sensor-03","tempC":21.9,"seq":3}' \
    | docker exec -i "$REDPANDA_CONTAINER" rpk topic produce "$TOPIC_NAME" -f '%k:%v\n' >/dev/null

SINK_RESP=""
for _ in $(seq 1 30); do
    SINK_RESP=$(api_json GET "$MOCK_URL/consumer-sink/events")
    COUNT=$(printf '%s' "$SINK_RESP" | jq -r '.count')
    if [[ "$COUNT" -ge 3 ]]; then
        break
    fi
    sleep 0.5
done

COUNT=$(printf '%s' "$SINK_RESP" | jq -r '.count')
[[ "$COUNT" == "3" ]] || { echo "Expected 3 events at consumer sink, got $COUNT: $SINK_RESP" >&2; exit 1; }

FIRST_TOPIC=$(printf '%s' "$SINK_RESP" | jq -r '.items[0].headers["x-kafka-topic"]')
FIRST_OFFSET=$(printf '%s' "$SINK_RESP" | jq -r '.items[0].headers["x-kafka-offset"]')
THIRD_OFFSET=$(printf '%s' "$SINK_RESP" | jq -r '.items[2].headers["x-kafka-offset"]')
THIRD_KEY=$(printf '%s' "$SINK_RESP" | jq -r '.items[2].headers["x-kafka-key"]')
THIRD_SENSOR=$(printf '%s' "$SINK_RESP" | jq -r '.items[2].payload.sensorId')

[[ "$FIRST_TOPIC" == "$TOPIC_NAME" ]] || { echo "Expected X-Kafka-Topic '$TOPIC_NAME', got '$FIRST_TOPIC'" >&2; exit 1; }
[[ "$FIRST_OFFSET" == "0" && "$THIRD_OFFSET" == "2" ]] || { echo "Expected offsets 0..2, got $FIRST_OFFSET..$THIRD_OFFSET" >&2; exit 1; }
[[ "$THIRD_KEY" == "sensor-03" && "$THIRD_SENSOR" == "sensor-03" ]] || { echo "Unexpected payload/key: $SINK_RESP" >&2; exit 1; }
log_ok "Consumer HTTP sink received all 3 Kafka records in order (offsets 0..2, X-Kafka-Topic=$FIRST_TOPIC, X-Kafka-Key=$THIRD_KEY)"

log_step "7. Exercising Suspend -> produce while paused -> Resume (committed offset continuity) -> Terminate"
api_json POST "$PROVIDER_URL/dsp/current/transfers/rpc/setup-suspension" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$PROVIDER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferSuspensionMessage", "@id": $id}')" >/dev/null

printf '%s\n' 'sensor-04:{"sensorId":"sensor-04","tempC":24.0,"seq":4}' \
    | docker exec -i "$REDPANDA_CONTAINER" rpk topic produce "$TOPIC_NAME" -f '%k:%v\n' >/dev/null
sleep 1.5
PAUSED_COUNT=$(api_json GET "$MOCK_URL/consumer-sink/events" | jq -r '.count')
[[ "$PAUSED_COUNT" == "3" ]] || { echo "Expected sink count to stay at 3 while SUSPENDED, got $PAUSED_COUNT" >&2; exit 1; }
log_ok "Relay halted cleanly during SUSPENDED state (queued offset 3 held in Kafka)"

api_json POST "$PROVIDER_URL/dsp/current/transfers/rpc/setup-start" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$PROVIDER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferStartMessage", "@id": $id}')" >/dev/null

for _ in $(seq 1 30); do
    SINK_RESP=$(api_json GET "$MOCK_URL/consumer-sink/events")
    COUNT=$(printf '%s' "$SINK_RESP" | jq -r '.count')
    if [[ "$COUNT" -ge 4 ]]; then
        break
    fi
    sleep 0.5
done

COUNT=$(printf '%s' "$SINK_RESP" | jq -r '.count')
FOURTH_OFFSET=$(printf '%s' "$SINK_RESP" | jq -r '.items[3].headers["x-kafka-offset"]')
[[ "$COUNT" == "4" && "$FOURTH_OFFSET" == "3" ]] || { echo "Expected resumed delivery of offset 3 (total 4), got count=$COUNT offset=$FOURTH_OFFSET" >&2; exit 1; }
log_ok "Resumed transfer continued from committed offset 3 without duplicate replay (total delivered: 4)"

api_json POST "$CONSUMER_URL/dsp/current/transfers/rpc/setup-termination" "$(jq -n \
    --arg ctx "$EUNOMIA_CONTEXT" --arg id "$CONSUMER_TP_ID" \
    '{"@context": $ctx, "@type": "RpcTransferTerminationMessage", "@id": $id}')" >/dev/null
TERM_STATE=$(api_json GET "$PROVIDER_URL/api/v1/transfer-agent/dataplane/dataplane-processes/$PROVIDER_DP_ID" | jq -r '.state')
[[ "$TERM_STATE" == "Terminated" ]] || { echo "Expected Provider dataplane Terminated, got $TERM_STATE" >&2; exit 1; }
log_ok "Terminated transfer stopped KafkaRelayWorker and moved dataplane to '$TERM_STATE'"

printf '\n\033[1;32mKafka Provider PUSH relay demo succeeded end-to-end!\033[0m\n'
