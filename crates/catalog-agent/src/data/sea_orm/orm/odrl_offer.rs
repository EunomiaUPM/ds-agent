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

//! `catalog_offers` rows.

use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use ymir::errors::Outcome;
use ymir::oauth::Owner;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;
use ymir::utils::parse_urn;

use crate::entities::odrl_offer::OdrlOffer;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "catalog_offers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub odrl_offer: Json,
    pub entity: String,
    pub entity_type: String,
    pub created_at: DateTimeWithTimeZone,
    pub source_template_id: Option<String>,
    pub source_template_version: Option<String>,
    pub instantiation_parameters: Option<Json>,
    pub description: Option<String>,
    pub composition: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn into_domain(self) -> Outcome<OdrlOffer> {
        Ok(OdrlOffer {
            id: parse_urn(&self.id)?,
            owner: Owner::new(self.user_id, self.user_role, self.visibility),
            odrl_offer: self.odrl_offer,
            entity: parse_urn(&self.entity)?,
            entity_type: self.entity_type,
            created_at: self.created_at.with_timezone(&Utc),
            source_template_id: self.source_template_id,
            source_template_version: self.source_template_version,
            instantiation_parameters: self.instantiation_parameters,
            description: self.description,
            composition: self.composition,
        })
    }
}

impl ActiveModel {
    pub fn from_domain(offer: &OdrlOffer) -> Self {
        Self {
            id: Set(offer.id.to_string()),
            user_id: Set(offer.owner.user_id.clone()),
            user_role: Set(offer.owner.role.clone()),
            visibility: Set(offer.owner.visibility.clone()),
            odrl_offer: Set(offer.odrl_offer.clone()),
            entity: Set(offer.entity.to_string()),
            entity_type: Set(offer.entity_type.clone()),
            created_at: Set(offer.created_at.into()),
            source_template_id: Set(offer.source_template_id.clone()),
            source_template_version: Set(offer.source_template_version.clone()),
            instantiation_parameters: Set(offer.instantiation_parameters.clone()),
            description: Set(offer.description.clone()),
            composition: Set(offer.composition.clone()),
        }
    }
}
