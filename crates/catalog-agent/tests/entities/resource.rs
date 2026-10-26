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

//! CatalogResource stamping and revision; no doubles.

use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;

use crate::support::builders::{dataset, id, titled};

/// A new resource carries its id, class, issue date and a default identifier.
#[test]
fn creation_stamps_the_managed_properties() {
    let dataset = dataset(3, 1);
    let description = dataset.description();

    assert_eq!(description.id(), Some(id(ResourceKind::Dataset, 3).as_str()));
    assert!(description.has_type(Vocab::DCAT_DATASET));
    assert!(description.has(Vocab::DCT_ISSUED));
    assert!(!description.has(Vocab::DCT_MODIFIED));
    assert_eq!(description.first_value(Vocab::DCT_IDENTIFIER), Some(id(ResourceKind::Dataset, 3)));
    assert_eq!(dataset.version(), 1);
    assert_eq!(dataset.etag(), "W/\"1\"");
}

/// A revision keeps the issue date, sets the modification date and moves the version on.
#[test]
fn revision_keeps_issued_and_moves_the_version() {
    let original = dataset(3, 1);
    let revised = original.revise(titled("renamed"));

    assert_eq!(revised.issued(), original.issued());
    assert!(revised.modified().is_some());
    assert!(revised.description().has(Vocab::DCT_MODIFIED));
    assert_eq!(revised.version(), 2);
    assert_eq!(revised.title().as_deref(), Some("renamed"));
    assert_eq!(revised.catalog_id(), original.catalog_id());
}
