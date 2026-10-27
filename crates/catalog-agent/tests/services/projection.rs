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

//! ProjectionService over a mocked closure service and the registered profiles.

use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::services::closure::MockClosureServiceTrait;
use catalog_agent::services::projection::views::ProjectionBody;
use catalog_agent::services::projection::ProjectionRequest;
use common::rdf::RdfSyntax;
use common::test_utils::scopes::TestUsers;

use crate::support::builders::{provider_set, urn};
use crate::support::context::TestContext;

fn closures() -> MockClosureServiceTrait {
    let mut closures = MockClosureServiceTrait::new();
    closures.expect_resource().returning(|_, id| {
        let (_, set) = provider_set();
        Ok(set.get(id.as_ref()).cloned().map(|root| (root, set)))
    });
    closures
}

fn request(profile: &str, limit: Option<usize>, syntax: RdfSyntax) -> ProjectionRequest {
    ProjectionRequest {
        profile: profile.into(),
        limit,
        syntax,
    }
}

/// A preview keeps the asked number of datasets and says how many there are.
#[tokio::test]
async fn preview_is_cut() {
    let ctx = TestContext::new().closures(closures()).build();

    let projection = ctx
        .projections()
        .project(
            &TestUsers::root(),
            ResourceKind::Catalog,
            &urn(ResourceKind::Catalog, 1),
            &request("dcat-ap", Some(1), RdfSyntax::JsonLd),
        )
        .await
        .unwrap();

    assert_eq!(
        (projection.truncation.total, projection.truncation.shown),
        (2, 1)
    );
    let ProjectionBody::JsonLd(document) = projection.body else {
        panic!("JSON-LD expected")
    };
    let datasets = document["@graph"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["@type"] == "Dataset")
        .count();
    assert_eq!(datasets, 1);
    assert_eq!(
        projection.profile_iri,
        "https://semiceu.github.io/DCAT-AP/releases/3.0.1/"
    );
}

/// Turtle is written from the expanded projection, DSP's context included.
#[tokio::test]
async fn turtle_of_every_profile() {
    let ctx = TestContext::new().closures(closures()).build();
    for profile in ["dsp", "dcat3", "dcat-ap"] {
        let projection = ctx
            .projections()
            .project(
                &TestUsers::root(),
                ResourceKind::Catalog,
                &urn(ResourceKind::Catalog, 1),
                &request(profile, None, RdfSyntax::Turtle),
            )
            .await
            .unwrap();

        let ProjectionBody::Text(turtle) = &projection.body else {
            panic!("text expected")
        };
        assert!(turtle.contains("dcat:Catalog"), "{profile}: {turtle}");
        assert_eq!(
            projection.file_name(),
            format!("catalog-00000000-0000-0000-0000-000000000001-{profile}.ttl")
        );
    }
}

/// An unknown profile is a bad request; a resource of another kind is not found.
#[tokio::test]
async fn unknown_profile_and_wrong_kind() {
    let ctx = TestContext::new().closures(closures()).build();
    let projections = ctx.projections();

    let unknown = projections
        .project(
            &TestUsers::root(),
            ResourceKind::Catalog,
            &urn(ResourceKind::Catalog, 1),
            &request("geodcat", None, RdfSyntax::JsonLd),
        )
        .await
        .unwrap_err();
    let wrong = projections
        .project(
            &TestUsers::root(),
            ResourceKind::Catalog,
            &urn(ResourceKind::Dataset, 3),
            &request("dsp", None, RdfSyntax::JsonLd),
        )
        .await
        .unwrap_err();

    assert_eq!(unknown.info().status_code.as_u16(), 400);
    assert_eq!(wrong.info().status_code.as_u16(), 404);
}
