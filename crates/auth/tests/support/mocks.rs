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

//! Doubles of every port `AuthCore` holds: set expectations on the fields, then build the core.
//! A port with no expectation panics when called, so each test also proves what is not touched.

use std::sync::Arc;

use auth::core::AuthCore;
use auth::data::factory::MockAuthRepoTrait;
use auth::services::callback::MockCallbackTrait;
use auth::services::gaia_self_attester::MockGaiaSelfAttesterTrait;
use auth::services::gatekeeper::MockGateKeeperTrait;
use auth::services::peer_connector::MockPeerConnectorTrait;
use auth::services::vc_requester::MockVcRequesterTrait;
use ymir::services::issuer::MockIssuerTrait;
use ymir::services::repo::traits::received::{
    MockRecvGrantRepoTrait, MockRecvInteractionRepoTrait, MockRecvVerificationRepoTrait,
};
use ymir::services::repo::traits::sent::{
    MockSentGrantRepoTrait, MockSentInteractionRepoTrait, MockSentVerificationRepoTrait,
};
use ymir::services::repo::traits::shared::{
    MockParticipantRelationRepoTrait, MockParticipantRepoTrait, MockResourceReqRepoTrait,
};
use ymir::services::verifier::MockVerifierTrait;
use ymir::services::wallet::MockWalletTrait;

use crate::support::fixtures::config;

#[derive(Default)]
pub struct Repos {
    pub sent_grant: MockSentGrantRepoTrait,
    pub sent_interaction: MockSentInteractionRepoTrait,
    pub sent_verification: MockSentVerificationRepoTrait,
    pub participant: MockParticipantRepoTrait,
    pub participant_relation: MockParticipantRelationRepoTrait,
    pub resource_req: MockResourceReqRepoTrait,
    pub recv_grant: MockRecvGrantRepoTrait,
    pub recv_interaction: MockRecvInteractionRepoTrait,
    pub recv_verification: MockRecvVerificationRepoTrait,
}

#[derive(Default)]
pub struct Doubles {
    pub repos: Repos,
    pub vc_requester: MockVcRequesterTrait,
    pub peer_connector: MockPeerConnectorTrait,
    pub callback: MockCallbackTrait,
    pub gatekeeper: MockGateKeeperTrait,
    pub verifier: MockVerifierTrait,
    pub wallet: MockWalletTrait,
    pub gaia: MockGaiaSelfAttesterTrait,
    pub issuer: MockIssuerTrait,
}

impl Repos {
    /// Factory handing out these repositories.
    fn factory(self) -> MockAuthRepoTrait {
        let mut factory = MockAuthRepoTrait::new();
        let repo = Arc::new(self.sent_grant);
        factory.expect_sent_grant().returning(move || repo.clone());
        let repo = Arc::new(self.sent_interaction);
        factory
            .expect_sent_interaction()
            .returning(move || repo.clone());
        let repo = Arc::new(self.sent_verification);
        factory
            .expect_sent_verification()
            .returning(move || repo.clone());
        let repo = Arc::new(self.participant);
        factory.expect_participant().returning(move || repo.clone());
        let repo = Arc::new(self.participant_relation);
        factory
            .expect_participant_relation()
            .returning(move || repo.clone());
        let repo = Arc::new(self.resource_req);
        factory
            .expect_resource_req()
            .returning(move || repo.clone());
        let repo = Arc::new(self.recv_grant);
        factory.expect_recv_grant().returning(move || repo.clone());
        let repo = Arc::new(self.recv_interaction);
        factory
            .expect_recv_interaction()
            .returning(move || repo.clone());
        let repo = Arc::new(self.recv_verification);
        factory
            .expect_recv_verification()
            .returning(move || repo.clone());
        factory
    }
}

impl Doubles {
    /// `AuthCore` over these doubles, with Gaia-X and the issuer enabled.
    pub fn core(self) -> AuthCore {
        AuthCore::new(
            Arc::new(self.vc_requester),
            Arc::new(self.peer_connector),
            Arc::new(self.callback),
            Arc::new(self.gatekeeper),
            Arc::new(self.verifier),
            Arc::new(self.repos.factory()),
            Arc::new(self.wallet),
            Some(Arc::new(self.gaia)),
            Some(Arc::new(self.issuer)),
            config(),
        )
    }
}
