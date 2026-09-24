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

use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::rpc::persistence::OrchestrationPersistenceForProtocolForRPC;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcCatalogMessageTrait, RpcCatalogRequestMessageDto, RpcCatalogResponseMessageDto,
    RpcDatasetRequestMessageDto,
};
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use crate::protocols::dsp::protocol_types::{
    CatalogMessageWrapper, CatalogRequestMessageDto, DatasetRequestMessage,
};
use crate::protocols::dsp::types::catalog_definition::Catalog;
use crate::protocols::dsp::types::dataset_definition::Dataset;
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use axum::http::HeaderMap;
use common::auth::AccessScope;
use common::errors::{CommonErrors, ErrorLog};
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::well_known::rpc::WellKnownRPCRequest;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::error;
use ymir::errors::{Errors, Outcome};
use ymir::services::client::ClientExt;
use ymir::types::http::{HttpBody, Method};
use ymir::utils::{bearer_headers, http_client};

pub struct RPCOrchestratorService {
    validator: Arc<dyn ValidationRpcSteps>,
    facades: Arc<dyn FacadeTrait>,
    persistence: Arc<OrchestrationPersistenceForProtocolForRPC>,
    mates_facade: Arc<dyn MatesFacadeTrait>,
}

impl RPCOrchestratorService {
    pub fn new(
        validator: Arc<dyn ValidationRpcSteps>,
        facades: Arc<dyn FacadeTrait>,
        persistence: Arc<OrchestrationPersistenceForProtocolForRPC>,
        mates_facade: Arc<dyn MatesFacadeTrait>,
    ) -> RPCOrchestratorService {
        Self {
            validator,
            facades,
            persistence,
            mates_facade,
        }
    }

    /// Bearer headers for the peer's token, if the tenant holds one for that mate.
    async fn peer_headers(&self, scope: &AccessScope, peer: String) -> Outcome<Option<HeaderMap>> {
        match self
            .mates_facade
            .get_mate_by_id(scope.acting_tenant().clone(), peer)
            .await
        {
            Ok(mate) => mate.token.as_deref().map(bearer_headers).transpose(),
            Err(_) => Ok(None),
        }
    }
}

#[async_trait::async_trait]
impl RPCOrchestratorTrait for RPCOrchestratorService {
    async fn setup_catalog_request_rpc(
        &self,
        scope: &AccessScope,
        input: &RpcCatalogRequestMessageDto,
    ) -> Outcome<RpcCatalogResponseMessageDto<RpcCatalogRequestMessageDto, Catalog>> {
        // agent_peer
        let agent_peer = input
            .get_associated_agent_peer()
            .ok_or(Errors::crazy("No associated agent", None))?;

        // validation
        self.validator.on_catalog_request(input).await?;

        if input.no_cache == false {
            // hit caché and return guard
            let catalog_in_cache = self.persistence.get_catalog(scope, &agent_peer).await?;
            if let Some(catalog) = catalog_in_cache {
                let response = RpcCatalogResponseMessageDto {
                    request: input.clone(),
                    response: catalog,
                };
                return Ok(response);
            }
        }

        // send message to peer
        // resolve path
        let participant_id = input
            .get_associated_agent_peer()
            .ok_or(Errors::crazy("No associated agent", None))?;
        let provider_address = self
            .facades
            .get_catalog_rpc_path_facade()
            .await
            .resolve_dataspace_current_path(&WellKnownRPCRequest {
                tenant_id: scope.acting_tenant().clone(),
                participant_id,
            })
            .await?;

        // send dsp message to peer to fetch catalog
        let peer_url = format!("{}/catalog/request", provider_address);
        let request_body: CatalogMessageWrapper<CatalogRequestMessageDto> = input.clone().into();
        let headers = self.peer_headers(scope, agent_peer.clone()).await?;
        let response = http_client()
            .post_json::<CatalogMessageWrapper<CatalogRequestMessageDto>, Catalog>(
                peer_url.as_str(),
                headers,
                &request_body,
            )
            .await?;

        if input.no_cache == false {
            // hydrate cache
            let _ = self
                .persistence
                .set_catalog(scope, &agent_peer, &response)
                .await?;
        }

        // return response
        let response = RpcCatalogResponseMessageDto {
            request: input.clone(),
            response,
        };
        Ok(response)
    }

    async fn setup_dataset_request_rpc(
        &self,
        scope: &AccessScope,
        input: &RpcDatasetRequestMessageDto,
    ) -> Outcome<RpcCatalogResponseMessageDto<RpcDatasetRequestMessageDto, Dataset>> {
        // validation
        self.validator.on_dataset_request(input).await?;

        let participant_id = input
            .get_associated_agent_peer()
            .ok_or(Errors::crazy("No associated agent", None))?;
        let provider_address = self
            .facades
            .get_catalog_rpc_path_facade()
            .await
            .resolve_dataspace_current_path(&WellKnownRPCRequest {
                tenant_id: scope.acting_tenant().clone(),
                participant_id,
            })
            .await?;
        let dataset = input.get_dataset_id().unwrap_or("".to_string());
        let peer_url = format!("{}/catalog/datasets/{}", provider_address, dataset);
        let request_body: CatalogMessageWrapper<DatasetRequestMessage> = input.clone().into();
        let peer_id = input.get_associated_agent_peer().unwrap_or_default();
        let headers = self.peer_headers(scope, peer_id).await?;
        let response: Dataset = http_client()
            .send_json(
                Method::GET,
                peer_url.as_str(),
                headers,
                HttpBody::json(&request_body)?,
            )
            .await?;

        let response = RpcCatalogResponseMessageDto {
            request: input.clone(),
            response,
        };
        Ok(response)
    }
}
