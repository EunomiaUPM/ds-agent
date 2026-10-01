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

use crate::entities::auth_code::AuthCode;
use crate::entities::role::RbacRole;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_auth_codes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub code: String,
    pub client_id: String,
    pub redirect_uri: Option<String>,
    pub tenant_id: String,
    pub role: String,
    pub scopes: Json,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub expires_at: DateTimeWithTimeZone,
    pub used: bool,
}

impl Model {
    pub(crate) fn into_domain(self) -> Outcome<AuthCode> {
        let role = self
            .role
            .parse::<RbacRole>()
            .map_err(|e| Errors::crazy("invalid role in auth code", Some(e.to_string().into())))?;
        let scopes: Vec<String> = serde_json::from_value(self.scopes).unwrap_or_default();
        Ok(AuthCode {
            code: self.code,
            client_id: self.client_id,
            redirect_uri: self.redirect_uri,
            tenant_id: self.tenant_id,
            role,
            scopes,
            code_challenge: self.code_challenge,
            code_challenge_method: self.code_challenge_method,
            expires_at: self.expires_at.with_timezone(&Utc),
            used: self.used,
        })
    }
}

impl ActiveModel {
    pub(crate) fn from_domain(a: &AuthCode) -> Self {
        Self {
            code: Set(a.code.clone()),
            client_id: Set(a.client_id.clone()),
            redirect_uri: Set(a.redirect_uri.clone()),
            tenant_id: Set(a.tenant_id.clone()),
            role: Set(a.role.to_string()),
            scopes: Set(serde_json::to_value(&a.scopes).unwrap_or(serde_json::Value::Array(vec![]))),
            code_challenge: Set(a.code_challenge.clone()),
            code_challenge_method: Set(a.code_challenge_method.clone()),
            expires_at: Set(a.expires_at.into()),
            used: Set(a.used),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
