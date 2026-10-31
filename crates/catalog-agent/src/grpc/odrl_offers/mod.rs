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

//! The offer RPCs over the offer service.

use std::sync::Arc;

use ymir::grpc::{IntoStatus, PageParams, ProtoField};
use ymir::oauth::grpc::GrpcAuth;
use tonic::{Request, Response, Status};
use ymir::errors::Errors;

use crate::entities::commands::NewOdrlOfferCommand;
use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::policy_composition::{NewPolicyCompositionDto, PolicyCompositionPreviewDto};
use crate::grpc::api::catalog_agent::odrl_offer_service_server::OdrlOfferService;
use crate::grpc::api::catalog_agent::{IdRequest, JsonDocument, ListJsonRequest};
use crate::grpc::GrpcJson;
use crate::services::odrl_policies::views::OdrlPolicyView;
use crate::setup::context::AppContext;

pub struct OdrlOfferGrpc {
    ctx: Arc<AppContext>,
    auth: GrpcAuth,
}

impl OdrlOfferGrpc {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            auth: GrpcAuth::new(ctx.oauth_validator.clone()),
            ctx,
        }
    }
}

#[tonic::async_trait]
impl OdrlOfferService for OdrlOfferGrpc {
    async fn list(&self, request: Request<ListJsonRequest>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let request = request.into_inner();
        let filter: OdrlPolicyFilter = match request.filter.as_deref().filter(|f| !f.is_empty()) {
            Some(json) => GrpcJson::parse(json, "filter")?,
            None => OdrlPolicyFilter::default(),
        };
        let (page, sort) = PageParams::from_proto(
            request.limit.unwrap_or(0),
            request.cursor.as_deref().unwrap_or_default(),
            request.sort.as_deref().unwrap_or_default(),
        )?;
        let found = self
            .ctx
            .offers()
            .list(&user, &filter, &page, &sort)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&found.map(OdrlPolicyView::assemble))?))
    }

    async fn get(&self, request: Request<IdRequest>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        let offer = self.ctx.offers().get(&user, &id).await.map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&OdrlPolicyView::assemble(offer))?))
    }

    async fn by_entity(&self, request: Request<IdRequest>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let entity = request.into_inner().id.urn("id")?;
        let offers = self
            .ctx
            .offers()
            .by_entities(&user, std::slice::from_ref(&entity))
            .await
            .map_err(Errors::into_status)?;
        let views: Vec<OdrlPolicyView> = offers.into_iter().map(OdrlPolicyView::assemble).collect();
        Ok(Response::new(GrpcJson::document(&views)?))
    }

    async fn create(&self, request: Request<JsonDocument>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let command: NewOdrlOfferCommand = GrpcJson::parse(&request.into_inner().json, "json")?;
        let offer = self.ctx.offers().create(&user, &command).await.map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&OdrlPolicyView::assemble(offer))?))
    }

    async fn compose(&self, request: Request<JsonDocument>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let composition: NewPolicyCompositionDto = GrpcJson::parse(&request.into_inner().json, "json")?;
        let offer = self
            .ctx
            .composition()
            .compose(&user, &composition)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&OdrlPolicyView::assemble(offer))?))
    }

    async fn compose_preview(&self, request: Request<JsonDocument>) -> Result<Response<JsonDocument>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let preview: PolicyCompositionPreviewDto = GrpcJson::parse(&request.into_inner().json, "json")?;
        let found = self
            .ctx
            .composition()
            .preview(&user, &preview.parts)
            .await
            .map_err(Errors::into_status)?;
        Ok(Response::new(GrpcJson::document(&found)?))
    }

    async fn delete(&self, request: Request<IdRequest>) -> Result<Response<()>, Status> {
        let user = self.auth.user(request.metadata()).await?;
        let id = request.into_inner().id.urn("id")?;
        self.ctx.offers().delete(&user, &id).await.map_err(Errors::into_status)?;
        Ok(Response::new(()))
    }
}
