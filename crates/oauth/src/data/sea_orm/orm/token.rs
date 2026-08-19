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
use ymir::errors::Outcome;

use crate::entities::refresh_token::RefreshToken;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: String,
    pub jti: String,
    pub expires_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
    pub revoked: bool,
}

impl Model {
    pub(crate) fn into_domain(self) -> Outcome<RefreshToken> {
        Ok(RefreshToken {
            id: self.id,
            tenant_id: self.tenant_id,
            jti: self.jti,
            expires_at: self.expires_at.with_timezone(&Utc),
            created_at: self.created_at.with_timezone(&Utc),
            revoked: self.revoked,
        })
    }
}

impl ActiveModel {
    pub(crate) fn from_domain(rt: &RefreshToken) -> Self {
        Self {
            id: Set(rt.id),
            tenant_id: Set(rt.tenant_id.clone()),
            jti: Set(rt.jti.clone()),
            expires_at: Set(rt.expires_at.into()),
            created_at: Set(rt.created_at.into()),
            revoked: Set(rt.revoked),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
