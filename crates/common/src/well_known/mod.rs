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

use ymir::errors::Outcome;

use crate::auth::ServiceHttpClient;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::facades::ssi_auth_facade::mates_facade::MatesFacadeService;
use crate::well_known::dspace_version::dspace_version::WellKnownDSpaceVersionService;
use crate::well_known::router::WellKnownRouter;
use crate::well_known::rpc::rpc::WellKnownRPCService;
use ymir::config::types::HostType;

pub mod dspace_version;
pub mod router;
pub mod rpc;

pub struct WellKnownRoot;
impl WellKnownRoot {
    pub fn get_well_known_router(config: &MinKnownConfig) -> Outcome<axum::Router> {
        let config = Arc::new(config.clone());
        let service_client = Arc::new(ServiceHttpClient::new(
            &config.service_client,
            &config.get_host(HostType::Http),
        ));
        let mates_facade = Arc::new(MatesFacadeService::new(config.clone(), service_client));

        let dspace_version_service = WellKnownDSpaceVersionService::new();
        let dspace_version_rpc = Arc::new(WellKnownRPCService::new(mates_facade.clone()));
        let router = WellKnownRouter::new(dspace_version_service, dspace_version_rpc.clone());
        Ok(router.router())
    }
}
