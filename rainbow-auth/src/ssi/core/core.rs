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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::Arc;

use rainbow_common::config::services::SsiAuthConfig;
use tracing::error;
use ymir::core_traits::CoreWalletTrait;
use ymir::errors::{ErrorLogTrait, Errors};
use ymir::services::issuer::IssuerTrait;
use ymir::services::repo::subtraits::{MatesTrait, MinionsTrait};
use ymir::services::verifier::VerifierTrait;
use ymir::services::wallet::WalletTrait;

use crate::ssi::core::traits::{
    AuthCoreTrait, CoreBusinessTrait, CoreGaiaSelfIssuerTrait, CoreGateKeeperTrait, CoreMateTrait,
    CoreOnboarderTrait, CoreVcRequesterTrait, CoreVerifierTrait,
};
use crate::ssi::services::business::BusinessTrait;
use crate::ssi::services::callback::CallbackTrait;
use crate::ssi::services::gaia_self_issuer::GaiaOwnIssuerTrait;
use crate::ssi::services::gatekeeper::GateKeeperTrait;
use crate::ssi::services::onboarder::OnboarderTrait;
use crate::ssi::services::repo::repo_trait::AuthRepoTrait;
use crate::ssi::services::vc_requester::VcRequesterTrait;

pub struct AuthCore {
    vc_requester: Arc<dyn VcRequesterTrait>,
    onboarder: Arc<dyn OnboarderTrait>,
    callback: Arc<dyn CallbackTrait>,
    business: Arc<dyn BusinessTrait>,
    gatekeeper: Arc<dyn GateKeeperTrait>,
    verifier: Arc<dyn VerifierTrait>,
    repo: Arc<dyn AuthRepoTrait>,
    config: Arc<SsiAuthConfig>,
    // EXTRA MODULES
    wallet: Option<Arc<dyn WalletTrait>>,
    issuer: Option<Arc<dyn IssuerTrait>>,
    own_issuer: Option<Arc<dyn GaiaOwnIssuerTrait>>,
}

impl AuthCore {
    pub fn new(
        vc_requester: Arc<dyn VcRequesterTrait>,
        onboarder: Arc<dyn OnboarderTrait>,
        callback: Arc<dyn CallbackTrait>,
        business: Arc<dyn BusinessTrait>,
        gatekeeper: Arc<dyn GateKeeperTrait>,
        verifier: Arc<dyn VerifierTrait>,
        repo: Arc<dyn AuthRepoTrait>,
        config: Arc<SsiAuthConfig>,
        // EXTRA MODULES
        wallet: Option<Arc<dyn WalletTrait>>,
        issuer: Option<Arc<dyn IssuerTrait>>,
        self_issuer: Option<Arc<dyn GaiaOwnIssuerTrait>>,
    ) -> AuthCore {
        AuthCore {
            vc_requester,
            onboarder,
            callback,
            business,
            gatekeeper,
            verifier,
            issuer,
            repo,
            config,
            wallet,
            own_issuer: self_issuer,
        }
    }
}

impl CoreOnboarderTrait for AuthCore {
    fn onboarder(&self) -> Arc<dyn OnboarderTrait> {
        self.onboarder.clone()
    }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> {
        self.repo.clone()
    }

    fn callback(&self) -> Arc<dyn CallbackTrait> {
        self.callback.clone()
    }
}

impl CoreWalletTrait for AuthCore {
    fn wallet(&self) -> Arc<dyn WalletTrait> {
        let wallet = self.wallet.clone().or_else(|| {
            let error = Errors::module_new("Wallet");
            error!("{}", error.log());
            None
        });
        wallet.expect("Wallet module activated")
    }

    fn mate(&self) -> Option<Arc<dyn MatesTrait>> {
        Some(self.repo.mates().clone())
    }

    fn minion(&self) -> Option<Arc<dyn MinionsTrait>> {
        None
    }
}

impl CoreVcRequesterTrait for AuthCore {
    fn vc_req(&self) -> Arc<dyn VcRequesterTrait> {
        self.vc_requester.clone()
    }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> {
        self.repo.clone()
    }

    fn callback(&self) -> Arc<dyn CallbackTrait> {
        self.callback.clone()
    }
}

impl CoreMateTrait for AuthCore {
    fn repo(&self) -> Arc<dyn AuthRepoTrait> {
        self.repo.clone()
    }
}

impl CoreGaiaSelfIssuerTrait for AuthCore {
    fn issuer(&self) -> Arc<dyn IssuerTrait> {
        let issuer = self.issuer.clone().or_else(|| {
            let error = Errors::module_new("Issuer");
            error!("{}", error.log());
            None
        });
        issuer.expect("Issuer module is not active")
    }

    fn gaia(&self) -> Arc<dyn GaiaOwnIssuerTrait> {
        let self_issuer = self.own_issuer.clone().or_else(|| {
            let error = Errors::module_new("Wallet");
            error!("{}", error.log());
            None
        });
        self_issuer.expect("Self issuer module not activated")
    }

    fn wallet(&self) -> Option<Arc<dyn WalletTrait>> {
        self.wallet.clone()
    }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> {
        self.repo.clone()
    }
}

impl CoreVerifierTrait for AuthCore {
    fn verifier(&self) -> Arc<dyn VerifierTrait> {
        self.verifier.clone()
    }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> {
        self.repo.clone()
    }

    fn business(&self) -> Arc<dyn BusinessTrait> {
        self.business.clone()
    }
}

impl CoreBusinessTrait for AuthCore {
    fn business(&self) -> Arc<dyn BusinessTrait> {
        self.business.clone()
    }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> {
        self.repo.clone()
    }

    fn verifier(&self) -> Arc<dyn VerifierTrait> {
        self.verifier.clone()
    }
}

impl CoreGateKeeperTrait for AuthCore {
    fn gatekeeper(&self) -> Arc<dyn GateKeeperTrait> {
        self.gatekeeper.clone()
    }

    fn verifier(&self) -> Arc<dyn VerifierTrait> {
        self.verifier.clone()
    }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> {
        self.repo.clone()
    }
}

impl AuthCoreTrait for AuthCore {
    fn is_gaia_active(&self) -> bool {
        match self.own_issuer {
            Some(_) => true,
            None => false,
        }
    }

    fn is_wallet_active(&self) -> bool {
        match self.wallet {
            Some(_) => true,
            None => false,
        }
    }
    fn config(&self) -> Arc<SsiAuthConfig> {
        self.config.clone()
    }
}
