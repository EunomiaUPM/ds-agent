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

use axum::Router;
use common::config::types::traits::CommonConfigTrait;
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

use crate::data::sea_orm::repos::parameter::SeaOrmParameterRepo;
use crate::data::sea_orm::repos::secret::SeaOrmSecretRepo;
use crate::http::KeystoreRouter;
use crate::services::parameters::service::ParameterStoreImpl;
use crate::services::secrets::service::SecretStoreImpl;

pub struct KeystoreSetup;

impl KeystoreSetup {
    pub fn new() -> Self {
        Self
    }

    pub async fn build_keystore_router<C>(&self, config: &C, vault: Arc<VaultService>) -> Router
    where
        C: CommonConfigTrait + Send + Sync,
    {
        let db = vault.get_db_connection(config.common()).await;

        let parameter_repo = Arc::new(SeaOrmParameterRepo::new(db.clone()));
        let secret_repo = Arc::new(SeaOrmSecretRepo::new(db.clone()));

        let parameter_service = Arc::new(ParameterStoreImpl::new(parameter_repo));
        let secret_service = Arc::new(SecretStoreImpl::new(secret_repo));

        KeystoreRouter::new(parameter_service, secret_service).router()
    }
}
