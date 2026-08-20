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
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::ApiConfigTrait;
use ymir::services::vault::VaultService;
use ymir::services::vault::VaultTrait;

pub struct KeystoreModule {
    prefix: String,
    router: Router,
}

impl KeystoreModule {
    pub async fn build_stores<C>(
        config: &C,
        vault: Arc<VaultService>,
    ) -> (
        Arc<dyn ParameterStore<serde_json::Value>>,
        Arc<dyn SecretStore>,
    )
    where
        C: CommonConfigTrait + Send + Sync,
    {
        let db = vault
            .get_db_connection(config.common())
            .await
            .expect("Unable to retrieve db connection");
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

    pub async fn build<C>(
        config: &C,
        app_config: Arc<ApplicationConfig>,
        vault: Arc<VaultService>,
    ) -> Self
    where
        C: CommonConfigTrait + Send + Sync,
    {
        let prefix = format!("{}/keystore", config.common().get_api_version());
        let (parameter_service, secret_service) = Self::build_stores(config, vault).await;
        let config_service =
            Arc::new(ConfigStoreImpl::new(Arc::new(ConfigPassthroughRepo::new(app_config))));

        let router =
            KeystoreRouter::new(parameter_service, secret_service, config_service).router();
        Self { prefix, router }
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::get_keystore_migrations()
    }
}

impl ServiceModuleTrait for KeystoreModule {
    fn name(&self) -> &'static str {
        "keystore"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((self.prefix.clone(), self.router.clone()))
    }
}
