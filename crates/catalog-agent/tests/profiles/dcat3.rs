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

//! The DCAT 3 adapter: everything optional, value kinds checked when given.

use catalog_agent::entities::graph::ResourceGraph;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;
use catalog_agent::profiles::dcat3::Dcat3;
use catalog_agent::profiles::shared::dcat::DcatAdapter;
use catalog_agent::profiles::{ProfileAdapterTrait, Severity};

use crate::support::builders::{id, provider_set, resource};

/// The provider's catalog conforms to DCAT 3 as it is.
#[test]
fn provider_catalog_conforms() {
    let (main, set) = provider_set();
    let diagnosis = DcatAdapter::<Dcat3>::default().diagnose(&set, &main, true).unwrap();

    assert!(diagnosis.compliant, "{:#?}", diagnosis.findings);
}

/// A literal where DCAT 3 takes a resource is a violation.
#[test]
fn literal_media_type_is_a_violation() {
    let (_, mut set) = provider_set();
    let mut description = ResourceGraph::default();
    description.put(Vocab::DCAT_MEDIA_TYPE, vec![ResourceGraph::literal("text/csv", None)]);
    let distribution = resource(ResourceKind::Distribution, 7, None, Some(3), description);
    set.insert(distribution.clone());
    let diagnosis = DcatAdapter::<Dcat3>::default().diagnose(&set, &distribution, false).unwrap();

    assert!(diagnosis.findings.iter().any(|f| f.severity == Severity::Violation
        && f.property.as_deref() == Some(Vocab::DCAT_MEDIA_TYPE)));
}

/// The projection writes `dcterms:` the way the recommendation does, membership included.
#[test]
fn projection_uses_dcterms_and_lists_members() {
    let (main, set) = provider_set();
    let document = DcatAdapter::<Dcat3>::default().project(&set, &main).unwrap();

    assert_eq!(document["@context"]["dcterms"], Vocab::DCT);
    let catalog = &document["@graph"][0];
    assert!(catalog["dataset"].as_array().unwrap().iter().any(|d| d == &serde_json::json!(id(ResourceKind::Dataset, 3))));
    assert_eq!(catalog["catalog"][0], id(ResourceKind::Catalog, 5));
}
