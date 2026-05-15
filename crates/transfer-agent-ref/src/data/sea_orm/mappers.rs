use std::str::FromStr;

use chrono::Utc;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::data::sea_orm::orm::transfer_identifier as ident_orm;
use crate::data::sea_orm::orm::transfer_message as msg_orm;
use crate::data::sea_orm::orm::transfer_process as proc_orm;
use crate::entities::commands::NewTransferProcessCommand;
use crate::entities::ids::{
    CorrelationId, MessageId, ParticipantId, RequestId, TenantId, TransferProcessId,
};
use crate::entities::message_envelope::{Direction, MessageEnvelopeRef};
use crate::entities::protocol::{
    ProtocolId, ProtocolMessageType, ProtocolState, StateMetadata, TransferCorrelation,
    TransferRole,
};
use crate::entities::transfer_message::{MessageEnvelope, MessageProcessingResult, TransferMessage};
use crate::entities::transfer_process::TransferProcess;
use crate::entities::transfer_process_identifier::TransferProcessIdentifier;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn deser_enum<T: for<'de> Deserialize<'de>>(s: &str) -> Outcome<T> {
    serde_json::from_str(&format!("\"{}\"", s))
        .map_err(|e| Errors::crazy("invalid enum value in database", Some(Box::new(e))))
}

fn ser_enum<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .expect("enum serialization never fails")
        .as_str()
        .expect("enum serializes to a string")
        .to_string()
}

fn ser_json<T: Serialize>(v: &T) -> Json {
    serde_json::to_value(v).expect("domain type serialization never fails")
}

fn deser_json<T: for<'de> Deserialize<'de>>(v: Json, field: &'static str) -> Outcome<T> {
    serde_json::from_value(v)
        .map_err(|e| Errors::crazy(field, Some(Box::new(e))))
}

fn parse_urn(s: &str, field: &'static str) -> Outcome<Urn> {
    Urn::from_str(s).map_err(|e| Errors::crazy(field, Some(Box::new(e))))
}

// ── TransferProcess ───────────────────────────────────────────────────────────

pub(crate) fn process_from_orm(m: proc_orm::Model) -> Outcome<TransferProcess> {
    let id = TransferProcessId::new(parse_urn(&m.id, "transfer_process.id")?);
    let tenant_id = TenantId::new(m.tenant_id);
    let role = deser_enum::<TransferRole>(&m.role)?;
    let created_at = m.created_at.with_timezone(&Utc);
    let updated_at = m.updated_at.with_timezone(&Utc);
    let version = m.version as u32;
    let protocol = deser_enum::<ProtocolId>(&m.protocol)?;
    let protocol_state = ProtocolState(CompactString::from(m.protocol_state));
    let state_metadata = deser_json::<StateMetadata>(m.state_metadata, "transfer_process.state_metadata")?;
    let correlation = deser_json::<TransferCorrelation>(m.correlation, "transfer_process.correlation")?;
    let properties = m.properties;
    let error_details = m.error_details;

    let last_inbound_envelope = m
        .last_inbound_envelope
        .map(|v| deser_json::<MessageEnvelopeRef>(v, "transfer_process.last_inbound_envelope"))
        .transpose()?;
    let last_outbound_envelope = m
        .last_outbound_envelope
        .map(|v| deser_json::<MessageEnvelopeRef>(v, "transfer_process.last_outbound_envelope"))
        .transpose()?;

    Ok(TransferProcess::rehydrate(
        id,
        tenant_id,
        role,
        created_at,
        updated_at,
        version,
        protocol,
        protocol_state,
        state_metadata,
        correlation,
        properties,
        error_details,
        last_inbound_envelope,
        last_outbound_envelope,
    ))
}

pub(crate) fn process_to_active_model(
    process: &TransferProcess,
) -> proc_orm::ActiveModel {
    use sea_orm::ActiveValue::Set;
    let correlation = process.correlation();
    proc_orm::ActiveModel {
        id: Set(process.id().to_string()),
        tenant_id: Set(process.tenant_id().as_str().to_string()),
        role: Set(ser_enum(&process.role())),
        created_at: Set(process.created_at().into()),
        updated_at: Set(process.updated_at().into()),
        version: Set(process.version() as i32),
        protocol: Set(ser_enum(process.protocol())),
        protocol_state: Set(process.state().0.to_string()),
        state_metadata: Set(ser_json(process.state_metadata())),
        agreement_id: Set(correlation.agreement_id.as_ref().map(|u| u.to_string())),
        peer_participant_id: Set(
            correlation.peer_participant_id.as_ref().map(|p| p.to_string()),
        ),
        correlation: Set(ser_json(correlation)),
        properties: Set(process.properties().clone()),
        error_details: Set(process.error_details().cloned()),
        last_inbound_envelope: Set(
            process.last_inbound_envelope().map(ser_json),
        ),
        last_outbound_envelope: Set(
            process.last_outbound_envelope().map(ser_json),
        ),
    }
}

