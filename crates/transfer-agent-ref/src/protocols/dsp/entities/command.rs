use std::collections::BTreeMap;
use std::str::FromStr;

use bytes::Bytes;
use sha2::{Digest, Sha256};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::entities::transfer_message::MessageEnvelope;
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot,
};
use crate::protocols::dsp::entities::context_dsp::TransferDSPContextDomain;
use crate::protocols::dsp::entities::context_rpc::TransferRPCContextDomain;
use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::entities::dataplane_signal::DataplaneSignal;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use common::dsp_common::data_address::{DataAddress, EndpointProperty};

/// Command data for manager.
/// Each context such as DSP or RPC contexts, even Dataplane signaling turns
/// into a TransferManagerCommand when it comes to reach the manager
/// It works as a more suitable and unified view for the Manager

/// The distilled, normalized input the manager runs its template over.
/// Every source (inbound DSP, outbound RPC, dataplane signal) produces one of these
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
        let raw = self.typed.rdf.parsed.raw.body_bytes.clone();
        let canonical = Some((
            Bytes::from(self.typed.rdf.canonical_n_quads.clone().into_bytes()),
            self.typed.rdf.canonical_hash,
        ));
        Ok(TransferManagerCommand {
            trigger: TransferTransitionTrigger::Dsp(self.typed.message.clone()),
            direction: TransferCommandDirection::Inbound,
            role: self.role,
            transfer_direction: self.transfer_direction,
            data_address: self.typed.data_address.clone(),
            is_restart: self.is_restart,
            agreement_id: Some(self.agreement.id.clone()),
            envelope: build_envelope(raw, "application/ld+json", canonical),
            connector_instance: self.connector_instance,
            process: self.process,
        })
    }
}

impl ExtractCommand for TransferRPCContextDomain {
    fn extract(self) -> Outcome<TransferManagerCommand> {
        // RPC is plain JSON — no canonical form.
        let raw = Bytes::from(
            serde_json::to_vec(&self.typed.parsed.json_value).map_err(|_| {
                Errors::format(
                    BadFormat::Received,
                    "RPC body is not serializable JSON",
                    None,
                )
            })?,
        );
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
            data_address: self.typed.data_address.clone().map(dto_to_data_address),
            is_restart: self.is_restart,
            agreement_id,
            envelope: build_envelope(raw, "application/json", None),
            connector_instance: self.connector_instance,
            process: self.process,
        })
    }
}

// Helpers

/// Assemble a [`MessageEnvelope`] from the raw bytes and an optional canonical
/// form. Headers/signature are left empty here; the persist stage fills them if
/// the protocol requires signing.
fn build_envelope(
    raw_bytes: Bytes,
    content_type: &str,
    canonical: Option<(Bytes, [u8; 32])>,
) -> MessageEnvelope {
    let content_hash: [u8; 32] = Sha256::digest(&raw_bytes).into();
    let (canonical_form, canonical_hash) = match canonical {
        Some((form, hash)) => (Some(form), Some(hash)),
        None => (None, None),
    };
    MessageEnvelope {
        raw_bytes,
        content_type: content_type.into(),
        content_hash,
        headers: BTreeMap::new(),
        canonical_form,
        canonical_hash,
        signature: None,
    }
}

/// Widen the RPC data-plane DTO into the common wire `DataAddress`. Lossy on the
/// `@type` tags (not present in the DTO); defaulted, since only the endpoint and
/// properties matter downstream.
fn dto_to_data_address(dto: DataAddressDto) -> DataAddress {
    DataAddress {
        _type: "DataAddress".to_string(),
        endpoint_type: dto.endpoint_type,
        endpoint: dto.endpoint.unwrap_or_default(),
        endpoint_properties: dto
            .endpoint_properties
            .unwrap_or_default()
            .into_iter()
            .map(|p| EndpointProperty {
                _type: "EndpointProperty".to_string(),
                name: p.name,
                value: p.value,
            })
            .collect(),
    }
}
