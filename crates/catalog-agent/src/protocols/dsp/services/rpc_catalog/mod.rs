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

//! Asking a peer's catalog on a user's behalf (the consumer side of DSP 6.2).

pub mod service;
pub mod views;

use common::rdf::RdfMessage;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::protocols::dsp::entities::rdf_extractor_rpc::EunomiaCatalog;
use crate::protocols::dsp::services::rpc_catalog::views::RpcCatalogResult;

/// The consumer side of the catalog protocol.
#[mockall::automock]
#[async_trait::async_trait]
pub trait RpcCatalogServiceTrait: Send + Sync {
    /// The peer's catalog, from the cache unless the request says `noCache`.
    async fn request_catalog(
        &self,
        user: &UserInfo,
        request: &RdfMessage<EunomiaCatalog>,
    ) -> Outcome<RpcCatalogResult>;
    /// One dataset of the peer, always asked.
    async fn request_dataset(
        &self,
        user: &UserInfo,
        request: &RdfMessage<EunomiaCatalog>,
    ) -> Outcome<RpcCatalogResult>;
}
