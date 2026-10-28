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

//! The outbound negotiation flow: load with the loader, send to the peer, apply with the
//! message's handler.

use std::sync::Arc;

use common::dsp_common::DspCaller;
use common::facades::grants_facade::send_with_peer_token;
use common::flow::{FlowCx, FlowTrait};
use common::rdf::RdfMessage;
use serde_json::{json, Value};
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientTrait;
use ymir::types::gnap::grant_request::access::AccessNeed;
use ymir::types::http::HttpBody;
use ymir::utils::http_client;

use crate::protocols::dsp::entities::context_rpc::{
    NegotiationRpcDomain, NegotiationRpcOutcome, NegotiationRpcTyped,
};
use crate::protocols::dsp::entities::message_types::NegotiationRpcMessage;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspNegotiation;
use crate::protocols::dsp::entities::role::NegotiationRole;
use crate::protocols::dsp::services::dsp_persistence::service::NegotiationDspPersistence;
use crate::protocols::dsp::services::rpc_flow::handlers::agreement::AgreementHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::event::EventHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::offer::OfferHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::request::RequestHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::termination::TerminationHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::verification::VerificationHandler;
use crate::protocols::dsp::services::rpc_flow::loader::NegotiationRpcLoader;
use crate::setup::context::AppContext;

/// Sends the negotiation messages to the peer and moves the local process with them.
pub struct NegotiationRpcFlow {
    ctx: Arc<AppContext>,
    loader: NegotiationRpcLoader,
    persistence: NegotiationDspPersistence,
}

impl NegotiationRpcFlow {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            loader: NegotiationRpcLoader::new(ctx.clone()),
            persistence: NegotiationDspPersistence::new(ctx.clone()),
            ctx,
        }
    }

    /// POSTs `body` to `url`; any non-2xx answer is an error carrying the peer's body.
    async fn post(url: String, headers: Option<http::HeaderMap>, body: Value) -> Outcome<Value> {
        let response = http_client()
            .post(&url, headers, HttpBody::Json(body))
            .await?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(Errors::petition(
                &url,
                "POST",
                Some(status),
                PetitionFailure::HttpStatus(status),
                text,
                None,
            ));
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
    }
}

#[async_trait::async_trait]
impl FlowTrait for NegotiationRpcFlow {
    type Key = NegotiationRpcMessage;
    type Caller = DspCaller;
    type Input = NegotiationRpcTyped;
    type Facts = NegotiationRpcDomain;
    type Output = NegotiationRpcOutcome;

    /// The route's RPC; the body's own type is checked against it by a shape rule.
    fn key(&self, typed: &NegotiationRpcTyped) -> NegotiationRpcMessage {
        typed.route()
    }

    async fn load(
        &self,
        cx: &FlowCx<'_, DspCaller>,
        typed: NegotiationRpcTyped,
    ) -> Outcome<NegotiationRpcDomain> {
        self.loader.load(cx.caller, typed).await
    }

    /// Sends the message as built; the answer to an opening message is read back through RDF.
    async fn send(
        &self,
        cx: &FlowCx<'_, DspCaller>,
        domain: &NegotiationRpcDomain,
    ) -> Outcome<Option<Value>> {
        let (url, body) = (domain.target_url.clone(), domain.outgoing.json.clone());
        let requested = domain.role == NegotiationRole::Consumer;
        let dataset = domain.opening_target.as_deref().filter(|_| requested);
        let need = AccessNeed::of_call(&url, dataset, !requested);
        let raw = send_with_peer_token(
            self.ctx.ports.auth.grants.as_ref(),
            cx.caller.user()?,
            &domain.peer_id,
            &need,
            requested,
            |headers| Self::post(url.clone(), headers, body.clone()),
        )
        .await?;
        // Only an opening message is answered with a message (DSP 8.3.1); a state change gets a
        // 200 whose body DSP leaves open.
        let read = match (domain.typed.message.opens(), &raw) {
            (true, answer) if !answer.is_null() => {
                let message = RdfMessage::<DspNegotiation>::read(answer.clone()).await?;
                json!({ "type": message.message.wire_type(), "fields": message.fields, "raw": answer })
            }
            _ => json!({ "raw": raw }),
        };
        Ok(Some(read))
    }

    async fn apply(
        &self,
        cx: &FlowCx<'_, DspCaller>,
        domain: NegotiationRpcDomain,
        reply: Option<Value>,
    ) -> Outcome<NegotiationRpcOutcome> {
        use NegotiationRpcMessage::*;
        let reply = reply.unwrap_or(Value::Null);
        let (persistence, user) = (&self.persistence, cx.caller.user()?);
        match domain.typed.message {
            RequestInit | Request => {
                RequestHandler::handle(persistence, user, domain, &reply).await
            }
            OfferInit | Offer => OfferHandler::handle(persistence, user, domain, &reply).await,
            Acceptance | Finalization => {
                EventHandler::handle(persistence, domain, &reply).await
            }
            Agreement => AgreementHandler::handle(persistence, domain, &reply).await,
            Verification => VerificationHandler::handle(persistence, domain, &reply).await,
            Termination => TerminationHandler::handle(persistence, domain, &reply).await,
        }
    }
}
