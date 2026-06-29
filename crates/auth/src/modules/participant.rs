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

use crate::services::HasRepo;
use async_trait::async_trait;
use common::batch_requests::BatchRequestsAsString;
use common::facades::VerifyTokenRequest;
use json_value_merge::Merge;
use ymir::data::entities::shared::participant::{Model, Plan};
use ymir::errors::Outcome;
use ymir::services::HasWallet;
use ymir::types::participants::ParticipantType;

#[async_trait]
pub trait ParticipantModule: HasWallet + HasRepo + Send + Sync + 'static {
    async fn get_all(&self, query_type: ParticipantType, exclude: bool) -> Outcome<Vec<Model>> {
        let mates = self.repo().participant().filter_by_type(query_type).await?;
        let filtered_in_mates = mates
            .into_iter()
            .filter(|mate| !exclude || !mate.is_me)
            .collect();
        Ok(filtered_in_mates)
    }

    async fn get_by_id(&self, id: String) -> Outcome<Model> {
        self.repo().participant().get_by_id(&id).await
    }

    async fn get_me(&self) -> Outcome<Model> {
        self.repo().participant().get_me().await
    }

    async fn get_participant_batch(&self, payload: BatchRequestsAsString) -> Outcome<Vec<Model>> {
        self.repo().participant().get_batch(&payload.ids).await
    }

    async fn get_by_token(&self, payload: VerifyTokenRequest) -> Outcome<Model> {
        self.repo().participant().get_by_token(&payload.token).await
    }
    async fn update_extra_fields_by_id(
        &self,
        id: String,
        extra_fields: serde_json::Value,
    ) -> Outcome<Model> {
        let mut mate = self.repo().participant().get_by_id(&id).await?;
        let mut merged_extra_fields = mate.extra_fields.clone();
        merged_extra_fields.merge(&extra_fields);
        mate.extra_fields = merged_extra_fields;
        self.repo().participant().update(mate).await
    }

    async fn create_participant(&self, payload: Plan) -> Outcome<Model> {
        self.repo().participant().create(payload).await
    }

    async fn update_myself(&self) -> Outcome<Model> {
        let mut model = self.repo().participant().get_me().await?;
        let lock = self.wallet().get_identity();
        let identity = lock.read().await;
        model.participant_id = identity.did().id().to_string();
        self.repo().participant().update(model).await
    }
}
