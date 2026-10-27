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

//! What a DSP call needs from a GNAP token, and whether a granted `access` covers it (GNAP4DS).

use serde::{Deserialize, Serialize};
use ymir::types::gnap::grant_request::access::{AccessType, ResourceAccess};

const DATASETS: &str = "/catalog/datasets";
const OFFERS: &str = "/negotiations/offers";

/// What a DSP call needs, with `base` the DSP base of the participant that receives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AccessNeed {
    Catalog { base: String },
    Dataset { base: String, id: String },
    /// A message on a process already open: any token of the grant, then the counterparty rule.
    Process { base: String },
    Callback { base: String },
    Offer { base: String },
}

impl AccessNeed {
    /// The need of a DSP call to `url`, from its path; `dataset` for a dataset request or N1.
    pub fn of_call(url: &str, dataset: Option<&str>, answering: bool) -> Self {
        let base = dsp_base(url);
        if url.ends_with(OFFERS) {
            return Self::Offer { base };
        }
        if answering {
            return Self::Callback { base };
        }
        match dataset {
            Some(id) => Self::Dataset { base, id: id.to_string() },
            None if url.contains("/catalog/") => Self::Catalog { base },
            None => Self::Process { base },
        }
    }

    /// The `access` to ask for when no grant covers this need.
    pub fn to_request(&self) -> Vec<ResourceAccess> {
        match self {
            Self::Catalog { base } | Self::Dataset { base, .. } => {
                vec![ResourceAccess::datasets_of(format!("{base}{DATASETS}"))]
            }
            Self::Process { base } | Self::Callback { base } => {
                vec![ResourceAccess::callback(base.clone())]
            }
            Self::Offer { base } => vec![
                ResourceAccess::offer(format!("{base}{OFFERS}")),
                ResourceAccess::callback(base.clone()),
            ],
        }
    }

    /// Whether `granted` covers this need.
    pub fn covered_by(&self, granted: &[ResourceAccess]) -> bool {
        let located = |r#type: AccessType, url: &str| {
            granted.iter().any(|access| {
                access.r#type == r#type
                    && access
                        .locations
                        .iter()
                        .flatten()
                        .any(|location| location_matches(location, url))
            })
        };
        match self {
            Self::Catalog { base } => {
                located(AccessType::Catalog, base)
                    || located(AccessType::Dataset, &format!("{base}{DATASETS}"))
            }
            Self::Dataset { base, id } => {
                granted.iter().any(|access| {
                    access.r#type == AccessType::Dataset
                        && access.identifier.as_deref() == Some(id.as_str())
                }) || located(AccessType::Dataset, &format!("{base}{DATASETS}"))
            }
            Self::Process { .. } => !granted.is_empty(),
            Self::Callback { base } => located(AccessType::Callback, base),
            Self::Offer { base } => located(AccessType::Offer, &format!("{base}{OFFERS}")),
        }
    }
}

/// The DSP base of `url`: what comes before its catalog, negotiation or transfer path.
pub fn dsp_base(url: &str) -> String {
    ["/catalog", "/negotiations", "/transfers"]
        .iter()
        .filter_map(|segment| url.find(segment))
        .min()
        .map_or(url, |at| &url[..at])
        .trim_end_matches('/')
        .to_string()
}

/// Whether `url` lies under `location` (GNAP4DS 8.1), comparing paths only.
pub fn location_matches(location: &str, url: &str) -> bool {
    let location = path_of(location).trim_end_matches('/');
    let url = path_of(url).trim_end_matches('/');
    url == location || url.starts_with(&format!("{location}/"))
}

fn path_of(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    if url.contains("://") {
        rest.find('/').map_or("", |at| &rest[at..])
    } else {
        rest
    }
}
