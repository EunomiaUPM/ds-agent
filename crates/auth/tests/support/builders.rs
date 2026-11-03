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

//! Records of the auth flows as the repositories would return them.
//!
//! Sent grants belong to a user (`user_id` + `role`); received grants only have the role that
//! handles them. Resource requests hang 1:1 from their grant by id; interactions and
//! verifications by `grant_id` (here, the same id).

use chrono::Utc;
use serde_json::json;
use ymir::data::entities::received;
use ymir::data::entities::sent;
use ymir::data::entities::shared::{participant, participant_relation, resource_req};
use ymir::data::entities::wallet::vc;
use ymir::types::gnap::access_token::{BoundToken, TokenManagement};
use ymir::types::gnap::grant_request::access::ResourceAccess;
use ymir::types::gnap::grant_request::client::{Client, ClientKey, KeyProof};
use ymir::types::gnap::grant_request::interact::{
    FinishMethod, HashMethod, InteractStart,
};
use ymir::types::gnap::grant_request::{GrantKind, GrantRequest};
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::GrantStatus;
use ymir::types::issuance::VcBody;
// use ymir::types::vcs::VCJwtClaims;
use ymir::types::jwt::{Jwt, JwtClaims};
use ymir::types::vcs::VcPayload;
use ymir::types::keys::DbKeySource;
use ymir::types::oauth::RolePath;
use ymir::types::participants::{ParticipantType, Visibility};
use ymir::types::vcs::{VcFormat, VcType, VcTypeConfig};
use ymir::types::verification::VerificationStatus;
use ymir::utils::encode_url_safe_no_pad;

use auth::types::entities::{ReachAuthority, ReachProvider};

/// Role path from a literal of the tests.
pub fn role(path: &str) -> RolePath {
    path.parse().expect("valid role path")
}

// ==========================================================================================
// Sent grants (ours): access tokens and VC requests, owned by a user
// ==========================================================================================

/// Access-token grant `user_id` (under `role_path`) sent to the peer; `auto` presents without
/// manual steps.
pub fn sent_grant(user_id: &str, role_path: &str, id: &str, auto: bool) -> sent::grant::Model {
    sent::grant::Model {
        id: id.to_string(),
        role: role(role_path),
        user_id: user_id.to_string(),
        username: None,
        participant_id: "did:web:peer".to_string(),
        participant_nick: "peer".to_string(),
        visibility: Visibility::Private,
        grant_endpoint: "http://peer/gnap/grant".to_string(),
        kind: GrantKind::AccessToken,
        status: GrantStatus::Pending,
        final_token: None,
        final_expires_at: None,
        managing_uri: None,
        managing_token: None,
        managing_expires_at: None,
        vc_type_config: None,
        vc_uri: None,
        as_assigned_id: None,
        auto,
        requested: true,
        created_at: Utc::now(),
        ended_at: None,
    }
}

/// VC request `user_id` (under `role_path`) sent to the authority; `auto` redeems the offer
/// without manual steps.
pub fn vc_request(user_id: &str, role_path: &str, id: &str, auto: bool) -> sent::grant::Model {
    sent::grant::Model {
        participant_id: "did:web:authority".to_string(),
        participant_nick: "authority".to_string(),
        visibility: Visibility::Public,
        kind: GrantKind::CredentialRequest,
        ..sent_grant(user_id, role_path, id, auto)
    }
}

pub fn sent_grant_plan(user_id: &str, role_path: &str, id: &str) -> sent::grant::Plan {
    sent::grant::Plan {
        id: id.to_string(),
        role: role(role_path),
        user_id: user_id.to_string(),
        username: None,
        participant_id: "did:web:peer".to_string(),
        participant_nick: "peer".to_string(),
        visibility: Visibility::Private,
        vc_type_config: None,
        grant_endpoint: "http://peer/gnap/grant".to_string(),
        kind: GrantKind::AccessToken,
        auto: Some(true),
        requested: true,
    }
}

pub fn sent_interaction(id: &str) -> sent::interaction::Model {
    sent::interaction::Model {
        id: id.to_string(),
        grant_id: id.to_string(),
        start: vec![InteractStart::Oid4VP],
        method: FinishMethod::Push,
        callback_uri: "http://me/callback".to_string(),
        client_nonce: "client-nonce".to_string(),
        hash_method: HashMethod::Sha256,
        hints: None,
        continuation_endpoint: None,
        continuation_token: None,
        continuation_wait: None,
        as_nonce: None,
        oidc_vp_uri: None,
        interact_ref: None,
        hash: None,
        created_at: Utc::now(),
    }
}

pub fn sent_interaction_plan(id: &str) -> sent::interaction::Plan {
    sent::interaction::Plan {
        grant_id: id.to_string(),
        start: vec![InteractStart::Oid4VP],
        method: FinishMethod::Push,
        callback_uri: "http://me/callback".to_string(),
        hash_method: None,
        hints: None,
    }
}

