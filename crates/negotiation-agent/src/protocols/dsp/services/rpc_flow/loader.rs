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

//! The outbound flow's load step: the process, peer and target of a message, and the DSP
//! message itself, built in JSON-LD and read back through RDF.

use std::str::FromStr;
use std::sync::Arc;

use ymir::config::traits::CommonConfigTrait;
use common::dsp_common::paths;
use common::dsp_common::{DspActor, DspCaller};
use ymir::oauth::Owner;
use ymir::types::oauth::UserInfo;
use common::rdf::RdfMessage;
use serde_json::{json, Map, Value};
use urn::Urn;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::protocols::dsp::entities::context_rpc::{NegotiationRpcDomain, NegotiationRpcTyped};
use crate::protocols::dsp::entities::message_types::NegotiationRpcMessage;
use crate::protocols::dsp::entities::process::NegotiationProcess;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspNegotiation;
use crate::protocols::dsp::entities::role::NegotiationRole;
use crate::protocols::dsp::services::dsp_persistence::service::NegotiationDspPersistence;
use crate::setup::context::AppContext;

/// Resolves what an outbound message needs before it can be checked and sent.
pub struct NegotiationRpcLoader {
    ctx: Arc<AppContext>,
    persistence: NegotiationDspPersistence,
}

impl NegotiationRpcLoader {
    /// ODRL members an agreement copies from the offer it closes.
    const POLICY_MEMBERS: [&'static str; 4] =
        ["profile", "permission", "obligation", "prohibition"];

    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            persistence: NegotiationDspPersistence::new(ctx.clone()),
            ctx,
        }
    }

    /// Resolves the domain stage of a typed RPC body.
    #[tracing::instrument(level = "info", skip_all, err)]
    pub async fn load(
        &self,
        caller: &DspCaller,
        typed: NegotiationRpcTyped,
    ) -> Outcome<NegotiationRpcDomain> {
        let user = caller.user()?;
        if typed.message.opens() {
            self.load_opening(user, typed).await
        } else {
            self.load_continuation(user, typed).await
        }
    }

    /// An opening request or offer starts a process; nothing is stored until the peer accepts.
    async fn load_opening(
        &self,
        user: &UserInfo,
        typed: NegotiationRpcTyped,
    ) -> Outcome<NegotiationRpcDomain> {
        let body = typed.body.clone();
        let own_pid = format!("urn:uuid:{}", uuid::Uuid::new_v4());
        let callback = body
            .callback_address
            .clone()
            .unwrap_or_else(|| self.own_dsp_base());
        let (role, peer_address, segment, pid_member) = match typed.message {
            NegotiationRpcMessage::OfferInit => (
                NegotiationRole::Provider,
                body.consumer_address.clone(),
                "offers",
                "providerPid",
            ),
            _ => (
                NegotiationRole::Consumer,
                body.provider_address.clone(),
                "request",
                "consumerPid",
            ),
        };
        let peer_address = peer_address.unwrap_or_default();
        let peer_id = body.associated_agent_peer.clone().unwrap_or_default();
        let offer = self.resolved_offer(role, body.offer.clone()).await?;
        let outgoing = Self::build(
            typed.message,
            json!({ pid_member: own_pid, "offer": offer, "callbackAddress": callback }),
        )
        .await?;
        Ok(NegotiationRpcDomain {
            process: None,
            role,
            owner: Owner::for_new(user, None, None),
            peer_mate: self.peer_mate(user, &peer_id).await,
            peer_id,
            target_url: format!("{}/negotiations/{segment}", Self::base(&peer_address)),
            peer_address,
            opening_target: outgoing
                .fields
                .offer
                .as_ref()
                .and_then(|o| o.target.clone()),
            outgoing,
            typed,
        })
    }

    /// Any other message moves the process `@id` names, if the caller acts on it, addressed to
    /// the peer's pid.
    async fn load_continuation(
        &self,
        user: &UserInfo,
        typed: NegotiationRpcTyped,
    ) -> Outcome<NegotiationRpcDomain> {
        let id = typed.process_id().unwrap_or_default().to_string();
        let process = self
            .persistence
            .process_by_id(&id)
            .await?
            .ok_or_else(|| Errors::missing_resource(id.clone(), "negotiation not found", None))?;
        DspActor::user(user).authorize(&process.owner(), process.peer_id(), &process.id()?)?;

        let body = &typed.body;
        let mut members = json!({
            "consumerPid": process.consumer_pid,
            "providerPid": process.provider_pid,
        });
        match typed.message {
            NegotiationRpcMessage::Request | NegotiationRpcMessage::Offer => {
                members["offer"] = self
                    .resolved_offer(process.role, body.offer.clone())
                    .await?
                    .unwrap_or(Value::Null);
            }
            NegotiationRpcMessage::Agreement => {
                members["agreement"] = self.agreement(&process).await?;
            }
            NegotiationRpcMessage::Termination => {
                members["code"] = json!(body.code);
                members["reason"] = json!(body.reason);
            }
            _ => {}
        }
        let peer_address = process.callback_address().unwrap_or_default().to_string();
        let target_url = format!(
            "{}/negotiations/{}/{}",
            Self::base(&peer_address),
            process.peer_pid().unwrap_or_default(),
            Self::segment(typed.message)
        );
        let peer_id = process.peer_id().to_string();
        Ok(NegotiationRpcDomain {
            outgoing: Self::build(typed.message, members).await?,
            opening_target: self.persistence.opening_target(&process).await?,
            role: process.role,
            owner: process.owner(),
            peer_mate: self.peer_mate(user, &peer_id).await,
            peer_id,
            peer_address,
            target_url,
            process: Some(process),
            typed,
        })
    }

    /// The agreement closing the last offer: its rules and target (the published offer's when it
    /// went by id), us as assigner and the peer as assignee.
    async fn agreement(&self, process: &NegotiationProcess) -> Outcome<Value> {
        let offer = self.persistence.last_offer(process).await?.ok_or_else(|| {
            Errors::missing_resource(process.model.id.clone(), "no offer to agree on", None)
        })?;
        let (content, target) = match offer.offer_content.get("target") {
            Some(target) => (offer.offer_content.clone(), target.clone()),
            None => self.published(&offer.offer_id).await?,
        };
        let assigner = self
            .ctx
            .ports
            .auth
            .mates
            .get_me_mate()
            .await
            .map(|me| me.participant_id)
            .unwrap_or_default();
        let mut agreement = json!({
            "@id": format!("urn:agreement:{}", uuid::Uuid::new_v4()),
            "@type": "Agreement",
            "target": target,
            "assigner": assigner,
            "assignee": process.peer_id(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        for member in Self::POLICY_MEMBERS {
            if let Some(rules) = content.get(member) {
                agreement[member] = rules.clone();
            }
        }
        Ok(agreement)
    }

    /// A provider's offer given only by `@id` becomes the one its catalog publishes, with the
    /// rules and the `target` DSP 8.3.3 requires; any other offer goes as sent.
    async fn resolved_offer(
        &self,
        role: NegotiationRole,
        offer: Option<Value>,
    ) -> Outcome<Option<Value>> {
        let reference = match &offer {
            Some(o) if role == NegotiationRole::Provider && o.get("target").is_none() => {
                o.get("@id").and_then(Value::as_str)
            }
            _ => None,
        };
        let Some(id) = reference else {
            return Ok(offer);
        };
        let (content, target) = self.published(id).await?;
        let mut resolved = json!({ "@id": id, "@type": "Offer", "target": target });
        for member in Self::POLICY_MEMBERS {
            if let Some(rules) = content.get(member) {
                resolved[member] = rules.clone();
            }
        }
        Ok(Some(resolved))
    }

    /// The rules and target of the offer the catalog publishes under `offer_id`.
    async fn published(&self, offer_id: &str) -> Outcome<(Value, Value)> {
        let id = Urn::from_str(offer_id).map_err(|_| {
            Errors::format(
                BadFormat::Received,
                format!("offer {offer_id} has no target and is not a URN"),
                None,
            )
        })?;
        let published = self.ctx.ports.catalog.get_offer(&id).await?;
        Ok((published.odrl_offer, Value::String(published.entity)))
    }

    /// The DSP message `message` sends, built from `members` and read back through RDF.
    async fn build(
        message: NegotiationRpcMessage,
        members: Value,
    ) -> Outcome<RdfMessage<DspNegotiation>> {
        let members = match members {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        let message = message.message();
        RdfMessage::<DspNegotiation>::build(&message, DspNegotiation::members(message, members))
            .await
    }

    /// This agent's DSP base, the default callback the peer answers to.
    fn own_dsp_base(&self) -> String {
        let host = self.ctx.config.common().get_host(HostType::Http);
        format!("{host}{}", paths::DSP_CURRENT)
    }

    /// The peer's participant record, when `user` sees it; an unknown peer is no error here.
    async fn peer_mate(&self, user: &UserInfo, peer_id: &str) -> Option<Mates> {
        self.ctx
            .ports
            .auth
            .mates
            .get_mate_by_id(user, peer_id.to_string())
            .await
            .inspect_err(
                |e| tracing::debug!(peer = peer_id, error = %e, "peer is not a known mate"),
            )
            .ok()
    }

    fn base(url: &str) -> &str {
        url.trim_end_matches('/')
    }

    /// DSP 8.2, 8.3 path segment below the peer's pid.
    fn segment(message: NegotiationRpcMessage) -> &'static str {
        match message {
            NegotiationRpcMessage::Request => "request",
            NegotiationRpcMessage::Offer => "offers",
            NegotiationRpcMessage::Agreement => "agreement",
            NegotiationRpcMessage::Verification => "agreement/verification",
            NegotiationRpcMessage::Acceptance | NegotiationRpcMessage::Finalization => "events",
            _ => "termination",
        }
    }
}
