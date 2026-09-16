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

use common::auth::AuthRules;
use common::dsp_common::DspRules;
use common::validation::codes;
use serde_json::json;

#[test]
fn dsp_rules_urn_validation() {
    assert!(DspRules::is_urn("urn:uuid:1234-5678", "pid").is_ok());
    assert!(DspRules::is_urn("urn:custom:item", "pid").is_ok());

    let malformed = DspRules::is_urn("not-a-urn", "pid").unwrap_err();
    assert_eq!(malformed.code(), Some(codes::MALFORMED));

    assert!(DspRules::is_urn_opt(Some("urn:uuid:abc"), "pid").is_ok());
    let missing = DspRules::is_urn_opt(None, "pid").unwrap_err();
    assert_eq!(missing.code(), Some(codes::MISSING));
}

#[test]
fn dsp_rules_pid_correlation() {
    assert!(DspRules::correlate_pids("urn:uuid:a", "urn:uuid:a", "pid").is_ok());

    let mismatch = DspRules::correlate_pids("urn:uuid:a", "urn:uuid:b", "pid").unwrap_err();
    assert_eq!(mismatch.code(), Some(codes::NOT_ALLOWED));
}

#[test]
fn dsp_rules_context_and_type() {
    let doc_str = json!({
        "@context": "https://w3id.org/dspace/2025/1/context.json",
        "@type": "dspace:TransferRequestMessage"
    });
    assert!(DspRules::valid_dsp_context(&doc_str, "@context").is_ok());
    assert!(DspRules::expected_type(&doc_str, "TransferRequestMessage", "@type").is_ok());

    let doc_arr = json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.json", "https://example.org/extra"],
        "@type": ["dspace:TransferRequestMessage", "CustomType"]
    });
    assert!(DspRules::valid_dsp_context(&doc_arr, "@context").is_ok());
    assert!(DspRules::expected_type(&doc_arr, "TransferRequestMessage", "@type").is_ok());

    let bad_ctx = json!({ "@context": "https://schema.org" });
    assert!(DspRules::valid_dsp_context(&bad_ctx, "@context").is_err());

    let bad_type = json!({ "@type": "TransferStartMessage" });
    assert!(DspRules::expected_type(&bad_type, "TransferRequestMessage", "@type").is_err());
}



#[test]
fn auth_rules_tokens_and_rbac() {
    let now = 1000;
    assert!(AuthRules::token_not_expired(1500, now, "exp").is_ok());
    let expired = AuthRules::token_not_expired(900, now, "exp").unwrap_err();
    assert_eq!(expired.code(), Some(codes::NOT_ALLOWED));

    assert!(AuthRules::audience_matches("https://agent.local", "https://agent.local", "aud").is_ok());
    assert!(AuthRules::audience_matches("https://other", "https://agent.local", "aud").is_err());

    assert!(AuthRules::issuer_matches("https://idp.local", "https://idp.local", "iss").is_ok());
    assert!(AuthRules::issuer_matches("https://fake", "https://idp.local", "iss").is_err());

    let roles = vec!["admin".to_string(), "reader".to_string()];
    assert!(AuthRules::has_role(&roles, "admin", "roles").is_ok());
    assert!(AuthRules::has_role(&roles, "writer", "roles").is_err());
}
