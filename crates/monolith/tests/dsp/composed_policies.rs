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

//! Composed policies over Postgres: templates referencing templates, an offer composed from
//! them, who uses what, and the plain ODRL a peer gets when negotiating it.

use serde_json::{json, Value};

use crate::negotiation::publish_offer;
use crate::support::dataspace::{Dataspace, CONSUMER_DID};

const TEMPLATES: &str = "/api/v1/catalog-agent/policy-templates";

/// The flat rules every check below expects.
fn expected_rules() -> Value {
    json!([{"action": "use", "constraint": [
        {"leftOperand": "ovc:credentialSubjectType", "operator": "eq", "rightOperand": "gx:Member"},
        {"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "P7D"}
    ]}])
}

/// The provider stores a constraint and a policy referencing it, composes an offer from both,
/// sees who uses the constraint, and offers the composed policy to a peer as plain ODRL.
#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn composed_offer_goes_out_as_plain_odrl() {
    let ds = Dataspace::start().await;
    let (provider, consumer) = (&ds.provider, &ds.consumer);
    let published = publish_offer(provider).await;

    provider
        .post(
            TEMPLATES,
            json!({
                "id": "test-expiry", "version": "1.0", "kind": "constraint",
                "controlPoints": ["validity"],
                "content": {"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "$d"},
                "parameters": {"$d": {"dataType": "duration", "defaultValue": "P30D"}}
            }),
        )
        .await;
    provider
        .post(
            TEMPLATES,
            json!({
                "id": "test-members", "version": "1.0", "kind": "policy",
                "controlPoints": ["negotiation"],
                "content": {"permission": [{"action": "use", "constraint": [
                    {"leftOperand": "ovc:credentialSubjectType", "operator": "eq", "rightOperand": "$type"}
                ]}]},
                "parameters": {"$type": {"dataType": "iri"}}
            }),
        )
        .await;

    let composed = provider
        .post(
            "/api/v1/catalog-agent/odrl-policies/compose",
            json!({
                "entityId": published.dataset_id,
                "entityType": "Dataset",
                "description": "Members, one week",
                "parts": [
                    {"@ref": "test-members@1.0", "with": {"$type": "gx:Member"}},
                    {"@ref": "test-expiry@1.0", "with": {"$d": "P7D"}, "into": "permission"}
                ]
            }),
        )
        .await;
    assert_eq!(composed["odrlOffer"]["permission"], expected_rules(), "{composed}");
    assert_eq!(composed["controlPoints"], json!(["negotiation", "validity"]), "{composed}");

    let usages = provider.get(&format!("{TEMPLATES}/test-expiry/1.0/usages")).await;
    assert_eq!(usages["offers"], json!([composed["id"]]), "{usages}");

    let version = consumer.get_unauthenticated("/.well-known/dspace-version/2025-1").await;
    let init = provider
        .post(
            "/dsp/current/negotiations/rpc/setup-offer-init",
            json!({
                "@context": "https://w3id.org/eunomia/v1/context.jsonld",
                "@type": "RpcNegotiationOfferInitMessage",
                "associatedAgentPeer": CONSUMER_DID,
                "consumerAddress": consumer.url(version["path"].as_str().unwrap()),
                "offer": {"@id": composed["id"]},
            }),
        )
        .await;
    let process = &init["negotiationAgentModel"];
    assert_eq!(process["state"], "OFFERED", "{init}");
    let offers = provider
        .get(&format!(
            "/api/v1/negotiation-agent/offers/process/{}",
            process["id"].as_str().unwrap()
        ))
        .await;
    let content = &offers["items"][0]["offerContent"];
    assert_eq!(content["permission"], expected_rules(), "{offers}");
    assert!(!content.to_string().contains("@ref"), "{content}");
}
