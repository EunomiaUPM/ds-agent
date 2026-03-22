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

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "catalog_catalogs")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub foaf_home_page: Option<String>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_identifier: Option<String>,
    pub dct_issued: DateTimeWithTimeZone,
    pub dct_modified: Option<DateTimeWithTimeZone>,
    pub dct_title: Option<String>,
    pub dspace_participant_id: Option<String>,
    pub dspace_main_catalog: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::dataset::Entity")]
    Dataset,
    #[sea_orm(has_many = "super::dataservice::Entity")]
    DataService,
    #[sea_orm(has_many = "super::odrl_offer::Entity")]
    OdrlOffer,
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

impl Related<super::odrl_offer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OdrlOffer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewCatalogModel {
    pub id: Option<Urn>,
    pub foaf_home_page: Option<String>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dspace_participant_id: Option<String>,
}

impl From<NewCatalogModel> for ActiveModel {
    fn from(dto: NewCatalogModel) -> Self {
        let new_urn = UrnBuilder::new("catalog", uuid::Uuid::new_v4().to_string().as_str())
            .build()
            .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(dto.id.clone().unwrap_or(new_urn.clone()).to_string()),
            foaf_home_page: ActiveValue::Set(dto.foaf_home_page),
            dct_conforms_to: ActiveValue::Set(dto.dct_conforms_to),
            dct_creator: ActiveValue::Set(dto.dct_creator),
            dct_identifier: ActiveValue::Set(Some(
                dto.id.clone().unwrap_or(new_urn.clone()).to_string(),
            )),
            dct_issued: ActiveValue::Set(chrono::Utc::now().into()),
            dct_modified: ActiveValue::Set(None),
            dct_title: ActiveValue::Set(dto.dct_title),
            dspace_participant_id: ActiveValue::Set(dto.dspace_participant_id),
            dspace_main_catalog: ActiveValue::Set(false),
        }
    }
}

impl From<&NewCatalogModel> for ActiveModel {
    fn from(dto: &NewCatalogModel) -> Self {
        dto.clone().into()
    }
}

pub struct EditCatalogModel {
    pub foaf_home_page: Option<String>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
}
