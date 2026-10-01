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

use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use ymir::errors::{Errors, Outcome};

use crate::entities::pat::PersonalAccessToken;
use crate::entities::role::RbacRole;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_pats")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: String,
    pub name: String,
    pub token_prefix: String,
    pub token_hash: String,
    pub role: String,
    pub scopes: Json,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub last_used_at: Option<DateTimeWithTimeZone>,
    pub revoked: bool,
}

impl Model {
    pub(crate) fn into_domain(self) -> Outcome<PersonalAccessToken> {
        let role = self.role.parse::<RbacRole>().map_err(|e| {
            Errors::crazy(
                "invalid role in personal access token",
                Some(e.to_string().into()),
            )
        })?;
        let scopes: Vec<String> = serde_json::from_value(self.scopes).unwrap_or_default();
        Ok(PersonalAccessToken {
            id: self.id,
            tenant_id: self.tenant_id,
            name: self.name,
            token_prefix: self.token_prefix,
            token_hash: self.token_hash,
            role,
            scopes,
            expires_at: self.expires_at.map(|dt| dt.with_timezone(&Utc)),
            created_at: self.created_at.with_timezone(&Utc),
            last_used_at: self.last_used_at.map(|dt| dt.with_timezone(&Utc)),
            revoked: self.revoked,
        })
    }
}

impl ActiveModel {
    pub(crate) fn from_domain(p: &PersonalAccessToken) -> Self {
        Self {
            id: Set(p.id),
            tenant_id: Set(p.tenant_id.clone()),
            name: Set(p.name.clone()),
            token_prefix: Set(p.token_prefix.clone()),
            token_hash: Set(p.token_hash.clone()),
            role: Set(p.role.to_string()),
            scopes: Set(serde_json::to_value(&p.scopes).unwrap_or(serde_json::Value::Array(vec![]))),
            expires_at: Set(p.expires_at.map(Into::into)),
            created_at: Set(p.created_at.into()),
            last_used_at: Set(p.last_used_at.map(Into::into)),
            revoked: Set(p.revoked),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
