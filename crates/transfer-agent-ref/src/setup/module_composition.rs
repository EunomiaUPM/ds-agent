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

//! Composition root: wires infrastructure → domain services → module tree,
//! and derives everything the agent exposes (HTTP, gRPC, DB migrations).

use axum::Router;
use common::config::services::TransferConfig;
use common::config::services::traits::TransferConfigTrait;
use common::config::types::traits::CommonConfigTrait;
use common::facades::ssi_auth_facade::ssi_auth_facade::SSIAuthFacadeService;
use common::http_client::HttpClient;
use common::module_loader::service_composer::ServiceComposer;
use oauth::config::OAuthConfig;
use oauth::setup::{OAuthModule, OAuthSetup};
use std::sync::Arc;
use tonic::service::Routes;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

use crate::SERVICE_NAME;
use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::protocols::dsp::setup::DspModule;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;
use crate::setup::root_api::ApiModule;

pub(crate) struct TransferAgentRefCompositionRoot {
    pub http_router: Router,
    pub grpc_routes: Routes,
}

impl TransferAgentRefCompositionRoot {
    pub async fn compose(config: &TransferConfig, vault: &VaultService) -> Outcome<Self> {
        // 0. Arc up config
        let config = Arc::new(config.clone());

        // 1. Shared infrastructure
        let db = vault.get_db_connection(config.common()).await?;
        let factory = SeaOrmDataFactory::new(db.clone());
        let oauth_validator = OAuthSetup::new().build_validator(config.jwt_secret());
        let http_client = Arc::new(HttpClient::new(10, 10));

        // 2. Domain services
        let process = Arc::new(TransferProcessService::new(
            factory.transfer_process_repo(),
            factory.transfer_identifier_repo(),
        ));
        let message = Arc::new(TransferMessageService::new(factory.transfer_message_repo()));
        let ssi_auth = Arc::new(SSIAuthFacadeService::new(
            Arc::new(config.ssi_auth().clone()),
            http_client,
        ));
        let oauth_config = OAuthConfig::new(
            config.jwt_secret(),
            config.common().get_host(HostType::Http),
            SERVICE_NAME,
        );

        // 3. Module tree — registration order = mount order
        let composer = ServiceComposer::new()
            .register(OAuthModule::new(oauth_config, db))
            .register(ApiModule::new(
                config.clone(),
                process.clone(),
                message.clone(),
                oauth_validator.clone(),
            ))
            .register(DspModule::new(
                config.clone(),
                process,
                message,
                oauth_validator,
                ssi_auth,
            ));

        Ok(Self {
            http_router: composer.http_router(),
            grpc_routes: composer.grpc_routes(),
        })
    }
}
