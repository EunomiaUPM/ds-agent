//! Atomic payload validators, ported from the mature `ValidatePayload`. Pure
//! functions over primitives — the command already carries the extracted fields,
//! so there is no `ValidationHelpers` re-extraction layer.

use std::str::FromStr;

use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;

/// Parse a value as a URN.
pub fn urn(value: &str) -> Outcome<Urn> {
    Urn::from_str(value).map_err(|e| Errors::parse(format!("value must be a URN: {e}"), None))
}

/// The pid the request URI carries (by role) must equal the body's pid — both
/// parsed as URNs so `urn:uuid:x` and a re-serialised form compare equal.
pub fn uri_and_pid(
    uri_id: &str,
    consumer_pid: Option<&str>,
    provider_pid: Option<&str>,
    role: &TransferRole,
) -> Outcome<()> {
    let body_pid = match role {
        TransferRole::Provider => provider_pid,
        TransferRole::Consumer => consumer_pid,
        TransferRole::Relay => return Err(Errors::parse("Relay has no pid", None)),
    }
    .ok_or_else(|| Errors::parse("body is missing the role's pid", None))?;

    if urn(uri_id)?.to_string() != urn(body_pid)?.to_string() {
        return Err(Errors::parse(
            "URI pid and body pid are not correlated",
            None,
        ));
    }
    Ok(())
}

/// The pids in the incoming message must match the stored process's pids.
pub fn correlation(
    process_consumer: Option<&str>,
    process_provider: Option<&str>,
    message_consumer: Option<&str>,
    message_provider: Option<&str>,
) -> Outcome<()> {
    if process_consumer != message_consumer || process_provider != message_provider {
        return Err(Errors::parse(
            "message pids and process pids are not correlated",
            None,
        ));
    }
    Ok(())
}

/// A `dataAddress` on a `TransferStart` is only valid from the provider on the
/// first start (`OnRequest`); a consumer sending one on a later start is invalid.
pub fn data_address_in_start(
    has_data_address: bool,
    role: &TransferRole,
    attribute: &TransferDSPStateAttribute,
) -> Outcome<()> {
    if has_data_address
        && matches!(role, TransferRole::Consumer)
        && !matches!(attribute, TransferDSPStateAttribute::OnRequest)
    {
        return Err(Errors::parse(
            "dataAddress is only allowed in the first provider TransferStart",
            None,
        ));
    }
    Ok(())
}

/// JSON-schema validation of the raw message body.
// ponytail: stub, exactly like the mature crate — wire a schema validator when
// the per-message schemas land.
pub fn json_schema(_body: &serde_json::Value) -> Outcome<()> {
    Ok(())
}

/// Authorization / token check against the peer store.
// ponytail: stub — the mature crate defers this too; wire when the peer/token
// store is available.
pub fn auth() -> Outcome<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_pid_correlation() {
        // provider role → matches provider pid
        assert!(
            uri_and_pid(
                "urn:uuid:pp",
                None,
                Some("urn:uuid:pp"),
                &TransferRole::Provider
            )
            .is_ok()
        );
        // mismatch
        assert!(
            uri_and_pid(
                "urn:uuid:pp",
                None,
                Some("urn:uuid:xx"),
                &TransferRole::Provider
            )
            .is_err()
        );
        // missing the role's pid
        assert!(uri_and_pid("urn:uuid:pp", None, None, &TransferRole::Provider).is_err());
    }

    #[test]
    fn pid_correlation() {
        assert!(correlation(Some("c"), Some("p"), Some("c"), Some("p")).is_ok());
        assert!(correlation(Some("c"), Some("p"), Some("c"), Some("x")).is_err());
        assert!(correlation(None, None, None, None).is_ok());
    }

    #[test]
    fn data_address_rule() {
        use TransferDSPStateAttribute::*;
        // provider first start with a data address → fine
        assert!(data_address_in_start(true, &TransferRole::Provider, &OnRequest).is_ok());
        // consumer with a data address on a non-first start → rejected
        assert!(data_address_in_start(true, &TransferRole::Consumer, &ByProvider).is_err());
        // consumer on the first start → fine
        assert!(data_address_in_start(true, &TransferRole::Consumer, &OnRequest).is_ok());
        // no data address → always fine
        assert!(data_address_in_start(false, &TransferRole::Consumer, &ByProvider).is_ok());
    }
}
