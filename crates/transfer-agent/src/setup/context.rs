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
use crate::protocols::dsp::facades::{FacadeService, FacadeTrait};
use crate::protocols::dsp::services::connector_resolver::ConnectorResolverTrait;
use crate::protocols::dsp::services::connector_resolver::connector_resolver::ConnectorResolver;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;
use crate::setup::ports::TransferPorts;
use common::auth::OauthTokenValidator;
use common::config::services::TransferConfig;
use common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use common::module_loader::root_context::RootContext;

#[derive(Clone)]
pub struct AppContext {
    pub config: Arc<TransferConfig>,
    pub transfer_process_svc: Arc<TransferProcessService>,
    pub transfer_message_svc: Arc<TransferMessageService>,
    pub oauth_validator: Arc<dyn OauthTokenValidator>,
    pub ssi_auth_facade: Arc<dyn SSIAuthFacadeTrait>,
    /// DSP collaborators, consumed once the domain loader and manager are wired.
    pub connector_resolver: Arc<dyn ConnectorResolverTrait>,
    pub dsp_facades: Arc<dyn FacadeTrait>,
}

impl AppContext {
    pub fn build(
        config: &TransferConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
        ports: &TransferPorts,
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

        Self {
            config,
            transfer_process_svc,
            transfer_message_svc,
            oauth_validator: root.validator.clone(),
            ssi_auth_facade: ports.auth.ssi_auth.clone(),
            connector_resolver: Arc::new(ConnectorResolver::new(
                ports.negotiation.clone(),
                ports.catalog.clone(),
            )),
            dsp_facades: Arc::new(FacadeService::new(ports.dataplane.clone())),
        }
    }
}
