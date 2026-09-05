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

//! The inbound DSP pipeline, as a template. Each stage consumes the previous
//! context, so `Raw -> Parsed -> Rdf -> Typed -> Domain` is enforced by types.

use axum::extract::Request;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::protocol::{ProtocolId, TransferDirection, TransferRole};
use crate::protocols::dsp::entities::auth::TransferDSPAuthn;
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot, TransferContextRaw,
};
use crate::protocols::dsp::entities::context_dsp::{
    TransferDSPContextDomain, TransferDSPContextParsed, TransferDSPContextRdf,
    TransferDSPContextTyped,
};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::http::dsp::DspRouter;

#[async_trait::async_trait]
pub trait DSPHandlerPipeline: Send + Sync + 'static {
    /// Read body and headers, and pick up the participant the auth middleware
    /// resolved. Consumes the request: a body can only be read once.
    async fn extract_wire(request: Request) -> Outcome<TransferContextRaw<TransferDSPAuthn>> {
        TransferContextRaw::<TransferDSPAuthn>::from_request(request).await
    }

    /// Read the body as JSON and pair it with what the route already settled:
    /// the protocol version and the message type this endpoint handles.
    fn parse(
        raw: TransferContextRaw<TransferDSPAuthn>,
        message_route: &TransferDSPMessageType,
        protocol_id: &ProtocolId,
    ) -> Outcome<TransferDSPContextParsed> {
        let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).map_err(|e| {
            Errors::format(BadFormat::Received, format!("body is not JSON: {e}"), None)
        })?;
        TransferDSPContextParsed::from_raw(raw, protocol_id, message_route, json)
    }

    /// Expand to RDF once, keeping both products: the expanded document to read
    /// fields from, and the canonical n-quads the content hash is taken over.
    async fn extract_rdf(parsed: TransferDSPContextParsed) -> Outcome<TransferDSPContextRdf> {
        TransferDSPContextRdf::from_parsed(parsed).await
    }

    /// Pull the typed fields out of the expanded document and derive the
    /// idempotency key. Synchronous — it reads a value already in memory, and
    /// declaring it `async` would box a future for nothing.
    fn extract_typed(rdf: TransferDSPContextRdf) -> Outcome<TransferDSPContextTyped> {
        TransferDSPContextTyped::from_rdf(rdf)
    }

    /// Resolve the process, agreement, connector and role. The only required
    /// stage, because it is the one that varies; `path_id` is `None` on `/request`.
    async fn to_domain(
        typed: TransferDSPContextTyped,
        path_id: Option<String>,
        // TODO(loader): returns `Outcome<TransferDSPContextDomain>` once wired.
    ) -> Outcome<()> {
        Ok(())
    }

    async fn run(
        request: Request,
        message_route: &TransferDSPMessageType,
        protocol_id: &ProtocolId,
    ) -> Outcome<TransferDSPContextDomain> {
        let raw = Self::extract_wire(request).await?;
        let parsed = Self::parse(raw, message_route, protocol_id)?;
        let rdf = Self::extract_rdf(parsed).await.or_else(|err| {
            dbg!("Err: {}", &err.reason());
            Err(err)
        })?;
        let typed = Self::extract_typed(rdf)?;
        //Self::to_domain(typed, None).await
        Ok(TransferDSPContextDomain {
            typed,
            process: TransferContextProcessSlot::New {
                consumer_pid: "".to_string(),
            },
            agreement: Default::default(),
            role: TransferRole::Provider,
            transfer_direction: TransferDirection::Push,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            is_restart: false,
            is_idempotent_replay: false,
            resolved_data_address: None,
        })
    }
}

/// `DspRouter` is the pipeline implementor; every stage but `to_domain` comes
/// from the default bodies above.
#[async_trait::async_trait]
impl DSPHandlerPipeline for DspRouter {
    async fn to_domain(_typed: TransferDSPContextTyped, _path_id: Option<String>) -> Outcome<()> {
        // TODO(loader): resolve process, agreement, connector and role. Needs the
        // repositories, so this signature grows a deps parameter when it lands.
        Ok(())
    }
}
