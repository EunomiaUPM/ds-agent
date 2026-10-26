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

//! What DSP 2025-1 asks of a catalog, as rules over the effective graph and as the published
//! JSON Schemas over the projection.

use common::validation::{Rule, SchemaRule};
use serde_json::Value;

use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;
use crate::profiles::shared::rules::{ClassRules, Expect, PropertyRule, RuleSet};
use crate::profiles::{Finding, Severity};

/// The DSP rules and schemas.
pub struct DspRules {
    catalog_schema: SchemaRule,
    dataset_schema: SchemaRule,
}

impl DspRules {
    const CATALOG_SCHEMA: &'static str = include_str!("../../../dsp_schema/catalog-schema.json");
    const DATASET_SCHEMA: &'static str = include_str!("../../../dsp_schema/dataset-schema.json");
    const CONTRACT_SCHEMA: &'static str = include_str!("../../../dsp_schema/contract-schema.json");
    const CONTEXT_SCHEMA: &'static str = include_str!("../../../dsp_schema/context-schema.json");
    const DATASET_SCHEMA_ID: &'static str = "https://w3id.org/dspace/2025/1/catalog/dataset-schema.json";
    const CONTRACT_SCHEMA_ID: &'static str = "https://w3id.org/dspace/2025/1/negotiation/contract-schema.json";
    const CONTEXT_SCHEMA_ID: &'static str = "https://w3id.org/dspace/2025/1/common/context-schema.json";

    /// The schemas are embedded, so a failure to compile them is a build defect.
    pub fn new() -> Self {
        let resources = [
            (Self::DATASET_SCHEMA_ID, Self::DATASET_SCHEMA),
            (Self::CONTRACT_SCHEMA_ID, Self::CONTRACT_SCHEMA),
            (Self::CONTEXT_SCHEMA_ID, Self::CONTEXT_SCHEMA),
        ];
        Self {
            catalog_schema: SchemaRule::with_resources(Self::CATALOG_SCHEMA, &resources)
                .expect("embedded DSP catalog schema compiles"),
            dataset_schema: SchemaRule::with_resources(Self::DATASET_SCHEMA, &resources[1..])
                .expect("embedded DSP dataset schema compiles"),
        }
    }

    /// The rules over each class's effective graph.
    pub fn rule_set() -> RuleSet {
        RuleSet {
            classes: vec![
                ClassRules {
                    kind: ResourceKind::Dataset,
                    properties: vec![
                        PropertyRule::mandatory(Vocab::ODRL_HAS_POLICY, Expect::Iri),
                        PropertyRule::mandatory(Vocab::DCAT_DISTRIBUTION_P, Expect::Iri),
                    ],
                },
                ClassRules {
                    kind: ResourceKind::Distribution,
                    properties: vec![
                        PropertyRule::mandatory(Vocab::DCT_FORMAT, Expect::Any).single(),
                        PropertyRule::mandatory(Vocab::DCAT_ACCESS_SERVICE, Expect::Iri).single(),
                    ],
                },
                ClassRules {
                    kind: ResourceKind::DataService,
                    properties: vec![PropertyRule::mandatory(Vocab::DCAT_ENDPOINT_URL, Expect::Iri).single()],
                },
            ],
            language_tags: None,
        }
    }

    /// A projected root catalog or dataset against its schema; other kinds have none.
    pub fn check_schema(&self, kind: ResourceKind, resource: &str, projection: &Value) -> Vec<Finding> {
        let schema = match kind {
            ResourceKind::Catalog => &self.catalog_schema,
            ResourceKind::Dataset => &self.dataset_schema,
            _ => return vec![],
        };
        match schema.check(projection) {
            Ok(()) => vec![],
            Err(violations) => violations
                .iter()
                .map(|v| Finding {
                    severity: Severity::Violation,
                    resource: resource.to_string(),
                    property: Some(v.path.to_string()),
                    message: format!("DSP schema: {}", v.message),
                })
                .collect(),
        }
    }
}

impl Default for DspRules {
    fn default() -> Self {
        Self::new()
    }
}
