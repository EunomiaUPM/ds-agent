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

//! CatalogLocalFacade over mocked offer and resource services: what other agents may negotiate.

use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::facades::CatalogLocalFacade;
use catalog_agent::services::odrl_policies::MockOdrlPolicyServiceTrait;
use catalog_agent::services::resources::MockResourceServiceTrait;
use common::facades::catalog::CatalogFacadeTrait;
use ymir::errors::Errors;

use crate::support::builders::{dataset, offer, urn};
use crate::support::context::TestContext;

/// An offer on a published dataset is handed over; one on a draft is as missing as none.
#[tokio::test]
async fn offers_on_drafts_are_not_negotiable() {
    let published = urn(ResourceKind::Dataset, 3);
    let draft = urn(ResourceKind::Dataset, 7);
    let mut offers = MockOdrlPolicyServiceTrait::new();
    offers.expect_get().returning(|_, id| {
        let on = if id.to_string().ends_with('1') { 3 } else { 7 };
        let mut found = offer(1, &urn(ResourceKind::Dataset, on));
        found.id = id.clone();
        Ok(found)
    });
    let mut resources = MockResourceServiceTrait::new();
    resources.expect_get().returning(|_, _, id| {
        let n = if id == &urn(ResourceKind::Dataset, 7) {
            7
        } else {
            3
        };
        let found = dataset(n, 1);
        Ok(if n == 7 {
            found.with_publication(true)
        } else {
            found
        })
    });
    let ctx = TestContext::new()
        .offers(offers)
        .resources(resources)
        .build();
    let facade = CatalogLocalFacade::new(ctx);
    let offer_on = |n: u32| {
        format!("urn:odrl-policy:00000000-0000-0000-0000-{n:012}")
            .parse::<urn::Urn>()
            .unwrap()
    };

    let negotiable = facade.get_offer(&offer_on(1)).await.unwrap();
    let refused = facade.get_offer(&offer_on(2)).await;

    assert_eq!(negotiable.entity, published.to_string());
    assert!(
        matches!(refused, Err(Errors::MissingResourceError { .. })),
        "{refused:?}"
    );
    assert!(facade.get_dataset(&draft).await.unwrap().draft);
}
