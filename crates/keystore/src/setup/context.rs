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

//! Wires the keystore stores once: parameters in the DB, secrets in Vault when there is one.

use std::sync::Arc;

use crate::data::config::ConfigPassthroughRepo;
use crate::data::repo::secrets::SecretRepoTrait;
use crate::data::sea_orm::repos::parameter::SeaOrmParameterRepo;
use crate::data::sea_orm::repos::secret::SeaOrmSecretRepo;
use crate::data::vault::VaultSecretRepo;
use crate::services::config::config::ConfigStoreImpl;
use crate::services::parameters::ParameterStore;
use crate::services::parameters::service::ParameterStoreImpl;
use crate::services::secrets::SecretStore;
use crate::services::secrets::service::SecretStoreImpl;
use common::auth::OauthTokenValidator;
use common::config::ApplicationConfig;
use common::module_loader::root_context::RootContext;
use ymir::services::vault::VaultService;

#[derive(Clone)]
pub(crate) struct AppContext {
    pub parameter_svc: Arc<dyn ParameterStore<serde_json::Value>>,
    pub secret_svc: Arc<dyn SecretStore>,
    pub config_svc: Arc<ConfigStoreImpl>,
    pub oauth_validator: Arc<dyn OauthTokenValidator>,
}

impl AppContext {
    pub fn build(
        config: &ApplicationConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Self {
        let (parameter_svc, secret_svc) = Self::build_stores(root, event_bus);
        let config_svc = Arc::new(ConfigStoreImpl::new(Arc::new(ConfigPassthroughRepo::new(
            Arc::new(config.clone()),
        ))));
        Self {
            parameter_svc,
            secret_svc,
            config_svc,
            oauth_validator: root.validator.clone(),
        }
    }

    pub fn build_stores(
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> (
        Arc<dyn ParameterStore<serde_json::Value>>,
        Arc<dyn SecretStore>,
    ) {
        let db = root.db.clone();
        let parameter_repo = Arc::new(SeaOrmParameterRepo::new(db.clone()));
        let secret_repo: Arc<dyn SecretRepoTrait> = match &*root.vault {
            VaultService::Real(_) => Arc::new(VaultSecretRepo::new(
                root.vault.clone(),
                Arc::new(SeaOrmSecretRepo::new(db)),
            )),
            VaultService::Fake(_) => Arc::new(SeaOrmSecretRepo::new(db)),
        };
        (
            Arc::new(ParameterStoreImpl::new(parameter_repo).with_event_bus(event_bus.clone())),
            Arc::new(SecretStoreImpl::new(secret_repo).with_event_bus(event_bus)),
        )
    }
}
