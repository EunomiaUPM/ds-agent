/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use async_trait::async_trait;
use rainbow_common::batch_requests::BatchRequests;
use rainbow_common::mates::mates::VerifyTokenRequest;
use ymir::data::entities::mates::Model;

use crate::ssi::services::repo::repo_trait::AuthRepoTrait;

#[async_trait]
pub trait CoreMateTrait: Send + Sync + 'static {
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;

    async fn get_all(&self) -> anyhow::Result<Vec<Model>> {
        self.repo().mates().get_all(None, None).await
    }

    async fn get_by_id(&self, id: String) -> anyhow::Result<Model> {
        self.repo().mates().get_by_id(&id).await
    }

    async fn get_me(&self) -> anyhow::Result<Model> {
        self.repo().mates().get_me().await
    }

    async fn get_mate_batch(&self, payload: BatchRequests) -> anyhow::Result<Vec<Model>> {
        self.repo().mates().get_batch(&payload.ids).await
    }

    async fn get_by_token(&self, payload: VerifyTokenRequest) -> anyhow::Result<Model> {
        self.repo().mates().get_by_token(&payload.token).await
    }
}
