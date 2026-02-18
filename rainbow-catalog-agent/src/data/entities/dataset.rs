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
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use urn::{Urn, UrnBuilder};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "catalog_datasets")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_identifier: Option<String>,
    pub dct_issued: DateTimeWithTimeZone,
    pub dct_modified: Option<DateTimeWithTimeZone>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub catalog_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::catalog::Entity",
        from = "Column::CatalogId",
        to = "super::catalog::Column::Id"
    )]
    Catalog,
    #[sea_orm(has_many = "super::distribution::Entity")]
    Distribution,
    #[sea_orm(has_many = "super::odrl_offer::Entity")]
    OdrlOffer,
}

impl Related<super::catalog::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Catalog.def()
    }
}

impl Related<super::distribution::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Distribution.def()
    }
}

impl Related<super::odrl_offer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OdrlOffer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewDatasetModel {
    pub id: Option<Urn>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub catalog_id: Urn,
}

impl From<NewDatasetModel> for ActiveModel {
    fn from(dto: NewDatasetModel) -> Self {
        let new_urn = UrnBuilder::new("dataset", uuid::Uuid::new_v4().to_string().as_str())
            .build()
            .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(dto.id.clone().unwrap_or(new_urn.clone()).to_string()),
            dct_conforms_to: ActiveValue::Set(dto.dct_conforms_to),
            dct_creator: ActiveValue::Set(dto.dct_creator),
            dct_identifier: ActiveValue::Set(Some(
                dto.id.clone().unwrap_or(new_urn.clone()).to_string(),
            )),
            dct_issued: ActiveValue::Set(chrono::Utc::now().into()),
            dct_modified: ActiveValue::Set(None),
            dct_title: ActiveValue::Set(dto.dct_title),
            dct_description: ActiveValue::Set(dto.dct_description),
            catalog_id: ActiveValue::Set(dto.catalog_id.to_string()),
        }
    }
}

impl From<&NewDatasetModel> for ActiveModel {
    fn from(dto: &NewDatasetModel) -> Self {
        dto.clone().into()
    }
}

pub struct EditDatasetModel {
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
}