pub(crate) fn process_from_cmd(cmd: &NewTransferProcessCommand) -> TransferProcess {
    let id = cmd.id.clone().unwrap_or_else(TransferProcessId::generate);
    let correlation = TransferCorrelation {
        identifiers: std::collections::HashMap::new(),
        consumer_pid: None,
        provider_pid: None,
        agreement_id: Some(cmd.agreement_id.clone()),
        callback_address: cmd.callback_address.clone(),
        peer_participant_id: Some(cmd.peer_participant_id.clone()),
    };

    let now = chrono::Utc::now();
    TransferProcess::rehydrate(
        id,
        cmd.tenant_id.clone(),
        cmd.role,
        now,
        now,
        0,
        cmd.protocol.clone(),
        cmd.initial_state.clone(),
        cmd.initial_state_metadata.clone(),
        correlation,
        cmd.properties.clone().unwrap_or(serde_json::json!({})),
        None,
        None,
        None,
    )
}

// ── TransferMessage ───────────────────────────────────────────────────────────

pub(crate) fn message_from_orm(m: msg_orm::Model) -> Outcome<TransferMessage> {
    let id = MessageId::new(parse_urn(&m.id, "transfer_message.id")?);
    let transfer_process_id =
        TransferProcessId::new(parse_urn(&m.transfer_process_id, "transfer_message.transfer_process_id")?);
    let tenant_id = TenantId::new(m.tenant_id);
    let direction = deser_enum::<Direction>(&m.direction)?;
    let protocol = deser_enum::<ProtocolId>(&m.protocol)?;
    let message_type = ProtocolMessageType(CompactString::from(m.message_type));
    let protocol_version = CompactString::from(m.protocol_version);
    let envelope = deser_json::<MessageEnvelope>(m.envelope, "transfer_message.envelope")?;
    let occurred_at = m.occurred_at.with_timezone(&Utc);
    let correlation_id = m.correlation_id.map(CorrelationId::new);
    let request_id = RequestId::new(m.request_id);
    let peer_participant_id =
        ParticipantId::new(parse_urn(&m.peer_participant_id, "transfer_message.peer_participant_id")?);
    let processing_result =
        deser_json::<MessageProcessingResult>(m.processing_result, "transfer_message.processing_result")?;

    Ok(TransferMessage {
        id,
        transfer_process_id,
        tenant_id,
        direction,
        protocol,
        message_type,
        protocol_version,
        envelope,
        occurred_at,
        correlation_id,
        request_id,
        peer_participant_id,
        processing_result,
    })
}

pub(crate) fn message_to_active_model(msg: &TransferMessage) -> msg_orm::ActiveModel {
    use sea_orm::ActiveValue::Set;
    let state_transition_to = msg.resulting_state().map(|s| s.0.to_string());
    msg_orm::ActiveModel {
        id: Set(msg.id().to_string()),
        transfer_process_id: Set(msg.transfer_process_id().to_string()),
        tenant_id: Set(msg.tenant_id().as_str().to_string()),
        direction: Set(ser_enum(&msg.direction())),
        protocol: Set(ser_enum(msg.protocol())),
        message_type: Set(msg.message_type().0.to_string()),
        protocol_version: Set(msg.protocol_version().to_string()),
        envelope: Set(ser_json(msg.envelope())),
        occurred_at: Set(msg.occurred_at().into()),
        correlation_id: Set(msg.correlation_id().map(|c| c.as_str().to_string())),
        request_id: Set(msg.request_id().as_str().to_string()),
        peer_participant_id: Set(msg.peer_participant_id().to_string()),
        processing_result: Set(ser_json(msg.processing_result())),
        state_transition_to: Set(state_transition_to),
    }
}

// ── TransferIdentifier ────────────────────────────────────────────────────────

pub(crate) fn identifier_from_orm(m: ident_orm::Model) -> Outcome<TransferProcessIdentifier> {
    let process_id = parse_urn(&m.transfer_process_id, "transfer_identifier.transfer_process_id")?;
    Ok(TransferProcessIdentifier {
        transfer_process_id: process_id,
        key: m.key,
        value: m.value,
    })
}

pub(crate) fn identifier_to_active_model(
    process_id: &Urn,
    ident: &TransferProcessIdentifier,
) -> ident_orm::ActiveModel {
    use sea_orm::ActiveValue::Set;
    ident_orm::ActiveModel {
        transfer_process_id: Set(process_id.to_string()),
        key: Set(ident.key.clone()),
        value: Set(ident.value.clone()),
    }
}
