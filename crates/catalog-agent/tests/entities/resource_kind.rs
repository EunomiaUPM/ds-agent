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

//! ResourceKind mappings between classes, columns and URNs; no doubles.

use std::str::FromStr;

use catalog_agent::entities::resource_kind::{Parent, ResourceKind};
use catalog_agent::entities::vocab::Vocab;

/// Organizations and persons are agents; unknown classes are no kind.
#[test]
fn classes_map_to_kinds() {
    assert_eq!(ResourceKind::from_class(Vocab::DCAT_DATASET), Some(ResourceKind::Dataset));
    assert_eq!(ResourceKind::from_class(Vocab::FOAF_ORGANIZATION), Some(ResourceKind::Agent));
    assert_eq!(ResourceKind::from_class("https://example.org/Thing"), None);
}

/// The column value and the class term both parse.
#[test]
fn kind_parses_from_column_or_term() {
    assert_eq!(ResourceKind::from_str("data_service").unwrap(), ResourceKind::DataService);
    assert_eq!(ResourceKind::from_str("DataService").unwrap(), ResourceKind::DataService);
    assert!(ResourceKind::from_str("table").is_err());
}

/// Minted URNs use the kind's namespace; parents follow DCAT membership.
#[test]
fn urns_and_parents_follow_the_kind() {
    assert!(ResourceKind::DatasetSeries.mint().to_string().starts_with("urn:dataset-series:"));
    assert_eq!(ResourceKind::Distribution.parent(), Parent::Dataset);
    assert_eq!(ResourceKind::DataService.membership(), Some(Vocab::DCAT_SERVICE));
    assert_eq!(ResourceKind::Agent.membership(), None);
}
