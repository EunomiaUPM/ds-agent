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

use std::sync::Arc;

use crate::data::config::ConfigPassthroughRepo;
use crate::data::repo::secrets::SecretRepoTrait;
use crate::data::sea_orm::repos::parameter::SeaOrmParameterRepo;
use crate::data::sea_orm::repos::secret::SeaOrmSecretRepo;
use crate::data::vault::VaultSecretRepo;
use crate::http::KeystoreRouter;
use crate::services::config::config::ConfigStoreImpl;
use crate::services::parameters::ParameterStore;
use crate::services::parameters::service::ParameterStoreImpl;
use crate::services::secrets::SecretStore;
use crate::services::secrets::service::SecretStoreImpl;
use axum::Router;
use common::config::ApplicationConfig;
use common::config::types::traits::CommonConfigTrait;
use ymir::services::vault::VaultService;
use ymir::services::vault::VaultTrait;

pub struct KeystoreSetup;

impl KeystoreSetup {
    pub fn new() -> Self {
        Self
    }

    pub async fn build_keystore_stores<C>(
        &self,
        config: &C,
        vault: Arc<VaultService>,
    ) -> (
        Arc<dyn ParameterStore<serde_json::Value>>,
        Arc<dyn SecretStore>,
    )
    where
        C: CommonConfigTrait + Send + Sync,
    {
        let db = vault.get_db_connection(config.common()).await.unwrap();
        let parameter_repo = Arc::new(SeaOrmParameterRepo::new(db.clone()));
        let secret_repo: Arc<dyn SecretRepoTrait> = match &*vault {
            VaultService::Real(_) => Arc::new(VaultSecretRepo::new(
                vault.clone(),
                Arc::new(SeaOrmSecretRepo::new(db.clone())),
            )),
            VaultService::Fake(_) => Arc::new(SeaOrmSecretRepo::new(db.clone())),
        };
        (
            Arc::new(ParameterStoreImpl::new(parameter_repo)),
            Arc::new(SecretStoreImpl::new(secret_repo)),
        )
    }

    pub async fn build_keystore_router<C>(
        &self,
        config: &C,
        app_config: Arc<ApplicationConfig>,
        vault: Arc<VaultService>,
    ) -> Router
    where
        C: CommonConfigTrait + Send + Sync,
    {
        let db = vault.get_db_connection(config.common()).await.expect("Unable to retrieve db connection");

        let config_repo = Arc::new(ConfigPassthroughRepo::new(app_config));
        let parameter_repo = Arc::new(SeaOrmParameterRepo::new(db.clone()));
        let secret_repo: Arc<dyn SecretRepoTrait> = match &*vault {
            VaultService::Real(_) => Arc::new(VaultSecretRepo::new(
                vault.clone(),
                Arc::new(SeaOrmSecretRepo::new(db.clone())),
            )),
            VaultService::Fake(_) => Arc::new(SeaOrmSecretRepo::new(db.clone())),
        };

        let config_service = Arc::new(ConfigStoreImpl::new(config_repo));
        let parameter_service = Arc::new(ParameterStoreImpl::new(parameter_repo));
        let secret_service = Arc::new(SecretStoreImpl::new(secret_repo));

        KeystoreRouter::new(parameter_service, secret_service, config_service).router()
    }
}
