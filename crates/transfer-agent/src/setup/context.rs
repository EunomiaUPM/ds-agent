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

use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;
use common::auth::OauthTokenValidator;
use common::config::services::TransferConfig;
use common::config::services::traits::TransferConfigTrait;
use common::facades::ssi_auth_facade::ssi_auth_facade::SSIAuthFacadeService;
use common::module_loader::root_context::RootContext;

#[derive(Clone)]
pub struct AppContext {
    pub config: Arc<TransferConfig>,
    pub transfer_process_svc: Arc<TransferProcessService>,
    pub transfer_message_svc: Arc<TransferMessageService>,
    pub oauth_validator: Arc<dyn OauthTokenValidator>,
    pub ssi_auth_facade: Arc<SSIAuthFacadeService>,
}

impl AppContext {
    pub fn build(
        config: &TransferConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Self {
        let config = Arc::new(config.clone());
        let db_factory = SeaOrmDataFactory::new(root.db.clone());

        // Domain services
        let transfer_process_svc = Arc::new(
            TransferProcessService::new(
                db_factory.transfer_process_repo(),
                db_factory.transfer_identifier_repo(),
            )
            .with_event_bus(event_bus.clone()),
        );
        let transfer_message_svc = Arc::new(
            TransferMessageService::new(db_factory.transfer_message_repo())
                .with_event_bus(event_bus),
        );
        let ssi_auth_facade = Arc::new(SSIAuthFacadeService::new(
            Arc::new(config.ssi_auth().clone()),
            root.service_client.clone(),
        ));

        Self {
            config,
            transfer_process_svc,
            transfer_message_svc,
            oauth_validator: root.validator.clone(),
            ssi_auth_facade,
        }
    }
}
