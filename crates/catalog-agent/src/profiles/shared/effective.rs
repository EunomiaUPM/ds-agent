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

//! A resource as a profile sees it: its description plus the links the catalog keeps outside
//! it (a catalog's datasets, a dataset's distributions, the offers on it).

use crate::entities::graph::ResourceGraph;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::resource_set::ResourceSet;
use crate::entities::vocab::Vocab;

/// The effective graph of one resource of a set.
pub struct EffectiveGraph;

impl EffectiveGraph {
    /// The description with membership and policy links drawn from `set`.
    pub fn of(resource: &CatalogResource, set: &ResourceSet) -> ResourceGraph {
        let mut graph = resource.description().clone();
        let id = resource.id().to_string();
        let mut link = |predicate: &str, children: Vec<&CatalogResource>| {
            for child in children {
                graph.push(predicate, ResourceGraph::iri(child.id().as_ref()));
            }
        };
        match resource.kind() {
            ResourceKind::Catalog => {
                link(Vocab::DCAT_DATASET_P, set.children_of(&id, ResourceKind::Dataset));
                link(Vocab::DCAT_DATASET_P, set.children_of(&id, ResourceKind::DatasetSeries));
                link(Vocab::DCAT_SERVICE, set.children_of(&id, ResourceKind::DataService));
                link(Vocab::DCAT_RECORD, set.children_of(&id, ResourceKind::CatalogRecord));
                if resource.is_main() {
                    link(Vocab::DCAT_CATALOG_P, set.sub_catalogs(&id));
                }
            }
            ResourceKind::Dataset => {
                link(Vocab::DCAT_DISTRIBUTION_P, set.children_of(&id, ResourceKind::Distribution));
            }
            _ => {}
        }
        for offer in set.offers_of(&id) {
            graph.push(Vocab::ODRL_HAS_POLICY, ResourceGraph::iri(offer.id.as_ref()));
        }
        graph
    }
}
