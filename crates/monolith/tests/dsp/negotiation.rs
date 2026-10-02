/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Contract negotiation over DSP, from the consumer's request to FINALIZED, as the tutorial's
//! pull flow walks it: offers and counter-offers, acceptance, agreement, verification.

use serde_json::{json, Value};

use crate::support::dataspace::{Dataspace, PROVIDER_DID};
use crate::support::participant::Participant;

/// Provider publishes a dataset with a distribution and a usage policy on its main catalog.
async fn publish_offer(provider: &Participant) {
    let catalog = provider.get("/api/v1/catalog-agent/catalogs/main").await;
    let service = provider
        .get("/api/v1/catalog-agent/data-services/main")
        .await;
    let dataset = provider
        .post(
            "/api/v1/catalog-agent/datasets",
            json!({"dctTitle": "Weather", "catalogId": catalog["id"]}),
        )
        .await;
    provider
        .post(
            "/api/v1/catalog-agent/distributions",
            json!({
                "dctTitle": "Weather API",
                "dcatAccessService": service["id"],
                "datasetId": dataset["id"],
                "dctFormats": "http+pull"
            }),
        )
        .await;
    provider
        .post(
            "/api/v1/catalog-agent/odrl-policies",
            json!({
                "odrlOffer": {"permission": [{"action": "use", "constraint": []}]},
                "entityId": dataset["id"],
                "entityType": "Dataset",
                "description": "Use the weather data"
            }),
        )
        .await;
}

fn offer(policy_id: &Value, target: &Value, action: &str) -> Value {
    json!({"@id": policy_id, "target": target, "@type": "Offer", "permission": [{"action": action}]})
}

/// What a participant's RPC answers about its own process.
fn process_of(answer: &Value) -> &Value {
    &answer["negotiationAgentModel"]
}

/// The consumer negotiates the provider's offer to FINALIZED: each step moves the process to
/// the expected state, and both sides end FINALIZED holding the same agreement.
#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn negotiation_reaches_finalized_with_an_agreement() {
    let ds = Dataspace::start().await;
    let (provider, consumer) = (&ds.provider, &ds.consumer);
    publish_offer(provider).await;

    let catalog = consumer
        .post(
            "/dsp/current/catalog/rpc/setup-catalog-request",
            json!({"associatedAgentPeer": PROVIDER_DID, "filter": [], "noCache": true}),
        )
        .await;
    let dataset = &catalog["response"]["dataset"][0];
    let policy_id = &dataset["hasPolicy"][0]["@id"];
    let target = &dataset["@id"];
    let provider_dsp = &catalog["response"]["service"]["endpointURL"];
    let version = consumer
        .get_unauthenticated("/.well-known/dspace-version/2025-1")
        .await;
    let callback = consumer.url(version["path"].as_str().unwrap());

    let init = consumer
        .post(
            "/dsp/current/negotiations/rpc/setup-request-init",
            json!({
                "associatedAgentPeer": PROVIDER_DID,
                "providerAddress": provider_dsp,
                "callbackAddress": callback,
                "offer": offer(policy_id, target, "use"),
            }),
        )
        .await;
    let consumer_process = process_of(&init).clone();
    assert_eq!(consumer_process["state"], "REQUESTED", "{init}");
    let pids = consumer_process["identifiers"].clone();
    assert!(pids["consumerPid"].is_string() && pids["providerPid"].is_string());
    let with_offer = |action: &str| {
        let mut body = pids.clone();
        body["offer"] = offer(policy_id, target, action);
        body
    };

    let steps: [(&Participant, &str, Value, &str); 7] = [
        (provider, "setup-offer", with_offer("use"), "OFFERED"),
        (consumer, "setup-request", with_offer("use"), "REQUESTED"),
        (provider, "setup-offer", with_offer("use"), "OFFERED"),
        (consumer, "setup-acceptance", pids.clone(), "ACCEPTED"),
        (provider, "setup-agreement", pids.clone(), "AGREED"),
        (consumer, "setup-verification", pids.clone(), "VERIFIED"),
        (provider, "setup-finalization", pids.clone(), "FINALIZED"),
    ];
    let mut provider_process = Value::Null;
    for (who, step, body, expected) in steps {
        let answer = who
            .post(&format!("/dsp/current/negotiations/rpc/{step}"), body)
            .await;
        assert_eq!(process_of(&answer)["state"], expected, "{step}: {answer}");
        if std::ptr::eq(who, provider) {
            provider_process = process_of(&answer).clone();
        }
    }

    let consumer_view = consumer
        .get(&format!(
            "/api/v1/negotiation-agent/negotiation-processes/{}",
            consumer_process["id"].as_str().unwrap()
        ))
        .await;
    assert_eq!(consumer_view["state"], "FINALIZED", "{consumer_view}");
    let agreement = &provider_process["agreement"];
    assert_eq!(agreement["target"], *target);
    assert_eq!(
        consumer_view["agreement"]["id"], agreement["id"],
        "{consumer_view}"
    );
}
