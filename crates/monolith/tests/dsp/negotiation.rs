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

use common::dsp_common::odrl::ODRL_CONTEXT_URL;
use common::eunomia::EUNOMIA_CATALOG_CONTEXT_URL;
use serde_json::{json, Value};

use crate::support::dataspace::{Dataspace, CONSUMER_DID, PROVIDER_DID};
use crate::support::participant::Participant;

/// What the provider published: the dataset, its `http+pull` distribution and usage policy.
pub(crate) struct PublishedOffer {
    pub dataset_id: Value,
    pub distribution_id: Value,
    pub policy_id: Value,
}

/// A negotiation both sides finished: the offer, its target and the provider's agreement.
pub(crate) struct Negotiated {
    pub offer: PublishedOffer,
    pub target: Value,
    pub agreement: Value,
    pub consumer_process: Value,
}

/// Provider publishes a dataset with a distribution and a usage policy on its main catalog.
pub(crate) async fn publish_offer(provider: &Participant) -> PublishedOffer {
    let catalog = provider.get("/api/v1/catalog-agent/catalogs/main").await;
    let service = provider
        .get("/api/v1/catalog-agent/data-services/main")
        .await;
    let dataset = provider
        .post(
            "/api/v1/catalog-agent/datasets",
            json!({
                "@context": EUNOMIA_CATALOG_CONTEXT_URL,
                "@type": "Dataset",
                "title": {"en": "Weather"},
                "inCatalog": catalog["@id"]
            }),
        )
        .await;
    let distribution = provider
        .post(
            "/api/v1/catalog-agent/distributions",
            json!({
                "@context": EUNOMIA_CATALOG_CONTEXT_URL,
                "@type": "Distribution",
                "title": {"en": "Weather API"},
                "accessService": [service["@id"]],
                "ofDataset": dataset["@id"],
                "format": "http+pull"
            }),
        )
        .await;
    let policy = provider
        .post(
            "/api/v1/catalog-agent/odrl-policies",
            json!({
                "odrlOffer": {
                    "@context": ODRL_CONTEXT_URL,
                    "@type": "Offer",
                    "permission": [{"action": "use", "constraint": []}]
                },
                "entityId": dataset["@id"],
                "entityType": "Dataset",
                "description": "Use the weather data"
            }),
        )
        .await;
    PublishedOffer {
        dataset_id: dataset["@id"].clone(),
        distribution_id: distribution["@id"].clone(),
        policy_id: policy["id"].clone(),
    }
}

fn offer(policy_id: &Value, target: &Value, action: &str) -> Value {
    json!({"@id": policy_id, "target": target, "@type": "Offer", "permission": [{"action": action}]})
}

/// What a participant's RPC answers about its own process.
fn process_of(answer: &Value) -> &Value {
    &answer["negotiationAgentModel"]
}

/// The agreement `participant` stored for its process `process_id`.
async fn agreement_of(participant: &Participant, process_id: &Value) -> Value {
    let agreements = participant
        .get(&format!(
            "/api/v1/negotiation-agent/agreements/process/{}",
            process_id.as_str().unwrap()
        ))
        .await;
    agreements["items"][0].clone()
}

