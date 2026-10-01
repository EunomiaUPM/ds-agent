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

//! The manager's input, and how each context distils itself into one. Unifies
//! inbound DSP, outbound RPC and data-plane signals behind a single shape.

use std::str::FromStr;

use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot,
};
use crate::protocols::dsp::entities::context_dsp::TransferDSPContextDomain;
use crate::protocols::dsp::entities::context_rpc::TransferRPCContextDomain;
use crate::protocols::dsp::entities::dataplane_signal::DataplaneSignal;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use common::dsp_common::data_address::DataAddress;

/// The distilled, normalized input the manager runs its template over. Every
/// source (inbound DSP, outbound RPC, dataplane signal) produces one of these.
#[derive(Debug)]
pub struct TransferManagerCommand {
    // identity / routing
    pub process: TransferContextProcessSlot,
    pub role: TransferRole,
    pub direction: TransferCommandDirection,
    pub trigger: TransferTransitionTrigger,
    // dataplane
    pub transfer_direction: TransferDirection,
    pub connector_instance: TransferContextConnectorRole,
    pub data_address: Option<DataAddress>,
    pub is_restart: bool,
    // persist + ack
    pub envelope: MessageEnvelope,
    pub agreement_id: Option<Urn>,
}

#[derive(Debug)]
pub enum TransferCommandDirection {
    Inbound,
    Outbound,
    Inner,
}

#[derive(Debug)]
pub enum TransferTransitionTrigger {
    Dsp(TransferDSPMessageType),
    DataplaneSignal(DataplaneSignal),
}

/// Each domain context knows how to distil itself into a [`TransferManagerCommand`].
pub trait ExtractCommand {
    fn extract(self) -> Outcome<TransferManagerCommand>;
}

impl ExtractCommand for TransferDSPContextDomain {
    fn extract(self) -> Outcome<TransferManagerCommand> {
        // DSP carries a real canonical form (n-quads + hash) from the RDF stage.
        let payload = self.typed.rdf.parsed.json_value.clone();
        let canonical = Some((
            self.typed.rdf.canonical_n_quads.clone(),
            self.typed.rdf.canonical_hash,
        ));
        Ok(TransferManagerCommand {
            trigger: TransferTransitionTrigger::Dsp(self.typed.message.clone()),
            direction: TransferCommandDirection::Inbound,
            role: self.role,
            transfer_direction: self.transfer_direction,
            data_address: self.typed.fields.data_address.clone(),
            is_restart: self.is_restart,
            agreement_id: Some(self.agreement.id.clone()),
            envelope: MessageEnvelope::new(payload, canonical),
            connector_instance: self.connector_instance,
            process: self.process,
        })
    }
}

impl ExtractCommand for TransferRPCContextDomain {
    fn extract(self) -> Outcome<TransferManagerCommand> {
        // RPC is plain JSON — no canonical form.
        let payload = self.typed.parsed.json_value.clone();
        let agreement_id = self
            .typed
            .agreement_id
            .as_deref()
            .map(Urn::from_str)
            .transpose()
            .map_err(|_| Errors::format(BadFormat::Received, "invalid agreementId URN", None))?;
        Ok(TransferManagerCommand {
            trigger: TransferTransitionTrigger::Dsp(self.typed.message.clone()),
            direction: TransferCommandDirection::Outbound,
            role: self.role,
            transfer_direction: self.transfer_direction,
            data_address: self.typed.data_address.clone().map(DataAddress::from),
            is_restart: self.is_restart,
            agreement_id,
            envelope: MessageEnvelope::new(payload, None),
            connector_instance: self.connector_instance,
            process: self.process,
        })
    }
}
