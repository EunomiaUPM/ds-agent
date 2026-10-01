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

use crate::data::config::ConfigPassthroughRepo;
use crate::data::factory::DataFactory;
use crate::data::repo::config::KeystoreConfigRepo;
use crate::data::repo::parameters::ParameterRepoTrait;
use crate::data::repo::secrets::SecretRepoTrait;
use crate::data::sea_orm::repos::parameter::SeaOrmParameterRepo;
use crate::data::sea_orm::repos::secret::SeaOrmSecretRepo;
use common::config::ApplicationConfig;
use sea_orm::DatabaseConnection;
use std::ops::Deref;
use std::sync::Arc;

#[allow(dead_code)]
pub(crate) struct SeaOrmFactory {
    config: Arc<ApplicationConfig>,
    db: Arc<DatabaseConnection>,
}

#[allow(dead_code)]
impl SeaOrmFactory {
    pub fn new(db: Arc<DatabaseConnection>, config: Arc<ApplicationConfig>) -> Self {
        Self { db, config }
    }
}

impl DataFactory for SeaOrmFactory {
    fn keystore_config_repo(&self) -> Arc<dyn KeystoreConfigRepo> {
        Arc::new(ConfigPassthroughRepo::new(self.config.clone()))
    }

    fn keystore_secrets_repo(&self) -> Arc<dyn SecretRepoTrait> {
        Arc::new(SeaOrmSecretRepo::new(self.db.deref().clone()))
    }

    fn keystore_parameters_repo(&self) -> Arc<dyn ParameterRepoTrait<Value = serde_json::Value>> {
        Arc::new(SeaOrmParameterRepo::new(self.db.deref().clone()))
    }
}
