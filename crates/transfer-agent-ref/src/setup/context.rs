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

//! Dependency context: everything the modules need, built once.

use std::sync::Arc;

use common::auth::middleware::TokenValidator;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use sea_orm::DatabaseConnection;
use ymir::errors::Outcome;
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;

/// The domain services of the agent, wired over one DB connection.
pub(crate) struct DomainServices {
    pub process: Arc<TransferProcessService>,
    pub message: Arc<TransferMessageService>,
    pub validator: Arc<dyn TokenValidator>,
    pub db: DatabaseConnection,
}

/// Shared context injected into service modules: the agent config plus the
/// already-built domain services. `Ctx` for [`common::module_loader`].
/// Owns its config (cloned once at startup) so module trees can be `'static`.
pub(crate) struct ModuleCtx {
    pub config: TransferConfig,
    pub services: DomainServices,
}

/// How this agent satisfies the OAuth module's dependencies.
impl oauth::setup::OAuthModuleCtx for ModuleCtx {
    fn oauth_config(&self) -> oauth::config::OAuthConfig {
        use ymir::config::traits::HostsConfigTrait;
        use ymir::config::types::HostType;
        oauth::config::OAuthConfig::new(
            self.config.jwt_secret(),
            self.config.common().get_host(HostType::Http),
            "transfer-agent-ref",
        )
    }

    fn oauth_db(&self) -> sea_orm::DatabaseConnection {
        self.services.db.clone()
    }
}

impl ModuleCtx {
    /// Opens the DB connection and wires repositories, services and the
    /// token validator — the single dependency-wiring step of the agent.
    pub async fn build(config: &TransferConfig, vault: &VaultService) -> Outcome<Self> {
        let db = vault.get_db_connection(config.common()).await?;
        let factory = SeaOrmDataFactory::new(db.clone());
        let services = DomainServices {
            process: Arc::new(TransferProcessService::new(
                factory.transfer_process_repo(),
                factory.transfer_identifier_repo(),
            )),
            message: Arc::new(TransferMessageService::new(factory.transfer_message_repo())),
            validator: oauth::setup::OAuthSetup::new().build_validator(config.jwt_secret()),
            db,
        };
        Ok(Self {
            config: config.clone(),
            services,
        })
    }
}
