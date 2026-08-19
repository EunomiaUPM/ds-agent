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

use common::auth::middleware::OauthTokenValidator;
use common::config::services::TransferConfig;
use common::config::services::traits::TransferConfigTrait;
use common::config::types::traits::CommonConfigTrait;
use common::facades::ssi_auth_facade::ssi_auth_facade::SSIAuthFacadeService;
use common::http_client::HttpClient;
use oauth::setup::OAuthSetup;
use sea_orm::DatabaseConnection;
use ymir::errors::Outcome;
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;

/// Shared infrastructure and domain services, provisioned once and handed to
/// every module. Building modules reads from here instead of threading a fresh
/// bag of dependencies through each constructor — add a field once, use it in
/// as many modules as you like.
pub(crate) struct AppContext {
    pub config: Arc<TransferConfig>,
    pub db: DatabaseConnection,
    pub transfer_process_svc: Arc<TransferProcessService>,
    pub transfer_message_svc: Arc<TransferMessageService>,
    pub oauth_validator: Arc<dyn OauthTokenValidator>,
    pub ssi_auth_facade: Arc<SSIAuthFacadeService>,
}

impl AppContext {
    pub async fn build(config: &TransferConfig, vault: &VaultService) -> Outcome<Self> {
        let config = Arc::new(config.clone());

        // Shared infrastructure
        let db = vault.get_db_connection(config.common()).await?;
        let db_factory = SeaOrmDataFactory::new(db.clone());
        let http_client = Arc::new(HttpClient::new(10, 10));

        // Oauth module validator
        let oauth_validator: Arc<dyn OauthTokenValidator> =
            OAuthSetup::new().build_token_service(config.common().clone().into(), db.clone());

        // Domain services
        let transfer_process_svc = Arc::new(TransferProcessService::new(
            db_factory.transfer_process_repo(),
            db_factory.transfer_identifier_repo(),
        ));
        let transfer_message_svc = Arc::new(TransferMessageService::new(
            db_factory.transfer_message_repo(),
        ));
        let ssi_auth_facade = Arc::new(SSIAuthFacadeService::new(
            Arc::new(config.ssi_auth().clone()),
            http_client,
        ));

        Ok(Self {
            config,
            db,
            transfer_process_svc,
            transfer_message_svc,
            oauth_validator,
            ssi_auth_facade,
        })
    }
}
