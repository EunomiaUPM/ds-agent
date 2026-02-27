/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use ymir::config::traits::{
    ApiConfigTrait, ConnectionConfigTrait, DidConfigTrait, HostsConfigTrait, VcConfigTrait,
    VerifyReqConfigTrait, WalletConfigTrait
};
use ymir::services::issuer::basic::config::{BasicIssuerConfig, BasicIssuerConfigBuilder};
use ymir::services::verifier::basic::config::{BasicVerifierConfig, BasicVerifierConfigBuilder};
use ymir::services::wallet::walt_id::config::{WaltIdConfig, WaltIdConfigBuilder};

use crate::config::services::SsiAuthConfig;
use crate::config::types::traits::CommonConfigTrait;

impl From<SsiAuthConfig> for WaltIdConfig {
    fn from(value: SsiAuthConfig) -> Self {
        WaltIdConfigBuilder::new()
            .hosts(value.common().hosts().clone())
            .ssi_wallet_config(value.wallet_config().clone())
            .did_config(value.did_config().clone())
            .build()
    }
}

impl From<SsiAuthConfig> for BasicVerifierConfig {
    fn from(value: SsiAuthConfig) -> Self {
        BasicVerifierConfigBuilder::new()
            .hosts(value.common().hosts().clone())
            .local(value.common().is_local())
            .requested_vcs(value.get_requested_vcs().to_vec())
            .api_path(value.common().get_api_version())
            .vc_config(value.vc_config().clone())
            .build()
    }
}

impl From<SsiAuthConfig> for BasicIssuerConfig {
    fn from(value: SsiAuthConfig) -> Self {
        BasicIssuerConfigBuilder::new()
            .hosts(value.common().hosts().clone())
            .local(value.common().is_local())
            .api_path(value.common().get_api_version())
            .did_config(value.did_config().clone())
            .build()
    }
}