/// `participant`'s own id of the negotiation whose consumer pid is `consumer_pid`.
async fn process_id_by_consumer_pid(participant: &Participant, consumer_pid: &Value) -> Value {
    let processes = participant
        .get("/api/v1/negotiation-agent/negotiation-processes")
        .await;
    processes["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["correlation"]["identifiers"]["consumerPid"] == *consumer_pid)
        .map(|p| p["id"].clone())
        .unwrap_or_else(|| panic!("no negotiation for {consumer_pid}: {processes}"))
}

/// An eunomia RPC body of type `RpcNegotiation{kind}Message` with `members`.
fn rpc(kind: &str, members: Value) -> Value {
    let mut body = json!({
        "@context": "https://w3id.org/eunomia/v1/context.jsonld",
        "@type": format!("RpcNegotiation{kind}Message"),
    });
    body.as_object_mut()
        .unwrap()
        .extend(members.as_object().unwrap().clone());
    body
}

/// Walks the consumer's request to FINALIZED, checking each step's state on the way.
pub(crate) async fn negotiate_to_finalized(ds: &Dataspace) -> Negotiated {
    let (provider, consumer) = (&ds.provider, &ds.consumer);
    let published = publish_offer(provider).await;

    let catalog = consumer
        .post(
            "/dsp/current/catalog/rpc/setup-catalog-request",
            json!({
                "@context": "https://w3id.org/eunomia/v1/context.jsonld",
                "@type": "RpcCatalogRequestMessage",
                "associatedAgentPeer": PROVIDER_DID,
                "noCache": true
            }),
        )
        .await;
    let dataset = &catalog["response"]["dataset"][0];
    let policy_id = &dataset["hasPolicy"][0]["@id"];
    let target = &dataset["@id"];
    let provider_dsp = &catalog["response"]["service"][0]["endpointURL"];
    let version = consumer
        .get_unauthenticated("/.well-known/dspace-version/2025-1")
        .await;
    let callback = consumer.url(version["path"].as_str().unwrap());

    let init = consumer
        .post(
            "/dsp/current/negotiations/rpc/setup-request-init",
            rpc(
                "RequestInit",
                json!({
                    "associatedAgentPeer": PROVIDER_DID,
                    "providerAddress": provider_dsp,
                    "callbackAddress": callback,
                    "offer": offer(policy_id, target, "use"),
                }),
            ),
        )
        .await;
    let consumer_process = process_of(&init).clone();
    assert_eq!(consumer_process["state"], "REQUESTED", "{init}");
    let pids = consumer_process["correlation"]["identifiers"].clone();
    assert!(pids["consumerPid"].is_string() && pids["providerPid"].is_string(), "{consumer_process}");
    // Each side names the negotiation by its own process id, never by the DSP pids.
    let mine = consumer_process["id"].clone();
    let theirs = process_id_by_consumer_pid(provider, &pids["consumerPid"]).await;
    let with_offer = |kind: &str, id: &Value| {
        rpc(kind, json!({"@id": id, "offer": offer(policy_id, target, "use")}))
    };
    let with_id = |kind: &str, id: &Value| rpc(kind, json!({"@id": id}));

    let steps: [(&Participant, &str, Value, &str); 7] = [
        (provider, "setup-offer", with_offer("Offer", &theirs), "OFFERED"),
        (consumer, "setup-request", with_offer("Request", &mine), "REQUESTED"),
        (provider, "setup-offer", with_offer("Offer", &theirs), "OFFERED"),
        (consumer, "setup-acceptance", with_id("Acceptance", &mine), "ACCEPTED"),
        (provider, "setup-agreement", with_id("Agreement", &theirs), "AGREED"),
        (consumer, "setup-verification", with_id("Verification", &mine), "VERIFIED"),
        (provider, "setup-finalization", with_id("Finalization", &theirs), "FINALIZED"),
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
    Negotiated {
        offer: published,
        target: target.clone(),
        agreement: agreement_of(provider, &provider_process["id"]).await,
        consumer_process,
    }
}

/// The consumer negotiates the provider's offer to FINALIZED: each step moves the process to
/// the expected state, and both sides end FINALIZED holding the same agreement.
#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn negotiation_reaches_finalized_with_an_agreement() {
    let ds = Dataspace::start().await;
    let negotiated = negotiate_to_finalized(&ds).await;
    let (consumer_process, agreement, target) =
        (&negotiated.consumer_process, &negotiated.agreement, &negotiated.target);

    let consumer_view = ds
        .consumer
        .get(&format!(
            "/api/v1/negotiation-agent/negotiation-processes/{}",
            consumer_process["id"].as_str().unwrap()
        ))
        .await;
    assert_eq!(consumer_view["state"], "FINALIZED", "{consumer_view}");
    assert_eq!(agreement["target"], *target, "{agreement}");
    let consumer_agreement = agreement_of(&ds.consumer, &consumer_process["id"]).await;
    assert_eq!(consumer_agreement["id"], agreement["id"], "{consumer_agreement}");
}

/// A provider offering its published policy by `@id` sends it with the catalog's rules and
/// target (DSP 8.3.3), so both sides store the full offer.
#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn an_offer_by_id_goes_out_with_its_rules_and_target() {
    let ds = Dataspace::start().await;
    let (provider, consumer) = (&ds.provider, &ds.consumer);
    let published = publish_offer(provider).await;
    let version = consumer
        .get_unauthenticated("/.well-known/dspace-version/2025-1")
        .await;
    let init = provider
        .post(
            "/dsp/current/negotiations/rpc/setup-offer-init",
            rpc(
                "OfferInit",
                json!({
                    "associatedAgentPeer": CONSUMER_DID,
                    "consumerAddress": consumer.url(version["path"].as_str().unwrap()),
                    "offer": {"@id": published.policy_id},
                }),
            ),
        )
        .await;
    let provider_process = process_of(&init).clone();
    assert_eq!(provider_process["state"], "OFFERED", "{init}");
    let consumer_pid = &provider_process["correlation"]["identifiers"]["consumerPid"];
    let consumer_process = process_id_by_consumer_pid(consumer, consumer_pid).await;

    for (side, process_id) in [(provider, &provider_process["id"]), (consumer, &consumer_process)] {
        let offers = side
            .get(&format!(
                "/api/v1/negotiation-agent/offers/process/{}",
                process_id.as_str().unwrap()
            ))
            .await;
        let content = &offers["items"][0]["offerContent"];
        assert_eq!(content["@id"], published.policy_id, "{offers}");
        assert_eq!(content["target"], published.dataset_id, "{offers}");
        assert_eq!(content["permission"][0]["action"], "use", "{offers}");
    }
}
