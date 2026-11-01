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

//! NegotiationPublicResolver over the real AgreementService and a mocked agreement repository.

use std::str::FromStr;

use chrono::Utc;
use common::public_refs::{PublicQuery, PublicResolverTrait};
use negotiation_agent::data::repo::agreement::MockAgreementRepoTrait;
use negotiation_agent::data::sea_orm::orm::agreement::Model as AgreementModel;
use negotiation_agent::facades::NegotiationPublicResolver;
use serde_json::{json, Value};
use urn::Urn;
use ymir::oauth::OwnerScope;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::support::context::TestContext;

const ID: &str = "urn:agreement:7f1c2b4e-1111-4222-8333-944444444444";

fn stored(content: Value) -> AgreementModel {
    AgreementModel {
        id: ID.to_string(),
        user_id: "ana".to_string(),
        user_role: RolePath::root(),
        visibility: Visibility::Private,
        negotiation_agent_process_id: "urn:uuid:process-1".to_string(),
        negotiation_agent_message_id: "urn:uuid:message-1".to_string(),
        consumer_participant_id: "did:web:consumer".to_string(),
        provider_participant_id: "did:web:provider".to_string(),
        agreement_content: content,
        target: "urn:dataset:1".to_string(),
        state: "AGREED".to_string(),
        created_at: Utc::now().into(),
        updated_at: None,
    }
}

/// A repository holding one agreement, read with every owner in scope.
fn resolver(content: Value) -> NegotiationPublicResolver {
    let mut repo = MockAgreementRepoTrait::new();
    repo.expect_get_agreement_by_id()
        .withf(|scope, _| *scope == OwnerScope::All)
        .returning(move |_, id| Ok((id.as_str() == ID).then(|| stored(content.clone()))));
    let ctx = TestContext::new().agreement_repo(repo).build();
    NegotiationPublicResolver::new(ctx.agreements().clone())
}

/// A private agreement is still public, as its ODRL document with the DSP context.
#[tokio::test]
async fn private_agreement_is_public_with_dsp_context() {
    let content = json!({ "@id": ID, "@type": "Agreement", "target": "urn:dataset:1" });
    let document = resolver(content)
        .resolve(&Urn::from_str(ID).unwrap(), &PublicQuery::default())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(document.media_type, "application/ld+json");
    assert_eq!(
        document.body["@context"],
        "https://w3id.org/dspace/2025/1/context.jsonld"
    );
    assert_eq!(document.body["@type"], "Agreement");
}

/// A context the agreement already carries is kept as it is.
#[tokio::test]
async fn own_context_is_kept() {
    let content = json!({ "@context": ["https://example.org/ctx"], "@id": ID });
    let document = resolver(content.clone())
        .resolve(&Urn::from_str(ID).unwrap(), &PublicQuery::default())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(document.body, content);
}

/// A URN with no agreement behind it is absent.
#[tokio::test]
async fn unknown_agreement_is_absent() {
    let document = resolver(json!({}))
        .resolve(
            &Urn::from_str("urn:agreement:other").unwrap(),
            &PublicQuery::default(),
        )
        .await
        .unwrap();

    assert!(document.is_none());
}
