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

//! The outbound flow's load step: the process, peer, agreement and target of a message, and
//! the DSP message itself, built in JSON-LD and read back through RDF.

use std::str::FromStr;
use std::sync::Arc;
use common::config::types::traits::CommonConfigTrait;

use common::dsp_common::data_address::DataAddress;
use common::dsp_common::odrl::OdrlAgreement;
use common::dsp_common::paths;
use common::dsp_common::{DspActor, DspCaller};
use common::oauth::Owner;
use ymir::types::oauth::UserInfo;
use common::rdf::{RdfMessage, RdfProtocolTrait};
use serde_json::{json, Map, Value};
use url::Url;
use urn::Urn;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::ids::TransferProcessId;
use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::protocols::dsp::entities::context_common::DIRECTION_PROPERTY;
use crate::protocols::dsp::entities::context_rpc::{TransferRpcDomain, TransferRpcTyped};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspTransfer;
use crate::protocols::dsp::entities::state::TransferDSPState;
use crate::protocols::dsp::services::dsp_persistence::service::TransferDspPersistence;
use crate::setup::context::AppContext;

/// Resolves what an outbound message needs before it can be checked and sent.
pub struct TransferRpcLoader {
    ctx: Arc<AppContext>,
    persistence: TransferDspPersistence,
}

