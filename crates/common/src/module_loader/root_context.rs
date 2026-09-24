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

//! Process-wide infrastructure built once at the composition root and injected into every module.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

use crate::auth::{OauthTokenValidator, ServiceHttpClient};
use crate::config::services::CommonConfig;

/// Seconds the shared service client waits for a peer agent.
const SERVICE_CLIENT_TIMEOUT: u64 = 10;

/// Builds the process token validator; lives in `oauth`, which `common` cannot depend on.
pub type ValidatorFactory = fn(&CommonConfig, DatabaseConnection) -> Arc<dyn OauthTokenValidator>;

/// One vault, one DB pool, one token validator and one service client for the whole process.
#[derive(Clone)]
pub struct RootContext {
    pub vault: Arc<VaultService>,
    pub db: DatabaseConnection,
    pub validator: Arc<dyn OauthTokenValidator>,
    pub service_client: Arc<ServiceHttpClient>,
}

impl RootContext {
    pub async fn connect(
        common: &CommonConfig,
        vault: Arc<VaultService>,
        validator: ValidatorFactory,
    ) -> Outcome<Self> {
        let db = vault.get_db_connection(common).await?;
        Ok(Self {
            validator: validator(common, db.clone()),
            service_client: Arc::new(ServiceHttpClient::from_common(
                common,
                SERVICE_CLIENT_TIMEOUT,
            )),
            vault,
            db,
        })
    }
}
