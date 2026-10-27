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

//! A loaded slice of the catalog graph: a root with what hangs from it and what it links to,
//! read in a fixed number of queries and handed whole to a profile adapter.

use std::collections::{HashMap, HashSet};

use crate::entities::odrl_offer::OdrlOffer;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;

/// How much of a set a preview kept: datasets and series shown out of those there are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Truncation {
    pub total: usize,
    pub shown: usize,
}

impl Truncation {
    pub fn is_cut(&self) -> bool {
        self.shown < self.total
    }
}

/// Resources by id plus the offers attached to them.
#[derive(Debug, Clone, Default)]
pub struct ResourceSet {
    resources: HashMap<String, CatalogResource>,
    /// Ids in the order they were added, so projections are stable.
    order: Vec<String>,
    offers: HashMap<String, Vec<OdrlOffer>>,
}

impl ResourceSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a resource; one already there is kept.
    pub fn insert(&mut self, resource: CatalogResource) {
        let id = resource.id().to_string();
        if !self.resources.contains_key(&id) {
            self.order.push(id.clone());
            self.resources.insert(id, resource);
        }
    }

    pub fn extend(&mut self, resources: impl IntoIterator<Item = CatalogResource>) {
        for resource in resources {
            self.insert(resource);
        }
    }

    pub fn insert_offers(&mut self, offers: impl IntoIterator<Item = OdrlOffer>) {
        for offer in offers {
            self.offers
                .entry(offer.entity.to_string())
                .or_default()
                .push(offer);
        }
    }

    pub fn get(&self, id: &str) -> Option<&CatalogResource> {
        self.resources.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.resources.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Every resource, in the order added.
    pub fn iter(&self) -> impl Iterator<Item = &CatalogResource> {
        self.order.iter().filter_map(|id| self.resources.get(id))
    }

    /// The ids of every resource held.
    pub fn ids(&self) -> Vec<String> {
        self.order.clone()
    }

    /// Resources of `kind` hanging from `parent`, oldest first.
    pub fn children_of(&self, parent: &str, kind: ResourceKind) -> Vec<&CatalogResource> {
        let mut children: Vec<&CatalogResource> = self
            .iter()
            .filter(|r| r.kind() == kind)
            .filter(|r| r.parent_id().is_some_and(|p| p.to_string() == parent))
            .collect();
        children.sort_by_key(|r| r.issued());
        children
    }

    /// Every catalog but `main`, oldest first: the sub-catalogs a main catalog shows.
    pub fn sub_catalogs(&self, main: &str) -> Vec<&CatalogResource> {
        let mut catalogs: Vec<&CatalogResource> = self
            .iter()
            .filter(|r| r.kind() == ResourceKind::Catalog && r.id().to_string() != main)
            .collect();
        catalogs.sort_by_key(|r| r.issued());
        catalogs
    }

    /// The offers attached to `id`.
    pub fn offers_of(&self, id: &str) -> &[OdrlOffer] {
        self.offers.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Every offer held, whatever it is attached to.
    pub fn all_offers(&self) -> impl Iterator<Item = &OdrlOffer> {
        self.offers.values().flatten()
    }

    /// The datasets and series of the set, oldest first.
    fn members(&self) -> Vec<&CatalogResource> {
        let mut members: Vec<&CatalogResource> = self
            .iter()
            .filter(|r| {
                matches!(
                    r.kind(),
                    ResourceKind::Dataset | ResourceKind::DatasetSeries
                )
            })
            .collect();
        members.sort_by_key(|r| r.issued());
        members
    }

    /// The set with only its first `limit` datasets and series and what hangs from them
    /// (distributions, records about them, offers); the root and everything else stays.
    pub fn truncated(&self, root: &str, limit: Option<usize>) -> (ResourceSet, Truncation) {
        let members = self.members();
        let total = members
            .iter()
            .filter(|m| m.id().to_string() != root)
            .count();
        let dropped: HashSet<String> = members
            .iter()
            .filter(|m| m.id().to_string() != root)
            .skip(limit.unwrap_or(usize::MAX))
            .map(|m| m.id().to_string())
            .collect();
        let shown = total - dropped.len();
        if dropped.is_empty() {
            return (self.clone(), Truncation { total, shown });
        }
        let hangs_from_dropped = |r: &CatalogResource| {
            r.dataset_id()
                .is_some_and(|d| dropped.contains(&d.to_string()))
                || (r.kind() == ResourceKind::CatalogRecord
                    && r.description()
                        .iris(Vocab::FOAF_PRIMARY_TOPIC)
                        .iter()
                        .any(|topic| dropped.contains(*topic)))
        };
        let mut kept = ResourceSet::new();
        for resource in self.iter() {
            let id = resource.id().to_string();
            if dropped.contains(&id) || hangs_from_dropped(resource) {
                continue;
            }
            kept.insert_offers(self.offers_of(&id).iter().cloned());
            kept.insert(resource.clone());
        }
        (kept, Truncation { total, shown })
    }
}