impl TransferRpcLoader {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            persistence: TransferDspPersistence::new(ctx.clone()),
            ctx,
        }
    }

    /// Resolves the domain stage of a typed RPC body.
    pub async fn load(
        &self,
        caller: &DspCaller,
        typed: TransferRpcTyped,
    ) -> Outcome<TransferRpcDomain> {
        let user = caller.user()?;
        match typed.message {
            TransferDSPMessageType::TransferRequestMessage => self.load_request(user, typed).await,
            _ => self.load_continuation(user, typed).await,
        }
    }

    /// This agent's DSP base, the default callback a provider answers to.
    fn own_dsp_base(&self) -> String {
        let host = self.ctx.config.common().get_host(HostType::Http);
        format!("{host}{}", paths::DSP_CURRENT)
    }

    /// The agreement a message runs under, from the negotiation agent.
    async fn agreement(&self, id: &str) -> Outcome<OdrlAgreement> {
        let id = Urn::from_str(id).map_err(|_| {
            Errors::format(
                BadFormat::Received,
                format!("agreementId is not a URN: {id}"),
                None,
            )
        })?;
        self.ctx.ports.negotiation.get_agreement(&id).await
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

    /// A request opens a new consumer process; nothing is stored until the provider accepts.
    async fn load_request(
        &self,
        caller: &UserInfo,
        typed: TransferRpcTyped,
    ) -> Outcome<TransferRpcDomain> {
        let body = typed.body.clone();
        let process_id = TransferProcessId::generate().as_urn().clone();
        let consumer_pid = format!("urn:uuid:{}", uuid::Uuid::new_v4());
        let transfer_direction = match body.data_address {
            Some(_) => TransferDirection::Push,
            None => TransferDirection::Pull,
        };
        let callback = body
            .callback_address
            .clone()
            .unwrap_or_else(|| self.own_dsp_base());
        let message = typed.message.clone();
        let peer_id = body.associated_agent_peer.clone().unwrap_or_default();
        let mut domain = TransferRpcDomain {
            process: None,
            process_id,
            role: TransferRole::Consumer,
            transfer_direction,
            owner: Owner::for_new(caller, None, None),
            agreement: self
                .agreement(body.agreement_id.as_deref().unwrap_or_default())
                .await?,
            peer_mate: self.peer_mate(caller, &peer_id).await,
            peer_id,
            target_url: format!(
                "{}/transfers/request",
                Self::base(body.provider_address.as_deref().unwrap_or_default())
            ),
            outgoing: RdfMessage::<DspTransfer>::build(&message, Map::new()).await?,
            is_restart: false,
            typed,
        };
        // A push consumer sets its sink up first: the request carries its address.
        let sink = body.data_address.clone().map(DataAddress::from);
        let sink = self
            .ctx
            .ports
            .dataplane
            .on_transfer_request_pre(&domain.hooks(sink.clone()))
            .await?
            .map(DataAddress::from)
            .or(sink);
        let members = Self::members(json!({
            "consumerPid": consumer_pid,
            "agreementId": body.agreement_id,
            "format": body.format,
            "callbackAddress": callback,
            "dataAddress": sink,
        }));
        domain.outgoing =
            Self::request_message(&message, members, body.auto_start == Some(true)).await?;
        Ok(domain)
    }

    /// The DSP 9.2.1 request; with `auto_start` it also carries `eunomia:autoStart`.
    async fn request_message(
        message: &TransferDSPMessageType,
        members: Map<String, Value>,
        auto_start: bool,
    ) -> Outcome<RdfMessage<DspTransfer>> {
        if !auto_start {
            return RdfMessage::<DspTransfer>::build(message, members).await;
        }
        let mut json = Map::new();
        json.insert("@context".into(), DspTransfer::auto_start_context());
        json.insert("@type".into(), json!(DspTransfer::compact_type(message)));
        json.insert(DspTransfer::auto_start_member(), json!(true));
        json.extend(members.into_iter().filter(|(_, v)| !v.is_null()));
        RdfMessage::<DspTransfer>::read(Value::Object(json)).await
    }

    /// Any other message moves the process `@id` names, if the caller may act on it, addressed
    /// to the peer's pid.
    async fn load_continuation(
        &self,
        caller: &UserInfo,
        typed: TransferRpcTyped,
    ) -> Outcome<TransferRpcDomain> {
        let id = typed.process_id().unwrap_or_default().to_string();
        let process = self
            .persistence
            .process_by_id(&id)
            .await?
            .ok_or_else(|| {
                Errors::missing_resource(id.clone(), "transfer process not found", None)
            })?;
        let correlation = process.correlation().clone();
        let peer_id = correlation
            .peer_participant_id
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_default();
        DspActor::user(caller).authorize(process.owner(), &peer_id, process.id().as_urn())?;

        let role = process.role();
        let (consumer_pid, provider_pid) = (
            correlation.consumer_pid.clone().unwrap_or_default(),
            correlation.provider_pid.clone().unwrap_or_default(),
        );
        let peer_pid = match role {
            TransferRole::Provider => &consumer_pid,
            _ => &provider_pid,
        };
        let base = correlation
            .callback_address
            .as_ref()
            .map(Url::to_string)
            .unwrap_or_default();
        let message = typed.message.clone();
        let is_restart = process.state().0 == TransferDSPState::SUSPENDED.to_string()
            && message == TransferDSPMessageType::TransferStartMessage;
        let (code, reason) = (typed.body.code.clone(), typed.body.reason.clone());
        let agreement_id = correlation
            .agreement_id
            .as_ref()
            .ok_or_else(|| Errors::crazy("stored transfer process has no agreement", None))?;
        let mut domain = TransferRpcDomain {
            agreement: self
                .ctx
                .ports
                .negotiation
                .get_agreement(agreement_id)
                .await?,
            peer_mate: self.peer_mate(caller, &peer_id).await,
            target_url: format!(
                "{}/transfers/{peer_pid}/{}",
                Self::base(&base),
                Self::segment(&message)
            ),
            process_id: process.id().as_urn().clone(),
            transfer_direction: process
                .properties()
                .get(DIRECTION_PROPERTY)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(TransferDirection::Pull),
            owner: process.owner().clone(),
            role,
            peer_id,
            outgoing: RdfMessage::<DspTransfer>::build(&message, Map::new()).await?,
            is_restart,
            process: Some(process),
            typed,
        };
        // A provider starting a pull hands the consumer its endpoint in the start message.
        let endpoint = match (&message, role) {
            (TransferDSPMessageType::TransferStartMessage, TransferRole::Provider) => self
                .ctx
                .ports
                .dataplane
                .on_transfer_start_pre(&domain.hooks(None))
                .await?
                .map(DataAddress::from),
            _ => None,
        };
        domain.outgoing = RdfMessage::<DspTransfer>::build(
            &message,
            Self::members(json!({
                "consumerPid": consumer_pid,
                "providerPid": provider_pid,
                "dataAddress": endpoint,
                "code": code,
                "reason": reason,
            })),
        )
        .await?;
        Ok(domain)
    }

    fn members(value: Value) -> Map<String, Value> {
        match value {
            Value::Object(map) => map,
            _ => Map::new(),
        }
    }

    fn base(url: &str) -> &str {
        url.trim_end_matches('/')
    }

    /// DSP 10.2 path segment of a state change.
    fn segment(message: &TransferDSPMessageType) -> &'static str {
        match message {
            TransferDSPMessageType::TransferStartMessage => "start",
            TransferDSPMessageType::TransferCompletionMessage => "completion",
            TransferDSPMessageType::TransferSuspensionMessage => "suspension",
            _ => "termination",
        }
    }
}
