/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::config::services::CommonConfig;
use crate::config::traits::{CommonConfigTrait, ConfigLoader};
use crate::config::types::{ClientConfig, GaiaConfig};
use serde::{Deserialize, Serialize};
use tracing::error;
use ymir::config::types::{DidConfig, VcConfig, VerifyReqConfig, WalletConfig};
use ymir::errors::{ErrorLogTrait, Errors};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SsiAuthConfig {
    common_config: CommonConfig,
    wallet_config: Option<WalletConfig>,
    client_config: ClientConfig,
    did_config: DidConfig,
    vc_config: VcConfig,
    verify_req_config: VerifyReqConfig,
    gaia_config: Option<GaiaConfig>,
}

impl SsiAuthConfig {
    pub fn wallet_config(&self) -> WalletConfig {
        let wallet = match self.wallet_config.as_ref() {
            Some(wallet) => Some(wallet.clone()),
            None => {
                let error = Errors::module_new("wallet");
                error!("{}", error.log());
                None
            }
        };
        wallet.expect("Wallet not active")
    }
    pub fn client_config(&self) -> ClientConfig {
        self.client_config.clone()
    }
    pub fn gaia_config(&self) -> GaiaConfig {
        let gaia = match self.gaia_config.as_ref() {
            Some(gaia) => Some(gaia.clone()),
            None => {
                let error = Errors::module_new("gaia");
                error!("{}", error.log());
                None
            }
        };
        gaia.expect("Gaia not active")
    }
    pub fn is_gaia_active(&self) -> bool {
        self.gaia_config.is_some()
    }
    pub fn is_wallet_active(&self) -> bool {
        self.wallet_config.is_some()
    }
    pub fn did_config(&self) -> DidConfig {
        self.did_config.clone()
    }
    pub fn vc_config(&self) -> VcConfig {
        self.vc_config.clone()
    }
    pub fn verify_req_config(&self) -> VerifyReqConfig {
        self.verify_req_config.clone()
    }
}

impl ConfigLoader for SsiAuthConfig {
    fn load(env_file: String) -> Self {
        match Self::global_load(env_file.clone()) {
            Ok(data) => data.ssi_auth(),
            Err(_) => Self::local_load(env_file).expect("Unable to load auth config"),
        }
    }
}

impl CommonConfigTrait for SsiAuthConfig {
    fn common(&self) -> &CommonConfig {
        &self.common_config
    }
}