pub fn sent_verification(id: &str) -> sent::verification::Model {
    sent::verification::Model {
        id: id.to_string(),
        grant_id: id.to_string(),
        uri: "openid4vp://peer".to_string(),
        scheme: "openid4vp".to_string(),
        response_type: "vp_token".to_string(),
        client_id: "peer".to_string(),
        response_mode: "direct_post".to_string(),
        pd_uri: "http://peer/pd".to_string(),
        client_id_scheme: "redirect_uri".to_string(),
        nonce: "nonce".to_string(),
        response_uri: "http://peer/verify".to_string(),
        status: VerificationStatus::Pending,
        created_at: Utc::now(),
        ended_at: None,
    }
}

pub fn sent_verification_plan(id: &str) -> sent::verification::Plan {
    sent::verification::Plan {
        grant_id: id.to_string(),
        uri: "openid4vp://peer".to_string(),
        scheme: "openid4vp".to_string(),
        response_type: "vp_token".to_string(),
        client_id: "peer".to_string(),
        response_mode: "direct_post".to_string(),
        pd_uri: "http://peer/pd".to_string(),
        client_id_scheme: "redirect_uri".to_string(),
        nonce: "nonce".to_string(),
        response_uri: "http://peer/verify".to_string(),
    }
}

pub fn issued_token_response(token: &str, req: &resource_req::Model) -> GrantResponse {
    let managing = TokenManagement::new("http://peer/gate/token/m-1", BoundToken::new("managing"));
    GrantResponse::token_issued(token, req, 3600, managing, None)
}

pub fn resource_req(id: &str) -> resource_req::Model {
    resource_req::Model {
        id: id.to_string(),
        access: vec![ResourceAccess::datasets_of("http://peer/dsp/current/catalog/datasets")],
        granted: None,
        label: None,
        flags: None,
    }
}

// ==========================================================================================
// Received grants (a peer's): handled by a role
// ==========================================================================================

/// Grant a peer sent us, handled by `role_path`, public and not approved yet.
pub fn recv_grant(role_path: &str, id: &str) -> received::grant::Model {
    received::grant::Model {
        id: id.to_string(),
        role: role(role_path),
        visibility: Visibility::Public,
        participant_nick: "peer".to_string(),
        participant_id: None,
        kind: GrantKind::AccessToken,
        final_token_hash: None,
        final_expires_at: None,
        managing_id: None,
        managing_token_hash: None,
        managing_expires_at: None,
        vc_type_config: None,
        status: GrantStatus::Pending,
        created_at: Utc::now(),
        ended_at: None,
    }
}

pub fn recv_grant_plan(role_path: &str, id: &str) -> received::grant::Plan {
    received::grant::Plan {
        id: id.to_string(),
        role: role(role_path),
        visibility: Visibility::Public,
        participant_nick: "peer".to_string(),
        vc_type_config: None,
        kind: GrantKind::AccessToken,
    }
}

pub fn recv_interaction(id: &str) -> received::interaction::Model {
    received::interaction::Model {
        id: id.to_string(),
        grant_id: id.to_string(),
        start: vec![InteractStart::Oid4VP],
        method: FinishMethod::Push,
        callback_uri: "http://peer/callback".to_string(),
        key_source: DbKeySource::PublicKey(json!({})),
        client_nonce: "client-nonce".to_string(),
        hash_method: HashMethod::Sha256,
        hints: None,
        continuation_endpoint: "http://me/gnap/continue/cont-1".to_string(),
        continuation_id: "cont-1".to_string(),
        continuation_token: "cont-token".to_string(),
        continuation_wait: None,
        as_nonce: "as-nonce".to_string(),
        interact_ref: "interact-ref".to_string(),
        hash: "hash".to_string(),
        created_at: Utc::now(),
    }
}

pub fn recv_interaction_plan(id: &str) -> received::interaction::Plan {
    received::interaction::Plan {
        grant_id: id.to_string(),
        start: vec![InteractStart::Oid4VP],
        method: FinishMethod::Push,
        callback_uri: "http://peer/callback".to_string(),
        key_source: DbKeySource::PublicKey(json!({})),
        client_nonce: "client-nonce".to_string(),
        hash_method: None,
        hints: None,
        grant_endpoint: "http://me/gnap/grant".to_string(),
        continuation_endpoint: "http://me/gnap/continue/cont-1".to_string(),
        continuation_id: None,
        continuation_token: "cont-token".to_string(),
        continuation_wait: None,
    }
}

