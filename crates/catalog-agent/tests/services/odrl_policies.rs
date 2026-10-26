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

//! OdrlPolicyService over mocked repositories: an offer sits on a seen resource of its kind.

use catalog_agent::data::repo::odrl_offer::MockOdrlOfferRepoTrait;
use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::entities::commands::NewOdrlOfferCommand;
use catalog_agent::entities::odrl_offer::CatalogEntityTypes;
use catalog_agent::entities::resource_kind::ResourceKind;
use common::dsp_common::odrl::ODRL_CONTEXT_URL;
use common::test_utils::scopes::TestUsers;
use serde_json::json;

use crate::support::builders::{data_service, dataset, urn};
use crate::support::context::TestContext;

fn command(entity_type: CatalogEntityTypes) -> NewOdrlOfferCommand {
    serde_json::from_value(json!({
        "odrlOffer": {"@context": ODRL_CONTEXT_URL, "@type": "Offer", "permission": [{"action": "use"}]},
        "entityId": urn(ResourceKind::Dataset, 3).to_string(),
        "entityType": entity_type.to_string(),
    }))
    .unwrap()
}

/// An offer on a dataset the user sees is stored as the user's.
#[tokio::test]
async fn offer_on_a_seen_dataset_is_stored() {
    let mut resources = MockResourceRepoTrait::new();
    resources.expect_get_by_id().returning(|_, _| Ok(Some(dataset(3, 1))));
    let mut offers = MockOdrlOfferRepoTrait::new();
    offers.expect_insert().returning(|o| Ok(o.clone()));
    let ctx = TestContext::new().resource_repo(resources).offer_repo(offers).build();

    let stored = ctx
        .offers()
        .create(&TestUsers::alone("ana"), &command(CatalogEntityTypes::Dataset))
        .await
        .unwrap();

    assert_eq!(stored.owner.user_id, "ana");
    assert_eq!(stored.entity_type, "Dataset");
    assert!(stored.id.to_string().starts_with("urn:odrl-policy:"));
}

/// An offer naming the wrong kind, or a resource the user cannot see, is refused unstored.
#[tokio::test]
async fn offer_on_the_wrong_resource_is_refused() {
    let mut resources = MockResourceRepoTrait::new();
    resources.expect_get_by_id().returning(|_, _| Ok(Some(data_service(3, 1))));
    let ctx = TestContext::new().resource_repo(resources).build();

    let refused = ctx
        .offers()
        .create(&TestUsers::alone("ana"), &command(CatalogEntityTypes::Dataset))
        .await
        .unwrap_err();

    assert_eq!(refused.info().status_code.as_u16(), 400);
}
