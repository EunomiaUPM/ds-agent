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

use super::AuthOrchestratorTrait;
use crate::modules::{
    GaiaSelfAttesterModule, GateKeeperModule, ParticipantModule, PeerConnectorModule,
    VcRequesterModule, VerifierModule,
};
use crate::services::callback::CallbackTrait;
use crate::services::gaia_self_attester::GaiaSelfAttesterTrait;
use crate::services::gatekeeper::GateKeeperTrait;
use crate::services::peer_connector::PeerConnectorTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;
use crate::services::vc_requester::VcRequesterTrait;
use crate::services::{
    HasCallback, HasGaiaSelfAttester, HasGateKeeper, HasPeerConnector, HasRepo, HasVcRequester,
};
use common::config::services::SsiAuthConfig;
use std::sync::Arc;
use ymir::modules::WalletModuleTrait;
use ymir::services::issuer::IssuerTrait;
use ymir::services::verifier::VerifierTrait;
use ymir::services::wallet::WalletTrait;
use ymir::services::{HasIssuer, HasVerifier, HasWallet};

pub struct AuthCore {
    vc_requester: Arc<dyn VcRequesterTrait>,
    peer_connector: Arc<dyn PeerConnectorTrait>,
    callback: Arc<dyn CallbackTrait>,
    gatekeeper: Arc<dyn GateKeeperTrait>,
    verifier: Arc<dyn VerifierTrait>,
    repo: Arc<dyn AuthRepoTrait>,
    wallet: Arc<dyn WalletTrait>,
    gaia: Option<Arc<dyn GaiaSelfAttesterTrait>>,
    issuer: Option<Arc<dyn IssuerTrait>>,
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
        gaia: Option<Arc<dyn GaiaSelfAttesterTrait>>,
        issuer: Option<Arc<dyn IssuerTrait>>,
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
            gaia,
            issuer,
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

impl HasGaiaSelfAttester for AuthCore {
    fn gaia(&self) -> Arc<dyn GaiaSelfAttesterTrait> {
        self.gaia.as_ref().expect("Gaia Module not active").clone()
    }
}

impl HasIssuer for AuthCore {
    fn issuer(&self) -> Arc<dyn IssuerTrait> {
        self.issuer
            .as_ref()
            .expect("Issuer Module not active")
            .clone()
    }
}

// ========================================== MODULES ==============================================
impl PeerConnectorModule for AuthCore {}
impl ParticipantModule for AuthCore {}
impl VcRequesterModule for AuthCore {}

impl GaiaSelfAttesterModule for AuthCore {}
impl VerifierModule for AuthCore {}

impl GateKeeperModule for AuthCore {}
impl WalletModuleTrait for AuthCore {}

// ======================================== ORCHESTATOR ============================================
impl AuthOrchestratorTrait for AuthCore {
    fn config(&self) -> Arc<SsiAuthConfig> {
        self.config.clone()
    }
}
