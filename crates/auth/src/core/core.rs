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

use common::config::services::SsiAuthConfig;
use ymir::core_traits::CoreWalletTrait;
use ymir::services::issuer::IssuerTrait;
use ymir::services::repo::subtraits::{MatesTrait, MinionsTrait};
use ymir::services::verifier::VerifierTrait;
use ymir::services::wallet::WalletTrait;

use crate::core::traits::{
    AuthCoreTrait, CoreBusinessTrait, CoreGaiaSelfIssuerTrait, CoreGateKeeperTrait, CoreMateTrait,
    CoreOnboarderTrait, CoreVcRequesterTrait, CoreVerifierTrait
};
use crate::services::business::BusinessTrait;
use crate::services::callback::CallbackTrait;
use crate::services::gaia_self_issuer::GaiaOwnIssuerTrait;
use crate::services::gatekeeper::GateKeeperTrait;
use crate::services::onboarder::OnboarderTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;
use crate::services::vc_requester::VcRequesterTrait;

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
    own_issuer: Option<Arc<dyn GaiaOwnIssuerTrait>>
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
        self_issuer: Option<Arc<dyn GaiaOwnIssuerTrait>>
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
            own_issuer: self_issuer
        }
    }
}

impl CoreOnboarderTrait for AuthCore {
    fn onboarder(&self) -> Arc<dyn OnboarderTrait> { self.onboarder.clone() }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> { self.repo.clone() }

    fn callback(&self) -> Arc<dyn CallbackTrait> { self.callback.clone() }

    fn wallet(&self) -> Option<Arc<dyn WalletTrait>> { self.wallet.as_ref().cloned() }
}

impl CoreWalletTrait for AuthCore {
    fn wallet(&self) -> Arc<dyn WalletTrait> {
        self.wallet
            .as_ref()
            .map(Clone::clone)
            .expect("Wallet module is required for this operation but is not active in the current configuration")
    }

    fn mate(&self) -> Option<Arc<dyn MatesTrait>> { Some(self.repo.mates().clone()) }

    fn minion(&self) -> Option<Arc<dyn MinionsTrait>> { None }
}

impl CoreVcRequesterTrait for AuthCore {
    fn vc_req(&self) -> Arc<dyn VcRequesterTrait> { self.vc_requester.clone() }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> { self.repo.clone() }

    fn callback(&self) -> Arc<dyn CallbackTrait> { self.callback.clone() }

    fn wallet(&self) -> Option<Arc<dyn WalletTrait>> { self.wallet.as_ref().cloned() }
}

impl CoreMateTrait for AuthCore {
    fn repo(&self) -> Arc<dyn AuthRepoTrait> { self.repo.clone() }
}

impl CoreGaiaSelfIssuerTrait for AuthCore {
    fn issuer(&self) -> Arc<dyn IssuerTrait> {
        self.issuer
            .as_ref()
            .map(Clone::clone)
            .expect("Issuer module is required for this operation but is not active in the current configuration")
    }

    fn gaia(&self) -> Arc<dyn GaiaOwnIssuerTrait> {
        self.own_issuer
            .as_ref()
            .map(Clone::clone)
            .expect("Gaia module is required for this operation but is not active in the current configuration")
    }

    fn wallet(&self) -> Option<Arc<dyn WalletTrait>> { self.wallet.clone() }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> { self.repo.clone() }
}

impl CoreVerifierTrait for AuthCore {
    fn verifier(&self) -> Arc<dyn VerifierTrait> { self.verifier.clone() }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> { self.repo.clone() }

    fn business(&self) -> Arc<dyn BusinessTrait> { self.business.clone() }
}

impl CoreBusinessTrait for AuthCore {
    fn business(&self) -> Arc<dyn BusinessTrait> { self.business.clone() }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> { self.repo.clone() }

    fn verifier(&self) -> Arc<dyn VerifierTrait> { self.verifier.clone() }
}

impl CoreGateKeeperTrait for AuthCore {
    fn gatekeeper(&self) -> Arc<dyn GateKeeperTrait> { self.gatekeeper.clone() }

    fn verifier(&self) -> Arc<dyn VerifierTrait> { self.verifier.clone() }

    fn repo(&self) -> Arc<dyn AuthRepoTrait> { self.repo.clone() }
}

impl AuthCoreTrait for AuthCore {
    fn is_gaia_active(&self) -> bool {
        match self.own_issuer {
            Some(_) => true,
            None => false
        }
    }

    fn is_wallet_active(&self) -> bool {
        match self.wallet {
            Some(_) => true,
            None => false
        }
    }
    fn config(&self) -> Arc<SsiAuthConfig> { self.config.clone() }
}
