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

//! DoctorService over a mocked closure service and the registered profiles.

use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::services::closure::MockClosureServiceTrait;
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

/// Every registered profile gives its verdict on the resource.
#[tokio::test]
async fn every_profile_is_asked() {
    let ctx = TestContext::new().closures(closures()).build();

    let report = ctx
        .doctor()
        .diagnose(&TestUsers::root(), ResourceKind::Dataset, &urn(ResourceKind::Dataset, 3), None, false)
        .await
        .unwrap();

    let profiles: Vec<_> = report.profiles.iter().map(|p| p.profile.as_str()).collect();
    assert_eq!(profiles, vec!["dsp", "dcat3", "dcat-ap"]);
    assert_eq!(report.checked, vec![urn(ResourceKind::Dataset, 3).to_string()]);
    let dsp = &report.profiles[0];
    assert!(dsp.compliant, "{:#?}", dsp.findings);
    let dcat_ap = &report.profiles[2];
    assert!(!dcat_ap.compliant, "a dataset without description breaks DCAT-AP");
}

/// `deep` checks the whole closure; one profile can be asked alone.
#[tokio::test]
async fn deep_and_single_profile() {
    let ctx = TestContext::new().closures(closures()).build();

    let report = ctx
        .doctor()
        .diagnose(&TestUsers::root(), ResourceKind::Catalog, &urn(ResourceKind::Catalog, 1), Some("dsp".into()), true)
        .await
        .unwrap();

    assert_eq!(report.profiles.len(), 1);
    assert_eq!(report.checked.len(), provider_set().1.len());
}

/// An unknown profile is a bad request naming the ones there are.
#[tokio::test]
async fn unknown_profile_lists_the_available_ones() {
    let ctx = TestContext::new().build();

    let refused = ctx
        .doctor()
        .diagnose(&TestUsers::root(), ResourceKind::Dataset, &urn(ResourceKind::Dataset, 3), Some("geodcat".into()), false)
        .await
        .unwrap_err();

    assert_eq!(refused.info().status_code.as_u16(), 400);
    assert!(refused.reason().contains("dcat-ap"));
}

/// Asking for a resource as the wrong kind is a missing resource.
#[tokio::test]
async fn wrong_kind_is_not_found() {
    let ctx = TestContext::new().closures(closures()).build();

    let missing = ctx
        .doctor()
        .diagnose(&TestUsers::root(), ResourceKind::Catalog, &urn(ResourceKind::Dataset, 3), None, false)
        .await
        .unwrap_err();

    assert_eq!(missing.info().status_code.as_u16(), 404);
}
