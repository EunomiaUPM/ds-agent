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

//! ClosureService over mocked repositories: a closure takes a fixed number of queries.

use catalog_agent::data::repo::odrl_offer::MockOdrlOfferRepoTrait;
use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::entities::graph::ResourceGraph;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;
use common::test_utils::scopes::TestUsers;
use ymir::oauth::OwnerScope;

use crate::support::builders::{agent, catalog, data_service, dataset, distribution, id, main_catalog, offer, urn};
use crate::support::context::TestContext;

/// The main catalog loads roots, members, distributions, linked resources and offers once
/// each, whatever the number of datasets.
#[tokio::test]
async fn main_catalog_closure_takes_five_queries() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_main().times(1).returning(|_| Ok(Some(main_catalog(1))));
    repo.expect_get_all().times(1).returning(|_, f| {
        assert_eq!(f.main, Some(false));
        Ok(vec![catalog(5)])
    });
    repo.expect_get_children()
        .times(1)
        .withf(|_, kind, parents| kind.is_none() && parents.len() == 2)
        .returning(|_, _, _| {
            let mut published = dataset(3, 1);
            let mut description = published.description().clone();
            description.put(Vocab::DCT_PUBLISHER, vec![ResourceGraph::iri(&id(ResourceKind::Agent, 9))]);
            published = published.revise(description);
            Ok(vec![data_service(2, 1), published, dataset(6, 5)])
        });
    repo.expect_get_children()
        .times(1)
        .withf(|_, kind, parents| *kind == Some(ResourceKind::Distribution) && parents.len() == 2)
        .returning(|_, _, _| Ok(vec![distribution(4, 3, 2, "http+pull")]));
    repo.expect_get_batch()
        .times(1)
        .withf(|_, ids| ids == [urn(ResourceKind::Agent, 9)])
        .returning(|_, _| Ok(vec![agent(9, "ACME")]));
    let mut offers = MockOdrlOfferRepoTrait::new();
    offers.expect_get_by_entities()
        .times(1)
        .returning(|_, _| Ok(vec![offer(1, &urn(ResourceKind::Dataset, 3))]));
    let ctx = TestContext::new().resource_repo(repo).offer_repo(offers).build();

    let (main, set) = ctx
        .closures()
        .main_catalog(&OwnerScope::seeing(&TestUsers::alone("peer")))
        .await
        .unwrap();

    assert_eq!(main.id(), &urn(ResourceKind::Catalog, 1));
    assert_eq!(set.len(), 7);
    assert_eq!(set.offers_of(&id(ResourceKind::Dataset, 3)).len(), 1);
    assert_eq!(set.children_of(&id(ResourceKind::Catalog, 5), ResourceKind::Dataset).len(), 1);
}

/// A resource the scope does not see has no closure.
#[tokio::test]
async fn unseen_resource_has_no_closure() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(None));
    let ctx = TestContext::new().resource_repo(repo).build();

    let closure = ctx
        .closures()
        .resource(&OwnerScope::seeing(&TestUsers::alone("peer")), &urn(ResourceKind::Dataset, 3))
        .await
        .unwrap();

    assert!(closure.is_none());
}
