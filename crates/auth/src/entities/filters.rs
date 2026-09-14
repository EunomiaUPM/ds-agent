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

//! Domain filters for ssi-auth queries.

use chrono::{DateTime, Utc};
use common::query::{validate_date_range, QueryFilter};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::GrantStatus;
use ymir::types::participants::ParticipantType;

/// Filter criteria for querying participants/mates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantFilter {
    #[serde(alias = "type")]
    pub r#type: Option<ParticipantType>,
    pub participant_nick: Option<String>,
    pub participant_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "common::paginated_spec::deserialize_opt_bool_from_str_or_bool"
    )]
    pub exclude_myself: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for ParticipantFilter {
    fn is_empty(&self) -> bool {
        self.r#type.is_none()
            && self.participant_nick.is_none()
            && self.participant_id.is_none()
            && self.exclude_myself.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Deserializes optional grant status case-insensitively.
pub fn deserialize_opt_grant_status<'de, D>(
    deserializer: D,
) -> Result<Option<GrantStatus>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None => Ok(None),
        Some(s) => match s.to_lowercase().as_str() {
            "approved" => Ok(Some(GrantStatus::Approved)),
            "pending" => Ok(Some(GrantStatus::Pending)),
            "rejected" => Ok(Some(GrantStatus::Rejected)),
            "processing" => Ok(Some(GrantStatus::Processing)),
            "finalized" => Ok(Some(GrantStatus::Finalized)),
            _ => Ok(None),
        },
    }
}

/// Filter criteria for querying sent grants (peer connections & VC requests).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SentGrantFilter {
    pub participant_id: Option<String>,
    pub participant_nick: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_grant_status"
    )]
    pub status: Option<GrantStatus>,
    pub kind: Option<GrantKind>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for SentGrantFilter {
    fn is_empty(&self) -> bool {
        self.participant_id.is_none()
            && self.participant_nick.is_none()
            && self.status.is_none()
            && self.kind.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying received grants (gatekeeper requests).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecvGrantFilter {
    pub participant_nick: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_grant_status"
    )]
    pub status: Option<GrantStatus>,
    pub kind: Option<GrantKind>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for RecvGrantFilter {
    fn is_empty(&self) -> bool {
        self.participant_nick.is_none()
            && self.status.is_none()
            && self.kind.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}
