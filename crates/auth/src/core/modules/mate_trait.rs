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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::Arc;

use crate::services::repo::repo_trait::AuthRepoTrait;
use async_trait::async_trait;
use common::batch_requests::BatchRequests;
use common::facades::VerifyTokenRequest;
use json_value_merge::Merge;
use serde::{Deserialize, Deserializer};
use ymir::data::entities::shared::participant::{Model, Plan};
use ymir::errors::Outcome;

#[derive(Debug, PartialEq)]
pub enum MateRouterGetAllQueryParamsType {
    Agents,
    Authorities,
    All,
    Other(String),
}

impl<'de> Deserialize<'de> for MateRouterGetAllQueryParamsType {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.as_str() {
            "agents" => Self::Agents,
            "authorities" => Self::Authorities,
            "all" => Self::All,
            other => Self::Other(other.to_string()),
        })
    }
}

#[async_trait]
pub trait CoreMateTrait: Send + Sync + 'static {
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;

    async fn get_all(
        &self,
        query_type: &MateRouterGetAllQueryParamsType,
        exclude: &bool,
    ) -> Outcome<Vec<Model>> {
        let mates = self.repo().mates().get_all(None, None).await?;
        let filtered_in_mates = mates
            .into_iter()
            .filter(|mate| !*exclude || !mate.is_me)
            .filter(|mate| match query_type {
                MateRouterGetAllQueryParamsType::Authorities => {
                    mate.participant_type == "Authority".to_string()
                }
                MateRouterGetAllQueryParamsType::Agents => {
                    mate.participant_type == "Agent".to_string()
                }
                _ => true,
            })
            .collect();
        Ok(filtered_in_mates)
    }

    async fn get_by_id(&self, id: String) -> Outcome<Model> {
        self.repo().mates().get_by_id(&id).await
    }

    async fn get_me(&self) -> Outcome<Model> {
        self.repo().mates().get_me().await
    }

    async fn get_mate_batch(&self, payload: BatchRequests) -> Outcome<Vec<Model>> {
        self.repo().mates().get_batch(&payload.ids).await
    }

    async fn get_by_token(&self, payload: VerifyTokenRequest) -> Outcome<Model> {
        self.repo().mates().get_by_token(&payload.token).await
    }
    async fn update_extra_fields_by_id(
        &self,
        id: String,
        extra_fields: serde_json::Value,
    ) -> Outcome<Model> {
        let mut mate = self.repo().mates().get_by_id(&id).await?;
        let mut merged_extra_fields = mate.extra_fields.clone();
        merged_extra_fields.merge(&extra_fields);
        mate.extra_fields = merged_extra_fields;
        self.repo().mates().update(mate).await
    }

    async fn create_mate(&self, payload: &NewModel) -> Outcome<Model> {
        self.repo().mates().create(payload.clone()).await
    }
}
