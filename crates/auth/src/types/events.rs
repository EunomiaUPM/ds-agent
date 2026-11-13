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

//! Canonical event adapters for identity, verification, and credential issuance flows in `auth`.

use events::schemas::identity::{
    CredentialIssuancePayload, GrantAction, GrantDirection, IdentityGrantPayload, IssuanceAction,
    VerificationAction, VerificationDirection, VerificationPayload,
};
use events::{
    ActorType, EventActor, EventAttestation, EventContext, EventPayload, EventSinkTrait,
    TypedEventSink,
};
use tracing::warn;
use ymir::data::entities::{received, sent};
use ymir::oauth::Owner;
use ymir::types::{gnap::GrantStatus, vcs::VcTypeConfig, verification::VerificationStatus};

/// Stateless helper namespace translating `auth` domain models into typed `ymir-events` payloads.
pub struct AuthEvents;

impl AuthEvents {
    /// Emits a lifecycle event for a received GNAP grant request or update.
    pub async fn recv_grant(
        sink: &dyn EventSinkTrait,
        grant: &received::grant::Model,
        action: GrantAction,
        actor: Option<EventActor>,
        reason: Option<String>,
    ) {
        let payload = IdentityGrantPayload {
            action,
            grant_id: grant.id.clone(),
            direction: GrantDirection::Received,
            kind: grant.kind.to_string(),
            status: Self::grant_status_str(&grant.status).to_string(),
            previous_status: None,
            peer_participant_id: grant.participant_id.clone(),
            peer_participant_nick: Some(grant.participant_nick.clone()),
            vc_types: Self::vc_types(grant.vc_type_config.as_deref()),
            token_fingerprint: grant
                .final_token_hash
                .as_deref()
                .map(IdentityGrantPayload::fingerprint_token),
            expires_at: grant.final_expires_at,
            reason,
        };
        let actor = actor.or_else(|| {
            grant
                .participant_id
                .as_deref()
                .map(|did| EventActor::peer(did, None))
        });
        let mut ctx = EventContext::for_owner(Self::recv_owner(grant))
            .with_grant_id(&grant.id)
            .with_correlation(&grant.id);
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        if let Some(did) = grant.participant_id.as_deref() {
            ctx = ctx.with_peer_id(did);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a lifecycle event for an outgoing GNAP grant request, mapping string actions cleanly.
    pub async fn sent_grant_action(
        sink: &dyn EventSinkTrait,
        grant: &sent::grant::Model,
        action: &str,
    ) {
        let (mapped, reason) = Self::map_grant_action(action);
        Self::sent_grant(sink, grant, mapped, reason).await;
    }

    /// Emits a typed lifecycle event for an outgoing GNAP grant request.
    pub async fn sent_grant(
        sink: &dyn EventSinkTrait,
        grant: &sent::grant::Model,
        action: GrantAction,
        reason: Option<String>,
    ) {
        let payload = IdentityGrantPayload {
            action,
            grant_id: grant.id.clone(),
            direction: GrantDirection::Requested,
            kind: grant.kind.to_string(),
            status: Self::grant_status_str(&grant.status).to_string(),
            previous_status: None,
            peer_participant_id: Some(grant.participant_id.clone()),
            peer_participant_nick: Some(grant.participant_nick.clone()),
            vc_types: Self::vc_types(grant.vc_type_config.as_deref()),
            token_fingerprint: grant
                .final_token
                .as_deref()
                .map(IdentityGrantPayload::fingerprint_token),
            expires_at: grant.final_expires_at,
            reason,
        };
        let actor = EventActor {
            actor_type: ActorType::HumanUser,
            id: grant.user_id.clone(),
            participant_id: None,
            role: Some(grant.role.to_string()),
        };
        let ctx = EventContext::for_owner(Self::sent_owner(grant))
            .with_actor(actor)
            .with_grant_id(&grant.id)
            .with_correlation(&grant.id)
            .with_peer_id(&grant.participant_id);
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a lifecycle event for a received GNAP grant, mapping string actions cleanly.
    pub async fn recv_grant_action(
        sink: &dyn EventSinkTrait,
        grant: &received::grant::Model,
        action: &str,
        actor: Option<EventActor>,
    ) {
        let (mapped, reason) = Self::map_grant_action(action);
        Self::recv_grant(sink, grant, mapped, actor, reason).await;
    }

    /// Emits an incoming OID4VP verification event with fingerprinted VC material.
    pub async fn recv_verification(
        sink: &dyn EventSinkTrait,
        grant: &received::grant::Model,
        verification: &received::verification::Model,
        action: VerificationAction,
        failure_reason: Option<String>,
    ) {
        let presented_vc_ids = verification
            .vcs
            .iter()
            .map(|vc| IdentityGrantPayload::fingerprint_token(vc))
            .collect();
        let payload = VerificationPayload {
            action,
            grant_id: verification.grant_id.clone(),
            verification_id: verification.state.clone(),
            direction: VerificationDirection::Incoming,
            holder_did: verification.holder.clone(),
            verifier_did: None,
            vc_types: verification.vc_type.iter().map(ToString::to_string).collect(),
            presented_vc_ids,
            status: Self::verification_status_str(&verification.status).to_string(),
            failure_reason,
        };
        let mut ctx = EventContext::for_owner(Self::recv_owner(grant))
            .with_grant_id(&grant.id)
            .with_correlation(&grant.id);
        if let Some(holder) = verification.holder.as_deref() {
            ctx = ctx
                .with_actor(EventActor::peer(holder, None))
                .with_peer_id(holder);
            if matches!(action, VerificationAction::Verified) {
                if let Some(vpt) = verification.vpt.as_deref() {
                    ctx = ctx.with_attestation(EventAttestation::from_material_digest(
                        "jwt_vp_json",
                        holder,
                        vpt,
                    ));
                }
            }
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits an outgoing OID4VP presentation event for a client-initiated grant.
    pub async fn sent_verification(
        sink: &dyn EventSinkTrait,
        grant: &sent::grant::Model,
        verification: &sent::verification::Model,
        action: VerificationAction,
    ) {
        let payload = VerificationPayload {
            action,
            grant_id: verification.grant_id.clone(),
            verification_id: verification.id.clone(),
            direction: VerificationDirection::Outgoing,
            holder_did: None,
            verifier_did: Some(grant.participant_id.clone()),
            vc_types: Self::vc_types(grant.vc_type_config.as_deref()),
            presented_vc_ids: Vec::new(),
            status: Self::verification_status_str(&verification.status).to_string(),
            failure_reason: verification.reason.clone(),
        };
        let actor = EventActor {
            actor_type: ActorType::HumanUser,
            id: grant.user_id.clone(),
            participant_id: None,
            role: Some(grant.role.to_string()),
        };
        let ctx = EventContext::for_owner(Self::sent_owner(grant))
            .with_actor(actor)
            .with_grant_id(&grant.id)
            .with_correlation(&grant.id)
            .with_peer_id(&grant.participant_id);
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits an attestation issuance event when self-attested Gaia credentials are stored.
    pub async fn gaia_self_issued(sink: &dyn EventSinkTrait, vc_types: Vec<String>) {
        let payload = CredentialIssuancePayload {
            action: IssuanceAction::Issued,
            grant_id: "gaia-self-attestation".to_string(),
            credential_id: "gaia-self-attested".to_string(),
            issuer_did: None,
            holder_did: None,
            subject_name: None,
            vc_types,
            issued: true,
            format: Some("JwtVcJson".to_string()),
        };
        let ctx = EventContext::for_owner(Owner::connector()).with_actor(EventActor::system());
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Maps legacy `auth` string actions onto canonical `GrantAction` variants and details.
    pub fn map_grant_action(action: &str) -> (GrantAction, Option<String>) {
        match action {
            "requested" => (GrantAction::Created, None),
            "approved" => (GrantAction::Approved, None),
            "rejected" => (GrantAction::Rejected, None),
            "modification-rejected" => (
                GrantAction::Rejected,
                Some("modification rejected".to_string()),
            ),
            "revoked" => (GrantAction::Revoked, None),
            "disconnected" => (GrantAction::Revoked, Some("disconnected".to_string())),
            "finalized" => (GrantAction::Finalized, None),
            "issued" => (GrantAction::Finalized, Some("credential issued".to_string())),
            "rotated" => (GrantAction::Updated, Some("token rotated".to_string())),
            "modified" => (GrantAction::Updated, Some("grant modified".to_string())),
            "modifying" => (
                GrantAction::Updated,
                Some("modification pending".to_string()),
            ),
            "renewing" => (GrantAction::Updated, Some("token renewing".to_string())),
            other => (GrantAction::Updated, Some(other.to_string())),
        }
    }

    /// Computes ownership visibility for an incoming received grant.
    pub fn recv_owner(grant: &received::grant::Model) -> Owner {
        Owner::team(grant.role.clone(), grant.visibility.clone())
    }

    /// Computes ownership visibility for an outgoing sent grant.
    pub fn sent_owner(grant: &sent::grant::Model) -> Owner {
        Owner::new(
            grant.user_id.clone(),
            grant.role.clone(),
            grant.visibility.clone(),
        )
    }

    fn vc_types(cfgs: Option<&[VcTypeConfig]>) -> Vec<String> {
        cfgs.map(|items| items.iter().map(ToString::to_string).collect())
            .unwrap_or_default()
    }

    fn grant_status_str(status: &GrantStatus) -> &'static str {
        match status {
            GrantStatus::Processing => "Processing",
            GrantStatus::Pending => "Pending",
            GrantStatus::Approved => "Approved",
            GrantStatus::Finalized => "Finalized",
        }
    }

    fn verification_status_str(status: &VerificationStatus) -> &'static str {
        match status {
            VerificationStatus::Pending => "Pending",
            VerificationStatus::Verified => "Verified",
            VerificationStatus::Failed => "Failed",
            VerificationStatus::Refused => "Refused",
            VerificationStatus::Expired => "Expired",
        }
    }

    async fn emit_typed<P: EventPayload>(sink: &dyn EventSinkTrait, ctx: &EventContext, payload: &P) {
        if let Err(err) = sink.emit(ctx, payload).await {
            warn!(topic = %payload.topic(), error = %err, "auth event emit failed");
        }
    }
}
