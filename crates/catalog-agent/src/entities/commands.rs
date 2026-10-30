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

//! Write intents: a resource to create or replace, an offer, a template version.

use chrono::{DateTime, FixedOffset};
use common::dsp_common::odrl::OdrlPolicyInfo;
use common::eunomia::document::JsonLd;
use common::oauth::Owner;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urn::Urn;
use ymir::types::participants::Visibility;

use crate::entities::graph::ResourceGraph;
use crate::entities::odrl_offer::CatalogEntityTypes;
use crate::entities::policy_templates::{ControlPoint, PolicyTemplateKind};
use crate::entities::resource_kind::ResourceKind;

/// A resource to create: its description and where it hangs from.
#[derive(Debug, Clone)]
pub struct NewResourceCommand {
    pub kind: ResourceKind,
    /// The id it asks for; a fresh URN when absent.
    pub id: Option<Urn>,
    /// Owner an in-process flow asks for; only honoured for the root.
    pub owner: Option<Owner>,
    /// Who else sees it; private by default.
    pub visibility: Option<Visibility>,
    /// Its catalog or dataset, as the kind requires.
    pub parent: Option<Urn>,
    pub description: ResourceGraph,
}

/// A new description for an existing resource, replacing the old one whole.
#[derive(Debug, Clone)]
pub struct ReplaceResourceCommand {
    pub description: ResourceGraph,
    /// The parent it names, which must be the one it has.
    pub parent: Option<Urn>,
    pub visibility: Option<Visibility>,
    /// The version the caller read (`If-Match`); a different stored one is a conflict.
    pub expected_version: Option<i64>,
}

/// What an import does with a resource that is already stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportMode {
    /// Replaces its description, one version on.
    #[default]
    Upsert,
    /// Refuses the import.
    Create,
}

/// A DCAT document to validate and load.
#[derive(Debug, Clone)]
pub struct ImportCatalogCommand {
    /// The profile whose shapes validate it (`dcat-ap`, `dcat3`).
    pub profile: String,
    pub mode: ImportMode,
    /// Validates and plans without writing.
    pub dry_run: bool,
    /// The catalog the document's resources go into; its own catalogs are then not stored.
    pub target: Option<Urn>,
    /// The shape groups to run; the profile's defaults when empty.
    pub checks: Vec<String>,
    pub visibility: Option<Visibility>,
}

/// New ODRL offer on a catalog resource.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewOdrlOfferCommand {
    pub id: Option<Urn>,
    /// Who else sees it; private by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    /// Owner an in-process flow asks for; only honoured for the root.
    #[serde(skip)]
    pub owner: Option<Owner>,
    pub odrl_offer: JsonLd<OdrlPolicyInfo>,
    pub entity_id: Urn,
    pub entity_type: CatalogEntityTypes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_template_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instantiation_parameters: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The parts it was composed from; set by composition, never by the API.
    #[serde(skip)]
    pub composition: Option<Value>,
    /// Template versions, `(id, version)`, the composition used.
    #[serde(skip)]
    pub references: Vec<(String, String)>,
}

/// A template version to store, its owner already resolved.
#[derive(Debug, Clone)]
pub struct NewPolicyTemplateCommand {
    pub id: Option<String>,
    pub owner: Owner,
    pub version: Option<String>,
    pub kind: PolicyTemplateKind,
    pub control_points: Vec<ControlPoint>,
    pub date: Option<DateTime<FixedOffset>>,
    pub author: Option<String>,
    pub title: Option<Value>,
    pub description: Option<Value>,
    pub content: Value,
    pub parameters: Value,
    /// Template versions the content references directly, as `(id, version)`.
    pub references: Vec<(String, String)>,
}

/// A schema version to store, owner resolved and content checked.
#[derive(Debug, Clone)]
pub struct NewSchemaVersionCommand {
    pub id: String,
    pub version: i32,
    pub owner: Owner,
    pub title: String,
    pub description: Option<String>,
    pub content: Value,
    pub sha256: String,
}
