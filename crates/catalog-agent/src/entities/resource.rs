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

//! A catalog resource: its RDF description plus what the agent keeps about it (owner, where it
//! hangs from, version).

use chrono::{DateTime, SecondsFormat, Utc};
use common::oauth::{OwnedTrait, Owner};
use urn::Urn;

use crate::entities::graph::ResourceGraph;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;

/// One DCAT resource of the catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogResource {
    id: Urn,
    kind: ResourceKind,
    owner: Owner,
    catalog_id: Option<Urn>,
    dataset_id: Option<Urn>,
    main: bool,
    description: ResourceGraph,
    issued: DateTime<Utc>,
    modified: Option<DateTime<Utc>>,
    version: i64,
}

/// Everything a stored resource is made of, to rebuild one.
pub struct CatalogResourceParts {
    pub id: Urn,
    pub kind: ResourceKind,
    pub owner: Owner,
    pub catalog_id: Option<Urn>,
    pub dataset_id: Option<Urn>,
    pub main: bool,
    pub description: ResourceGraph,
    pub issued: DateTime<Utc>,
    pub modified: Option<DateTime<Utc>>,
    pub version: i64,
}

impl CatalogResource {
    /// A resource as stored; the description is taken as it is.
    pub fn rehydrate(parts: CatalogResourceParts) -> Self {
        Self {
            id: parts.id,
            kind: parts.kind,
            owner: parts.owner,
            catalog_id: parts.catalog_id,
            dataset_id: parts.dataset_id,
            main: parts.main,
            description: parts.description,
            issued: parts.issued,
            modified: parts.modified,
            version: parts.version,
        }
    }

    /// A new resource issued now, with the managed properties stamped on its description.
    pub fn create(
        id: Urn,
        kind: ResourceKind,
        owner: Owner,
        catalog_id: Option<Urn>,
        dataset_id: Option<Urn>,
        main: bool,
        description: ResourceGraph,
    ) -> Self {
        let mut resource = Self {
            id,
            kind,
            owner,
            catalog_id,
            dataset_id,
            main,
            description,
            issued: Self::now(),
            modified: None,
            version: 1,
        };
        resource.stamp();
        resource
    }

    /// The resource with `description` replacing the old one, modified now, one version on.
    pub fn revise(&self, description: ResourceGraph) -> Self {
        let mut revised = Self {
            description,
            modified: Some(Self::now()),
            version: self.version + 1,
            ..self.clone()
        };
        revised.stamp();
        revised
    }

    /// The same resource under `owner`; a revision is what moves the version.
    pub fn with_owner(self, owner: Owner) -> Self {
        Self { owner, ..self }
    }

    pub fn id(&self) -> &Urn {
        &self.id
    }

    pub fn kind(&self) -> ResourceKind {
        self.kind
    }

    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    pub fn catalog_id(&self) -> Option<&Urn> {
        self.catalog_id.as_ref()
    }

    pub fn dataset_id(&self) -> Option<&Urn> {
        self.dataset_id.as_ref()
    }

    /// The parent it hangs from, catalog or dataset.
    pub fn parent_id(&self) -> Option<&Urn> {
        self.catalog_id.as_ref().or(self.dataset_id.as_ref())
    }

    pub fn is_main(&self) -> bool {
        self.main
    }

    pub fn description(&self) -> &ResourceGraph {
        &self.description
    }

    pub fn issued(&self) -> DateTime<Utc> {
        self.issued
    }

    pub fn modified(&self) -> Option<DateTime<Utc>> {
        self.modified
    }

    pub fn version(&self) -> i64 {
        self.version
    }

    /// The title an index or a listing shows: English first, then any.
    pub fn title(&self) -> Option<String> {
        self.description.text(Vocab::DCT_TITLE, &["en"])
    }

    /// The weak entity tag of this version.
    pub fn etag(&self) -> String {
        format!("W/\"{}\"", self.version)
    }

    /// Writes `@id`, class, issue and modification dates and a default identifier.
    fn stamp(&mut self) {
        let id = self.id.to_string();
        self.description.set_id(&id);
        self.description.ensure_type(self.kind.class_iri());
        self.description.put(
            Vocab::DCT_ISSUED,
            vec![ResourceGraph::typed(&Self::lexical(self.issued), Vocab::XSD_DATE_TIME)],
        );
        let modified = self
            .modified
            .map(|at| vec![ResourceGraph::typed(&Self::lexical(at), Vocab::XSD_DATE_TIME)])
            .unwrap_or_default();
        self.description.put(Vocab::DCT_MODIFIED, modified);
        if !self.description.has(Vocab::DCT_IDENTIFIER) {
            self.description
                .put(Vocab::DCT_IDENTIFIER, vec![ResourceGraph::literal(&id, None)]);
        }
    }

    fn lexical(at: DateTime<Utc>) -> String {
        at.to_rfc3339_opts(SecondsFormat::Millis, true)
    }

    /// Now, to the millisecond the database keeps, so a re-read compares equal.
    fn now() -> DateTime<Utc> {
        let now = Utc::now();
        DateTime::from_timestamp_millis(now.timestamp_millis()).unwrap_or(now)
    }
}

impl OwnedTrait for CatalogResource {
    fn owner(&self) -> Owner {
        self.owner.clone()
    }
}
