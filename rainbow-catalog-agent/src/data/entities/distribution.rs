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
use rainbow_common::dcat_formats::DctFormats;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use urn::{Urn, UrnBuilder};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "catalog_distributions")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub dct_issued: DateTimeWithTimeZone,
    pub dct_modified: Option<DateTimeWithTimeZone>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dcat_access_service: String,
    pub dataset_id: String,
    pub dct_format: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::dataset::Entity",
        from = "Column::DatasetId",
        to = "super::dataset::Column::Id"
    )]
    Dataset,
    #[sea_orm(
        belongs_to = "super::dataservice::Entity",
        from = "Column::DcatAccessService",
        to = "super::dataservice::Column::Id"
    )]
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

#[derive(Debug, Clone)]
pub struct NewDistributionModel {
    pub id: Option<Urn>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dct_formats: Option<String>,
    pub dcat_access_service: String,
    pub dataset_id: Urn,
}

impl From<NewDistributionModel> for ActiveModel {
    fn from(dto: NewDistributionModel) -> Self {
        let new_urn = UrnBuilder::new("distribution", uuid::Uuid::new_v4().to_string().as_str())
            .build()
            .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(dto.id.clone().unwrap_or(new_urn.clone()).to_string()),
            dct_issued: ActiveValue::Set(chrono::Utc::now().into()),
            dct_modified: ActiveValue::Set(None),
            dct_title: ActiveValue::Set(dto.dct_title),
            dct_description: ActiveValue::Set(dto.dct_description),
            dcat_access_service: ActiveValue::Set(dto.dcat_access_service),
            dataset_id: ActiveValue::Set(dto.dataset_id.to_string()),
            dct_format: ActiveValue::Set(dto.dct_formats.map(|d| d.to_string())),
        }
    }
}

impl From<&NewDistributionModel> for ActiveModel {
    fn from(dto: &NewDistributionModel) -> Self {
        dto.clone().into()
    }
}

pub struct EditDistributionModel {
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dcat_access_service: Option<String>,
}
