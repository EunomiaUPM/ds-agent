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

use serde::{Deserialize, Serialize};
use ymir::config::traits::{
    DidConfigTrait, VcConfigTrait, VerifyReqConfigTrait, WalletConfigTrait,
};
use ymir::config::types::{DidConfig, VcConfig, VerifyReqConfig, WalletConfig};
use ymir::errors::Outcome;

use crate::config::services::traits::SsiAuthConfigTrait;
use crate::config::services::CommonConfig;
use crate::config::types::traits::{
    CommonConfigTrait, ConfigLoader, EntityClientTrait, GaiaConfigTrait,
};
use crate::config::types::{EntityClientConfig, GaiaConfig};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SsiAuthConfig {
    common_config: CommonConfig,
    wallet_config: Option<WalletConfig>,
    client_config: EntityClientConfig,
    did_config: DidConfig,
    vc_config: VcConfig,
    verify_req_config: VerifyReqConfig,
    gaia_config: Option<GaiaConfig>,
}

impl VcConfigTrait for SsiAuthConfig {
    fn vc_config(&self) -> &VcConfig {
        &self.vc_config
    }
}

impl DidConfigTrait for SsiAuthConfig {
    fn did_config(&self) -> &DidConfig {
        &self.did_config
    }
}

impl VerifyReqConfigTrait for SsiAuthConfig {
    fn verify_req_config(&self) -> &VerifyReqConfig {
        &self.verify_req_config
    }
}

impl WalletConfigTrait for SsiAuthConfig {
    fn wallet_config(&self) -> &WalletConfig {
        self.wallet_config.as_ref().expect("Wallet is not active")
    }
}

impl ConfigLoader for SsiAuthConfig {
    fn load(env_file: &str) -> Outcome<Self> {
        Self::global_load(env_file)
            .map(|data| data.ssi_auth().clone())
            .or_else(|_| Self::local_load(env_file))
    }
}

impl CommonConfigTrait for SsiAuthConfig {
    fn common(&self) -> &CommonConfig {
        &self.common_config
    }
}

impl EntityClientTrait for SsiAuthConfig {
    fn client_config(&self) -> &EntityClientConfig {
        &self.client_config
    }
}

impl GaiaConfigTrait for SsiAuthConfig {
    fn gaia_config(&self) -> &GaiaConfig {
        self.gaia_config.as_ref().expect("GaiaConfig is not active")
    }
}

impl SsiAuthConfigTrait for SsiAuthConfig {
    fn is_gaia_active(&self) -> bool {
        self.gaia_config.is_some()
    }
    fn is_wallet_active(&self) -> bool {
        self.wallet_config.is_some()
    }
}
