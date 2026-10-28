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

//! Answering peers: what the connector publishes, the same for every peer; owners and
//! visibility only rule inside (DSP 6.2).

pub mod service;

use serde_json::Value;
use urn::Urn;
use ymir::errors::Outcome;

use crate::protocols::dsp::entities::protocol_fields::CatalogProtocolFields;

/// The provider side of the catalog protocol; `peer` is the participant asking, for the logs.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DspCatalogServiceTrait: Send + Sync {
    /// The main catalog with every published catalog, dataset and offer (DSP 6.2.1).
    async fn catalog(&self, peer: &str, request: &CatalogProtocolFields) -> Outcome<Value>;
    /// One published dataset, with its offers and distributions (DSP 6.2.2).
    async fn dataset(&self, peer: &str, id: &Urn) -> Outcome<Value>;
}
