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

use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use urn::Urn;
use ymir::errors::Outcome;

use crate::data::sea_orm::orm::helpers::parse_urn;
use crate::entities::transfer_process_identifier::TransferProcessIdentifier;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transfer_identifiers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub transfer_process_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub value: Option<String>,
}

impl Model {
    pub(crate) fn into_domain(self) -> Outcome<TransferProcessIdentifier> {
        let process_id = parse_urn(
            &self.transfer_process_id,
            "transfer_identifier.transfer_process_id",
        )?;
        Ok(TransferProcessIdentifier {
            transfer_process_id: process_id,
            key: self.key,
            value: self.value,
        })
    }
}

impl ActiveModel {
    pub(crate) fn from_domain(process_id: &Urn, ident: &TransferProcessIdentifier) -> Self {
        Self {
            transfer_process_id: Set(process_id.to_string()),
            key: Set(ident.key.clone()),
            value: Set(ident.value.clone()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
