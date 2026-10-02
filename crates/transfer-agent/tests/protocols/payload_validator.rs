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

//! Payload rules: pid correlation and when a data address may be sent.

use transfer_agent::entities::protocol::TransferRole;
use transfer_agent::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;

use PayloadValidator as V;
use transfer_agent::protocols::dsp::services::validator::payload::*;

/// as provider, the uri must match the provider pid
#[test]
fn uri_pid_correlation() {
    assert!(
        V::uri_and_pid(
            "urn:uuid:pp",
            None,
            Some("urn:uuid:pp"),
            &TransferRole::Provider
        )
        .is_ok()
    );
    // mismatch
    assert!(
        V::uri_and_pid(
            "urn:uuid:pp",
            None,
            Some("urn:uuid:xx"),
            &TransferRole::Provider
        )
        .is_err()
    );
    // missing the role's pid
    assert!(V::uri_and_pid("urn:uuid:pp", None, None, &TransferRole::Provider).is_err());
}

/// Path and body pids must match when both are present.
#[test]
fn pid_correlation() {
    assert!(V::correlation(Some("c"), Some("p"), Some("c"), Some("p")).is_ok());
    assert!(V::correlation(Some("c"), Some("p"), Some("c"), Some("x")).is_err());
    assert!(V::correlation(None, None, None, None).is_ok());
}

/// A data address may come on the first start from either role, but a consumer may not
/// send one on a later start.
#[test]
fn data_address_rule() {
    use TransferDSPStateAttribute::*;
    // a provider may send a data address on the first start
    assert!(V::data_address_in_start(true, &TransferRole::Provider, &OnRequest).is_ok());
    // a consumer may not send one on a later start
    assert!(V::data_address_in_start(true, &TransferRole::Consumer, &ByProvider).is_err());
    // a consumer may send one on the first start
    assert!(V::data_address_in_start(true, &TransferRole::Consumer, &OnRequest).is_ok());
    // without a data address the rule never fails
    assert!(V::data_address_in_start(false, &TransferRole::Consumer, &ByProvider).is_ok());
}
