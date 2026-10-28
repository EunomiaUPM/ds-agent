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

//! The consumer side over the well-known resolution, the peer tokens and the cache. What a peer
//! answers is read through RDF before it is kept: it must describe a catalog or a dataset.

use std::sync::Arc;

use common::dsp_common::rdf::DspProfile;
use common::facades::grants_facade::send_with_peer_token;
use common::rdf::{ExpandedDoc, RdfMessage};
use common::well_known::rpc::WellKnownRPCRequest;
use serde_json::{json, Map, Value};
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::services::client::ClientExt;
use ymir::types::gnap::grant_request::access::AccessNeed;
use ymir::types::oauth::UserInfo;
use ymir::utils::http_client;

use crate::entities::vocab::Vocab;
use crate::protocols::dsp::entities::message_types::CatalogDspMessageType;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspCatalog;
use crate::protocols::dsp::entities::rdf_extractor_rpc::EunomiaCatalog;
use crate::protocols::dsp::facades::well_known_rpc_facade::WellKnownRPCFacadeTrait;
use crate::protocols::dsp::services::rpc_catalog::views::RpcCatalogResult;
use crate::protocols::dsp::services::rpc_catalog::RpcCatalogServiceTrait;
use crate::protocols::dsp::services::validator::CatalogRpcRules;
use crate::setup::context::AppContext;

pub struct RpcCatalogService {
    ctx: Arc<AppContext>,
    well_known: Arc<dyn WellKnownRPCFacadeTrait>,
}

impl RpcCatalogService {
    pub fn new(ctx: Arc<AppContext>, well_known: Arc<dyn WellKnownRPCFacadeTrait>) -> Self {
        Self { ctx, well_known }
    }

    fn peer(request: &RdfMessage<EunomiaCatalog>) -> String {
        request.fields.associated_agent_peer.clone().unwrap_or_default()
    }

    /// The peer's current DSP endpoint, from its well-known document.
    async fn provider(&self, peer: &str) -> Outcome<String> {
        self.well_known
            .resolve_dataspace_current_path(&WellKnownRPCRequest {
                participant_id: peer.to_string(),
            })
            .await
    }

    /// Fails unless the peer's answer expands to a node of `class`.
    async fn describes(answer: &Value, class: &str) -> Outcome<()> {
        let expansion = DspProfile::shared().expand(answer).await?;
        let found = ExpandedDoc::new(&expansion.expanded)
            .is_some_and(|doc| doc.nodes().any(|node| node.has_type(class)));
        if found {
            Ok(())
        } else {
            Err(Errors::format(
                BadFormat::Sent,
                format!("the peer's answer describes no {class}"),
                None,
            ))
        }
    }
}

#[async_trait::async_trait]
impl RpcCatalogServiceTrait for RpcCatalogService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn request_catalog(
        &self,
        user: &UserInfo,
        request: &RdfMessage<EunomiaCatalog>,
    ) -> Outcome<RpcCatalogResult> {
        CatalogRpcRules::check(CatalogDspMessageType::CatalogRequestMessage, &request.fields)?;
        let peer = Self::peer(request);
        let no_cache = request.fields.no_cache;
        if !no_cache {
            if let Some(cached) = self.ctx.peer_catalogs().get(user, &peer).await? {
                return Ok(RpcCatalogResult::assemble(request.json.clone(), cached));
            }
        }
        let provider = self.provider(&peer).await?;
        let mut members = Map::new();
        if !request.fields.filter.is_empty() {
            members.insert("filter".into(), json!(request.fields.filter));
        }
        let message =
            RdfMessage::<DspCatalog>::build(&CatalogDspMessageType::CatalogRequestMessage, members).await?;
        let url = format!("{provider}/catalog/request");
        let need = AccessNeed::of_call(&url, None, false);
        let answer: Value = send_with_peer_token(
            self.ctx.ports.auth.grants.as_ref(),
            user,
            &peer,
            &need,
            true,
            |headers| http_client().post_json::<Value, Value>(&url, headers, &message.json),
        )
        .await?;
        Self::describes(&answer, Vocab::DCAT_CATALOG).await?;
        if !no_cache {
            self.ctx.peer_catalogs().set(user, &peer, &answer).await?;
        }
        Ok(RpcCatalogResult::assemble(request.json.clone(), answer))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn request_dataset(
        &self,
        user: &UserInfo,
        request: &RdfMessage<EunomiaCatalog>,
    ) -> Outcome<RpcCatalogResult> {
        CatalogRpcRules::check(CatalogDspMessageType::DatasetRequestMessage, &request.fields)?;
        let peer = Self::peer(request);
        let dataset = request.fields.dataset.clone().unwrap_or_default();
        let provider = self.provider(&peer).await?;
        let url = format!("{provider}/catalog/datasets/{dataset}");
        let need = AccessNeed::of_call(&url, Some(&dataset), false);
        let answer: Value = send_with_peer_token(
            self.ctx.ports.auth.grants.as_ref(),
            user,
            &peer,
            &need,
            true,
            |headers| http_client().get_json::<Value>(&url, headers),
        )
        .await?;
        Self::describes(&answer, Vocab::DCAT_DATASET).await?;
        Ok(RpcCatalogResult::assemble(request.json.clone(), answer))
    }
}