/// Verification of a presentation; `holder` is set once the peer presented.
pub fn recv_verification(id: &str, holder: Option<&str>) -> received::verification::Model {
    received::verification::Model {
        id: id.to_string(),
        grant_id: id.to_string(),
        state: "state-1".to_string(),
        nonce: "nonce".to_string(),
        vc_type: vec![VcType::LegalPerson],
        audience: "http://me/verify".to_string(),
        holder: holder.map(str::to_string),
        vpt: None,
        vcs: vec![],
        status: VerificationStatus::Pending,
        created_at: Utc::now(),
        ended_at: None,
    }
}

pub fn recv_verification_plan(id: &str) -> received::verification::Plan {
    received::verification::Plan {
        grant_id: id.to_string(),
        audience: "http://me/verify".to_string(),
        vc_type: vec![VcType::LegalPerson],
    }
}

// ==========================================================================================
// Participants and who added them
// ==========================================================================================

/// A participant, global to the connector.
pub fn participant(id: &str) -> participant::Model {
    participant::Model {
        participant_id: id.to_string(),
        participant_nick: "peer".to_string(),
        participant_type: ParticipantType::Agent,
        base_url: "http://peer".to_string(),
        saved_at: Utc::now(),
        last_interaction: Utc::now(),
        extra_fields: json!({"color": "blue"}),
    }
}

pub fn participant_plan(id: &str) -> participant::Plan {
    participant::Plan {
        participant_id: id.to_string(),
        participant_nick: "peer".to_string(),
        participant_type: ParticipantType::Agent,
        base_url: "http://peer".to_string(),
        extra_fields: None,
    }
}

/// Relation of `user_id` (under `role_path`) with participant `id`, private.
pub fn relation(user_id: &str, role_path: &str, id: &str) -> participant_relation::Model {
    participant_relation::Model {
        user_id: user_id.to_string(),
        participant_id: id.to_string(),
        username: None,
        role: role(role_path),
        visibility: Visibility::Private,
    }
}

// ==========================================================================================
// Payloads
// ==========================================================================================

pub fn reach_provider() -> ReachProvider {
    ReachProvider {
        id: "did:web:peer".to_string(),
        nick: "peer".to_string(),
        url: "http://peer".to_string(),
        access: vec![ResourceAccess::datasets_of("http://peer/dsp/current/catalog/datasets")],
        visibility: Visibility::Private,
        auto: Some(true),
        requested: None,
    }
}

pub fn reach_authority() -> ReachAuthority {
    ReachAuthority {
        id: "did:web:authority".to_string(),
        nick: "authority".to_string(),
        url: "http://authority".to_string(),
        vc_type: VcTypeConfig::new(VcType::LegalPerson, VcFormat::JwtVcJson),
        method: InteractStart::Oid4VP,
        auto: Some(true),
    }
}

/// Access token request a peer sends, as the gatekeeper parses it.
pub fn grant_request() -> GrantRequest {
    let client = Client {
        key: ClientKey::jwk(KeyProof::HttpSig, json!({})),
        class_id: Some("agent".to_string()),
        display: None,
    };
    GrantRequest::new_token(
        client,
        vec![ResourceAccess::datasets_of("http://me/dsp/current/catalog/datasets")],
        &sent_interaction("g-1"),
    )
}

/// Claims of a self-attested credential, before signing.
pub fn vc_claims(kind: &str) -> JwtClaims<VcPayload> {
    serde_json::from_value(json!({
        "@context": ["https://www.w3.org/ns/credentials/v2"],
        "id": format!("urn:vc:{kind}"),
        "type": ["VerifiableCredential", kind],
        "issuer": "did:web:me",
        "credentialSubject": {"id": "did:web:me"}
    }))
    .expect("valid VC claims")
}

/// `claims` as a JWT with a fake signature (`signed`), as the issuer double returns them.
pub fn signed_vc(claims: &JwtClaims<VcPayload>) -> Jwt<VcPayload> {
    let header = json!({"alg": "EdDSA", "typ": "vc+jwt", "kid": "did:web:me#key-1"});
    let jwt = format!(
        "{}.{}.{}",
        encode_url_safe_no_pad(header.to_string()),
        encode_url_safe_no_pad(serde_json::to_string(claims).expect("serializable claims")),
        encode_url_safe_no_pad("signed"),
    );
    Jwt::parse(&jwt).expect("valid JWT")
}

/// A credential as the wallet stores it.
pub fn stored_vc() -> vc::Model {
    vc::Model {
        id: "vc-1".to_string(),
        vc_body: VcBody::Jwt("signed.jwt".to_string()),
        vc_type: VcType::LegalPerson,
        vc_format: VcFormat::JwtVcJson,
        holder_did: "did:web:me".to_string(),
        issuer_did: "did:web:me".to_string(),
        parsed_document: json!({}),
        valid_until: None,
        added_on: Utc::now(),
    }
}
