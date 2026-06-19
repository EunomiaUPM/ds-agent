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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use common::config::services::SsiAuthConfig;
use std::sync::Arc;
use ymir::services::repo::traits::shared::ParticipantRepoTrait;
use ymir::services::verifier::VerifierTrait;
use ymir::services::wallet::WalletTrait;
use ymir::services::{HasVerifier, HasWallet};

use super::AuthOrchestatorTrait;
use crate::modules::{
    CoreMateTrait, GaiaSelfIssuerModuleTrait, GateKeeperModuleTrait, PeerConnectorModuleTrait,
    VcRequesterModuleTrait, VerifierModuleTrait,
};
use crate::services::callback::CallbackTrait;
use crate::services::gaia_self_issuer::GaiaOwnIssuerTrait;
use crate::services::gatekeeper::GateKeeperTrait;
use crate::services::peer_connector::PeerConnectorTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;
use crate::services::vc_requester::VcRequesterTrait;
use crate::services::{HasCallback, HasGateKeeper, HasPeerConnector, HasRepo, HasVcRequester};

pub struct AuthCore {
    vc_requester: Arc<dyn VcRequesterTrait>,
    peer_connector: Arc<dyn PeerConnectorTrait>,
    callback: Arc<dyn CallbackTrait>,
    gatekeeper: Arc<dyn GateKeeperTrait>,
    verifier: Arc<dyn VerifierTrait>,
    repo: Arc<dyn AuthRepoTrait>,
    wallet: Arc<dyn WalletTrait>,
    // gaia: Arc<dyn GaiaOwnIssuerTrait>,
    config: Arc<SsiAuthConfig>,
}

impl AuthCore {
    pub fn new(
        vc_requester: Arc<dyn VcRequesterTrait>,
        peer_connector: Arc<dyn PeerConnectorTrait>,
        callback: Arc<dyn CallbackTrait>,
        gatekeeper: Arc<dyn GateKeeperTrait>,
        verifier: Arc<dyn VerifierTrait>,
        repo: Arc<dyn AuthRepoTrait>,
        wallet: Arc<dyn WalletTrait>,
        // gaia: Arc<dyn GaiaOwnIssuerTrait>,
        config: Arc<SsiAuthConfig>,
    ) -> AuthCore {
        AuthCore {
            vc_requester,
            peer_connector,
            callback,
            gatekeeper,
            verifier,
            repo,
            config,
            wallet,
            // gaia,
        }
    }
}

// ========================================== SERVICES =============================================

impl HasPeerConnector for AuthCore {
    fn peer_connector(&self) -> Arc<dyn PeerConnectorTrait> {
        self.peer_connector.clone()
    }
}

impl HasRepo for AuthCore {
    fn repo(&self) -> Arc<dyn AuthRepoTrait> {
        self.repo.clone()
    }
}

impl HasCallback for AuthCore {
    fn callback(&self) -> Arc<dyn CallbackTrait> {
        self.callback.clone()
    }
}

impl HasWallet for AuthCore {
    fn wallet(&self) -> Arc<dyn WalletTrait> {
        self.wallet.clone()
    }
}

impl HasVcRequester for AuthCore {
    fn vc_requester(&self) -> Arc<dyn VcRequesterTrait> {
        self.vc_requester.clone()
    }
}

impl HasGateKeeper for AuthCore {
    fn gatekeeper(&self) -> Arc<dyn GateKeeperTrait> {
        self.gatekeeper.clone()
    }
}

impl HasVerifier for AuthCore {
    fn verifier(&self) -> Arc<dyn VerifierTrait> {
        self.verifier.clone()
    }
}

// ========================================== MODULES ==============================================
impl PeerConnectorModuleTrait for AuthCore {}
impl CoreMateTrait for AuthCore {}
impl VcRequesterModuleTrait for AuthCore {}
impl GaiaSelfIssuerModuleTrait for AuthCore {}
impl VerifierModuleTrait for AuthCore {}

impl GateKeeperModuleTrait for AuthCore {}
impl WalletModuleTrait for AuthCore {
    fn participant(&self) -> Arc<dyn ParticipantRepoTrait> {
        self.repo.participant().clone()
    }
}

// ======================================== ORCHESTATOR ============================================
impl AuthOrchestatorTrait for AuthCore {
    fn config(&self) -> Arc<SsiAuthConfig> {
        self.config.clone()
    }
}
