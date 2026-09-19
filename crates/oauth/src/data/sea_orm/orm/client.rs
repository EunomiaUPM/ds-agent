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

use crate::entities::client::Client;
use crate::entities::role::RbacRole;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_clients")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub client_id: String,
    pub tenant_id: String,
    pub client_secret_hash: String,
    pub client_name: String,
    pub role: String,
    pub scopes: Json,
    pub created_at: DateTimeWithTimeZone,
}

impl Model {
    pub(crate) fn into_domain(self) -> Outcome<Client> {
        let role = self.role.parse::<RbacRole>().map_err(|e| {
            Errors::crazy(
                "invalid role stored in database",
                Some(e.to_string().into()),
            )
        })?;
        let scopes: Vec<String> = serde_json::from_value(self.scopes).unwrap_or_default();
        Ok(Client {
            client_id: self.client_id,
            tenant_id: self.tenant_id,
            client_secret_hash: self.client_secret_hash,
            client_name: self.client_name,
            role,
            scopes,
            created_at: self.created_at.with_timezone(&Utc),
        })
    }
}

impl ActiveModel {
    pub(crate) fn from_domain(c: &Client) -> Self {
        Self {
            client_id: Set(c.client_id.clone()),
            tenant_id: Set(c.tenant_id.clone()),
            client_secret_hash: Set(c.client_secret_hash.clone()),
            client_name: Set(c.client_name.clone()),
            role: Set(c.role.to_string()),
            scopes: Set(serde_json::to_value(&c.scopes).unwrap_or(serde_json::Value::Array(vec![]))),
            created_at: Set(c.created_at.into()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
