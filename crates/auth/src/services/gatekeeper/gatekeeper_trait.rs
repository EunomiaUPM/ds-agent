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

use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use ymir::data::entities::received::{grant, interaction};
use ymir::data::entities::shared::{participant, participant_relation, resource_req};
use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::client::Client;
use ymir::types::gnap::grant_request::interact::InteractRequest;
use ymir::types::gnap::grant_request::{GrantRequest, GrantRequestKind};
use ymir::types::gnap::InteractionFinishResponse;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

/// Gatekeeper side of GNAP: building and checking what peers send.
#[mockall::automock]
#[async_trait]
pub trait GateKeeperTrait: Send + Sync + 'static {
    /// New received grant; `class_id` is the class the peer claims (its nick). Without a `role`
    /// the root handles it, and without a `visibility` it is public: once stored, a grant always
    /// has both.
    fn build_grant_plan(
        &self,
        role: Option<RolePath>,
        visibility: Option<Visibility>,
        class_id: Option<String>,
    ) -> Outcome<grant::Plan>;
    fn build_resource_req_plan(
        &self,
        id: &str,
        grant_request_kind: GrantRequestKind,
    ) -> Outcome<resource_req::Model>;
    fn build_interaction_plan(
        &self,
        id: &str,
        client: Client,
        interact: Option<InteractRequest>,
    ) -> Outcome<interaction::Plan>;
    /// Participant record of the peer once verified.
    fn build_mate_plan(&self, holder: &str, nick: &str, base_url: &str) -> participant::Plan;
    /// Private relation of the verification with the verified peer, under `role`.
    fn build_mate_rel_plan(&self, role: &RolePath, holder: &str) -> participant_relation::Model;
    /// Parses a grant request; only HTTP signature key proofs are accepted.
    fn validate_grant_req(&self, payload: &Bytes, headers: &HeaderMap) -> Outcome<GrantRequest>;

    /// Checks a continuation request against its interaction.
    fn validate_cont_req(
        &self,
        model: &interaction::Model,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<()>;
    /// Answer that closes the interaction, sent to the peer's callback.
    async fn finish_interaction(
        &self,
        model: &interaction::Model,
        verification_result: Outcome<()>,
    ) -> Outcome<InteractionFinishResponse>;
}
