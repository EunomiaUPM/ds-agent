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

use chrono::{DateTime, Utc};
use common::oauth::Owner;
use serde::Serialize;
use ymir::data::entities::{received, sent};
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::GrantStatus;
use ymir::types::vcs::VcType;
use ymir::types::verification::VerificationStatus;
use ymir::utils::hash_token;

#[derive(Serialize)]
pub struct GrantEvent {
    pub grant_id: String,
    pub kind: GrantKind,
    pub participant_id: Option<String>,
    pub participant_nick: String,
    pub status: GrantStatus,
    pub token_fingerprint: Option<String>,
    pub final_expires_at: Option<DateTime<Utc>>,
    pub managing_expires_at: Option<DateTime<Utc>>,
}

impl From<&received::grant::Model> for GrantEvent {
    fn from(grant: &received::grant::Model) -> Self {
        Self {
            grant_id: grant.id.clone(),
            kind: grant.kind.clone(),
            participant_id: grant.participant_id.clone(),
            participant_nick: grant.participant_nick.clone(),
            status: grant.status.clone(),
            token_fingerprint: grant.final_token_hash.clone(),
            final_expires_at: grant.final_expires_at,
            managing_expires_at: grant.managing_expires_at,
        }
    }
}

impl From<&sent::grant::Model> for GrantEvent {
    fn from(grant: &sent::grant::Model) -> Self {
        Self {
            grant_id: grant.id.clone(),
            kind: grant.kind.clone(),
            participant_id: Some(grant.participant_id.clone()),
            participant_nick: grant.participant_nick.clone(),
            status: grant.status.clone(),
            token_fingerprint: grant.final_token.as_deref().map(hash_token),
            final_expires_at: grant.final_expires_at,
            managing_expires_at: grant.managing_expires_at,
        }
    }
}

#[derive(Serialize)]
pub struct VerificationEvent {
    pub grant_id: String,
    pub holder: Option<String>,
    pub vc_types: Vec<VcType>,
    pub status: VerificationStatus,
}

impl From<&received::verification::Model> for VerificationEvent {
    fn from(verification: &received::verification::Model) -> Self {
        Self {
            grant_id: verification.id.clone(),
            holder: verification.holder.clone(),
            vc_types: verification.vc_type.clone(),
            status: verification.status.clone(),
        }
    }
}

impl From<&sent::verification::Model> for VerificationEvent {
    fn from(verification: &sent::verification::Model) -> Self {
        Self {
            grant_id: verification.id.clone(),
            holder: None,
            vc_types: Vec::new(),
            status: verification.status.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct GaiaIssuedEvent {
    pub credentials: Vec<String>,
}

pub fn recv_owner(grant: &received::grant::Model) -> Owner {
    Owner::team(grant.role.clone(), grant.visibility.clone())
}

pub fn sent_owner(grant: &sent::grant::Model) -> Owner {
    Owner::new(
        grant.user_id.clone(),
        grant.role.clone(),
        grant.visibility.clone(),
    )
}
