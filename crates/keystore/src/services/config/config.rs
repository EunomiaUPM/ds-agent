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

use crate::data::repo::config::KeystoreConfigRepo;
use crate::services::config::ConfigStore;
use common::config::ApplicationConfig;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct ConfigStoreImpl {
    repo: Arc<dyn KeystoreConfigRepo>,
}

impl ConfigStoreImpl {
    pub fn new(repo: Arc<dyn KeystoreConfigRepo>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl ConfigStore for ConfigStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_application_config(&self) -> Outcome<ApplicationConfig> {
        self.repo.get_config().await
    }
}
