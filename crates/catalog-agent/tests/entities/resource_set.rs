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

//! `ResourceSet` over the provider set: what a preview keeps and what the profiles share.

use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::resource_set::ResourceSet;

use crate::support::builders::{dataset, distribution, id, provider_set, provider_set_with_draft};

/// Without a limit nothing is cut and the datasets are counted.
#[test]
fn no_limit_keeps_everything() {
    let (main, set) = provider_set();
    let (kept, truncation) = set.truncated(main.id().as_ref(), None);

    assert_eq!(kept.len(), set.len());
    assert_eq!((truncation.total, truncation.shown), (2, 2));
    assert!(!truncation.is_cut());
}

/// A limit keeps the oldest datasets with their distributions and offers; the rest and what
/// hangs from them go, the catalogs and services stay.
#[test]
fn limit_keeps_the_first_datasets() {
    let (main, mut set) = provider_set();
    set.insert(dataset(7, 1));
    set.insert(distribution(8, 7, 2, "http+pull"));
    let (kept, truncation) = set.truncated(main.id().as_ref(), Some(1));

    assert_eq!((truncation.total, truncation.shown), (3, 1));
    assert!(truncation.is_cut());
    assert!(kept.contains(&id(ResourceKind::Dataset, 3)));
    assert!(kept.contains(&id(ResourceKind::Distribution, 4)));
    assert_eq!(kept.offers_of(&id(ResourceKind::Dataset, 3)).len(), 1);
    assert!(!kept.contains(&id(ResourceKind::Dataset, 7)));
    assert!(!kept.contains(&id(ResourceKind::Distribution, 8)));
    assert!(kept.contains(&id(ResourceKind::Catalog, 5)));
    assert!(kept.contains(&id(ResourceKind::DataService, 2)));
}

/// A draft dataset leaves the shared set with its distributions and offers; the rest stays.
#[test]
fn published_drops_drafts_and_what_hangs_from_them() {
    let (main, set) = provider_set_with_draft();
    let shared = set.published();

    assert!(!shared.contains(&id(ResourceKind::Dataset, 3)));
    assert!(!shared.contains(&id(ResourceKind::Distribution, 4)));
    assert!(shared.offers_of(&id(ResourceKind::Dataset, 3)).is_empty());
    assert!(shared.contains(&id(ResourceKind::Dataset, 6)));
    assert!(shared.shared_from(&main).is_some());
}

/// A draft root, or a distribution of a draft dataset held as its parent, shares nothing.
#[test]
fn drafts_and_their_children_are_not_shared() {
    let draft = dataset(3, 1).with_publication(true);
    let mut alone = ResourceSet::new();
    let child = distribution(4, 3, 2, "http+pull");
    alone.insert(child.clone());
    alone.insert_parent(draft.clone());

    assert!(draft.is_draft());
    assert!(alone.shared_from(&child).is_none());
    assert!(ResourceSet::new().shared_from(&draft).is_none());
    assert!(!draft.with_publication(false).is_draft());
}
