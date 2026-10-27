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

//! The outbound flow's HTTP adapter: the eunomia body through its stages in, the
//! `RpcTransferResult` or a `TransferError` out.

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use axum::Json;
use common::dsp_common::DspCaller;
use common::flow::{FlowAdapterTrait, FlowError};
use http::StatusCode;

use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::transfer_error::TransferErrorDto;
use crate::protocols::dsp::entities::context_rpc::{
    TransferRpcOutcome, TransferRpcRaw, TransferRpcRdf, TransferRpcTyped,
};
use crate::protocols::dsp::services::rpc_flow::service::TransferRpcFlow;
use crate::protocols::dsp::services::rpc_flow::views::TransferRpcResult;

/// Extracts the RPC body for the route's message and answers with the result.
pub struct TransferRpcAdapter;

#[async_trait::async_trait]
impl FlowAdapterTrait<TransferRpcFlow> for TransferRpcAdapter {
    type Raw = Request;
    type Response = Response;
    /// The caller names processes by id, so the error carries no pids.
    type ErrorContext = ();

    /// The user OAuth resolved, then the JSON body, expanded through the eunomia context and
    /// read into its fields.
    async fn extract(
        &self,
        key: &TransferDSPMessageType,
        raw: Request,
    ) -> Result<(DspCaller, TransferRpcTyped), FlowError> {
        let caller = DspCaller::from_extensions(raw.extensions())?;
        caller.user()?;
        let raw = TransferRpcRaw::from_request(key, raw).await?;
        let rdf = TransferRpcRdf::from_raw(raw).await?;
        Ok((caller, TransferRpcTyped::from_rdf(rdf)?))
    }

    fn error_context(&self, _input: &TransferRpcTyped) {}

    /// 201 when a request opened a process, 200 for a state change.
    fn respond(&self, key: &TransferDSPMessageType, output: TransferRpcOutcome) -> Response {
        let status = match key {
            TransferDSPMessageType::TransferRequestMessage => StatusCode::CREATED,
            _ => StatusCode::OK,
        };
        (status, Json(TransferRpcResult::assemble(output))).into_response()
    }

    fn reject(
        &self,
        _key: &TransferDSPMessageType,
        _context: (),
        error: FlowError,
    ) -> Response {
        let status =
            StatusCode::from_u16(error.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = TransferErrorDto::new(
            String::new(),
            String::new(),
            status.as_u16().to_string(),
            error.reasons(),
        );
        (status, Json(body)).into_response()
    }
}
