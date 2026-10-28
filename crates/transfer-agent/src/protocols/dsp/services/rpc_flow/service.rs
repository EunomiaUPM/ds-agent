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

//! The outbound transfer flow: load with the loader, send to the peer, apply with the
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

use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::context_rpc::{
    TransferRpcDomain, TransferRpcOutcome, TransferRpcTyped,
};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspTransfer;
use crate::protocols::dsp::services::dsp_persistence::service::TransferDspPersistence;
use crate::protocols::dsp::services::rpc_flow::handlers::completion::CompletionHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::request::RequestHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::start::StartHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::suspension::SuspensionHandler;
use crate::protocols::dsp::services::rpc_flow::handlers::termination::TerminationHandler;
use crate::protocols::dsp::services::rpc_flow::loader::TransferRpcLoader;
use crate::setup::context::AppContext;

/// Sends the five transfer messages to the peer and moves the local process with them.
pub struct TransferRpcFlow {
    ctx: Arc<AppContext>,
    loader: TransferRpcLoader,
    persistence: TransferDspPersistence,
}

impl TransferRpcFlow {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            loader: TransferRpcLoader::new(ctx.clone()),
            persistence: TransferDspPersistence::new(ctx.clone()),
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
impl FlowTrait for TransferRpcFlow {
    type Key = TransferDSPMessageType;
    type Caller = DspCaller;
    type Input = TransferRpcTyped;
    type Facts = TransferRpcDomain;
    type Output = TransferRpcOutcome;

    /// The route's message type; the body's own type is checked against it by a shape rule.
    fn key(&self, typed: &TransferRpcTyped) -> TransferDSPMessageType {
        typed.route().clone()
    }

    async fn load(
        &self,
        cx: &FlowCx<'_, DspCaller>,
        typed: TransferRpcTyped,
    ) -> Outcome<TransferRpcDomain> {
        self.loader.load(cx.caller, typed).await
    }

    /// Sends the message as built; the answer to a request is read back through RDF.
    async fn send(
        &self,
        cx: &FlowCx<'_, DspCaller>,
        domain: &TransferRpcDomain,
    ) -> Outcome<Option<Value>> {
        let (url, body) = (domain.target_url.clone(), domain.outgoing.json.clone());
        let requested = domain.role == TransferRole::Consumer;
        let need = AccessNeed::of_call(&url, None, !requested);
        let raw = send_with_peer_token(
            self.ctx.ports.auth.grants.as_ref(),
            cx.caller.user()?,
            &domain.peer_id,
            &need,
            requested,
            |headers| Self::post(url.clone(), headers, body.clone()),
        )
        .await?;
        // Only a request is answered with a message (the TransferProcess, DSP 9.3.1); a state
        // change gets a bare 200, whatever body the peer adds.
        let read = match (&domain.typed.message, &raw) {
            (TransferDSPMessageType::TransferRequestMessage, answer) if !answer.is_null() => {
                let message = RdfMessage::<DspTransfer>::read(answer.clone()).await?;
                json!({ "type": message.message.to_string(), "fields": message.fields, "raw": answer })
            }
            _ => json!({ "raw": raw }),
        };
        Ok(Some(read))
    }

    async fn apply(
        &self,
        cx: &FlowCx<'_, DspCaller>,
        domain: TransferRpcDomain,
        reply: Option<Value>,
    ) -> Outcome<TransferRpcOutcome> {
        use TransferDSPMessageType::*;
        let reply = reply.unwrap_or(Value::Null);
        let (ctx, persistence) = (self.ctx.as_ref(), &self.persistence);
        match domain.typed.message {
            TransferRequestMessage => {
                let user = cx.caller.user()?;
                RequestHandler::handle(ctx, persistence, user, domain, &reply).await
            }
            TransferStartMessage => StartHandler::handle(ctx, persistence, domain, &reply).await,
            TransferCompletionMessage => {
                CompletionHandler::handle(ctx, persistence, domain, &reply).await
            }
            TransferSuspensionMessage => {
                SuspensionHandler::handle(ctx, persistence, domain, &reply).await
            }
            TransferTerminationMessage => {
                TerminationHandler::handle(ctx, persistence, domain, &reply).await
            }
            other => Err(Errors::crazy(
                format!("{other} is not an outbound message"),
                None,
            )),
        }
    }
}
