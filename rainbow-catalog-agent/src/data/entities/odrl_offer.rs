/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */
use crate::entities::odrl_policies::CatalogEntityTypes;
use rainbow_common::dsp_common::odrl::OdrlPolicyInfo;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use urn::{Urn, UrnBuilder};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "catalog_odrl_offers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub odrl_offer: serde_json::Value,
    pub entity: String,
    pub entity_type: String,
    pub created_at: DateTimeWithTimeZone,
    pub source_template_id: Option<String>,
    pub source_template_version: Option<String>,
    pub instantiation_parameters: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::catalog::Entity",
        from = "Column::Entity",
        to = "super::catalog::Column::Id"
    )]
    Catalog,
    #[sea_orm(
        belongs_to = "super::dataset::Entity",
        from = "Column::Entity",
        to = "super::dataset::Column::Id"
    )]
    Dataset,
    #[sea_orm(
        belongs_to = "super::dataservice::Entity",
        from = "Column::Entity",
        to = "super::dataservice::Column::Id"
    )]
    DataService,
    #[sea_orm(
        belongs_to = "super::distribution::Entity",
        from = "Column::Entity",
        to = "super::distribution::Column::Id"
    )]
    Distribution,
    #[sea_orm(
        belongs_to = "super::policy_template::Entity",
        from = "(Column::SourceTemplateId, Column::SourceTemplateVersion)",
        to = "(super::policy_template::Column::Id, super::policy_template::Column::Version)"
    )]
    Template,
}

impl Related<super::policy_template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}
impl Related<super::catalog::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Catalog.def()
    }
}
impl Related<super::dataset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dataset.def()
    }
}
impl Related<super::dataservice::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataService.def()
    }
}
impl Related<super::distribution::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Distribution.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewOdrlOfferModel {
    pub id: Option<Urn>,
    pub odrl_offer: OdrlPolicyInfo,
    pub entity_id: Urn,
    pub entity_type: CatalogEntityTypes,
    pub source_template_id: Option<String>,
    pub source_template_version: Option<String>,
    pub instantiation_parameters: Option<serde_json::Value>,
    pub description: Option<String>,
}

impl From<NewOdrlOfferModel> for ActiveModel {
    fn from(dto: NewOdrlOfferModel) -> Self {
        let new_urn = UrnBuilder::new("odrl-policy", uuid::Uuid::new_v4().to_string().as_str())
            .build()
            .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(dto.id.clone().unwrap_or(new_urn.clone()).to_string()),
            odrl_offer: ActiveValue::Set(serde_json::to_value(dto.odrl_offer).unwrap_or_default()),
            entity: ActiveValue::Set(dto.entity_id.to_string()),
            entity_type: ActiveValue::Set(dto.entity_type.to_string()),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
            source_template_id: ActiveValue::Set(dto.source_template_id),
            source_template_version: ActiveValue::Set(dto.source_template_version),
            instantiation_parameters: ActiveValue::Set(dto.instantiation_parameters),
            description: ActiveValue::Set(dto.description),
        }
    }
}

impl From<&NewOdrlOfferModel> for ActiveModel {
    fn from(dto: &NewOdrlOfferModel) -> Self {
        dto.clone().into()
    }
}
