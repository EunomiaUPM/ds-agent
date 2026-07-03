use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::protocols::dsp::entities::auth::TransferRPCAuthn;
use crate::protocols::dsp::entities::context_common::{BuildAuthn, TransferContextRaw, header};
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot,
};
use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use common::facades::Mates;
use http::request::Parts;
use oauth::entities::user::User;
use serde::Deserialize;
use ymir::errors::{BadFormat, Errors, Outcome};

/// Context Created from RPC controllers
/// The context is used for validation at several steps, command for Manager creation

// TransferContextRaw --

impl BuildAuthn for TransferRPCAuthn {
    fn from_request_parts(parts: &Parts) -> Outcome<Self> {
        let me_participant = parts.extensions.get::<Mates>().cloned().ok_or_else(|| {
            Errors::crazy(
                "auth middleware did not resolve participant (Mates missing)",
                None,
            )
        })?;
        let me_user = parts.extensions.get::<User>().cloned().ok_or_else(|| {
            Errors::crazy("auth middleware did not resolve user (User missing)", None)
        })?;
        let raw = header(&parts.headers, "authorization").unwrap_or_default();
        let (token_type, token_content) = raw
            .split_once(' ')
            .map(|(t, c)| (t.to_string(), c.trim().to_string()))
            .unwrap_or_else(|| (String::new(), raw.clone()));
        Ok(TransferRPCAuthn {
            raw,
            token_type,
            token_content,
            me_participant,
            me_user,
        })
    }
}

// TransferRPCContextParsed --

#[derive(Debug)]
pub struct TransferRPCContextParsed {
    pub raw: TransferContextRaw<TransferRPCAuthn>,
    pub json_value: serde_json::Value,
}

impl TransferRPCContextParsed {
    pub fn from_raw(
        raw: TransferContextRaw<TransferRPCAuthn>,
        json_value: serde_json::Value,
    ) -> Outcome<Self> {
        Ok(Self { raw, json_value })
    }
}

// TransferRPCContextTyped --

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RpcMessageFields {
    consumer_pid: Option<String>,
    provider_pid: Option<String>,
    agreement_id: Option<String>,
    data_address: Option<DataAddressDto>,
    format: Option<String>,
    /// Routing fields consumed locally, never forwarded on the wire.
    provider_address: Option<String>,
    callback_address: Option<String>,
    associated_agent_peer: Option<String>,
}

#[derive(Debug)]
pub struct TransferRPCContextTyped {
    pub parsed: TransferRPCContextParsed,
    pub message: TransferDSPMessageType,
    pub consumer_pid: Option<String>,
    pub provider_pid: Option<String>,
    pub data_address: Option<DataAddressDto>,
    pub agreement_id: Option<String>,
    pub format: Option<String>,
    pub provider_address: Option<String>,
    pub callback_address: Option<String>,
    pub associated_agent_peer: Option<String>,
}

impl TransferRPCContextTyped {
    /// Deserialize the RPC body into its fields. One serde pass — the whole point
    /// of RPC being plain JSON.
    pub fn from_parsed(
        parsed: TransferRPCContextParsed,
        message: TransferDSPMessageType,
    ) -> Outcome<Self> {
        let f: RpcMessageFields =
            serde_json::from_value(parsed.json_value.clone()).map_err(|e| {
                Errors::format(BadFormat::Received, format!("invalid RPC body: {e}"), None)
            })?;
        Ok(Self {
            parsed,
            message,
            consumer_pid: f.consumer_pid,
            provider_pid: f.provider_pid,
            data_address: f.data_address,
            agreement_id: f.agreement_id,
            format: f.format,
            provider_address: f.provider_address,
            callback_address: f.callback_address,
            associated_agent_peer: f.associated_agent_peer,
        })
    }
}

// TransferRPCContextDomain --

#[derive(Debug)]
pub struct TransferRPCContextDomain {
    pub typed: TransferRPCContextTyped,
    pub process: TransferContextProcessSlot,
    pub role: TransferRole,
    pub transfer_direction: TransferDirection,
    pub connector_instance: TransferContextConnectorRole,
    pub is_restart: bool,
}

impl TransferRPCContextDomain {
    pub fn from_typed(
        typed: TransferRPCContextTyped,
        process: TransferContextProcessSlot,
        role: TransferRole,
        transfer_direction: TransferDirection,
        connector_instance: TransferContextConnectorRole,
        is_restart: bool,
    ) -> Outcome<Self> {
        Ok(Self {
            typed,
            process,
            role,
            transfer_direction,
            connector_instance,
            is_restart,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serde_extraction_pulls_request_fields_and_tolerates_missing() {
        // A TransferRequest-shaped RPC body: routing + agreement, no pids yet.
        let body = json!({
            "agreementId": "urn:uuid:agr",
            "format": "HttpData",
            "providerAddress": "https://provider.example/dsp",
            "callbackAddress": "https://me.example/cb",
            "associatedAgentPeer": "urn:peer:provider",
            "somethingWeIgnore": true
        });
        let f: RpcMessageFields = serde_json::from_value(body).unwrap();
        assert_eq!(f.agreement_id.as_deref(), Some("urn:uuid:agr"));
        assert_eq!(
            f.provider_address.as_deref(),
            Some("https://provider.example/dsp")
        );
        assert!(f.consumer_pid.is_none()); // minted later, not in the request body
        assert!(f.data_address.is_none());
    }
}
