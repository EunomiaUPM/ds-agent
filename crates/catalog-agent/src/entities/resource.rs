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

    /// A resource read from an imported document: the dates the document gives are kept, the
    /// agent's only fill in what it lacks.
    pub fn imported(
        id: Urn,
        kind: ResourceKind,
        owner: Owner,
        catalog_id: Option<Urn>,
        dataset_id: Option<Urn>,
        description: ResourceGraph,
    ) -> Self {
        let mut resource = Self {
            id,
            kind,
            owner,
            catalog_id,
            dataset_id,
            main: false,
            issued: Self::date_of(&description, Vocab::DCT_ISSUED).unwrap_or_else(Self::now),
            modified: Self::date_of(&description, Vocab::DCT_MODIFIED),
            description,
            version: 1,
        };
        resource.stamp_missing();
        resource
    }

    /// This resource with the description a new import of it gives, one version on.
    pub fn reimported(&self, description: ResourceGraph) -> Self {
        let mut revised = Self {
            modified: Some(
                Self::date_of(&description, Vocab::DCT_MODIFIED).unwrap_or_else(Self::now),
            ),
            description,
            version: self.version + 1,
            ..self.clone()
        };
        revised.stamp_missing();
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

    /// Kept out of the profiles (DSP, DCAT 3, DCAT-AP) until it is published.
    pub fn is_draft(&self) -> bool {
        self.description
            .first_value(Vocab::EUNOMIA_PUBLICATION_STATE)
            .is_some_and(|state| state == Vocab::DRAFT)
    }

    /// The same resource in `state`, a new version of it.
    pub fn with_publication(&self, draft: bool) -> Self {
        let mut description = self.description.clone();
        let state = if draft {
            Vocab::DRAFT
        } else {
            Vocab::PUBLISHED
        };
        description.put(
            Vocab::EUNOMIA_PUBLICATION_STATE,
            vec![ResourceGraph::literal(state, None)],
        );
        self.revise(description)
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
        self.description.remove(Vocab::DCT_ISSUED);
        self.description.remove(Vocab::DCT_MODIFIED);
        self.stamp_missing();
    }

    /// Writes `@id`, class and identifier, and the dates the description does not give.
    fn stamp_missing(&mut self) {
        let id = self.id.to_string();
        self.description.set_id(&id);
        self.description.ensure_type(self.kind.class_iri());
        if !self.description.has(Vocab::DCT_ISSUED) {
            self.description.put(
                Vocab::DCT_ISSUED,
                vec![ResourceGraph::typed(
                    &Self::lexical(self.issued),
                    Vocab::XSD_DATE_TIME,
                )],
            );
        }
        if let (Some(at), false) = (self.modified, self.description.has(Vocab::DCT_MODIFIED)) {
            self.description.put(
                Vocab::DCT_MODIFIED,
                vec![ResourceGraph::typed(
                    &Self::lexical(at),
                    Vocab::XSD_DATE_TIME,
                )],
            );
        }
        if !self.description.has(Vocab::DCT_IDENTIFIER) {
            self.description.put(
                Vocab::DCT_IDENTIFIER,
                vec![ResourceGraph::literal(&id, None)],
            );
        }
    }

    /// The first date `predicate` gives, as `xsd:dateTime` or `xsd:date` (midnight UTC).
    fn date_of(description: &ResourceGraph, predicate: &str) -> Option<DateTime<Utc>> {
        let lexical = description.first_value(predicate)?;
        DateTime::parse_from_rfc3339(&lexical)
            .map(|at| at.with_timezone(&Utc))
            .ok()
            .or_else(|| {
                chrono::NaiveDate::parse_from_str(lexical.get(..10)?, "%Y-%m-%d")
                    .ok()?
                    .and_hms_opt(0, 0, 0)
                    .map(|at| at.and_utc())
            })
            .map(|at| DateTime::from_timestamp_millis(at.timestamp_millis()).unwrap_or(at))
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
